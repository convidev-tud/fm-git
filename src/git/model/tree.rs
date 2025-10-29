use crate::git::model::*;

const FEATURES_PREFIX: &str = "feature";
const PRODUCTS_PREFIX: &str = "product";

#[derive(Clone, Debug)]
pub struct TreeDataModel {
    node: Node<AreaNode>,
}
impl TreeDataModel {
    pub fn feature_prefix() -> String {
        FEATURES_PREFIX.to_string()
    }
    pub fn product_prefix() -> String {
        PRODUCTS_PREFIX.to_string()
    }
    pub fn feature_path_prefix() -> String {
        FEATURES_PREFIX.to_string() + "/"
    }
    pub fn product_path_prefix() -> String {
        PRODUCTS_PREFIX.to_string() + "/"
    }
    pub fn transform_to_qualified_path<S: Into<String>>(path: S) -> String {
        path.into().trim().replace("*", "").replace("_", "")
    }

    pub fn new() -> Self {
        Self {
            node: Node::new(""),
        }
    }
    pub fn insert_from_git_native_branch(&mut self, branch: &str) {
        let without_star = branch.replace("*", "").trim().to_string();
        let converted_branch = Self::transform_to_qualified_path(&without_star);
        self.node
            .insert_path(converted_branch.split("/").collect::<Vec<&str>>());
    }
    pub fn get_area_node(&self, name: &str) -> Option<&AreaNode> {
        self.node.get_child(name)
    }
}
