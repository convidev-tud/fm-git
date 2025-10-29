use crate::git::model::feature::FeatureNode;
use crate::git::model::product::ProductNode;
use crate::git::model::*;

#[derive(Clone, Debug)]
pub struct AreaNode {
    name: String,
    features: Node<FeatureNode>,
    products: Node<ProductNode>,
}
impl NodeBase for AreaNode {
    fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            features: Node::new("feature"),
            products: Node::new("product"),
        }
    }
    fn get_name(&self) -> &String {
        &self.name
    }
    fn insert_path(&mut self, path: Vec<&str>) {
        if path.is_empty() {
            return;
        }
        if path[0] == TreeDataModel::feature_prefix() {
            self.features.insert_path(path[1..].to_vec());
        } else if path[0] == TreeDataModel::product_prefix() {
            self.products.insert_path(path[1..].to_vec());
        } else {
            panic!("Wrong branch layout")
        }
    }
}
