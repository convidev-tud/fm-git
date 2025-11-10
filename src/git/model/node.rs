use crate::git::model::*;
use std::collections::HashMap;
use std::hash::Hash;
use termtree::Tree;

#[derive(Clone, Debug)]
pub struct NodeMetadata {
    has_branch: bool,
}
impl NodeMetadata {
    pub fn new(has_branch: bool) -> Self {
        Self { has_branch }
    }
    pub fn default() -> Self {
        Self { has_branch: false }
    }
}

pub trait NodeTypeInterface {
    fn get_type(&self) -> &NodeType;
}

#[derive(Clone, Debug)]
pub struct Node {
    name: String,
    node_type: NodeType,
    metadata: NodeMetadata,
    children: HashMap<String, Node>,
}

impl NodeTypeInterface for Node {
    fn get_type(&self) -> &NodeType {
        &self.node_type
    }
}

impl Node {
    pub fn new<S: Into<String>>(name: S, node_type: NodeType, metadata: NodeMetadata) -> Self {
        Self {
            name: name.into(),
            node_type,
            metadata,
            children: HashMap::new(),
        }
    }
    pub fn update_metadata(&mut self, metadata: NodeMetadata) {
        self.metadata = metadata;
    }
    fn add_child<S: Into<String>>(&mut self, name: S, metadata: NodeMetadata) -> Result<(), WrongNodeTypeError> {
        let real_name = name.into();
        let new_type = self.node_type.build_child_from_name(real_name.as_str())?;
        self.children.insert(real_name.clone(), Node::new(real_name, new_type, metadata));
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
    pub fn insert_path(&mut self, path: &QualifiedPath, metadata: NodeMetadata) -> Result<(), WrongNodeTypeError> {
        match path.len() {
            0 => Ok(()),
            1 => {
                let name = path.get(0).unwrap().to_string();
                match self.get_child_mut(&name) {
                    Some(node) => node.update_metadata(metadata),
                    None => {
                        self.add_child(name.clone(), metadata)?;
                    }
                };
                Ok(())
            },
            _ => {
                let name = path.get(0).unwrap().to_string();
                let next_child = match self.get_child_mut(&name) {
                    Some(node) => node,
                    None => {
                        self.add_child(name.clone(), NodeMetadata::default())?;
                        self.get_child_mut(&name).unwrap()
                    }
                };
                next_child.insert_path(&path.trim_n_left(1), metadata)
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
    pub fn as_node_path(&self) -> NodePath<'_> {
        NodePath::new(&self)
    }
    pub fn as_qualified_path(&self) -> QualifiedPath {
        QualifiedPath::from(self.name.clone())
    }
    pub fn get_qualified_paths_by<T, P>(&self, initial_path: &QualifiedPath, predicates: &HashMap<T, P>) -> HashMap<T, Vec<QualifiedPath>>
    where P: Fn(&Node) -> bool,
          T: Hash + Eq + Clone,
    {
        let mut result: HashMap<T, Vec<QualifiedPath>> = HashMap::new();
        for child in self.children.values() {
            let path = initial_path.clone() + child.as_qualified_path();
            for (t, predicate) in predicates {
                if predicate(child) {
                    result.insert(t.clone(), vec![path.clone()]);
                } else {
                    result.insert(t.clone(), vec![]);
                }
            }
            let from_child = child.get_qualified_paths_by(&path, predicates);
            for (t, value) in from_child {
                result.get_mut(&t).unwrap().extend(value);
            }
        };
        result
    }
    pub fn get_child_paths_by_branch(&self, has_branch: bool) -> Vec<QualifiedPath> {
        let predicate = |node: &Node| -> bool {
            node.metadata.has_branch == has_branch
        };
        let mut map = HashMap::new();
        map.insert(0, predicate);
        self.get_qualified_paths_by(&QualifiedPath::new(), &map).get(&0).unwrap().clone()
    }
}
