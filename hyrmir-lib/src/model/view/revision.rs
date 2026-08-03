use crate::model::*;
use crate::vcs::{VCS, VCSError};
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

/*
    ###################
    # Main Definition #
    ###################
*/

#[derive(Debug)]
pub struct RevisionView<'a, S, R, M, V>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
    V: VCS,
{
    structure_view: StructureView<'a, S, Shared, V>,
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
        if node.is_revision_locked() {
            drop(node);
            panic!(
                "Cannot create revision view for for '{}': node is locked",
                view.to_normalized_path(),
            )
        }
        if M::lock() {
            if node.revision_views_referenced() > 0 {
                let referenced = node.revision_views_referenced();
                drop(node);
                panic!(
                    "Cannot lock node for revision view '{}': there are {referenced} other revision views referencing it",
                    view.to_normalized_path(),
                )
            }
            node.lock_revision();
        }
        node.reference_revision_view();
    }

    pub fn assert_revision(
        structure_view: &StructureView<S, Shared, V>,
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

    pub fn get_structure_view(&self) -> &StructureView<'a, S, Shared, V> {
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
            RevisionPointer::Revision(revision.clone())
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
    pub(crate) fn new(structure_view: StructureView<'a, S, Shared, V>) -> Self {
        let revision = structure_view
            .get_node()
            .get()
            .borrow()
            .get_branch_info()
            .unwrap()
            .get_head()
            .clone();
        let new = Self {
            structure_view,
            revision,
            _revision_type: PhantomData,
            _access_mode: PhantomData,
        };
        new.assert_lock();
        new
    }
}

impl<'a, S, M, V> RevisionView<'a, S, Rev, M, V>
where
    S: IsConcrete,
    M: AccessMode,
    V: VCS,
{
    pub(crate) fn new(
        structure_view: StructureView<'a, S, Shared, V>,
        revision: impl Into<String>,
    ) -> Result<Self, RevisionError<V, V::VCSError>> {
        let revision = Self::assert_revision(&structure_view, revision)?;
        let new = Self {
            structure_view,
            revision,
            _revision_type: PhantomData,
            _access_mode: PhantomData,
        };
        new.assert_lock();
        Ok(new)
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
