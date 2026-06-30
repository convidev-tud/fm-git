use crate::model::NodeType;
use crate::model::view::*;
use crate::vcs::VCS;

/// Marker for the virtual root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VirtualRoot;

impl SymbolicNodeType for VirtualRoot {
    type Classification = Abstract;

    fn new() -> Self {
        Self
    }

    fn compatible() -> Vec<NodeType> {
        vec![NodeType::VirtualRoot]
    }
}

/// Reachability for virtual root
impl<'a, V: VCS> SemanticView<'a, VirtualRoot, V> {
    pub fn move_to_area<C: NodeClassification>(
        self,
        area: &impl ToNormalizedPath,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, Area<C>, V>, TreeViewError<V::VersionId>> {
        self.move_to(area, repo)
    }
}
