use crate::vcs::{RevisionId, VCS};
use colored::{ColoredString, Colorize};
use indextree::NodeId;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
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
    Channel(bool),
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
            Self::VirtualRoot => Ok(Self::Channel(concrete)),
            Self::Channel(_) => match name {
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
            Self::Channel(true) | Self::Feature(true) | Self::Product(true) => true,
            _ => false,
        }
    }

    pub fn format_node_display(&self, name: ColoredString) -> ColoredString {
        match self {
            Self::Channel(_) => name.yellow(),
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
            Self::Channel(_) => "channel",
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
            Self::Channel(_) => "c",
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
    head: V::RevisionId,
}

impl<V: VCS> BranchInfo<V> {
    pub fn new(id: usize, head: V::RevisionId) -> Self {
        Self { id, head }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_head(&self) -> &V::RevisionId {
        &self.head
    }
}

#[derive(Debug)]
pub struct NodeData<V: VCS> {
    name: String,
    node_type: NodeType,
    branch_info: Option<BranchInfo<V>>,
    children: HashMap<String, NodeId>,
    structure_lock: bool,
    revision_lock: bool,
    structure_views_referenced: usize,
    revision_views_referenced: usize,
}

impl<V: VCS> NodeData<V> {
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
            structure_lock: false,
            revision_lock: false,
            structure_views_referenced: 0,
            revision_views_referenced: 0,
        }
    }

    pub(crate) fn update_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub(crate) fn update_type(&mut self, node_type: NodeType) {
        self.node_type = node_type;
    }

    pub(crate) fn update_branch_info(&mut self, branch_info: Option<BranchInfo<V>>) {
        self.branch_info = branch_info;
    }

    pub(crate) fn add_child(&mut self, id: NodeId, name: impl Into<String>) {
        self.children.insert(name.into(), id);
    }

    pub(crate) fn remove_child(&mut self, name: &str) {
        self.children.remove(name);
    }

    pub(crate) fn is_structure_locked(&self) -> bool {
        self.structure_lock
    }

    pub(crate) fn lock_structure(&mut self) {
        self.structure_lock = true;
    }

    pub(crate) fn unlock_structure(&mut self) {
        self.structure_lock = false;
    }

    pub(crate) fn is_revision_locked(&self) -> bool {
        self.revision_lock
    }

    pub(crate) fn lock_revision(&mut self) {
        self.revision_lock = true;
    }

    pub(crate) fn unlock_revision(&mut self) {
        self.revision_lock = false;
    }

    pub(crate) fn structure_views_referenced(&self) -> usize {
        self.structure_views_referenced
    }

    pub(crate) fn revision_views_referenced(&self) -> usize {
        self.revision_views_referenced
    }

    pub(crate) fn reference_structure_view(&mut self) {
        self.structure_views_referenced += 1
    }

    pub(crate) fn reference_revision_view(&mut self) {
        self.revision_views_referenced += 1
    }

    pub(crate) fn dereference_structure_view(&mut self) {
        self.structure_views_referenced -= 1
    }

    pub(crate) fn dereference_revision_view(&mut self) {
        self.revision_views_referenced -= 1
    }

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

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub(crate) fn get_child(&self, name: &str) -> Option<&NodeId> {
        self.children.get(name)
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
}
