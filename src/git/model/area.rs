use crate::git::model::*;

#[derive(Clone, Debug)]
pub struct Area;
impl NodeTypeBehavior for Area {
    fn build_child_from_path(&mut self, path: &Vec<&str>) -> NodeType {
        let first = path.first().unwrap();
        if first.to_string() == ModelConstants::feature_prefix() {
            NodeType::FeatureRoot(FeatureRoot::new())
        } else if first.to_string() == ModelConstants::product_prefix() {
            NodeType::ProductRoot(ProductRoot)
        } else {
            panic!("'{}' is no valid child of an area node", first.to_string())
        }
    }
}
