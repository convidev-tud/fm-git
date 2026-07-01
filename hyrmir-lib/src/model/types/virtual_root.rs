use crate::model::NodeType;
use crate::model::view::*;
use crate::vcs::VCS;


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
