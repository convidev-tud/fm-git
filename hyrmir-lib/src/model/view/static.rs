use crate::model::*;
use crate::vcs::VersionId;
use colored::Colorize;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// A static view onto the model.
#[derive(Clone, Debug)]
pub struct StaticView<V: VersionId> {
    path: Vec<Rc<RefCell<Node<V>>>>,
    version: RevisionPointer<V>,
}

impl<V: VersionId> StaticView<V> {
    pub fn new(path: Vec<Rc<RefCell<Node<V>>>>, version: RevisionPointer<V>) -> Self {
        Self { path, version }
    }

    pub fn formatted(
        &self,
        show_type: bool,
        show_version: bool,
        colored: bool,
    ) -> String {
        let mut path = self
            .normalize()
            .get_path()
            .to_string()
            .blue()
            .to_string();
        if show_type {
            let node = self.get_node().borrow();
            let node_type = node.get_type();
            let type_name = node_type.get_type_name();
            let formatted = node_type
                .format_node_display(format!("({type_name})").normal());
            path = path + " " + formatted.to_string().as_str();
        }
        if show_version {
            if let Some(branch_info) = self.get_node().borrow().get_branch_info() {
                let version = match &self.version {
                    RevisionPointer::Head => {
                        let head = branch_info.get_head();
                        format!("({})", head.get_printable_id()).yellow()
                    },
                    RevisionPointer::Revision(version) => {
                        if branch_info.contains_version(version) {
                            format!("({})", version.get_printable_id()).yellow()
                        } else {
                            format!("(Invalid: {})", version.get_printable_id()).red()
                        }
                    },
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

impl<V: VersionId> NodeHolder<V> for StaticView<V> {
    fn get_node(&self) -> &Rc<RefCell<Node<V>>> {
        &self.path.last().unwrap()
    }
}

impl<V: VersionId> Normalize for StaticView<V> {
    fn normalize(&self) -> Normalized {
        let path = self.path.to_normalized_path();
        let revision = match &self.version {
            RevisionPointer::Head => NormalizedRevision::None,
            RevisionPointer::Revision(version) => NormalizedRevision::Revision(version.get_full_id()),
        };
        Normalized::new(path, revision)
    }
}

impl<V: VersionId> Eq for StaticView<V> {}

impl<V: VersionId> Hash for StaticView<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.normalize().hash(state);
    }
}

impl<V: VersionId> PartialEq for StaticView<V> {
    fn eq(&self, other: &StaticView<V>) -> bool {
        self.normalize() == other.normalize()
    }
}

impl<V: VersionId> Display for StaticView<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.normalize().to_string().as_str())
    }
}

impl<V: VersionId> PartialOrd for StaticView<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.normalize()
            .partial_cmp(&other.normalize())
    }
}

impl<V: VersionId> Ord for StaticView<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}