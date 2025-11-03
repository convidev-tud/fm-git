use crate::git::model::*;
#[derive(Clone, Debug)]
pub struct VirtualRoot;
impl NodeTypeBehavior for VirtualRoot {
    fn build_child_from_path(&mut self, _: &Vec<&str>) -> NodeType {
        NodeType::Area(Area)
    }
}
