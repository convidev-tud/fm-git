use crate::git::model::*;

#[derive(Clone, Debug)]
pub struct Area;
impl NodeTypeBehavior for Area {
    fn build_child_from_path(&mut self, path: &Vec<&str>) -> NodeType {
        let first = path.first().unwrap();
        if first.to_string() == TreeDataModel::feature_prefix() {
            NodeType::FeatureRoot(FeatureRoot::new())
        } else if first.to_string() == TreeDataModel::product_prefix() {
            NodeType::ProductRoot(ProductRoot)
        } else {
            panic!()
        }
    }
}
