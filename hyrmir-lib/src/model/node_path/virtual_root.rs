use crate::model::node_path::*;
use crate::model::{NodeType, NormalizedPath, WrongNodeTypeError};
use crate::vcs::VCS;

/// Marker for the virtual root node.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VirtualRoot;

impl SymbolicNodeType for VirtualRoot {
    type Classification = Abstract;

    fn compatible(&self) -> Vec<NodeType> {
        vec![NodeType::VirtualRoot]
    }
}

/// Reachability for virtual root
impl<V: VCS> NodePath<VirtualRoot, V> {
    pub fn move_to_area<C: NodeClassification>(
        self,
        area: &NormalizedPath
    ) -> Result<NodePath<Area<C>, V>, PathNotFoundError> {
        match self.move_to(area) {
            Ok(node) => Ok(node),
            Err(error) => match error {
                NodePathError::NotFound(e) => Err(e),
                _ => unreachable!(),
            }
        }
    }
}

/// VCS connection for virtual root
impl<V: VCS> NodePath<VirtualRoot, V> {
    pub fn scan_repository(&self) -> Result<(), WrongNodeTypeError> {
        let mut node = self.get_node().borrow_mut();
        let vcs = self.get_vcs().borrow();
        for path in vcs.iter_concrete_paths() {
            let p = if path.is_absolute() {
                path.strip_n_left(1)
            } else { path };
            node.insert_path(&p, true)?;
        }
        Ok(())
    }
}