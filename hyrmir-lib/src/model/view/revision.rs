use crate::model::*;
use crate::vcs::{VCSError, VersionId, VCS};
use std::fmt::{Display, Formatter};
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
            },
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

#[derive(Debug)]
pub struct RevisionView<'a, S: IsConcrete, V: VCS> {
    semantic_view: SemanticView<'a, S, V>,
    revision_pointer: RevisionPointer<V::VersionId>,
}

impl<'a, S: IsConcrete, V: VCS> RevisionView<'a, S, V> {
    pub(crate) fn new(
        semantic_view: SemanticView<'a, S, V>,
        revision: &NormalizedRevision,
    ) -> Result<Self, RevisionError<V::VersionId, V::VCSError>> {
        let revision = semantic_view.assert_revision(&revision)?;
        let new = Self {
            semantic_view,
            revision_pointer: revision,
        };
        Ok(new)
    }
    
    pub fn get_ref(&'a self) -> RevisionRef<'a, S, V> {
        RevisionRef {
            semantic_view: &self.semantic_view,
            revision_pointer: self.revision_pointer.clone(),
        }
    }
    
    pub fn get_semantic_view(&self) -> &SemanticView<'a, S, V> {
        &self.semantic_view
    }
}

#[derive(Debug)]
pub struct RevisionRef<'a, S: IsConcrete, V: VCS> {
    semantic_view: &'a SemanticView<'a, S, V>,
    revision_pointer: RevisionPointer<V::VersionId>,
}

impl<'a, S: IsConcrete, V: VCS> RevisionRef<'a, S, V> {
    pub(crate) fn new(
        semantic_view: &'a SemanticView<'a, S, V>,
        revision: &NormalizedRevision,
    ) -> Result<Self, RevisionError<V::VersionId, V::VCSError>> {
        let revision = semantic_view.assert_revision(&revision)?;
        let new = Self {
            semantic_view, 
            revision_pointer: revision,
        };
        Ok(new)
    }
}