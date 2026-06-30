mod any;
mod area;
mod classification;
mod feature;
mod path;
mod product;
mod virtual_root;

use crate::model::*;
use crate::repository::Repository;
use crate::vcs::{VCS, VCSError, VersionId};
pub use any::*;
pub use area::*;
pub use classification::*;
use colored::Colorize;
pub use feature::*;
use itertools::Itertools;
pub use path::*;
pub use product::*;
use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use thiserror::Error;
pub use virtual_root::*;

#[derive(Error, Clone, Debug)]
#[error("")]
pub struct PathDoesNotExistError<V: VersionId> {
    path: NodePath<V>,
}

impl<V: VersionId> PathDoesNotExistError<V> {
    pub fn new(path: NodePath<V>) -> Self {
        Self { path }
    }
}

#[derive(Error, Clone, Debug)]
#[error("")]
pub struct InvalidTypeError {
    types_possible: Vec<NodeType>,
    type_found: NodeType,
}

impl InvalidTypeError {
    pub fn new(types_possible: Vec<NodeType>, type_found: NodeType) -> Self {
        Self {
            types_possible,
            type_found,
        }
    }
}

#[derive(Error, Clone, Debug)]
pub enum TreeViewError<V: VersionId> {
    #[error(transparent)]
    PathDoesNotExist(#[from] PathDoesNotExistError<V>),
    #[error(transparent)]
    InvalidType(#[from] InvalidTypeError),
}

/// Some node tree_view have the option of being concrete (with attached artifacts) or abstract.
/// This is the base trait for this classification.
pub trait NodeClassification: Clone + Debug + Eq + PartialEq + Hash {
    fn requires_artifact() -> Option<bool>;
}

/// Symbolic node type base trait.
/// This exists for generic type parameters.
pub trait SymbolicNodeType: Clone + Debug + Eq + PartialEq + Hash {
    type Classification: NodeClassification;
    fn new() -> Self;
    fn compatible() -> Vec<NodeType> {
        vec![]
    }
}

impl<V: VersionId> ToNormalizedPath for Vec<Rc<RefCell<Node<V>>>> {
    fn to_normalized_path(&self) -> NormalizedPath {
        let mut path = NormalizedPath::new();
        for p in self.iter() {
            path.push(p.borrow().get_name());
        }
        path
    }
}

/// The outward-facing abstraction of the path model.
///
/// This struct symbolizes a path in the tree model and is the primary interface to interact with the underlying VCS repository.
/// Its capabilities are defined by:
/// - the type of node it points to ([SymbolicNodeType] parameter),
/// - the VCS implementation ([VCS] parameter).
#[derive(Debug)]
pub struct TreeView<'a, S: SymbolicNodeType, V: VCS> {
    path: Vec<Rc<RefCell<Node<V::VersionId>>>>,
    sym_type: S,
    repo: &'a Repository<V>,
}

/// Construction and transformation
impl<'a, S: SymbolicNodeType, V: VCS> TreeView<'a, S, V> {
    pub(crate) fn new(
        path: Vec<Rc<RefCell<Node<V::VersionId>>>>,
        repo: &'a Repository<V>,
    ) -> Result<TreeView<'a, S, V>, TreeViewError<V::VersionId>> {
        let new = Self {
            path,
            repo,
            sym_type: S::new(),
        };
        new.lock_node();
        let new = new
            .check_path_not_existent()?
            .check_sym_type_compatibility()?;
        Ok(new)
    }

    pub fn try_convert_to<To: SymbolicNodeType>(
        self,
    ) -> Result<TreeView<'a, To, V>, InvalidTypeError> {
        let new = TreeView {
            path: self.path,
            repo: self.repo,
            sym_type: To::new(),
        };
        let new = new.check_sym_type_compatibility()?;
        Ok(new)
    }

    pub fn convert_to_any(self) -> TreeView<'a, AnyNode<AnyCls>, V> {
        self.try_convert_to().unwrap()
    }

    fn check_path_not_existent(self) -> Result<Self, PathDoesNotExistError<V::VersionId>> {
        if &self.get_real_type() == &NodeType::NonExistent {
            let path = NodePath::new(self.path, VersionPointer::Default);
            Err(PathDoesNotExistError::new(path))
        } else {
            Ok(self)
        }
    }

    fn check_sym_type_compatibility(self) -> Result<Self, InvalidTypeError> {
        if !S::compatible().contains(&self.get_real_type()) {
            let real_type = self.get_real_type();
            Err(InvalidTypeError::new(S::compatible(), real_type))
        } else {
            Ok(self)
        }
    }

    fn lock_node(&self) {
        if let Err(_) = self.get_node().borrow_mut().try_lock() {
            let path = self.to_normalized_path();
            panic!("Cannot lock path {path}: node is already locked")
        }
    }

    // fn check_version_compatibility(self) -> Result<Self, TreeViewError<V, V::VCSError>> {
    //     match &self.version_pointer {
    //         VersionPointer::Default => Ok(self),
    //         VersionPointer::Version(v) => {
    //             if !&self.get_real_type().accepts_explicit_version() {
    //                 Err(TreeView::<ErrorState, V>::new(self.path, self.vcs, self.version_pointer, NodePathError::VersionNotSupported).into())
    //             }
    //             else if !self.get_vcs().version_exists_on_path(&self.to_normalized_path(), &v)? {
    //                 Err(TreeView::<ErrorState, V>::new(self.path, self.vcs, self.version_pointer, NodePathError::VersionNotOnPath).into())
    //             } else {
    //                 Ok(self)
    //             }
    //         }
    //     }
    // }
}

/// Getters and setters
impl<'a, S: SymbolicNodeType, V: VCS> TreeView<'a, S, V> {
    fn get_repo(&self) -> &'a Repository<V> {
        self.repo
    }

    pub fn get_vcs(&self) -> &V {
        self.get_repo().get_vcs()
    }

    pub fn get_node(&self) -> &Rc<RefCell<Node<V::VersionId>>> {
        self.path.last().unwrap()
    }

    pub fn get_root(&self) -> &Rc<RefCell<Node<V::VersionId>>> {
        self.path.first().unwrap()
    }

    pub fn get_real_type(&self) -> NodeType {
        self.get_node().borrow().get_type().clone()
    }

    pub fn get_sym_type(&self) -> &S {
        &self.sym_type
    }

    pub fn get_child(
        &self,
        name: &str,
    ) -> Result<NodePath<V::VersionId>, PathDoesNotExistError<V::VersionId>> {
        let mut path = self.path.clone();
        if let Some(child) = self.get_node().borrow().get_child(name) {
            path.push(child);
            Ok(NodePath::new(path, VersionPointer::Default))
        } else {
            path.push(Rc::new(RefCell::new(Node::new(
                name.to_string(),
                NodeType::NonExistent,
                None,
            ))));
            Err(PathDoesNotExistError::new(NodePath::new(
                path,
                VersionPointer::Default,
            )))
        }
    }

    pub fn as_versioned_node_path(
        &self,
        version: VersionPointer<V::VersionId>,
    ) -> NodePath<V::VersionId> {
        NodePath::new(self.path.clone(), version)
    }

    pub fn as_node_path(&self) -> NodePath<V::VersionId> {
        self.as_versioned_node_path(VersionPointer::Default)
    }

    pub fn has_children(&self) -> bool {
        self.get_node().borrow().has_children()
    }
}

/// Iterators
impl<'a, S: SymbolicNodeType, V: VCS> TreeView<'a, S, V> {
    pub fn iter_children(&self) -> impl Iterator<Item = NodePath<V::VersionId>> {
        let path = self.as_node_path();
        path.iter_children()
    }

    pub fn iter_children_req(&self) -> impl Iterator<Item = NodePath<V::VersionId>> {
        let path = self.as_node_path();
        path.iter_children_req()
    }
}

/// Path pointer movement
impl<'a, S: SymbolicNodeType, V: VCS> TreeView<'a, S, V> {
    /// Moves path to a specific index of the node vector.
    pub fn move_to_index<To: SymbolicNodeType>(
        self,
        index: usize,
    ) -> Result<TreeView<'a, To, V>, TreeViewError<V::VersionId>> {
        let path = self.path[0..index + 1].to_vec();
        Ok(TreeView::<'a, To, V>::new(path, self.repo)?)
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
    pub fn move_to<To: SymbolicNodeType>(
        self,
        path: &impl ToNormalizedPath,
    ) -> Result<TreeView<'a, To, V>, TreeViewError<V::VersionId>> {
        let path = path.to_normalized_path();
        let normalized_self = self.to_normalized_path();
        let new_path = normalized_self + path.strip_version();
        let mut new_node_vec = vec![self.get_root().clone()];
        for p in new_path.iter_segments(1, new_path.len()) {
            let current = new_node_vec.last().unwrap();
            let node = if let Some(node) = current.borrow().get_child(p) {
                node
            } else {
                Rc::new(RefCell::new(Node::new(
                    p.clone(),
                    NodeType::NonExistent,
                    None,
                )))
            };
            new_node_vec.push(node);
        }
        Ok(TreeView::<'a, To, V>::new(self.path, self.repo)?)
    }
}

/// Display and pretty printing
impl<'a, S: SymbolicNodeType, V: VCS> TreeView<'a, S, V> {
    // pub fn display_tree(&self, show_tags: bool) -> String {
    //     self.get_node().borrow().display_tree(show_tags)
    // }

    pub fn formatted(&self, colored: bool) -> String {
        let path = self.to_normalized_path().strip_version();
        if colored {
            path.to_string().blue().to_string()
        } else {
            path.to_string()
        }
    }
}

impl<'a, T: SymbolicNodeType, V: VCS> ToNormalizedPath for TreeView<'a, T, V> {
    fn to_normalized_path(&self) -> NormalizedPath {
        self.path.to_normalized_path()
    }
}
