mod virtual_root;
mod area;
mod feature;
mod product;
mod any;
mod classif;

use crate::model::*;
use crate::vcs::VCS;
pub use area::*;
pub use feature::*;
pub use product::*;
pub use classif::*;
use std::cell::RefCell;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;
use thiserror::Error;
pub use virtual_root::*;

#[derive(Error, Debug)]
#[error("Path {path} des not exist.")]
pub struct PathNotFoundError {
    path: NormalizedPath,
}

impl PathNotFoundError {
    pub fn new(path: NormalizedPath) -> Self {
        Self { path }
    }
}

#[derive(Error, Debug)]
pub enum NodePathError {
    #[error(transparent)]
    WrongType(#[from] WrongNodeTypeError),
    #[error(transparent)]
    NotFound(#[from] PathNotFoundError),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum VersionPointer {
    Head,
    Commit(CommitHash),
    Tag(String),
}

impl VersionPointer {
    fn formatted(&self, colored: bool, current_head: CommitHash) -> String {
        fn make_head_info(head: &CommitHash) -> String {
            format!("(Head -> {head})")
        }

        let info = if colored {
            match self {
                Self::Head => make_head_info(&current_head).yellow(),
                Self::Commit(c) => {
                    if c == &current_head {
                        make_head_info(&current_head).yellow()
                    } else {
                        format!("({})", c.get_short_hash()).yellow()
                    }
                }
                Self::Tag(tag) => format!("({})", tag).green(),
            }
        } else {
            match self {
                Self::Head => make_head_info(&current_head).normal(),
                Self::Commit(c) => {
                    if c == &current_head {
                        make_head_info(&current_head).normal()
                    } else {
                        format!("({})", c.get_short_hash()).normal()
                    }
                }
                Self::Tag(tag) => format!("({})", tag).green().normal(),
            }
        };
        info.to_string()
    }
}

/// Some node node_path have the option of being concrete (with attached artifacts) or abstract.
/// This is the base trait for this classification.
pub trait NodeClassification: Clone + Debug + Eq + PartialEq + Hash {
    fn requires_artifact() -> Option<bool>;
}

/// Symbolic node type base trait.
/// This exists for generic type parameters.
pub trait SymbolicNodeType: Clone + Debug + Eq + PartialEq + Hash {
    type Classification: NodeClassification;
    fn new() -> Self { Self {} }
    fn compatible(&self) -> Vec<NodeType>;
}

/// Denotes that a [SymbolicNodeType] is concrete (with associated artifact).
///
/// Is automatically implemented if [Concrete] is used as parameter.
pub trait IsConcrete: SymbolicNodeType {
    fn get_version(&self) -> &VersionPointer;
    fn set_version(&mut self, version: VersionPointer);
}
impl<T: SymbolicNodeType<Classification=Concrete>> IsConcrete for T {
    fn get_version(&self) -> &VersionPointer {
        todo!()
    }

    fn set_version(&mut self, version: VersionPointer) {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct NodePath<S: SymbolicNodeType, V: VCS> {
    path: Vec<Rc<RefCell<Node>>>,
    sym_type: S,
    vcs: Rc<RefCell<V>>,
}

impl<S: SymbolicNodeType, V: VCS> NodePath<S, V> {
    pub(super) fn get_vcs(&self) -> &Rc<RefCell<V>> {
        &self.vcs
    }

    pub fn get_node(&self) -> &Rc<RefCell<Node>> {
        self.path.last().unwrap()
    }

    pub(crate) fn new(
        path: Vec<Rc<RefCell<Node>>>,
        vcs: Rc<RefCell<V>>,
    ) -> Result<NodePath<S, V>, WrongNodeTypeError> {
        let last = path.last().unwrap();
        let node = last.borrow();
        if !S::is_compatible(&node) {
            return Err(WrongNodeTypeError::new())
        }
        let new = Self {
            path,
            sym_type: S::new(),
            vcs,
        };
        Ok(new)
    }

    pub fn try_convert_to<To: SymbolicNodeType>(&self) -> Result<NodePath<To, V>, WrongNodeTypeError> {
        NodePath::new(self.path.clone(), self.vcs.clone())
    }

    pub fn move_to<To: SymbolicNodeType>(
        mut self,
        path: &NormalizedPath,
    ) -> Result<NodePath<To, V>, NodePathError> {
        let without_version = path.strip_version();
        for p in without_version.iter_segments() {
            let node = if let Some(node) = self.get_node().borrow().get_child(p) {
                node
            } else {
                return Err(PathNotFoundError::new(path.clone()).into());
            };
            self.path.push(node);
        }
        Ok(NodePath::new(self.path, self.vcs)?)
    }

    pub fn move_to_index<To: SymbolicNodeType>(self, index: usize) -> Result<NodePath<To, V>, WrongNodeTypeError> {
        let path = self.path[0..index + 1].to_vec();
        NodePath::new(path, self.vcs)
    }

    pub fn move_to_current<To: SymbolicNodeType>(self) -> Result<NodePath<To, V>, WrongNodeTypeError> {
        let current = self.get_vcs().borrow().get_current_path();
        match self.move_to(&current) {
            Ok(path) => Ok(path),
            Err(error) => {
                match error {
                    NodePathError::WrongType(e) => Err(e),
                    NodePathError::NotFound(_) => unreachable!(),
                }
            }
        }
    }

    pub fn has_children(&self) -> bool {
        self.get_node().borrow().has_children()
    }

    pub fn iter_children_by_type<I: SymbolicNodeType>(&self) -> impl Iterator<Item = NodePath<I, V>> {
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

    pub fn iter_children_by_type_req<I: SymbolicNodeType>(&self) -> impl Iterator<Item = NodePath<I, V>> {
        self.iter_children_by_type::<I>().flat_map(|path| {
            let mut to_iter = Vec::new();
            to_iter.push(path.clone());
            to_iter.extend(path.iter_children_by_type_req());
            to_iter
        })
    }

    pub fn get_tags(&self) -> Vec<CommitTag> {
        self.get_node().borrow().get_tags().clone()
    }

    pub fn has_tag<S: Into<String>>(&self, tag: S) -> bool {
        let mut has_tag = false;
        let into = tag.into();
        for tag in self.get_tags() {
            if tag.get_tag() == &into {
                has_tag = true;
                break;
            }
        }
        has_tag
    }

    pub fn get_real_type(&self) -> NodeType {
        self.get_node().borrow().get_type().clone()
    }

    pub fn as_any_type(&self) -> NodePath<AnyNode> {
        NodePath::new(self.path.clone(), self.version.clone(), self.vcs.clone()).unwrap()
    }

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

impl<S: IsConcrete, V: VCS> NodePath<S, V> {
    pub fn get_ref_name(&self) -> String {
        self.get_node()
            .borrow()
            .get_branch_data()
            .get_branch()
            .unwrap()
            .clone()
    }
    pub fn get_object(&self) -> String {
        match &self.version {
            VersionPointer::Head => self.get_head().get_full_hash().clone(),
            VersionPointer::Commit(hash) => hash.get_full_hash().clone(),
            VersionPointer::Tag(tag) => tag.clone(),
        }
    }
    pub fn get_qualified_object(&self) -> String {
        match &self.version {
            VersionPointer::Head => self.get_object(),
            VersionPointer::Commit(_) => self.get_object(),
            VersionPointer::Tag(_) => {
                todo!()
            }
        }
    }
    pub fn get_head(&self) -> CommitHash {
        self.get_metadata().get_head().unwrap().clone()
    }
    pub fn get_version(&self) -> &VersionPointer {
        &self.version
    }
    pub fn update_version(&mut self, head: VersionPointer) {
        self.version = head;
    }
    pub fn formatted_with_version(&self, colored: bool) -> String {
        let base = self.formatted(colored);
        let version = self.version.formatted(colored, self.get_head());
        format!("{base} {version}")
    }
    pub fn to_normalized_path_with_version(&self) -> NormalizedPath {
        let mut path = self.to_normalized_path();
        path.set_version_appendix(Some(self.get_object()));
        path
    }
}

impl<T: SymbolicNodeType> ToNormalizedPath for NodePath<T> {
    fn to_normalized_path(&self) -> NormalizedPath {
        let mut path = NormalizedPath::new();
        for p in self.path.iter() {
            path.push(p.borrow().get_name());
        }
        match &self.version {
            VersionPointer::Head => path.set_version_appendix::<String>(None),
            VersionPointer::Commit(hash) => path.set_version_appendix(Some(hash.get_full_hash())),
            VersionPointer::Tag(tag) => path.set_version_appendix(Some(tag)),
        }
        path
    }
}

impl<T: SymbolicNodeType> Hash for NodePath<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_normalized_path().hash(state);
    }
}

impl<A, B> PartialEq<NodePath<A>> for NodePath<B>
where
    A: SymbolicNodeType,
    B: SymbolicNodeType,
{
    fn eq(&self, other: &NodePath<A>) -> bool {
        self.to_normalized_path() == other.to_normalized_path()
    }
}

impl<T: SymbolicNodeType> Eq for NodePath<T> {}

impl<T: SymbolicNodeType> Display for NodePath<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_normalized_path().to_string().as_str())
    }
}

impl<T: SymbolicNodeType> PartialOrd for NodePath<T> {
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

impl<T: SymbolicNodeType> Ord for NodePath<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}