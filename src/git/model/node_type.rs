use crate::git::model::*;

pub trait NodeTypeBehavior {
    fn build_child_from_path(&mut self, path: &QualifiedPath) -> NodeType;
}

#[derive(Clone, Debug)]
pub struct FeatureRoot {
    features_with_branches: Vec<QualifiedPath>,
}
impl FeatureRoot {
    pub fn new() -> Self {
        FeatureRoot {
            features_with_branches: Vec::new(),
        }
    }
    pub fn iter_features_with_branches(&self) -> impl Iterator<Item = &QualifiedPath> {
        self.features_with_branches.iter()
    }
}
impl NodeTypeBehavior for FeatureRoot {
    fn build_child_from_path(&mut self, path: &QualifiedPath) -> NodeType {
        self.features_with_branches.push(path.clone());
        NodeType::Feature(Feature)
    }
}

#[derive(Clone, Debug)]
pub struct Feature;
impl NodeTypeBehavior for Feature {
    fn build_child_from_path(&mut self, _: &QualifiedPath) -> NodeType {
        NodeType::Feature(Self)
    }
}

#[derive(Clone, Debug)]
pub struct ProductRoot;
impl NodeTypeBehavior for ProductRoot {
    fn build_child_from_path(&mut self, _: &QualifiedPath) -> NodeType {
        NodeType::Product(Product)
    }
}

#[derive(Clone, Debug)]
pub struct Product;
impl NodeTypeBehavior for Product {
    fn build_child_from_path(&mut self, _: &QualifiedPath) -> NodeType {
        NodeType::Product(Self)
    }
}

#[derive(Clone, Debug)]
pub struct Area;
impl NodeTypeBehavior for Area {
    fn build_child_from_path(&mut self, path: &QualifiedPath) -> NodeType {
        let first = path.first().unwrap();
        if first.to_string() == ModelConstants::feature_prefix() {
            NodeType::FeatureRoot(FeatureRoot::new())
        } else if first.to_string() == ModelConstants::product_prefix() {
            NodeType::ProductRoot(ProductRoot)
        } else {
            panic!("'{}' is no valid child of an area node", first.to_string())
        }
    }
}

#[derive(Clone, Debug)]
pub struct VirtualRoot;
impl NodeTypeBehavior for VirtualRoot {
    fn build_child_from_path(&mut self, _: &QualifiedPath) -> NodeType {
        NodeType::Area(Area)
    }
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
    fn build_child_from_path(&mut self, path: &QualifiedPath) -> NodeType {
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
