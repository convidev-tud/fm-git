use crate::git::model::*;

pub trait NodeTypeBehavior {
    fn build_child_from_path(&mut self, path: &Vec<&str>) -> NodeType;
}

#[derive(Clone, Debug)]
pub enum NodeType {
    Feature(Feature),
    Product(Product),
    FeatureRoot(FeatureRoot),
    ProductRoot(ProductRoot),
    Area(Area),
    VirtualRoot(VirtualRoot),
}

impl NodeTypeBehavior for NodeType {
    fn build_child_from_path(&mut self, path: &Vec<&str>) -> NodeType {
        match self {
            Self::Feature(feature) => feature.build_child_from_path(path),
            Self::Product(product) => product.build_child_from_path(path),
            Self::FeatureRoot(feature_root) => feature_root.build_child_from_path(path),
            Self::ProductRoot(product_root) => product_root.build_child_from_path(path),
            Self::Area(area) => area.build_child_from_path(path),
            Self::VirtualRoot(virtual_root) => virtual_root.build_child_from_path(path),
        }
    }
}
