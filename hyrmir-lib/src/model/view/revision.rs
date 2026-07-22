use crate::model::*;
use crate::vcs::{VCSError, VersionId, VCS};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum RevisionError<V: VCS, VE: VCSError> {
    Invalid(#[from] DynamicView<V>),
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

#[derive(Debug)]
pub struct RevisionView<S, R, M>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
{
    semantic_view: StructureView<S, M>,
    revision: R,
}

impl<S, R, M> RevisionView<S, R, M>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
{
    fn lock_node(&self) -> Result<(), RevisionLockError> {
        let view = self.get_semantic_view();
        let mut node = view.get_node().borrow_mut();
        let lock = node.try_lock();
        drop(node);
        if let Err(_) = lock {
            Err(RevisionLockError::new(self.semantic_view.to_normalized_path()))
        } else {
            Ok(())
        }
    }
    
    pub fn get_semantic_view(&self) -> &StructureView<S, M> {
        &self.semantic_view
    }
    
    pub fn get_revision_id(&self) -> <M::V as VCS>::VersionId {
        self
            .get_semantic_view()
            .get_node()
            .borrow()
            .get_branch_info()
            .unwrap()
            .get_known_version(self.revision.get_revision())
            .unwrap()
            .clone()
    }

    pub fn get_head(&self) -> RevisionRef<S, M> {
        let view = self.get_semantic_view();
        let head = view
            .get_node()
            .borrow()
            .get_branch_info()
            .unwrap()
            .get_head()
            .clone();
        RevisionRef::new_no_check(view, head)
    }

    pub fn get_rev(
        &self,
        revision: impl Into<String>,
    ) -> Result<RevisionRef<S, M>, RevisionError<M::VersionId, M::VCSError>> {
        RevisionRef::new(self.get_semantic_view(), revision)
    }
    
    pub fn to_rev(&'a self) -> RevisionRef<S, M> {
        RevisionRef::new_no_check(self.get_semantic_view(), self.get_revision_id())
    }
}

impl<S, V> RevisionView<S, Head, V>
where
    S: IsConcrete,
    V: VCS,
{
    pub(crate) fn new(semantic_view: StructureView<S, V>) -> Self {
        let new = Self {
            semantic_view,
            revision: Head::new(),
        };
        new.lock_node();
        new
    }
}

impl<S, V> RevisionView<S, Rev, V>
where
    S: IsConcrete,
    V: VCS,
{
    pub(crate) fn new(
        semantic_view: StructureView<S, V>,
        revision: impl Into<String>,
    ) -> Result<Self, RevisionError<V::VersionId, V::VCSError>> {
        let revision = semantic_view.assert_revision(revision)?;
        let new = Self {
            semantic_view,
            revision: Rev::new(revision.get_full_id()),
        };
        new.lock_node();
        Ok(new)
    }
}

impl<S, R, V> Drop for RevisionView<S, R, V>
where
    S: IsConcrete,
    R: RevPointer,
    V: VCS,
{
    fn drop(&mut self) {
        self
            .get_semantic_view()
            .get_node()
            .borrow_mut()
            .unlock()
    }
}

#[derive(Debug)]
pub struct RevisionRef<'a, S: IsConcrete, M: AccessMode> {
    semantic_view: &'a StructureView<S, M>,
    revision: <M::V as VCS>::VersionId,
}

impl<'a, S, M> RevisionRef<'a, S, M>
where
    S: IsConcrete,
    M: AccessMode,
{
    pub(crate) fn new(
        semantic_view: &'a StructureView<S, M>,
        revision: impl Into<String>,
    ) -> Result<Self, RevisionError<M::VersionId, M::VCSError>> {
        let revision = semantic_view.assert_revision(revision)?;
        let new = Self {
            semantic_view,
            revision,
        };
        Ok(new)
    }

    pub(crate) fn new_no_check(
        semantic_view: &'a StructureView<S, M>,
        revision: M::VersionId,
    ) -> Self {
        Self {
            semantic_view,
            revision,
        }
    }
}
