use crate::model::node_path::*;
use crate::model::{NodeType, NormalizedPath};
use crate::vcs::VCS;

/// Marker for the virtual root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VirtualRoot;

impl ValidNodeType for VirtualRoot {
    type Classification = Abstract;

    fn compatible() -> Vec<NodeType> {
        vec![NodeType::VirtualRoot]
    }
}

/// Reachability for virtual root
impl<V: VCS> NodePath<VirtualRoot, V> {
    pub fn move_to_area<C: NodeClassification>(
        self,
        area: &NormalizedPath
    ) -> Result<NodePath<Area<C>, V>, NodePath<ErrorState, V>> {
        self.move_to(area)
    }
}