use std::marker::PhantomData;
use crate::model::tree_view::*;
use crate::model::{NormalizedPath, ToNormalizedPath};
use crate::vcs::VCS;

/// Marker for an area node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Area<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

impl<C: NodeClassification> SymbolicNodeType for Area<C> {
    type Classification = C;

    fn new() -> Self {
        Self { _phantom: PhantomData }
    }

    fn compatible() -> Vec<NodeType> {
        todo!()
    }
}

impl<'a, C: NodeClassification, V: VCS> TreeView<'a, Area<C>, V> {
    pub fn get_path_to_feature_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(FEATURE_ROOT)
    }
    
    pub fn get_path_to_product_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(PRODUCT_ROOT)
    }
    
    pub fn move_to_feature_root(self) -> Result<TreeView<'a, FeatureRoot, V>, TreeViewError<V::VersionId>> {
        Ok(self.move_to(&NormalizedPath::from(FEATURE_ROOT))?)
    }
    
    pub fn move_to_product_root(self) -> Result<TreeView<'a, ProductRoot, V>, TreeViewError<V::VersionId>> {
        Ok(self.move_to(&NormalizedPath::from(PRODUCT_ROOT))?)
    }
}

pub trait IsUnderArea: SymbolicNodeType {}

impl<'a, T: IsUnderArea, V: VCS> TreeView<'a, T, V> {
    pub fn move_to_area<C: NodeClassification>(self) -> TreeView<'a, Area<C>, V> {
        self.move_to_index(1).unwrap()
    }
}