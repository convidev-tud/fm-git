use std::collections::HashMap;

const FEATURES_PREFIX: &str = "feature";
const PRODUCTS_PREFIX: &str = "product";

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
    pub fn add_child(&mut self, name: &str) {
        self.children.push(SymNode::new(name));
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
    pub fn add_children_recursive(&mut self, qualified_path: Vec<&str>) {
        if qualified_path.is_empty() {
            return;
        }
        let name = qualified_path[0];
        let next_child: &mut SymNode = match self.get_child_mut(name) {
            Some(node) => node,
            None => {
                self.add_child(name);
                self.get_child_mut(name).unwrap()
            }
        };
        next_child.add_children_recursive(qualified_path[1..].to_vec());
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

#[derive(Clone, Debug)]
pub struct BranchDataModel {
    root_node: SymNode,
    feature_short_name_to_qualified_paths: HashMap<String, Vec<String>>,
    qualified_path_to_node_data: HashMap<String, NodeData>,
}
impl BranchDataModel {
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
            self.root_node.add_children_recursive(split_branch);
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
    pub fn get_unique_feature_names(&self) -> Vec<String> {
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
}
