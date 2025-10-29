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
    pub fn get_name(&self) -> &String {
        &self.name
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
    pub fn iter_children_flat(&self) -> impl Iterator<Item = &SymNode> {
        self.children.iter()
    }
    pub fn iter_children_req(&self) -> impl Iterator<Item = &SymNode> {
        self.iter_children_flat()
            .map(|child| child.iter_children_req())
            .flatten()
            .collect::<Vec<&SymNode>>()
            .into_iter()
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
    pub fn get_path<S: Into<String>>(&self, qualified_path: S) -> Option<SymPath<'_>> {
        let mut current = self;
        let mut node_vec: Vec<&SymNode> = Vec::new();
        for name in qualified_path.into().split('/') {
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
            .map(|e| e.get_name().to_string())
            .collect::<Vec<_>>()
            .join("/")
    }
    pub fn get_last(&self) -> Option<&&SymNode> {
        self.node_path.last()
    }
    pub fn get_first(&self) -> Option<&&SymNode> {
        self.node_path.first()
    }
}

pub enum NodeType {
    Feature,
    Product,
}

#[derive(Clone, Debug)]
pub struct BranchDataModel {
    root_node: SymNode,
    qualified_paths_of_existing_branches: Vec<String>,
}
impl BranchDataModel {
    pub fn feature_path_prefix() -> String {
        FEATURES_PREFIX.to_string() + "/"
    }
    pub fn product_path_prefix() -> String {
        PRODUCTS_PREFIX.to_string() + "/"
    }
    pub fn new() -> Self {
        Self {
            root_node: SymNode::new("", None),
            qualified_paths_of_existing_branches: Vec::new(),
        }
    }
    pub fn transform_to_qualified_path<S: Into<String>>(&self, path: S) -> String {
        path.into().trim().replace("*", "").replace("_", "")
    }
    pub fn insert_from_git_native_branch(&mut self, branch: &str) {
        let without_star = branch.replace("*", "").trim().to_string();
        let converted_branch = self.transform_to_qualified_path(&without_star);
        if !self
            .qualified_paths_of_existing_branches
            .contains(&converted_branch)
        {
            self.qualified_paths_of_existing_branches
                .push(converted_branch.clone());
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
    pub fn get_feature_root_of(&self, area: &str) -> Option<&SymNode> {
        let area_node = self.root_node.get_child(area)?;
        area_node.get_child(FEATURES_PREFIX)
    }
    pub fn get_product_root_of(&self, area: &str) -> Option<&SymNode> {
        let area_node = self.root_node.get_child(area)?;
        area_node.get_child(PRODUCTS_PREFIX)
    }
    pub fn iter_all_features_with_branches_of(&self, area: &str) -> impl Iterator<Item = &String> {
        self.qualified_paths_of_existing_branches
            .iter()
            .filter(|name| name.starts_with(&(area.to_string() + "/" + FEATURES_PREFIX)))
    }
    pub fn get_git_branch(&self, qualified_path: &str) -> Option<String> {
        let path = self.get_global_root().get_path(qualified_path)?;
        let branch = path.get_last()?.get_branch_data()?.get_git_branch();
        Some(branch.clone())
    }
    pub fn iter_all_qualified_paths(&self) -> impl Iterator<Item = &String> {
        self.qualified_paths_of_existing_branches.iter()
    }
    pub fn has_qualified_path(&self, qualified_path: &str) -> bool {
        self.qualified_paths_of_existing_branches
            .contains(&qualified_path.to_string())
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

    fn prepare_model() -> BranchDataModel {
        let mut model = BranchDataModel::new();
        model.insert_from_git_native_branch("* _main/feature/_root/feature1");
        model.insert_from_git_native_branch("  _main/feature/_root/feature2");
        model.insert_from_git_native_branch("  _main/feature/root");
        model.insert_from_git_native_branch("  main");
        model
    }

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
    #[test]
    fn test_branch_model_insert_branches() {
        let model = prepare_model();
        assert!(
            model
                .qualified_paths_of_existing_branches
                .contains(&"main".to_string())
        );
        assert!(
            model
                .qualified_paths_of_existing_branches
                .contains(&"main/feature/root".to_string())
        );
        assert!(
            model
                .qualified_paths_of_existing_branches
                .contains(&"main/feature/root/feature1".to_string())
        );
        assert!(
            model
                .qualified_paths_of_existing_branches
                .contains(&"main/feature/root/feature2".to_string())
        );
    }
    #[test]
    fn test_branch_model_get_git_branch() {
        let model = prepare_model();
        assert_eq!(model.get_git_branch("main").unwrap(), "main");
        assert_eq!(
            model.get_git_branch("main/feature/root").unwrap(),
            "_main/feature/root"
        );
        assert_eq!(
            model.get_git_branch("main/feature/root/feature1").unwrap(),
            "_main/feature/_root/feature1"
        );
        assert_eq!(
            model.get_git_branch("main/feature/root/feature2").unwrap(),
            "_main/feature/_root/feature2"
        );
    }
    #[test]
    fn test_sym_path_qualified_name() {
        let model = prepare_model();
        let path = model
            .get_global_root()
            .get_path("main/feature/root/feature1")
            .unwrap();
        assert_eq!(path.get_qualified_path(), "main/feature/root/feature1");
    }
}
