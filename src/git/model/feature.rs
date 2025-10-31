use crate::git::model::*;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct FeatureRoot;

#[derive(Debug)]
pub struct Feature;
impl NodeTypeBehavior for Feature { type DirectChild = Node<Feature>; }

impl Node<Feature> {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            node_type: Feature,
            children: HashMap::new(),
        }
    }
}
impl Into<NodeType> for Node<Feature> {
    fn into(self) -> NodeType {
        NodeType::Feature(Rc::new(self))
    }
}
impl From<NodeType> for Node<Feature> {
    fn from(value: NodeType) -> Self {
        match value {
            NodeType::Feature(node) => node.into(),
            _ => unreachable!(),
        }
    }
}
impl NodeBuild for Node<Feature> {
    fn build_node<S: Into<String>>(&self, name: S) -> Self {
        Self::new(name.into())
    }
}
