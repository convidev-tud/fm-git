use crate::model::*;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;

pub enum NodePathType {
    Feature(NodePath<Feature>),
    FeatureRoot(NodePath<FeatureRoot>),
    Product(NodePath<Product>),
    ProductRoot(NodePath<ProductRoot>),
    Area(NodePath<Area>),
    VirtualRoot(NodePath<VirtualRoot>),
    Tag(NodePath<Tag>),
}

pub struct NodePath<T> {
    path: Vec<Rc<Node>>,
    _phantom: PhantomData<T>,
}

impl NodePath<AnyNodeType> {
    fn to_concrete_type<T>(self) -> NodePath<T> {
        NodePath::<T>::from_path(self.path)
    }
    pub fn from_concrete<T>(other: NodePath<T>) -> Self {
        Self::from_path(other.path)
    }
    pub fn concretize(self) -> NodePathType {
        match self.get_node().get_type() {
            NodeType::Feature => NodePathType::Feature(self.to_concrete_type()),
            NodeType::FeatureRoot => NodePathType::FeatureRoot(self.to_concrete_type()),
            NodeType::Product => NodePathType::Product(self.to_concrete_type()),
            NodeType::ProductRoot => NodePathType::ProductRoot(self.to_concrete_type()),
            NodeType::Area => NodePathType::Area(self.to_concrete_type()),
            NodeType::VirtualRoot => NodePathType::VirtualRoot(self.to_concrete_type()),
            NodeType::Tag => NodePathType::Tag(self.to_concrete_type()),
        }
    }
}

impl NodePath<Area> {
    pub fn get_path_to_feature_root(&self) -> QualifiedPath {
        self.get_qualified_path() + QualifiedPath::from(FEATURES_PREFIX)
    }
    pub fn get_path_to_product_root(&self) -> QualifiedPath {
        self.get_qualified_path() + QualifiedPath::from(PRODUCTS_PREFIX)
    }
    pub fn to_feature_root(self) -> Option<NodePath<FeatureRoot>> {
        match self.to(&QualifiedPath::from(FEATURES_PREFIX))?.concretize() {
            NodePathType::FeatureRoot(path) => Some(path),
            _ => unreachable!(),
        }
    }
    pub fn to_product_root(self) -> Option<NodePath<ProductRoot>> {
        match self.to(&QualifiedPath::from(PRODUCTS_PREFIX))?.concretize() {
            NodePathType::ProductRoot(path) => Some(path),
            _ => unreachable!(),
        }
    }
}

impl<T> NodePath<T> {
    fn from_path(path: Vec<Rc<Node>>) -> Self {
        Self {
            path,
            _phantom: PhantomData,
        }
    }
    pub fn new(area: Rc<Node>) -> NodePath<T> {
        Self {
            path: vec![area],
            _phantom: PhantomData,
        }
    }
    fn get_node(&self) -> &Node {
        self.path.last().unwrap()
    }
    pub fn get_tags(&self) -> Vec<QualifiedPath> {
        self.get_node()
            .iter_children()
            .filter_map(|(name, child)| match child.get_type() {
                NodeType::Tag => Some(QualifiedPath::from(name.clone())),
                _ => None,
            })
            .collect()
    }
    pub fn to_any_type(self) -> NodePath<AnyNodeType> {
        NodePath::<AnyNodeType>::from_concrete(self)
    }
    pub fn to(mut self, path: &QualifiedPath) -> Option<NodePath<AnyNodeType>> {
        for p in path.iter() {
            self.path.push(self.get_node().get_child(p)?.clone());
        }
        Some(NodePath::<AnyNodeType>::from_path(self.path))
    }
    pub fn to_parent_area(self) -> NodePath<Area> {
        NodePath::<Area>::new(self.path.first().unwrap().clone())
    }
    pub fn get_qualified_path(&self) -> QualifiedPath {
        let mut path = QualifiedPath::new();
        for p in self.path.iter() {
            path.push(p.get_name());
        }
        path
    }
    pub fn get_child_paths_by_branch(&self) -> HashMap<bool, Vec<QualifiedPath>> {
        let has_branch = |node: &Node| -> bool { node.get_metadata().has_branch() };
        let has_no_branch = |node: &Node| -> bool { !node.get_metadata().has_branch() };
        let check = |flag: &bool, node: &Node| -> bool {
            if *flag {
                has_branch(node)
            } else {
                has_no_branch(node)
            }
        };
        self.get_node()
            .get_qualified_paths_by(&QualifiedPath::new(), &check, &vec![true, false])
    }
    pub fn display_tree(&self, show_tags: bool) -> String {
        self.get_node().display_tree(show_tags)
    }
}
