mod any;
mod area;
mod classification;
mod feature;
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
pub use product::*;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use thiserror::Error;
pub use virtual_root::*;

#[derive(Error, Clone, Debug)]
pub struct PathDoesNotExistError<V: VersionId> {
    path: StaticView<V>,
}

impl<V: VersionId> PathDoesNotExistError<V> {
    pub fn new(path: StaticView<V>) -> Self {
        Self { path }
    }
}

impl<V: VersionId> Display for PathDoesNotExistError<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("Path '{}' does not exist", self.path).as_str())
    }
}

#[derive(Error, Clone, Debug)]
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

impl Display for InvalidTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{} Path has invalid type", "Error:".red()).as_str())
    }
}

#[derive(Error, Clone, Debug)]
pub enum TreeViewError<V: VersionId> {
    #[error(transparent)]
    PathDoesNotExist(#[from] PathDoesNotExistError<V>),
    #[error(transparent)]
    InvalidType(#[from] InvalidTypeError),
}

/// Some paths have the option of being concrete (with attached artifacts) or abstract.
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

pub trait NodeHolder<V: VersionId> {
    fn get_node(&self) -> &Rc<RefCell<Node<V>>>;

    fn get_real_type(&self) -> NodeType {
        self.get_node().borrow().get_type().clone()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum VersionPointer<V: VersionId> {
    Default,
    Version(V),
}

/// A static view onto the model.
#[derive(Clone, Debug)]
pub struct StaticView<V: VersionId> {
    path: Vec<Rc<RefCell<Node<V>>>>,
    version: VersionPointer<V>,
}

impl<V: VersionId> StaticView<V> {
    pub fn new(path: Vec<Rc<RefCell<Node<V>>>>, version: VersionPointer<V>) -> Self {
        Self { path, version }
    }

    pub fn iter_children(self) -> impl Iterator<Item = StaticView<V>> {
        self.get_node()
            .borrow()
            .get_children()
            .into_iter()
            .map(|node| {
                let mut path = self.path.clone();
                path.push(node);
                StaticView::new(path, VersionPointer::Default)
            })
            .sorted()
    }

    pub fn iter_children_req(self) -> impl Iterator<Item = StaticView<V>> {
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

impl<V: VersionId> NodeHolder<V> for StaticView<V> {
    fn get_node(&self) -> &Rc<RefCell<Node<V>>> {
        &self.path.last().unwrap()
    }
}

impl<V: VersionId> ToNormalizedPath for StaticView<V> {
    fn to_normalized_path(&self) -> NormalizedPath {
        self.path.to_normalized_path()
    }
}

impl<V: VersionId> Eq for StaticView<V> {}

impl<V: VersionId> Hash for StaticView<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_normalized_path().hash(state);
    }
}

impl<V: VersionId> PartialEq for StaticView<V> {
    fn eq(&self, other: &StaticView<V>) -> bool {
        self.to_normalized_path() == other.to_normalized_path()
    }
}

impl<V: VersionId> Display for StaticView<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_normalized_path().to_string().as_str())
    }
}

impl<V: VersionId> PartialOrd for StaticView<V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_normalized_path()
            .partial_cmp(&other.to_normalized_path())
    }
}

impl<V: VersionId> Ord for StaticView<V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

/// Semantic view onto the path model.
///
/// This struct symbolizes a path in the tree model and is the primary interface to interact with the underlying VCS repository.
/// Its capabilities are defined by:
/// - the type of node it points to ([SymbolicNodeType] parameter),
/// - the VCS implementation ([VCS] parameter).
#[derive(Debug)]
pub struct SemanticView<'a, S: SymbolicNodeType, V: VCS> {
    path: Vec<Rc<RefCell<Node<V::VersionId>>>>,
    sym_type: S,
    repo: &'a Repository<V>,
}

/// Construction and transformation
impl<'a, S: SymbolicNodeType, V: VCS> SemanticView<'a, S, V> {
    pub(crate) fn new(
        path: Vec<Rc<RefCell<Node<V::VersionId>>>>,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, S, V>, TreeViewError<V::VersionId>> {
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
    ) -> Result<SemanticView<'a, To, V>, InvalidTypeError> {
        let new = SemanticView {
            path: self.path.clone(),
            repo: self.repo,
            sym_type: To::new(),
        };
        new.lock_node();
        let new = new.check_sym_type_compatibility()?;
        Ok(new)
    }

    pub fn convert_to_any(self) -> SemanticView<'a, AnyType<AnyCls>, V> {
        self.try_convert_to().unwrap()
    }

    fn check_path_not_existent(self) -> Result<Self, PathDoesNotExistError<V::VersionId>> {
        if &self.get_real_type() == &NodeType::NonExistent {
            let path = StaticView::new(self.path.clone(), VersionPointer::Default);
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
        let mut node = self.get_node().borrow_mut();
        let lock = node.try_lock();
        drop(node);
        if let Err(_) = lock {
            let path = self.to_normalized_path();
            panic!("Cannot lock path '{path}': a semantic view for this path already exists")
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
impl<'a, S: SymbolicNodeType, V: VCS> SemanticView<'a, S, V> {
    fn get_repo(&self) -> &'a Repository<V> {
        self.repo
    }

    pub fn get_vcs(&self) -> &V {
        self.get_repo().get_vcs()
    }

    pub fn get_root(&self) -> &Rc<RefCell<Node<V::VersionId>>> {
        self.path.first().unwrap()
    }

    pub fn get_sym_type(&self) -> &S {
        &self.sym_type
    }

    pub fn get_child(
        &self,
        name: &str,
    ) -> Result<StaticView<V::VersionId>, PathDoesNotExistError<V::VersionId>> {
        let mut path = self.path.clone();
        if let Some(child) = self.get_node().borrow().get_child(name) {
            path.push(child);
            Ok(StaticView::new(path, VersionPointer::Default))
        } else {
            path.push(Rc::new(RefCell::new(Node::new(
                name.to_string(),
                NodeType::NonExistent,
                None,
            ))));
            Err(PathDoesNotExistError::new(StaticView::new(
                path,
                VersionPointer::Default,
            )))
        }
    }

    pub fn as_static_view(&self) -> StaticView<V::VersionId> {
        StaticView::new(self.path.clone(), VersionPointer::Default)
    }

    pub fn has_children(&self) -> bool {
        self.get_node().borrow().has_children()
    }
}

/// Iterators
impl<'a, S: SymbolicNodeType, V: VCS> SemanticView<'a, S, V> {
    pub fn iter_children(&self) -> impl Iterator<Item = StaticView<V::VersionId>> {
        let path = self.as_static_view();
        path.iter_children()
    }

    pub fn iter_children_req(&self) -> impl Iterator<Item = StaticView<V::VersionId>> {
        let path = self.as_static_view();
        path.iter_children_req()
    }
}

/// Path pointer movement
impl<'a, S: SymbolicNodeType, V: VCS> SemanticView<'a, S, V> {
    pub fn get_at_index<To: SymbolicNodeType>(
        &self,
        index: usize,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, To, V>, TreeViewError<V::VersionId>> {
        let path = self.path[0..index + 1].to_vec();
        Ok(SemanticView::<'a, To, V>::new(path, repo)?)
    }

    /// Moves path to a specific index of the node vector.
    pub fn move_to_index<To: SymbolicNodeType>(
        self,
        index: usize,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, To, V>, TreeViewError<V::VersionId>> {
        self.get_at_index(index, repo)
    }

    pub fn get<To: SymbolicNodeType>(
        &self,
        path: &impl ToNormalizedPath,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, To, V>, TreeViewError<V::VersionId>> {
        repo.get_view(path)
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
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, To, V>, TreeViewError<V::VersionId>> {
        drop(self);
        repo.get_view(path)
    }
}

/// Display and pretty printing
impl<'a, S: SymbolicNodeType, V: VCS> SemanticView<'a, S, V> {
    // pub fn display_tree(&self, show_tags: bool) -> String {
    //     self.get_node().borrow().display_tree(show_tags)
    // }

    pub fn formatted(
        &self,
        show_type: bool,
        show_version: bool,
        colored: bool,
    ) -> String {
        self.as_static_view()
            .formatted(show_type, show_version, colored)
    }
}

impl<'a, S: SymbolicNodeType, V: VCS> NodeHolder<V::VersionId> for SemanticView<'a, S, V> {
    fn get_node(&self) -> &Rc<RefCell<Node<V::VersionId>>> {
        &self.path.last().unwrap()
    }
}

impl<'a, S: SymbolicNodeType, V: VCS> Drop for SemanticView<'a, S, V> {
    fn drop(&mut self) {
        self.get_node().borrow_mut().unlock()
    }
}

impl<'a, T: SymbolicNodeType, V: VCS> ToNormalizedPath for SemanticView<'a, T, V> {
    fn to_normalized_path(&self) -> NormalizedPath {
        self.path.to_normalized_path()
    }
}

impl<'a, S: SymbolicNodeType, V: VCS> PartialEq for SemanticView<'a, S, V> {
    fn eq(&self, other: &Self) -> bool {
        self.to_normalized_path() == other.to_normalized_path()
    }
}

impl<'a, S: SymbolicNodeType, V: VCS> Eq for SemanticView<'a, S, V> {}
