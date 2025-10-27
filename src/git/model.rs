use std::collections::HashMap;

// there is feature/ and product/
// full feature paths are like this: root/featureA/featureA1
// feature paths can be very long, so shortening them would be appropriate
// feature/.../featureA1 or feature/featureA1?
pub const FEATURES_PREFIX: &str = "feature";
pub const PRODUCTS_PREFIX: &str = "product";

#[derive(Clone, Debug)]
pub struct SymNode {
    pub name: String,
    children: Vec<SymNode>,
}
impl SymNode {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            children: Vec::new(),
        }
    }
    pub fn add_child(&mut self, node: SymNode) {
        self.children.push(node);
    }
    pub fn iter_children(&self) -> std::slice::Iter<'_, SymNode> {
        self.children.iter()
    }
    pub fn get_child(&self, name: &str) -> Option<&SymNode> {
        self.children.iter().find(|s| s.name == name)
    }
    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut SymNode> {
        self.children.iter_mut().find(|s| s.name == name)
    }
    fn insert_from_split_path(&mut self, path: Vec<&str>) {
        if path.is_empty() {
            return;
        }
        let name = path[0];
        let next_child: &mut SymNode = match self.get_child_mut(name) {
            Some(node) => node,
            None => {
                self.add_child(SymNode::new(name));
                self.get_child_mut(name).unwrap()
            }
        };
        next_child.insert_from_split_path(path[1..].to_vec());
    }
    pub fn insert_qualified_path(&mut self, qualified_path: &str) {
        self.insert_from_split_path(qualified_path.split('/').collect::<Vec<_>>());
    }
}

#[derive(Clone, Debug)]
pub struct NodeData {
    pub qualified_path: String,
    pub git_branch: String,
}
impl NodeData {
    pub fn new(qualified_path: &str, git_branch: &str) -> Self {
        Self {
            qualified_path: String::from(qualified_path),
            git_branch: String::from(git_branch),
        }
    }
}

pub enum NodeType {
    Feature,
    Product,
}

#[derive(Clone, Debug)]
pub struct BranchDataModel {
    root_node: SymNode,
    feature_short_name_to_qualified_paths: HashMap<String, Vec<String>>,
    qualified_path_to_node_data: HashMap<String, NodeData>,
}
impl BranchDataModel {
    pub fn feature_path_prefix() -> String {
        FEATURES_PREFIX.to_string() + "/"
    }
    pub fn product_path_prefix() -> String {
        PRODUCTS_PREFIX.to_string() + "/"
    }
    pub fn new(default_branch: &str) -> Self {
        Self {
            root_node: SymNode::new(default_branch),
            feature_short_name_to_qualified_paths: HashMap::new(),
            qualified_path_to_node_data: HashMap::new(),
        }
    }
    pub fn insert_from_git_native_branch(&mut self, branch: &str) {
        let without_star = branch.replace("*", "");
        let converted_branch = without_star.replace("_", "").trim().to_string();
        let split_branch = converted_branch.split("/").collect::<Vec<&str>>();
        if !self
            .qualified_path_to_node_data
            .contains_key(&converted_branch.to_string())
        {
            self.qualified_path_to_node_data.insert(
                converted_branch.clone(),
                NodeData::new(converted_branch.as_str(), without_star.as_str()),
            );
        }
        if converted_branch.starts_with(FEATURES_PREFIX) {
            let feature_name = split_branch.last().unwrap().to_string();
            if !self
                .feature_short_name_to_qualified_paths
                .contains_key(&feature_name)
            {
                self.feature_short_name_to_qualified_paths
                    .insert(feature_name.clone(), vec![converted_branch.clone()]);
            } else {
                self.feature_short_name_to_qualified_paths
                    .get_mut(&feature_name)
                    .unwrap()
                    .push(converted_branch.clone());
            }
        }
        if converted_branch != self.root_node.name {
            self.root_node
                .insert_qualified_path(converted_branch.as_str());
        }
    }
    pub fn get_global_root(&self) -> &SymNode {
        &self.root_node
    }
    pub fn get_feature_root(&self) -> Option<&SymNode> {
        self.root_node.get_child(FEATURES_PREFIX)
    }
    pub fn get_product_root(&self) -> Option<&SymNode> {
        self.root_node.get_child(PRODUCTS_PREFIX)
    }
    pub fn get_short_feature_names(&self) -> Vec<String> {
        let mut unique: Vec<String> = Vec::new();
        self.feature_short_name_to_qualified_paths
            .iter()
            .filter_map(|(k, v)| match v.len() {
                0 => {
                    panic!("Must be a bug")
                }
                1 => Some(vec![k.clone()]),
                _ => None,
            })
            .for_each(|e| unique.extend(e));
        unique
    }
    pub fn get_long_name_from_short(&self, name: &str) -> Option<&str> {
        let long_names = self.feature_short_name_to_qualified_paths.get(name)?;
        match long_names.len() {
            0 => None,
            1 => Some(&long_names[0]),
            _ => None,
        }
    }
    pub fn get_git_branch(&self, qualified_path: &str) -> Option<&String> {
        Some(
            &self
                .qualified_path_to_node_data
                .get(qualified_path)?
                .git_branch,
        )
    }
    pub fn expand_from_short(&self, name: &str) -> Option<String> {
        if self.qualified_path_to_node_data.contains_key(name) {
            return Some(name.to_string());
        }
        match self.get_long_name_from_short(self.strip_branch_type(name).as_str()) {
            Some(long_name) => Some(long_name.to_string()),
            None => None,
        }
    }
    pub fn branch_type(&self, branch: &str) -> Option<NodeType> {
        let feature_path = BranchDataModel::feature_path_prefix();
        let product_path = BranchDataModel::product_path_prefix();
        if branch.starts_with(feature_path.as_str()) {
            Some(NodeType::Feature)
        } else if branch.starts_with(product_path.as_str()) {
            Some(NodeType::Product)
        } else {
            None
        }
    }
    pub fn strip_branch_type(&self, branch: &str) -> String {
        match self.branch_type(branch) {
            Some(NodeType::Feature) => {
                branch.replace(BranchDataModel::feature_path_prefix().as_str(), "")
            }
            Some(NodeType::Product) => {
                branch.replace(BranchDataModel::product_path_prefix().as_str(), "")
            }
            None => branch.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sym_node_add_children_from_qualified_path_empty() {
        let mut root = SymNode::new("root");
        root.insert_qualified_path("foo/bar");
        let foo = root.get_child("foo");
        assert!(foo.is_some());
        let bar = foo.unwrap().get_child("bar");
        assert!(bar.is_some());
    }
    #[test]
    fn test_sym_node_add_children_from_qualified_path_prefilled() {
        let mut root = SymNode::new("root");
        {
            root.insert_qualified_path("foo");
            let foo = root.get_child_mut("foo");
            assert!(foo.is_some());
        };
        root.insert_qualified_path("foo/bar");
        let bar = root.get_child("foo").unwrap().get_child("bar");
        assert!(bar.is_some());
    }
}
