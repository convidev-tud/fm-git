use crate::git::model::node::NodeTypeInterface;
use crate::git::model::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Product;
impl NodeTypeInterface for Product {
    fn make_child_node<S: Into<String>>(&self, name: S) -> NodeType {
        todo!()
    }
}
impl Node<Product> {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            node_type: Product,
            children: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct ProductRoot;
