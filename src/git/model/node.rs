use crate::git::model::*;
use std::collections::HashMap;
use termtree::Tree;

#[derive(Clone, Debug)]
pub struct Node {
    name: String,
    node_type: NodeType,
    git_branch: Option<String>,
    children: HashMap<String, Node>,
}
impl Node {
    pub fn new<S: Into<String>>(name: S, node_type: NodeType, git_branch: Option<S>) -> Self {
        let maybe_branch = match git_branch {
            Some(git_branch) => Some(git_branch.into()),
            _ => None,
        };
        Self {
            name: name.into(),
            node_type,
            git_branch: maybe_branch,
            children: HashMap::new(),
        }
    }
    fn add_child(&mut self, child: Node) {
        self.children.insert(child.name.clone(), child);
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_child<S: Into<String>>(&self, name: S) -> Option<&Node> {
        Some(self.children.get(&name.into())?)
    }
    fn get_child_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut Node> {
        Some(self.children.get_mut(&name.into())?)
    }
    pub fn get_branch(&self) -> Option<&String> {
        self.git_branch.as_ref()
    }
    pub fn get_from_path(&self, path: Vec<&str>) -> Option<&Node> {
        if path.is_empty() {
            return None;
        }
        let maybe_child = self.get_child(path[0]);
        match maybe_child {
            Some(child) => match path.len() {
                1 => Some(child),
                _ => child.get_from_path(path[1..].to_vec()),
            },
            None => None,
        }
    }
    pub fn get_type(&self) -> &NodeType {
        &self.node_type
    }
    pub fn set_git_branch<S: Into<String>>(&mut self, branch: S) {
        self.git_branch = Some(branch.into());
    }
    pub fn insert_path<S: Into<String> + Clone>(&mut self, path: Vec<&str>, branch: S) {
        if path.is_empty() {
            return;
        }
        let name = path[0];
        let next_type = self.node_type.build_child_from_path(&path);
        match path.len() {
            1 => {
                match self.get_child_mut(name) {
                    Some(node) => node.set_git_branch(branch),
                    None => {
                        self.add_child(Node::new(
                            name,
                            next_type,
                            Some(branch.clone().into().as_str()),
                        ));
                        self.get_child_mut(name).unwrap();
                    }
                };
            }
            _ => {
                let next_child = match self.get_child_mut(name) {
                    Some(node) => node,
                    None => {
                        self.add_child(Node::new(
                            name,
                            next_type,
                            Some(branch.clone().into().as_str()),
                        ));
                        self.get_child_mut(name).unwrap()
                    }
                };
                next_child.insert_path(path[1..].to_vec(), branch);
            }
        }
    }
    fn build_display_tree(&self) -> Tree<String> {
        let mut tree = Tree::<String>::new(self.name.clone());
        for child in self.children.values() {
            tree.leaves.push(child.build_display_tree());
        }
        tree
    }
    pub fn display_tree(&self) -> String {
        self.build_display_tree().to_string()
    }
}
