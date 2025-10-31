use crate::git::model::*;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub enum AreaChild {
    FeatureRoot(Rc<Node<FeatureRoot>>),
    ProductRoot(Rc<Node<ProductRoot>>),
}
impl Into<NodeType> for AreaChild {
    fn into(self) -> NodeType {
        match self {
            Self::FeatureRoot(node) |
            Self::ProductRoot(node)
            => node.into(),
        }
    }
}
impl Into<AreaChild> for NodeType {
    fn into(self) -> AreaChild {
        match self {
            NodeType::FeatureRoot(node) => AreaChild::FeatureRoot(node),
            NodeType::ProductRoot(node) => AreaChild::ProductRoot(node),
            _ => panic!()
        }
    }
}
#[derive(Debug)]
pub struct Area;

impl Node<Area> {
    pub fn new(name: String) -> Self {
        Self {
            name,
            node_type: Area,
            children: HashMap::new(),
        }
    }
}

impl NodeSpecialization for Node<Area> {
    type Child = AreaChild;

    fn get_direct_child<S: Into<String>>(&self, name: S) -> Option<&Self::Child> {
        self.children.get(name)?.into()
    }
}
