mod virtual_root;
mod area;
mod feature;
mod product;
mod any;
mod classification;
mod error;

use crate::model::*;
use crate::vcs::VCS;
pub use any::*;
pub use area::*;
pub use classification::*;
use colored::Colorize;
pub use feature::*;
use itertools::Itertools;
pub use product::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
pub use virtual_root::*;
use crate::model::node_path::error::ErrorState;

/// Some node node_path have the option of being concrete (with attached artifacts) or abstract.
/// This is the base trait for this classification.
pub trait NodeClassification: Clone + Debug + Eq + PartialEq + Hash {
    fn requires_artifact() -> Option<bool>;
}

/// Symbolic node type base trait.
/// This exists for generic type parameters.
pub trait SymbolicNodeType: Clone + Debug + Eq + PartialEq + Hash {}

/// Symbolic node type for valid (non-error) paths.
pub trait ValidNodeType: SymbolicNodeType {
    type Classification: NodeClassification;
    fn new() -> Self { Self {} }
    fn compatible() -> Vec<NodeType> { vec![] }
}

impl<S: ValidNodeType> SymbolicNodeType for S {}

/// The outward-facing abstraction of the path model.
///
/// This struct symbolizes a path in the tree model and is the primary interface to interact with the underlying VCS repository.
/// Its capabilities are defined by:
/// - the type of node it points to ([ValidNodeType] parameter),
/// - the VCS implementation ([VCS] parameter).
#[derive(Clone, Debug)]
pub struct NodePath<S: SymbolicNodeType, V: VCS> {
    path: Vec<Rc<RefCell<Node>>>,
    sym_type: S,
    vcs: Rc<RefCell<V>>,
}

/// Construction and transformation
impl<S: ValidNodeType, V: VCS> NodePath<S, V> {
    fn new_internal() {
        
    }
    
    pub(crate) fn new(
        path: Vec<Rc<RefCell<Node>>>,
        vcs: Rc<RefCell<V>>,
        version: Option<String>,
    ) -> Result<NodePath<S, V>, NodePath<ErrorState, V>> {
        let last = path.last().unwrap();
        match last.borrow().get_type() {
            NodeType::NonExistent => {
                return Err(NodePath::<ErrorState, V>::new())
            }
            _ => {}
        }
        
        let node_type = last.borrow().get_type();
        if !S::compatible().contains(&node_type) {
            return Err(NodePath::new());
        }
        let new = Self {
            path,
            sym_type: S::new(),
            vcs,
        };
        Ok(new)
    }

    pub fn try_convert_to<To: ValidNodeType>(self) -> Result<NodePath<To, V>, NodePath<ErrorState, V>> {
        NodePath::new(self.path, self.vcs, self.get_sym_type())
    }

    pub fn convert_to_any(self) -> NodePath<AnyNode<AnyCls>, V> {
        NodePath::new(self.path, self.vcs).unwrap()
    }
}

/// Getters, setters, and iterators
impl<S: SymbolicNodeType, V: VCS> NodePath<S, V> {
    pub fn get_node(&self) -> &Rc<RefCell<Node>> {
        self.path.last().unwrap()
    }

    pub fn get_root(&self) -> &Rc<RefCell<Node>> { self.path.first().unwrap() }

    pub fn get_vcs(&self) -> &Rc<RefCell<V>> {
        &self.vcs
    }

    pub fn get_real_type(&self) -> NodeType {
        self.get_node().borrow().get_type().clone()
    }
    
    pub fn get_sym_type(&self) -> &S {
        &self.sym_type
    }

    pub fn has_children(&self) -> bool {
        self.get_node().borrow().has_children()
    }

    pub fn iter_children_by_type<I: ValidNodeType>(&self) -> impl Iterator<Item = NodePath<I, V>> {
        self.get_node()
            .borrow()
            .get_children()
            .into_iter()
            .filter_map(|node| {
                match self
                    .clone()
                    .move_to::<I>(&node.borrow().get_name().to_normalized_path()) {
                    Ok(path) => Some(path),
                    Err(_) => None,
                }
            })
            .sorted()
    }

    pub fn iter_children_by_type_req<I: ValidNodeType>(&self) -> impl Iterator<Item = NodePath<I, V>> {
        self.iter_children_by_type::<I>().flat_map(|path| {
            let mut to_iter = Vec::new();
            to_iter.push(path.clone());
            to_iter.extend(path.iter_children_by_type_req());
            to_iter
        })
    }
}

/// Path pointer movement
impl<S: ValidNodeType, V: VCS> NodePath<S, V> {
    /// Moves path to a specific index of the node vector.
    pub fn move_to_index<To: ValidNodeType>(self, index: usize) -> Result<NodePath<To, V>, InvalidNodeTypeError> {
        let path = self.path[0..index + 1].to_vec();
        NodePath::new(path, self.vcs)
    }
    
    /// Move path to another node.
    ///
    /// Relative paths such as `..` are allowed.
    ///
    /// ## Example:
    /// ```
    /// let path = NormalizedPath::from("foo")
    /// let node_path = NodePath::new(...)
    /// node_path.move_to<Feature<Concrete>>(&path);
    /// ```
    pub fn move_to<To: ValidNodeType>(
        self,
        path: &NormalizedPath,
    ) -> Result<NodePath<To, V>, NodePath<ErrorState, V>> {
        let normalized_self = self.to_normalized_path();
        let maybe_version = path.get_version_appendix();
        let new_path = normalized_self + path.strip_version();

        let mut new_node_vec = vec![self.get_root().clone()];
        for p in new_path.iter_segments(1, new_path.len()) {
            let current = new_node_vec.last().unwrap();
            let node = if let Some(node) = current.borrow().get_child(p) {
                node
            } else {
                Rc::new(RefCell::new(Node::new(p.clone(), NodeType::NonExistent)))
            };
            new_node_vec.push(node);
        }
        Ok(NodePath::new(self.path, self.vcs)?)
    }
}

/// Display and pretty printing
impl<S: ValidNodeType, V: VCS> NodePath<S, V> {
    pub fn display_tree(&self, show_tags: bool) -> String {
        self.get_node().borrow().display_tree(show_tags)
    }

    pub fn formatted(&self, colored: bool) -> String {
        let path = self.to_normalized_path().strip_version();
        if colored {
            path.to_string().blue().to_string()
        } else {
            path.to_string()
        }
    }
}

impl<T: ValidNodeType, V: VCS> ToNormalizedPath for NodePath<T, V> {
    fn to_normalized_path(&self) -> NormalizedPath {
        let mut path = NormalizedPath::new();
        for p in self.path.iter() {
            path.push(p.borrow().get_name());
        }
        path
    }
}

impl<T: ValidNodeType, V: VCS> Hash for NodePath<T, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_normalized_path().hash(state);
    }
}

impl<A, B, V> PartialEq<NodePath<A, V>> for NodePath<B, V>
where
    A: ValidNodeType,
    B: ValidNodeType,
    V: VCS,
{
    fn eq(&self, other: &NodePath<A, V>) -> bool {
        self.to_normalized_path() == other.to_normalized_path()
    }
}

impl<T: ValidNodeType, V: VCS> Eq for NodePath<T, V> {}

impl<T: ValidNodeType, V: VCS> Display for NodePath<T, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_normalized_path().to_string().as_str())
    }
}

impl<T: ValidNodeType, V: VCS> PartialOrd for NodePath<T, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.to_normalized_path() == other.to_normalized_path() {
            Some(Ordering::Equal)
        } else if self.to_normalized_path() > other.to_normalized_path() {
            Some(Ordering::Greater)
        } else if self.to_normalized_path() < other.to_normalized_path() {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl<T: ValidNodeType, V: VCS> Ord for NodePath<T, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}