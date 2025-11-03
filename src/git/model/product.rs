use crate::git::model::*;

#[derive(Clone, Debug)]
pub struct ProductRoot;
impl NodeTypeBehavior for ProductRoot {
    fn build_child_from_path(&mut self, _: &Vec<&str>) -> NodeType {
        NodeType::Product(Product)
    }
}

#[derive(Clone, Debug)]
pub struct Product;
impl NodeTypeBehavior for Product {
    fn build_child_from_path(&mut self, _: &Vec<&str>) -> NodeType {
        NodeType::Product(Self)
    }
}
