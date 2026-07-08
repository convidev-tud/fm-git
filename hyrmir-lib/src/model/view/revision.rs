use crate::model::*;
use crate::vcs::{VCS, VCSError, VersionId};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum RevisionError<VI: VersionId, VE: VCSError> {
    Invalid(#[from] DynamicView<VI>),
    VCS(#[from] VE),
}

impl<VI: VersionId, VE: VCSError> Display for RevisionError<VI, VE> {
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

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum RevisionPointer<V: VersionId> {
    Head,
    Revision(V),
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
pub struct RevisionView<'a, S: IsConcrete, R: RevPointer, V: VCS> {
    semantic_view: SemanticView<'a, S, V>,
    revision: R,
}

impl<'a, S, R, V> RevisionView<'a, S, R, V>
where
    S: IsConcrete,
    R: RevPointer,
    V: VCS,
{
    fn lock_node(&self) {
        let view = self.get_semantic_view();
        let mut node = view.get_node().borrow_mut();
        let lock = node.try_lock();
        drop(node);
        if let Err(_) = lock {
            let path = view.to_normalized_path();
            panic!("Cannot lock path '{path}': a semantic view for this path already exists")
        }
    }
    
    pub fn get_semantic_view(&self) -> &SemanticView<'a, S, V> {
        &self.semantic_view
    }
    
    pub fn get_revision_id(&self) -> V::VersionId {
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

    pub fn get_head(&'a self) -> RevisionRef<'a, S, V> {
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
        &'a self,
        revision: impl Into<String>,
    ) -> Result<RevisionRef<'a, S, V>, RevisionError<V::VersionId, V::VCSError>> {
        RevisionRef::new(self.get_semantic_view(), revision)
    }
    
    pub fn to_rev(&'a self) -> RevisionRef<'a, S, V> {
        RevisionRef::new_no_check(self.get_semantic_view(), self.get_revision_id())
    }
}

impl<'a, S, V> RevisionView<'a, S, Head, V>
where
    S: IsConcrete,
    V: VCS,
{
    pub(crate) fn new(semantic_view: SemanticView<'a, S, V>) -> Self {
        let new = Self {
            semantic_view,
            revision: Head::new(),
        };
        new.lock_node();
        new
    }
}

impl<'a, S, V> RevisionView<'a, S, Rev, V>
where
    S: IsConcrete,
    V: VCS,
{
    pub(crate) fn new(
        semantic_view: SemanticView<'a, S, V>,
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

impl<'a, S, R, V> Drop for RevisionView<'a, S, R, V>
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
pub struct RevisionRef<'a, S: IsConcrete, V: VCS> {
    semantic_view: &'a SemanticView<'a, S, V>,
    revision: V::VersionId,
}

impl<'a, S, V> RevisionRef<'a, S, V>
where
    S: IsConcrete,
    V: VCS,
{
    pub(crate) fn new(
        semantic_view: &'a SemanticView<'a, S, V>,
        revision: impl Into<String>,
    ) -> Result<Self, RevisionError<V::VersionId, V::VCSError>> {
        let revision = semantic_view.assert_revision(revision)?;
        let new = Self {
            semantic_view,
            revision,
        };
        Ok(new)
    }

    pub(crate) fn new_no_check(
        semantic_view: &'a SemanticView<'a, S, V>,
        revision: V::VersionId,
    ) -> Self {
        Self {
            semantic_view,
            revision,
        }
    }
}
