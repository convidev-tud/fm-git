use crate::model::*;
use crate::vcs::{VersionId, VCS};
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
        Self {
            reason: reason.into(),
        }
    }
}

impl Display for MalformedModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.reason.as_str())
    }
}

#[derive(Error, Debug)]
#[error("Node already locked")]
pub struct NodeLockError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VirtualRootMetadata {
    repo_scanned: bool,
}

impl VirtualRootMetadata {
    pub fn new() -> Self {
        Self {
            repo_scanned: false,
        }
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
    pub fn decide_next_type(
        &self,
        name: &str,
        concrete: bool,
    ) -> Result<NodeType, MalformedModelError> {
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
            Self::Area(true) | Self::Feature(true) | Self::Product(true) => true,
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
            _ => "bad",
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
            _ => "b",
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
pub struct BranchInfo<V: VCS> {
    id: usize,
    head: V::VersionId,
    known_versions: HashMap<String, V::VersionId>,
}

impl<V: VCS> BranchInfo<V> {
    pub fn new(id: usize, head: V::VersionId) -> Self {
        Self {
            id,
            head,
            known_versions: HashMap::new(),
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_head(&self) -> &V::VersionId {
        &self.head
    }
    
    pub fn get_known_version(&self, id: impl Into<String>) -> Option<&V::VersionId> {
        let id = id.into();
        if id == "HEAD" {
            Some(&self.head)
        } else {
            self.known_versions.get(&id)
        }
    }

    pub fn add_known_version(&mut self, version: V::VersionId) {
        self.known_versions.insert(version.get_full_id(), version);
    }

    pub fn remove_known_version(&mut self, version: &V::VersionId) {
        self.known_versions.remove(&version.get_full_id());
    }

    pub fn contains_version(&self, version: &V::VersionId) -> bool {
        self.known_versions.contains_key(&version.get_full_id())
    }
}

#[derive(Debug)]
pub struct Node<V: VCS> {
    name: String,
    node_type: NodeType,
    branch_info: Option<BranchInfo<V>>,
    children: HashMap<String, Rc<RefCell<Node<V>>>>,
    lock: bool,
}

impl<V: VCS> Node<V> {
    pub fn new(
        name: impl Into<String>,
        node_type: NodeType,
        branch_info: Option<BranchInfo<V>>,
    ) -> Self {
        Self {
            name: name.into(),
            node_type,
            branch_info,
            children: HashMap::new(),
            lock: false,
        }
    }

    pub(crate) fn update_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
    }

    pub(crate) fn update_branch_info(&mut self, branch_info: Option<BranchInfo<V>>) {
        self.branch_info = branch_info;
    }

    pub(crate) fn try_lock(&mut self) -> Result<(), NodeLockError> {
        if !self.lock {
            self.lock = true;
            Ok(())
        } else {
            Err(NodeLockError)
        }
    }

    pub(crate) fn unlock(&mut self) {
        self.lock = false;
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

    fn decide_child_type(
        &self,
        name: &str,
        branch_info: &Option<BranchInfo<V>>,
    ) -> Result<NodeType, MalformedModelError> {
        let concrete = match branch_info {
            Some(_) => true,
            None => false,
        };
        self.node_type.decide_next_type(name, concrete)
    }

    fn add_child(
        &mut self,
        name: String,
        branch_info: Option<BranchInfo<V>>,
    ) -> Result<NodeType, MalformedModelError> {
        let node_type = self.decide_child_type(name.as_str(), &branch_info)?;
        // let child = ;
        let child = Node::new(name.clone(), node_type.clone(), branch_info);
        self.children.insert(name, Rc::new(RefCell::new(child)));
        Ok(node_type)
    }

    fn update_child(
        &self,
        name: String,
        branch_info: Option<BranchInfo<V>>,
    ) -> Result<NodeType, MalformedModelError> {
        let new_type = self.decide_child_type(name.as_str(), &branch_info)?;
        let child = self.get_child(&name).unwrap();
        let mut child = child.borrow_mut();
        child.update_type(new_type.clone());
        child.update_branch_info(branch_info);
        Ok(new_type)
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_branch_info(&self) -> Option<&BranchInfo<V>> {
        self.branch_info.as_ref()
    }

    pub fn mut_get_branch_info(&mut self) -> Option<&mut BranchInfo<V>> {
        self.branch_info.as_mut()
    }

    pub fn get_type(&self) -> &NodeType {
        &self.node_type
    }

    pub fn get_child(&self, name: &str) -> Option<Rc<RefCell<Node<V>>>> {
        Some(self.children.get(name)?.clone())
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn get_children(&self) -> Vec<Rc<RefCell<Node<V>>>> {
        self.children.values().cloned().collect()
    }

    pub fn insert_path(
        &mut self,
        path: &NormalizedPath,
        branch_info: Option<BranchInfo<V>>,
    ) -> Result<NodeType, MalformedModelError> {
        match path.len() {
            0 => Ok(self.node_type.clone()),
            1 => {
                let name = path.get(0).unwrap().to_string();
                let new_type = match self.get_child(&name) {
                    Some(_) => self.update_child(name, branch_info),
                    None => self.add_child(name, branch_info),
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
                    .insert_path(&path.strip_n_left(1), branch_info)
            }
        }
    }

    // pub fn display_tree(&self, show_tags: bool) -> String {
    //     self.build_display_tree(show_tags).to_string()
    // }
}
