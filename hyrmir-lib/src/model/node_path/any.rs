use crate::model::node_path::*;

/// Placeholder if the exact node type is unknown or does not matter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyNode<C: NodeClassification>;

impl<C: NodeClassification> SymbolicNodeType for AnyNode<C> {
    type Classification = C;

    fn compatible(&self) -> Vec<NodeType> {
        todo!()
    }
}