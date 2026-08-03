use crate::model::*;
use crate::vcs::{VCS, VersionId};
use colored::{ColoredString, Colorize};
use itertools::Itertools;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum RevisionPointer<V: VCS> {
    None,
    Head(V::RevisionId),
    Revision(V::RevisionId),
    Invalid(String),
}

#[derive(Clone, Debug)]
pub struct FrozenNode {
    name: String,
    node_type: NodeType,
}

impl FrozenNode {
    pub fn new(name: String, node_type: NodeType) -> Self {
        Self { name, node_type }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_type(&self) -> &NodeType {
        &self.node_type
    }
}

impl ToNormalizedPath for Vec<FrozenNode> {
    fn to_normalized_path(&self) -> NormalizedPath {
        let mut path = NormalizedPath::new();
        for node in self {
            path.push(node.name.clone());
        }
        path
    }
}

/// A static, decoupled view onto the model, meant for long-term storage.
/// Does not react to changes in the source tree.
#[derive(Clone, Debug)]
pub struct FrozenView<V: VCS> {
    path: Vec<FrozenNode>,
    version: RevisionPointer<V>,
}

impl<V: VCS> FrozenView<V> {
    fn get_node(&self) -> &FrozenNode {
        &self.path.last().unwrap()
    }

    pub(crate) fn new(path: Vec<FrozenNode>, version: RevisionPointer<V>) -> Self {
        Self { path, version }
    }

    pub fn formatted(&self, show_type: bool, show_version: bool, colored: bool) -> String {
        let mut path = self.normalize().get_path().to_string().blue().to_string();
        let mut info: Vec<String> = vec![];

        if show_type {
            let node = self.get_node();
            let node_type = node.get_type();
            let type_name = node_type.get_formatted_name();
            info.push(type_name);
        }

        if show_version {
            let version: Option<ColoredString> = match &self.version {
                RevisionPointer::None => None,
                RevisionPointer::Head(head) => {
                    Some(format!("Head -> {}", head.get_printable_id()).yellow())
                }
                RevisionPointer::Revision(rev) => Some(rev.get_printable_id().yellow()),
                RevisionPointer::Invalid(invalid) => Some(invalid.red()),
            };
            if let Some(version) = version {
                info.push(version.to_string())
            }
        }

        let mut output = if info.is_empty() {
            path
        } else {
            format!(
                "{} {}{}{}",
                path,
                "(".yellow(),
                info.join(", ".yellow().to_string().as_str()),
                ")".yellow(),
            )
        };

        if !colored {
            output = output.normal().to_string();
        }
        output
    }
}

impl<V: VCS> Normalize for FrozenView<V> {
    fn normalize(&self) -> Normalized {
        let path = self.path.to_normalized_path();
        let revision = match &self.version {
            RevisionPointer::None | RevisionPointer::Head(_) => NormalizedRevision::Head,
            RevisionPointer::Revision(rev) => NormalizedRevision::Revision(rev.get_full_id()),
            RevisionPointer::Invalid(invalid) => NormalizedRevision::Revision(invalid.clone()),
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
