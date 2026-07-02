use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use colored::Colorize;
use itertools::Itertools;
use crate::model::{Node, NodeHolder, NormalizedPath, ToNormalizedPath, VersionPointer};
use crate::vcs::VersionId;

/// A static view onto the model.
#[derive(Clone, Debug)]
pub struct FuzzyView<V: VersionId> {
    path: Vec<Rc<RefCell<Node<V>>>>,
    version: VersionPointer<V>,
}

impl<V: VersionId> FuzzyView<V> {
    pub fn new(path: Vec<Rc<RefCell<Node<V>>>>, version: VersionPointer<V>) -> Self {
        Self { path, version }
    }

    pub fn iter_children(self) -> impl Iterator<Item = FuzzyView<V>> {
        self.get_node()
            .borrow()
            .get_children()
            .into_iter()
            .map(|node| {
                let mut path = self.path.clone();
                path.push(node);
                FuzzyView::new(path, VersionPointer::Default)
            })
            .sorted()
    }

    pub fn iter_children_req(self) -> impl Iterator<Item = FuzzyView<V>> {
        self.iter_children().flat_map(|path| {
            let mut to_iter = Vec::new();
            to_iter.push(path.clone());
            to_iter.extend(path.iter_children_req());
            to_iter
        })
    }

    pub fn formatted(
        &self,
        show_type: bool,
        show_version: bool,
        colored: bool,
    ) -> String {
        let mut path = self
            .to_normalized_path()
            .strip_version()
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
                    VersionPointer::Default => {
                        let head = branch_info.get_head();
                        format!("({})", head.get_printable_id()).yellow()
                    },
                    VersionPointer::Version(version) => {
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

impl<V: VersionId> NodeHolder<V> for FuzzyView<V> {
    fn get_node(&self) -> &Rc<RefCell<Node<V>>> {
        &self.path.last().unwrap()
    }
}

impl<V: VersionId> ToNormalizedPath for FuzzyView<V> {
    fn to_normalized_path(&self) -> NormalizedPath {
        let mut path = self.path.to_normalized_path();
        match &self.version {
            VersionPointer::Default => {},
            VersionPointer::Version(version) => {
                path.set_version_appendix(version.get_full_id())
            }
        }
        path
    }
}

impl<V: VersionId> Eq for FuzzyView<V> {}

impl<V: VersionId> Hash for FuzzyView<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_normalized_path().hash(state);
    }
}

impl<V: VersionId> PartialEq for FuzzyView<V> {
    fn eq(&self, other: &FuzzyView<V>) -> bool {
        self.to_normalized_path() == other.to_normalized_path()
    }
}

impl<V: VersionId> Display for FuzzyView<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_normalized_path().to_string().as_str())
    }
}

impl<V: VersionId> PartialOrd for FuzzyView<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_normalized_path()
            .partial_cmp(&other.to_normalized_path())
    }
}

impl<V: VersionId> Ord for FuzzyView<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}