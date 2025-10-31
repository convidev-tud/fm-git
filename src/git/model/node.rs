use crate::git::model::*;
use std::collections::HashMap;
use std::rc::Rc;

trait NodeBase {
    fn add_child(&mut self, child: NodeType);
    fn get_child_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut NodeType>;
}

pub trait NodeSpecialization: NodeBase {
    type DirectChild: Into<NodeType> + From<NodeType> + NodeBuild;
    
    fn get_direct_child<S: Into<String>>(&self, name: S) -> Option<&Self::DirectChild>;
    fn insert_path(&mut self, path: Vec<&str>) {
        if path.is_empty() {
            return;
        }
        let name = path[0];
        let next_child = match self.get_child_mut(name) {
            Some(node) => node,
            None => {
                self.add_child(self.build_child(name));
                self.get_child_mut(name).unwrap()
            }
        };
        next_child.insert_path(path[1..].to_vec());
    }
}

pub trait NodeBuild {
    fn build_node<S: Into<String>>(&self, name: S) -> Self;
}

pub trait NodeTypeBehavior {
    type DirectChild: Into<NodeType> + From<NodeType> + NodeBuild;
}

#[derive(Debug)]
pub enum NodeType {
    Feature(Rc<Node<Feature>>),
    Product(Rc<Node<Product>>),
    FeatureRoot(Rc<Node<FeatureRoot>>),
    ProductRoot(Rc<Node<ProductRoot>>),
    Area(Rc<Node<Area>>),
}
impl NodeBase for NodeType {
    fn add_child(&mut self, child: NodeType) {
        match self {
            Self::Feature(n)
            | Self::Product(n)
            | Self::FeatureRoot(n)
            | Self::ProductRoot(n)
            | Self::Area(n) => n.add_child(child),
        }
    }

    fn get_child_mut<S: Into<String>>(&mut self, name: S) -> Option<&mut NodeType> {
        match self {
            Self::Feature(n)
            | Self::Product(n)
            | Self::FeatureRoot(n)
            | Self::ProductRoot(n)
            | Self::Area(n) => n.get_child_mut(name),
        }
    }
}
impl NodeSpecialization for NodeType {
    type DirectChild = NodeType;

    fn build_child<S: Into<String>>(&self, name: S) -> NodeType {
        match self {
            Self::Feature(node)
            | Self::Product(node)
            | Self::FeatureRoot(node)
            | Self::ProductRoot(node)
            | Self::Area(node) => node.build_child(name),
        }
    }

    fn get_direct_child<S: Into<String>>(&self, name: S) -> Option<&Self::DirectChild> {
        match self {
            Self::Feature(node)
            | Self::Product(node)
            | Self::FeatureRoot(node)
            | Self::ProductRoot(node)
            | Self::Area(node) => node.get_direct_child(name)?.into(),
        }
    }
}

#[derive(Debug)]
pub struct Node<T: NodeTypeBehavior> {
    pub(super) name: String,
    pub(super) node_type: T,
    pub(super) children: HashMap<String, NodeType>,
}
impl<T: NodeTypeBehavior> NodeBase for Node<T> {}
impl<T: NodeTypeBehavior> NodeSpecialization for Node<T> {
    type DirectChild = T::DirectChild;

    fn get_direct_child<S: Into<String>>(&self, name: S) -> Option<&Self::DirectChild> {
        self.children.get(name)?.into()
    }
}
