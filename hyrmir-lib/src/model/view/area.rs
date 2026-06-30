use crate::model::view::*;
use crate::model::{NormalizedPath, ToNormalizedPath};
use crate::vcs::VCS;
use std::marker::PhantomData;

/// Marker for an area node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Area<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

impl<C: NodeClassification> SymbolicNodeType for Area<C> {
    type Classification = C;

    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    fn compatible() -> Vec<NodeType> {
        todo!()
    }
}

impl<'a, C: NodeClassification, V: VCS> SemanticView<'a, Area<C>, V> {
    pub fn get_path_to_feature_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(FEATURE_ROOT)
    }

    pub fn get_path_to_product_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(PRODUCT_ROOT)
    }

    pub fn move_to_feature_root(
        self,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, FeatureRoot, V>, TreeViewError<V::VersionId>> {
        Ok(self.move_to(&NormalizedPath::from(FEATURE_ROOT), repo)?)
    }

    pub fn move_to_product_root(
        self,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, ProductRoot, V>, TreeViewError<V::VersionId>> {
        Ok(self.move_to(&NormalizedPath::from(PRODUCT_ROOT), repo)?)
    }
}

pub trait IsOrUnderArea: SymbolicNodeType {}

impl<'a, T: IsOrUnderArea, V: VCS> SemanticView<'a, T, V> {
    pub fn get_area<C: NodeClassification>(
        &self,
        repo: &'a Repository<V>,
    ) -> SemanticView<'a, Area<C>, V> {
        self.get_at_index(1, repo).unwrap()
    }

    pub fn move_to_area<C: NodeClassification>(
        self,
        repo: &'a Repository<V>,
    ) -> SemanticView<'a, Area<C>, V> {
        self.move_to_index(1, repo).unwrap()
    }
}
