use crate::git::model::*;

const FEATURES_PREFIX: &str = "feature";
const PRODUCTS_PREFIX: &str = "product";

pub struct ModelConstants;
impl ModelConstants {
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
    pub fn transform_to_branch<S: Into<String>>(qualified_path: S) -> String {
        let path = qualified_path.into();
        let split = path.split('/').collect::<Vec<_>>();
        match split.len() {
            1 => split[0].to_string(),
            _ => {
                let mut prefix = split[..split.len() - 1]
                    .iter()
                    .map(|x| "_".to_string() + x)
                    .collect::<Vec<_>>();
                prefix.push(split[split.len() - 1].to_string());
                prefix.join("/")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreeDataModel {
    virtual_root: Node,
    qualified_paths_with_branch: Vec<String>,
}
impl TreeDataModel {
    pub fn new() -> Self {
        Self {
            virtual_root: Node::new("", NodeType::VirtualRoot(VirtualRoot), None),
            qualified_paths_with_branch: vec![],
        }
    }
    pub fn insert_from_git_native_branch(&mut self, branch: &str) {
        let without_star = branch.replace("*", "").trim().to_string();
        let converted_branch = ModelConstants::transform_to_qualified_path(&without_star);
        self.virtual_root.insert_path(
            converted_branch.split("/").collect::<Vec<&str>>(),
            without_star,
        );
        self.qualified_paths_with_branch.push(converted_branch);
    }
    pub fn get_area_node(&self, name: &str) -> Option<&Node> {
        self.virtual_root.get_child(name)
    }
    pub fn has_branch<S: Into<String> + Copy>(&self, qualified_path: S) -> bool {
        self.qualified_paths_with_branch
            .iter()
            .find(|e| **e == qualified_path.into())
            .is_some()
    }
    pub fn get_node_from_qualified_path<S: Into<String>>(&self, path: S) -> Option<&Node> {
        self.virtual_root
            .get_from_path(path.into().split("/").collect::<Vec<&str>>())
    }
    pub fn iter_qualified_paths_with_branches(&self) -> impl Iterator<Item = &String> {
        self.qualified_paths_with_branch.iter()
    }
}
