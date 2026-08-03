use crate::model::*;
use crate::vcs::{VersionId, VCS};
use colored::Colorize;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum RevisionPointer<V: VCS> {
    None,
    Head(V::VersionId),
    Revision(V::VersionId),
    Invalid(String),
}

#[derive(Clone, Debug)]
pub struct StaticNode {
    name: String,
    node_type: NodeType,
}

impl StaticNode {
    pub fn new(name: String, node_type: NodeType) -> Self {
        Self { name, node_type }
    }
    
    pub fn name(&self) -> &String {
        &self.name
    }
}

impl ToNormalizedPath for Vec<StaticNode> {
    fn to_normalized_path(&self) -> NormalizedPath {
        let mut path = NormalizedPath::new();
        for node in self {
            path.push(node.name.clone());
        };
        path
    }
}

/// A static, decoupled view onto the model, meant for long-term storage.
/// Does not react to changes in the source tree.
#[derive(Clone, Debug)]
pub struct FrozenView<V: VCS> {
    path: Vec<StaticNode>,
    version: RevisionPointer<V>,
}

impl<V: VCS> FrozenView<V> {
    pub(crate) fn new(
        path: Vec<StaticNode>,
        version: RevisionPointer<V>,
    ) -> Self {
        Self { path, version }
    }

    pub fn formatted(&self, show_type: bool, show_version: bool, colored: bool) -> String {
        let mut path = self.normalize().get_path().to_string().blue().to_string();
        if show_type {
            let node = self.get_node().borrow();
            let node_type = node.get_type();
            let type_name = node_type.get_type_name();
            let formatted = node_type.format_node_display(format!("({type_name})").normal());
            path = path + " " + formatted.to_string().as_str();
        }
        if show_version {
            if let Some(branch_info) = self.get_node().borrow().get_branch_info() {
                let version = match &self.version {
                    RevisionPointer::None => {
                        let head = branch_info.get_head();
                        format!("({})", head.get_printable_id()).yellow()
                    }
                    RevisionPointer::Revision(version) => {
                        if branch_info.contains_version(version) {
                            format!("({})", version.get_printable_id()).yellow()
                        } else {
                            format!("(Invalid: {})", version.get_printable_id()).red()
                        }
                    }
                };
                path = path + " " + version.to_string().as_str();
            }
        }
        if !colored {
            path = path.normal().to_string();
        }
        path
    }
}

impl<V: VCS> Normalize for FrozenView<V> {
    fn normalize(&self) -> Normalized {
        let path = self.path.to_normalized_path();
        let revision = match &self.version {
            RevisionPointer::None => NormalizedRevision::Head,
            RevisionPointer::Revision(version) => {
                NormalizedRevision::Revision(version.get_full_id())
            }
        };
        Normalized::new(path, revision)
    }
}

impl<V: VCS> Eq for FrozenView<V> {}

impl<V: VCS> Hash for FrozenView<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalize().hash(state);
    }
}

impl<V: VCS> PartialEq for FrozenView<V> {
    fn eq(&self, other: &FrozenView<V>) -> bool {
        self.normalize() == other.normalize()
    }
}

impl<V: VCS> Display for FrozenView<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.normalize().to_string().as_str())
    }
}

impl<V: VCS> PartialOrd for FrozenView<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.normalize().partial_cmp(&other.normalize())
    }
}

impl<V: VCS> Ord for FrozenView<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}
