use crate::model::*;
use crate::vcs::{VCS, VCSError, VersionId};
use std::fmt::{Debug, Display, Formatter};
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum RevisionError<VI: VersionId, VE: VCSError> {
    Invalid(#[from] StaticView<VI>),
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

pub trait RevPointer: Debug + Clone {}

#[derive(Debug, Clone)]
pub struct Head;

impl RevPointer for Head {}

#[derive(Debug, Clone)]
pub struct Rev<V: VersionId> {
    revision: V,
}

impl<V: VersionId> Rev<V> {
    pub fn new(revision: V) -> Self {
        Self { revision }
    }
}

impl<V: VersionId> RevPointer for Rev<V> {}

#[derive(Debug)]
pub struct RevisionView<'a, S: IsConcrete, R: RevPointer, V: VCS> {
    semantic_view: SemanticView<'a, S, V>,
    revision_pointer: R,
}

impl<'a, S, R, V> RevisionView<'a, S, R, V>
where
    S: IsConcrete,
    R: RevPointer,
    V: VCS,
{
    pub fn get_semantic_view(&self) -> &SemanticView<'a, S, V> {
        &self.semantic_view
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
            revision_pointer: Head,
        };
        new
    }
}

impl<'a, S, V> RevisionView<'a, S, Rev<V::VersionId>, V>
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
            revision_pointer: Rev::new(revision),
        };
        Ok(new)
    }
}

#[derive(Debug)]
pub struct RevisionRef<'a, S: IsConcrete, V: VCS> {
    semantic_view: &'a SemanticView<'a, S, V>,
    revision: Rev<V::VersionId>,
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
            revision: Rev::new(revision),
        };
        Ok(new)
    }

    pub(crate) fn new_no_check(
        semantic_view: &'a SemanticView<'a, S, V>,
        revision: Rev<V::VersionId>,
    ) -> Self {
        Self {
            semantic_view,
            revision,
        }
    }
}
