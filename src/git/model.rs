use std::collections::HashMap;

pub const FEATURES_PREFIX: &str = "feature";
pub const PRODUCTS_PREFIX: &str = "product";

#[derive(Clone, Debug)]
pub struct BranchData {
    git_branch: String,
}
impl BranchData {
    pub fn new<S: Into<String>>(git_branch: S) -> Self {
        Self {
            git_branch: git_branch.into(),
        }
    }
    pub fn get_git_branch(&self) -> &String {
        &self.git_branch
    }
}

#[derive(Clone, Debug)]
pub struct SymNode {
    name: String,
    children: Vec<SymNode>,
    branch_data: Option<BranchData>,
}
impl SymNode {
    pub fn new<S: Into<String>>(name: S, branch_data: Option<BranchData>) -> Self {
        Self {
            name: name.into(),
            children: Vec::new(),
            branch_data,
        }
    }
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn get_branch_data(&self) -> Option<&BranchData> {
        match &self.branch_data {
            Some(data) => Some(&data),
            None => None,
        }
    }
    pub fn update_branch_data(&mut self, branch_data: Option<BranchData>) {
        self.branch_data = branch_data;
    }
    pub fn add_child(&mut self, node: SymNode) {
        self.children.push(node);
    }
    pub fn iter_children_flat(&self) -> std::slice::Iter<'_, SymNode> {
        self.children.iter()
    }
    pub fn get_child(&self, name: &str) -> Option<&SymNode> {
        self.children.iter().find(|s| s.get_name() == name)
    }
    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut SymNode> {
        self.children.iter_mut().find(|s| s.name == name)
    }
    fn insert_from_split_path(&mut self, path: Vec<&str>, branch_data: Option<BranchData>) {
        let name = path[0];
        if path.len() == 1 {
            match self.get_child_mut(name) {
                Some(child) => child.update_branch_data(branch_data),
                None => self.add_child(SymNode::new(name, branch_data)),
            }
        } else {
            let next_child: &mut SymNode = match self.get_child_mut(name) {
                Some(node) => node,
                None => {
                    self.add_child(SymNode::new(name, None));
                    self.get_child_mut(name).unwrap()
                }
            };
            next_child.insert_from_split_path(path[1..].to_vec(), branch_data);
        }
    }
    pub fn insert_qualified_path(&mut self, qualified_path: &str, branch_data: Option<BranchData>) {
        self.insert_from_split_path(qualified_path.split('/').collect::<Vec<_>>(), branch_data);
    }
    pub fn get_path(&self, qualified_path: &str) -> Option<SymPath<'_>> {
        let mut current = self;
        let mut node_vec: Vec<&SymNode> = Vec::new();
        node_vec.push(current);
        for name in qualified_path.split('/') {
            let maybe_child = current.get_child(name);
            if maybe_child.is_some() {
                let child = maybe_child.unwrap();
                node_vec.push(child);
                current = child;
            } else {
                return None;
            }
        }
        Some(SymPath::new(node_vec))
    }
}

#[derive(Clone, Debug)]
pub struct SymPath<'a> {
    node_path: Vec<&'a SymNode>,
}
impl<'a> SymPath<'a> {
    pub fn new(node_path: Vec<&'a SymNode>) -> Self {
        Self { node_path }
    }
    pub fn get_qualified_path(&self) -> String {
        self.node_path
            .iter()
            .fold(String::new(), |acc, node| acc + "/" + node.get_name())
    }
    pub fn get_last(&self) -> Option<&&SymNode> {
        self.node_path.last()
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
    qualified_paths_of_existing_branches: Vec<String>,
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
            root_node: SymNode::new(default_branch, Some(BranchData::new(default_branch))),
            feature_short_name_to_qualified_paths: HashMap::new(),
            qualified_paths_of_existing_branches: Vec::new(),
        }
    }
    pub fn transform_to_qualified_paths<S: Into<String>>(&self, path: S) -> String {
        path.into().replace("*", "").replace("_", "")
    }
    pub fn insert_from_git_native_branch(&mut self, branch: &str) {
        let without_star = branch.replace("*", "");
        let converted_branch = self.transform_to_qualified_paths(&without_star);
        let split_branch = converted_branch.split("/").collect::<Vec<&str>>();
        if !self
            .qualified_paths_of_existing_branches
            .contains(&converted_branch)
        {
            self.qualified_paths_of_existing_branches
                .push(converted_branch.clone());
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
            self.root_node.insert_qualified_path(
                converted_branch.as_str(),
                Some(BranchData::new(without_star)),
            );
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
    pub fn get_git_branch(&self, qualified_path: &str) -> Option<String> {
        let path = self.get_global_root().get_path(qualified_path)?;
        let branch = path.get_last()?.get_branch_data()?.get_git_branch();
        Some(branch.clone())
    }
    pub fn get_all_qualified_paths(&self) -> &Vec<String> {
        &self.qualified_paths_of_existing_branches
    }
    pub fn expand_from_short(&self, name: &str) -> Option<String> {
        if self
            .qualified_paths_of_existing_branches
            .contains(&name.into())
        {
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
        let mut root = SymNode::new("root", None);
        root.insert_qualified_path("foo/bar", Some(BranchData::new("bar")));
        let foo = root.get_child("foo");
        assert!(foo.is_some());
        let bar = foo.unwrap().get_child("bar");
        assert!(bar.is_some());
        assert!(bar.unwrap().branch_data.is_some());
    }
    #[test]
    fn test_sym_node_add_children_from_qualified_path_prefilled() {
        let mut root = SymNode::new("root", None);
        {
            root.insert_qualified_path("foo", None);
            let foo = root.get_child_mut("foo");
            assert!(foo.is_some());
        };
        root.insert_qualified_path("foo/bar", Some(BranchData::new("bar")));
        let bar = root.get_child("foo").unwrap().get_child("bar");
        assert!(bar.is_some());
        assert!(bar.unwrap().branch_data.is_some());
    }
}
