use std::marker::PhantomData;
use crate::model::node_path::*;

/// Placeholder if the exact node type is unknown or does not matter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyNode<C: NodeClassification> {
    _phantom: PhantomData<C>
}

impl<C: NodeClassification> ValidNodeType for AnyNode<C> {
    type Classification = C;

    fn new() -> Self {
        Self { _phantom: PhantomData }
    }

    fn compatible() -> Vec<NodeType> {
        let mut base = vec![
            NodeType::VirtualRoot,
            NodeType::FeatureRoot,
            NodeType::ProductRoot,
        ];
        match Self::Classification::requires_artifact() {
            Some(true) => {
                base.push(NodeType::Area(true));
                base.push(NodeType::Feature(true));
                base.push(NodeType::Product(true));
            },
            Some(false) => {
                base.push(NodeType::Area(false));
                base.push(NodeType::Feature(false));
                base.push(NodeType::Product(false));
            },
            None => {
                base.push(NodeType::Area(true));
                base.push(NodeType::Feature(true));
                base.push(NodeType::Product(true));
                base.push(NodeType::Area(false));
                base.push(NodeType::Feature(false));
                base.push(NodeType::Product(false));
            },
        }
        base
    }
}