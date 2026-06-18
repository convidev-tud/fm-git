use std::marker::PhantomData;
use crate::model::node_path::*;
use crate::model::{NormalizedPath, ToNormalizedPath};
use crate::vcs::VCS;

/// Marker for an area node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Area<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

impl<C: NodeClassification> ValidNodeType for Area<C> {
    type Classification = C;

    fn new() -> Self {
        Self { _phantom: PhantomData }
    }

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
    
    pub fn move_to_feature_root(self) -> Result<NodePath<FeatureRoot, V>, VCSPathError<V, V::VCSError>> {
        Ok(self.move_to(&NormalizedPath::from(FEATURE_ROOT))?)
    }
    
    pub fn move_to_product_root(self) -> Result<NodePath<ProductRoot, V>, VCSPathError<V, V::VCSError>> {
        Ok(self.move_to(&NormalizedPath::from(PRODUCT_ROOT))?)
    }
}

pub trait IsUnderArea: ValidNodeType {}

impl<T: IsUnderArea, V: VCS> NodePath<T, V> {
    pub fn move_to_area<C: NodeClassification>(self) -> NodePath<Area<C>, V> {
        self.move_to_index(1).unwrap()
    }
}