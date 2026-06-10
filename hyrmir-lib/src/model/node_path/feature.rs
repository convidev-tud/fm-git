use crate::model::node_path::*;
use crate::vcs::VCS;
use std::marker::PhantomData;

/// Marker for the feature root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FeatureRoot;

impl SymbolicNodeType for FeatureRoot {
    type Classification = Abstract;

    fn compatible(&self) -> Vec<NodeType> {
        todo!()
    }
}

impl IsUnderArea for FeatureRoot {}

/// Marker of a feature node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Feature<C: NodeClassification> {
    _phantom: PhantomData<C>,
}

impl<C: NodeClassification> SymbolicNodeType for Feature<C> {
    type Classification = C;

    fn compatible(&self) -> Vec<NodeType> {
        todo!()
    }
}

impl<C: NodeClassification> IsUnderArea for Feature<C> {}

pub trait CanMergeWithFeature: SymbolicNodeType {}