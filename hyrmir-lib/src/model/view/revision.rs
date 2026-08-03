use crate::model::*;
use crate::vcs::{VCSError, VersionId, VCS};
use std::fmt::{Debug, Display, Formatter};
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
    fn get_revision(&self) -> &String;
}

#[derive(Debug, Clone)]
pub struct Head {
    head: String,
}

impl Head {
    pub fn new() -> Self {
        Self { head: "HEAD".to_string() }
    }
}

impl RevPointer for Head {
    fn get_revision(&self) -> &String {
        &self.head
    }
}

#[derive(Debug, Clone)]
pub struct Rev {
    revision: String,
}

impl Rev {
    pub fn new(revision: impl Into<String>) -> Self {
        Self { revision: revision.into() }
    }
}

impl RevPointer for Rev {
    fn get_revision(&self) -> &String {
        &self.revision
    }
}

/*
    #######################
        Main Definition
    #######################
*/

#[derive(Debug)]
pub struct RevisionView<'a, S, R, M, V>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
    V: VCS,
{
    structure_view: StructureView<'a, S, M, V>,
    revision: R,
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
        structure_view: &StructureView<S, M, V>,
        revision: impl Into<String>,
    ) -> Result<V::VersionId, RevisionError<V, V::VCSError>> {
        let rev = revision.into();
        let vcs = structure_view.get_vcs();
        if vcs
            .revision_exists_on_path(&structure_view.to_normalized_path(), &rev)?
        {
            let revision = vcs.get_revision(&rev)?.unwrap();
            structure_view.get_node()
                .get()
                .borrow_mut()
                .mut_get_branch_info()
                .unwrap()
                .add_known_version(revision.clone());
            Ok(revision)
        } else {
            Err(structure_view.to_frozen_view(RevisionPointer::Invalid(rev)).into())
        }
    }
    
    pub fn get_structure_view(&self) -> &StructureView<'a, S, M, V> {
        &self.structure_view
    }
    
    pub fn get_revision_id(&self) -> V::VersionId {
        self
            .get_structure_view()
            .get_node()
            .get()
            .borrow()
            .get_branch_info()
            .unwrap()
            .get_known_version(self.revision.get_revision())
            .unwrap()
            .clone()
    }
}

impl<'a, S, M, V> RevisionView<'a, S, Head, M, V>
where
    S: IsConcrete,
    M: AccessMode,
    V: VCS,
{
    pub(crate) fn new(structure_view: StructureView<'a, S, M, V>) -> Self {
        let new = Self {
            structure_view,
            revision: Head::new(),
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
        structure_view: StructureView<'a, S, M, V>,
        revision: impl Into<String>,
    ) -> Result<Self, RevisionError<V, V::VCSError>> {
        let revision = Self::assert_revision(&structure_view, revision)?;
        let new = Self {
            structure_view,
            revision: Rev::new(revision.get_full_id()),
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
        let mut node = self
            .get_structure_view()
            .get_node()
            .get()
            .borrow_mut();
        if M::lock() {
            node.unlock_revision()
        }
        node.dereference_revision_view()
    }
}
