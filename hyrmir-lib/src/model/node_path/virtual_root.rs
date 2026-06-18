use crate::model::node_path::*;
use crate::model::{NodeType, NormalizedPath};
use crate::vcs::VCS;

/// Marker for the virtual root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VirtualRoot;

impl ValidNodeType for VirtualRoot {
    type Classification = Abstract;

    fn new() -> Self {
        Self
    }

    fn compatible() -> Vec<NodeType> {
        vec![NodeType::VirtualRoot]
    }
}

/// Reachability for virtual root
impl<V: VCS> NodePath<VirtualRoot, V> {
    pub fn move_to_area<C: NodeClassification>(
        self,
        area: &NormalizedPath
    ) -> Result<NodePath<Area<C>, V>, VCSPathError<V, V::VCSError>> {
        self.move_to(area)
    }
}