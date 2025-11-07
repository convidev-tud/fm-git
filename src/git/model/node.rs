use crate::git::model::*;
use std::collections::HashMap;
use termtree::Tree;

#[derive(Clone, Debug)]
pub struct Node {
    name: String,
    node_type: NodeType,
    children: HashMap<String, Node>,
}
impl Node {
    pub fn new<S: Into<String>>(name: S, node_type: NodeType) -> Self {
        Self {
            name: name.into(),
            node_type,
            children: HashMap::new(),
        }
    }
    fn add_child<S: Into<String>>(&mut self, name: S) -> Result<(), WrongNodeTypeError> {
        let real_name = name.into();
        let new_type = self.node_type.build_child_from_name(real_name.as_str())?;
        self.children.insert(real_name.clone(), Node::new(real_name, new_type));
        Ok(())
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
    pub fn get_type(&self) -> &NodeType {
        &self.node_type
    }
    pub fn insert_path(&mut self, path: &QualifiedPath) -> Result<(), WrongNodeTypeError> {
        if path.is_empty() {
            return Ok(());
        }
        let name = path.get(0).unwrap().to_string();
        let next_child = match self.get_child_mut(&name) {
            Some(node) => node,
            None => {
                self.add_child(name.clone())?;
                self.get_child_mut(&name).unwrap()
            }
        };
        next_child.insert_path(&path.trim_n_left(1))
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
    pub fn as_node_path(&self) -> NodePath<'_> {
        NodePath::new(&self)
    }
}
