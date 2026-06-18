use crate::model::*;
use colored::{ColoredString, Colorize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::rc::Rc;
use thiserror::Error;

pub const FEATURE_ROOT: &str = "feature";
pub const PRODUCT_ROOT: &str = "product";
pub const TEMPORARY: &str = "tmp";

#[derive(Error, Debug)]
pub struct MalformedModelError {
    reason: String,
}

impl MalformedModelError {
    pub fn new(reason: impl Into<String>) -> Self {
        Self { reason: reason.into() }
    }
}

impl Display for MalformedModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VirtualRootMetadata {
    repo_scanned: bool
}

impl VirtualRootMetadata {
    pub fn new() -> Self {
        Self { repo_scanned: false }
    }
    
    pub fn repo_scanned(&self) -> bool {
        self.repo_scanned
    }
    
    pub fn set_repo_scanned(&mut self) {
        self.repo_scanned = true;
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeType {
    // Valid types
    VirtualRoot,
    Area(bool),
    FeatureRoot,
    ProductRoot,
    Feature(bool),
    Product(bool),

    // Error types
    Malformed,
    NonExistent,
}

impl NodeType {
    pub fn decide_next_type(&self, name: &str, concrete: bool) -> Result<NodeType, MalformedModelError> {
        match self {
            Self::VirtualRoot => Ok(Self::Area(concrete)),
            Self::Area(_) => match name {
                FEATURE_ROOT => Ok(Self::FeatureRoot),
                PRODUCT_ROOT => Ok(Self::ProductRoot),
                _ => Err(MalformedModelError::new("Expected 'feature' or 'product'")),
            },
            Self::Feature(_) | Self::FeatureRoot => Ok(Self::Feature(concrete)),
            Self::Product(_) | Self::ProductRoot => Ok(Self::Product(concrete)),
            Self::Malformed => Ok(Self::Malformed),
            Self::NonExistent => Ok(Self::NonExistent),
        }
    }

    pub fn accepts_explicit_version(&self) -> bool {
        match self {
            Self::Area(true) |
            Self::Feature(true) |
            Self::Product(true) => true,
            _ => false,
        }
    }

    pub fn format_node_display(&self, name: ColoredString) -> ColoredString {
        match self {
            Self::Area(_) => name.yellow(),
            Self::FeatureRoot => name.bright_purple(),
            Self::Feature(_) => name.purple(),
            Self::ProductRoot => name.truecolor(231, 100, 18),
            Self::Product(_) => name.truecolor(231, 100, 18),
            _ => name.red(),
        }
    }

    pub fn get_type_name(&self) -> String {
        let name: &str = match self {
            Self::VirtualRoot => "virtual root",
            Self::Area(_) => "area",
            Self::FeatureRoot => "feature root",
            Self::ProductRoot => "product root",
            Self::Feature(_) => "feature",
            Self::Product(_) => "product",
            _ => "bad"
        };
        name.to_string()
    }

    pub fn get_short_type_name(&self) -> String {
        let name: &str = match self {
            Self::VirtualRoot => "vr",
            Self::Area(_) => "a",
            Self::FeatureRoot => "fr",
            Self::ProductRoot => "pr",
            Self::Feature(_) => "f",
            Self::Product(_) => "p",
            _ => "b"
        };
        name.to_string()
    }

    pub fn get_formatted_name(&self) -> String {
        self.format_node_display(self.get_type_name().normal())
            .to_string()
    }

    pub fn get_formatted_short_name(&self) -> String {
        self.format_node_display(self.get_short_type_name().normal())
            .to_string()
    }
}

#[derive(Debug)]
pub struct Node {
    name: String,
    node_type: NodeType,
    children: HashMap<String, Rc<RefCell<Node>>>,
}

impl Node {
    pub fn new (
        name: String,
        node_type: NodeType,
    ) -> Self {
        Self {
            name,
            node_type,
            children: HashMap::new(),
        }
    }

    pub(crate) fn update_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
    }

    // fn build_display_tree(&self, show_tags: bool) -> Tree<String> {
    //     let mut formatted = ColoredString::from(self.name.clone());
    //     if self.branch.has_branch() {
    //         formatted = formatted.blue()
    //     }
    //     let type_display = match self.node_type {
    //         NodeType::AbstractFeature | NodeType::AbstractProduct => None,
    //         _ => Some(self.node_type.get_formatted_short_name()),
    //     };
    //     let content = if let Some(type_display) = type_display {
    //         format!("{formatted} [{type_display}]")
    //     } else {
    //         formatted.to_string()
    //     };
    //     let mut tree = Tree::<String>::new(content);
    //     let children = self.children.borrow();
    //     let mut sorted_children = children.iter().collect::<Vec<_>>();
    //     sorted_children.sort_by(|a, b| b.0.chars().cmp(a.0.chars()));
    //     sorted_children.reverse();
    //     for (_, child) in sorted_children {
    //         tree.leaves
    //             .push(child.borrow().build_display_tree(show_tags));
    //     }
    //     tree
    // }

    fn decide_child_type(&self, name: &str, concrete: bool) -> Result<NodeType, MalformedModelError> {
        self.node_type.decide_next_type(name, concrete)
    }

    fn add_child(&mut self, name: String, concrete: bool) -> Result<NodeType, MalformedModelError> {
        let node_type = self.decide_child_type(name.as_str(), concrete)?;
        // let child = ;
        let child = Node::new(
            name.clone(),
            node_type.clone(),
        );
        self.children.insert(name, Rc::new(RefCell::new(child)));
        Ok(node_type)
    }

    fn update_child(&self, name: String, concrete: bool) -> Result<NodeType, MalformedModelError> {
        let new_type = self.decide_child_type(name.as_str(), concrete)?;
        let child = self.get_child(&name).unwrap();
        child.borrow_mut().update_type(new_type.clone());
        Ok(new_type)
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_type(&self) -> &NodeType {
        &self.node_type
    }

    pub fn get_child<S: Into<String>>(&self, name: S) -> Option<Rc<RefCell<Node>>> {
        Some(self.children.get(&name.into())?.clone())
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn get_children(&self) -> Vec<Rc<RefCell<Node>>> {
        self.children.values().cloned().collect()
    }

    pub fn insert_path(&mut self, path: &NormalizedPath, concrete: bool) -> Result<NodeType, MalformedModelError> {
        match path.len() {
            0 => Ok(self.node_type.clone()),
            1 => {
                let name = path.get(0).unwrap().to_string();
                let new_type = match self.get_child(&name) {
                    Some(_) => self.update_child(name, concrete),
                    None => self.add_child(name, concrete),
                };
                new_type
            }
            _ => {
                let name = path.get(0).unwrap().to_string();
                let next_child = match self.get_child(&name) {
                    Some(node) => node,
                    None => {
                        self.add_child(name.clone(), false)?;
                        self.get_child(&name).unwrap()
                    }
                };
                next_child
                    .borrow_mut()
                    .insert_path(&path.strip_n_left(1), concrete)
            }
        }
    }

    // pub fn display_tree(&self, show_tags: bool) -> String {
    //     self.build_display_tree(show_tags).to_string()
    // }
}
