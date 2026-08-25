use crate::model::*;
use crate::vcs::{RevisionId, VCS, VCSError};
use indextree::Node;
use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use thiserror::Error;

/*
    ##########
    # Errors #
    ##########
*/

#[derive(Error, Clone, Debug)]
pub enum RevisionError<V: VCS, VE: VCSError> {
    Invalid(#[from] FrozenView<V>),
    VCS(#[from] VE),
}

impl<V: VCS, VE: VCSError> Display for RevisionError<V, VE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            RevisionError::Invalid(view) => {
                format!(
                    "Revision does not exist on this path\n  {}",
                    view.formatted(true, true, true)
                )
            }
            RevisionError::VCS(error) => error.to_string(),
        };
        f.write_str(&msg)
    }
}

#[derive(Error, Clone, Debug)]
pub struct RevisionLockError {
    path: NormalizedPath,
}

impl RevisionLockError {
    pub fn new(path: NormalizedPath) -> Self {
        Self { path }
    }
}

impl Display for RevisionLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let formatted = format!(
            "Cannot lock '{}': a revision view for this path already exists",
            self.path
        );
        f.write_str(&formatted)
    }
}

/*
    ########################
        Revision Pointer
    ########################
*/

pub trait RevPointer: Debug + Clone {
    fn is_head() -> bool;
}

#[derive(Debug, Clone)]
pub struct Head;

impl RevPointer for Head {
    fn is_head() -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct Rev;

impl RevPointer for Rev {
    fn is_head() -> bool {
        false
    }
}

// ###################
// # Main Definition #
// ###################

#[derive(Debug, Eq, PartialOrd, Ord)]
pub struct RevisionView<'a, S, R, M, V>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
    V: VCS,
{
    structure_view: StructureView<'a, S, Read, V>,
    revision: V::RevisionId,
    _revision_type: PhantomData<R>,
    _access_mode: PhantomData<M>,
}

impl<'a, S, R, M, V> RevisionView<'a, S, R, M, V>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
    V: VCS,
{
    fn assert_lock(&self) {
        let view = self.get_structure_view();
        let mut node = view.get_node().get().borrow_mut();
        node.reference_revision_view();
        if node.is_revision_locked() {
            drop(node);
            panic!(
                "Cannot create revision view for '{}': node is locked",
                view.to_normalized_path(),
            )
        }
        if M::lock() {
            let referenced = node.revision_views_referenced() - 1;
            if referenced > 0 {
                drop(node);
                panic!(
                    "Cannot lock node for revision view '{}': there are {referenced} other revision views referencing it",
                    view.to_normalized_path(),
                )
            }
            node.lock_revision();
        }
    }

    fn private_new(
        structure_view: StructureView<'a, S, Read, V>,
        revision: V::RevisionId,
    ) -> Self {
        let new = Self {
            structure_view,
            revision,
            _revision_type: PhantomData,
            _access_mode: PhantomData,
        };
        new.assert_lock();
        new
    }

    fn convert<R2, M2>(self) -> RevisionView<'a, S, R2, M2, V>
    where
        R2: RevPointer,
        M2: AccessMode,
    {
        let structure_view = self.structure_view.private_clone();
        let revision = self.revision.clone();
        drop(self);
        RevisionView::<'a, S, R2, M2, V>::private_new(structure_view, revision)
    }

    pub(crate) fn get_node(&self) -> &Node<RefCell<NodeData<V>>> {
        self.get_structure_view().get_node()
    }

    pub fn assert_revision(
        structure_view: &StructureView<S, Read, V>,
        revision: impl Into<String>,
    ) -> Result<V::RevisionId, RevisionError<V, V::VCSError>> {
        let rev = revision.into();
        let vcs = structure_view.get_vcs();
        if vcs.revision_exists_on_path(&structure_view.to_normalized_path(), &rev)? {
            let revision = vcs.get_revision(&rev)?.unwrap();
            Ok(revision)
        } else {
            Err(structure_view
                .to_frozen_view(RevisionPointer::Invalid(rev))
                .into())
        }
    }

    pub fn get_structure_view(&self) -> &StructureView<'a, S, Read, V> {
        &self.structure_view
    }

    pub fn get_revision(&self) -> &V::RevisionId {
        &self.revision
    }

    pub fn to_frozen_view(&self) -> FrozenView<V> {
        let revision = self.get_revision();
        let revision_pointer = if R::is_head() {
            RevisionPointer::Head(revision.clone())
        } else {
            let node = self.get_structure_view().get_node().get().borrow();
            let head = node.get_branch_info().unwrap().get_head();
            if revision == head {
                RevisionPointer::Head(revision.clone())
            } else {
                RevisionPointer::Revision(revision.clone())
            }
        };
        self.get_structure_view().to_frozen_view(revision_pointer)
    }

    pub fn formatted(&self, show_type: bool, show_version: bool, colored: bool) -> String {
        self.to_frozen_view()
            .formatted(show_type, show_version, colored)
    }
}

impl<'a, S, M, V> RevisionView<'a, S, Head, M, V>
where
    S: IsConcrete,
    M: AccessMode,
    V: VCS,
{
    pub(crate) fn new(structure_view: StructureView<'a, S, Read, V>) -> Self {
        let revision = structure_view
            .get_node()
            .get()
            .borrow()
            .get_branch_info()
            .unwrap()
            .get_head()
            .clone();
        Self::private_new(structure_view, revision)
    }

    pub fn convert_to_rev(self) -> RevisionView<'a, S, Rev, M, V> {
        self.convert()
    }
}

impl<'a, S, M, V> RevisionView<'a, S, Rev, M, V>
where
    S: IsConcrete,
    M: AccessMode,
    V: VCS,
{
    pub(crate) fn new(
        structure_view: StructureView<'a, S, Read, V>,
        revision: impl Into<String>,
    ) -> Result<Self, RevisionError<V, V::VCSError>> {
        let revision = Self::assert_revision(&structure_view, revision)?;
        Ok(Self::private_new(structure_view, revision))
    }
}

impl<'a, S, R, V> RevisionView<'a, S, R, Read, V>
where
    S: IsConcrete,
    R: RevPointer,
    V: VCS,
{
    pub fn lock(self) -> RevisionView<'a, S, R, ReadWrite, V> {
        self.convert()
    }
}

impl<'a, S, R, V> RevisionView<'a, S, R, ReadWrite, V>
where
    S: IsConcrete,
    R: RevPointer,
    V: VCS,
{
    pub fn unlock(self) -> RevisionView<'a, S, R, Read, V> {
        self.convert()
    }
}

// #########################
// # Trait Implementations #
// #########################

impl<'a, S, R, M, V> Normalize for RevisionView<'a, S, R, M, V>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
    V: VCS,
{
    fn try_normalize(&self) -> Result<Normalized, NormalizeError> {
        let path = self.get_structure_view().to_normalized_path();
        let revision = match R::is_head() {
            true => NormalizedRevision::None,
            false => NormalizedRevision::Revision(self.revision.get_full_id()),
        };
        Ok(Normalized::new(path, revision))
    }
}

impl<'a, S, R, M, V> Display for RevisionView<'a, S, R, M, V>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
    V: VCS,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let output = self.to_frozen_view().to_string();
        f.write_str(&output)
    }
}

impl<'a, S, R, M, V, T> PartialEq<T> for RevisionView<'a, S, R, M, V>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
    V: VCS,
    T: Normalize,
{
    fn eq(&self, other: &T) -> bool {
        self.normalize() == other.normalize()
    }
}

impl<'a, S, R, M, V> Drop for RevisionView<'a, S, R, M, V>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
    V: VCS,
{
    fn drop(&mut self) {
        let mut node = self.get_structure_view().get_node().get().borrow_mut();
        if M::lock() {
            node.unlock_revision()
        }
        node.dereference_revision_view()
    }
}

// #########
// # Tests #
// #########

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::test_utils::prepare_repo;

    #[test]
    #[should_panic(
        expected = "Cannot create revision view for '/main/feature/foo': node is locked"
    )]
    fn test_revision_view_lock_and_create_second() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let _view = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap()
            .head()
            .lock();
        root.move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap()
            .head();
    }

    #[test]
    #[should_panic(
        expected = "Cannot lock node for revision view '/main/feature/foo': there are 1 other revision views referencing it"
    )]
    fn test_revision_view_lock_while_other_exists() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let _view1 = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap()
            .head();
        let _view2 = root
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap()
            .head()
            .lock();
    }

    #[test]
    fn test_revision_view_lock_and_unlock() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let view1 = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap()
            .head()
            .lock();
        assert!(view1.get_node().get().borrow().is_revision_locked());
        let view1 = view1.unlock();
        assert!(!view1.get_node().get().borrow().is_revision_locked());
        let view2 = root
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap()
            .head();
        assert_eq!(view1, view2);
    }

    #[test]
    fn test_revision_view_count_increment() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let view1 = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert_eq!(
            view1.get_node().get().borrow().structure_views_referenced(),
            1
        );
        let view2 = root
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert_eq!(
            view1.get_node().get().borrow().structure_views_referenced(),
            2
        );
        assert_eq!(
            view2.get_node().get().borrow().structure_views_referenced(),
            2
        );
    }

    #[test]
    fn test_revision_view_count_decrement() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let view1 = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert_eq!(
            view1.get_node().get().borrow().structure_views_referenced(),
            1
        );
        let view2 = root
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert_eq!(
            view1.get_node().get().borrow().structure_views_referenced(),
            2
        );
        drop(view2);
        assert_eq!(
            view1.get_node().get().borrow().structure_views_referenced(),
            1
        );
    }
}
