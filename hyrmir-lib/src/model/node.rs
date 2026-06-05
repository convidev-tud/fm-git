use crate::model::*;
use colored::{ColoredString, Colorize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use termtree::Tree;
use thiserror::Error;

pub const FEATURE_ROOT: &str = "feature";
pub const PRODUCT_ROOT: &str = "product";
pub const TEMPORARY: &str = "tmp";

#[derive(Error, Debug)]
pub struct WrongNodeTypeError {
    types_expected: Vec<NodeType>,
    type_found: NodeType,
}

impl WrongNodeTypeError {
    pub fn new(types_expected: Vec<NodeType>, type_found: NodeType) -> Self {
        Self {
            types_expected,
            type_found,
        }
    }
}

impl Display for WrongNodeTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeType {
    VirtualRoot,
    Area(Option<String>),
    FeatureRoot,
    ProductRoot,
    Feature(Option<String>),
    Product(Option<String>),
    Temporary(Option<String>),
    Undefined,
}

impl NodeType {
    pub fn decide_next_type(&self, name: &str, branch: Option<String>) -> Result<NodeType, WrongNodeTypeError> {
        match self {
            Self::VirtualRoot => Ok(Self::Area(branch)),
            Self::Area(_) => match name {
                FEATURE_ROOT => Ok(Self::FeatureRoot),
                PRODUCT_ROOT => Ok(Self::ProductRoot),
                TEMPORARY => Ok(Self::Temporary(branch)),
                _ => Err(WrongNodeTypeError::new()),
            },
            Self::Feature(_) | Self::FeatureRoot => Ok(Self::Feature(branch)),
            Self::Product(_) | Self::ProductRoot => Ok(Self::Product(branch)),
            Self::Temporary(_) => Ok(Self::Temporary(branch)),
            Self::Undefined => Ok(Self::Undefined),
        }
    }

    pub fn format_node_display(&self, name: ColoredString) -> ColoredString {
        match self {
            Self::Area(_) => name.yellow(),
            Self::FeatureRoot => name.bright_purple(),
            Self::Feature(_) => name.purple(),
            Self::ProductRoot => name.truecolor(231, 100, 18),
            Self::Product(_) => name.truecolor(231, 100, 18),
            _ => name,
        }
    }

    pub fn get_type_name(&self) -> String {
        let name: &str = match self {
            Self::VirtualRoot => "virtual root",
            Self::Area => "area",
            Self::FeatureRoot => "feature root",
            Self::ProductRoot => "product root",
            Self::Feature => "feature",
            Self::Product => "product",
            Self::Temporary => "temporary",
            Self::Undefined => "",
        };
        name.to_string()
    }

    pub fn get_short_type_name(&self) -> String {
        let name: &str = match self {
            Self::VirtualRoot => "vr",
            Self::Area => "a",
            Self::FeatureRoot => "fr",
            Self::ProductRoot => "pr",
            Self::Feature => "f",
            Self::Product => "p",
            Self::Temporary => "temp",
            Self::Undefined => "",
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

    fn update_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
    }

    fn build_display_tree(&self, show_tags: bool) -> Tree<String> {
        let mut formatted = ColoredString::from(self.name.clone());
        if self.branch.has_branch() {
            formatted = formatted.blue()
        }
        let type_display = match self.node_type {
            NodeType::AbstractFeature | NodeType::AbstractProduct => None,
            _ => Some(self.node_type.get_formatted_short_name()),
        };
        let content = if let Some(type_display) = type_display {
            format!("{formatted} [{type_display}]")
        } else {
            formatted.to_string()
        };
        let mut tree = Tree::<String>::new(content);
        let children = self.children.borrow();
        let mut sorted_children = children.iter().collect::<Vec<_>>();
        sorted_children.sort_by(|a, b| b.0.chars().cmp(a.0.chars()));
        sorted_children.reverse();
        for (_, child) in sorted_children {
            tree.leaves
                .push(child.borrow().build_display_tree(show_tags));
        }
        tree
    }

    fn decide_child_type(&self, name: &str, object: Option<String>) -> Result<NodeType, WrongNodeTypeError> {
        self.node_type.decide_next_type(name, object)
    }

    fn add_child(&mut self, name: String, object: Option<String>) -> Result<NodeType, WrongNodeTypeError> {
        let node_type = self.decide_child_type(name.as_str(), object)?;
        // let child = ;
        let child = Node::new(
            name.clone(),
            node_type.clone(),
        );
        self.children.borrow_mut().insert(name, Rc::new(RefCell::new(child)));
        Ok(node_type)
    }

    fn update_child(&self, name: String, object: Option<String>) -> Result<NodeType, WrongNodeTypeError> {
        let new_type = self.decide_child_type(name.as_str(), object)?;
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
        Some(self.children.borrow().get(&name.into())?.clone())
    }

    pub fn has_children(&self) -> bool {
        !self.children.borrow().is_empty()
    }

    pub fn get_children(&self) -> Vec<Rc<RefCell<Node>>> {
        let nodes = self.children.borrow();
        nodes.values().cloned().collect()
    }

    pub fn insert_path(&mut self, path: &NormalizedPath, object: Option<String>) -> Result<NodeType, WrongNodeTypeError> {
        match path.len() {
            0 => Ok(self.node_type.clone()),
            1 => {
                let name = path.get(0).unwrap().to_string();
                let new_type = match self.get_child(&name) {
                    Some(_) => self.update_child(name, object),
                    None => self.add_child(name, object),
                };
                new_type
            }
            _ => {
                let name = path.get(0).unwrap().to_string();
                let next_child = match self.get_child(&name) {
                    Some(node) => node,
                    None => {
                        self.add_child(name.clone(), None)?;
                        self.get_child(&name).unwrap()
                    }
                };
                next_child
                    .borrow_mut()
                    .insert_path(&path.strip_n_left(1), object)
            }
        }
    }

    pub fn display_tree(&self, show_tags: bool) -> String {
        self.build_display_tree(show_tags).to_string()
    }
}
