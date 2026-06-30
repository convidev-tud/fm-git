use crate::model::{Node, NormalizedPath, SymbolicNodeType, ToNormalizedPath};
use crate::vcs::VersionId;
use itertools::Itertools;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum VersionPointer<V: VersionId> {
    Default,
    Version(V),
}

#[derive(Clone, Debug)]
pub struct NodePath<V: VersionId> {
    path: Vec<Rc<RefCell<Node<V>>>>,
    version: VersionPointer<V>,
}

impl<V: VersionId> NodePath<V> {
    pub fn new(path: Vec<Rc<RefCell<Node<V>>>>, version: VersionPointer<V>) -> Self {
        Self { path, version }
    }

    pub fn get_node(&self) -> &Rc<RefCell<Node<V>>> {
        &self.path.last().unwrap()
    }

    pub fn iter_children(self) -> impl Iterator<Item = NodePath<V>> {
        self.get_node()
            .borrow()
            .get_children()
            .into_iter()
            .map(|node| {
                let mut path = self.path.clone();
                path.push(node);
                NodePath::new(path, VersionPointer::Default)
            })
            .sorted()
    }

    pub fn iter_children_req(self) -> impl Iterator<Item = NodePath<V>> {
        self.iter_children().flat_map(|path| {
            let mut to_iter = Vec::new();
            to_iter.push(path.clone());
            to_iter.extend(path.iter_children_req());
            to_iter
        })
    }
}

impl<V: VersionId> ToNormalizedPath for NodePath<V> {
    fn to_normalized_path(&self) -> NormalizedPath {
        self.path.to_normalized_path()
    }
}

impl<V: VersionId> Eq for NodePath<V> {}

impl<V: VersionId> Hash for NodePath<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_normalized_path().hash(state);
    }
}

impl<V: VersionId> PartialEq for NodePath<V> {
    fn eq(&self, other: &NodePath<V>) -> bool {
        self.to_normalized_path() == other.to_normalized_path()
    }
}

impl<V: VersionId> Display for NodePath<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_normalized_path().to_string().as_str())
    }
}

impl<V: VersionId> PartialOrd for NodePath<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_normalized_path()
            .partial_cmp(&other.to_normalized_path())
    }
}

impl<V: VersionId> Ord for NodePath<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}
