use std::marker::PhantomData;
use crate::model::node_path::*;
use crate::model::{NormalizedPath, ToNormalizedPath};
use crate::vcs::VCS;

/// Marker for an area node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Area<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

impl<C: NodeClassification> SymbolicNodeType for Area<C> {
    type Classification = C;

    fn compatible() -> Vec<NodeType> {
        todo!()
    }
}

impl<C: NodeClassification, V: VCS> NodePath<Area<C>, V> {
    pub fn get_path_to_feature_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(FEATURE_ROOT)
    }
    pub fn get_path_to_product_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(PRODUCT_ROOT)
    }
    pub fn move_to_feature_root(self) -> Result<NodePath<FeatureRoot, V>, NodePathError> {
        self.move_to(&NormalizedPath::from(FEATURE_ROOT))?.into()
    }
    pub fn move_to_product_root(self) -> Result<NodePath<ProductRoot, V>, NodePathError> {
        self.move_to(&NormalizedPath::from(PRODUCT_ROOT))?.into()
    }
}

pub trait IsUnderArea: SymbolicNodeType {}

impl<T: IsUnderArea, V: VCS> NodePath<T, V> {
    pub fn move_to_area<C: NodeClassification>(self) -> NodePath<Area<C>, V> {
        self.move_to_index(1).unwrap()
    }
}