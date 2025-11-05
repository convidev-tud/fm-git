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
    pub fn _feature_path_prefix() -> String {
        FEATURES_PREFIX.to_string() + "/"
    }
    pub fn _product_path_prefix() -> String {
        PRODUCTS_PREFIX.to_string() + "/"
    }
}

#[derive(Clone, Debug)]
pub struct TreeDataModel {
    virtual_root: Node,
    qualified_paths_with_branch: Vec<QualifiedPath>,
}
impl TreeDataModel {
    pub fn new() -> Self {
        Self {
            virtual_root: Node::new("", NodeType::VirtualRoot(VirtualRoot)),
            qualified_paths_with_branch: vec![],
        }
    }
    pub fn insert_qualified_path(&mut self, path: QualifiedPath) {
        self.virtual_root.insert_path(&path);
        self.qualified_paths_with_branch.push(path);
    }
    pub fn get_node_path(&self, path: &QualifiedPath) -> Option<NodePath<'_>> {
        let area_node = self.virtual_root.get_child(path.first()?)?;
        let mut node_path = area_node.as_node_path();
        let new_path = path.trim_n_left(1);
        match node_path.push_path(new_path) {
            Ok(_) => Some(node_path),
            Err(_) => None,
        }
    }
    pub fn get_qualified_path_to_product_root(&self, area: &QualifiedPath) -> QualifiedPath {
        area.clone() + QualifiedPath::from(ModelConstants::product_prefix())
    }
    pub fn get_qualified_path_to_feature_root(&self, area: &QualifiedPath) -> QualifiedPath {
        area.clone() + QualifiedPath::from(ModelConstants::feature_prefix())
    }
    pub fn has_branch(&self, qualified_path: &QualifiedPath) -> bool {
        self.qualified_paths_with_branch
            .iter()
            .find(|e| *e == qualified_path)
            .is_some()
    }
    pub fn iter_qualified_paths_with_branches(&self) -> impl Iterator<Item = &QualifiedPath> {
        self.qualified_paths_with_branch.iter()
    }
}
