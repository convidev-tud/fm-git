use crate::model::*;
use crate::vcs::VCS;
use colored::Colorize;
use itertools::Itertools;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::rc::Rc;
use thiserror::Error;

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

#[derive(Clone, Debug)]
pub struct NodePath<SNT: SymbolicNodeType, V: VCS> {
    path: Vec<Rc<RefCell<Node>>>,
    version: VersionPointer,
    vcs: Rc<V>,
    _phantom: PhantomData<SNT>,
}

impl<T: HasFeatureChildren> NodePath<T> {
    pub fn move_to_feature(self, path: &NormalizedPath) -> Option<NodePath<Feature>> {
        self.move_to(path)?.try_convert_to()
    }
    pub fn iter_features(&self) -> impl Iterator<Item = NodePath<Feature>> {
        self.iter_children_by_type()
            .map(|p| p.try_convert_to().unwrap())
    }
    pub fn iter_features_req(&self) -> impl Iterator<Item = NodePath<Feature>> {
        self.iter_children_by_type_req()
            .map(|p| p.try_convert_to().unwrap())
    }
}

impl<T: HasProductChildren> NodePath<T> {
    pub fn move_to_product(self, path: &NormalizedPath) -> Option<NodePath<Product>> {
        self.move_to(path)?.try_convert_to()
    }
    pub fn iter_products(&self) -> impl Iterator<Item = NodePath<Product>> {
        self.iter_children_by_type()
            .map(|p| p.try_convert_to().unwrap())
    }
    pub fn iter_products_req(&self) -> impl Iterator<Item = NodePath<Product>> {
        self.iter_children_by_type_req()
            .map(|p| p.try_convert_to().unwrap())
    }
}

impl<T: IsOnOrUnderArea> NodePath<T> {
    pub fn move_to_area(self) -> NodePath<Area> {
        self.move_to_index(1).unwrap()
    }
}

impl<T: IsGitObject> NodePath<T> {
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

impl NodePath<VirtualRoot> {
    pub fn move_to_area(self, area: &NormalizedPath) -> Option<NodePath<ConcreteArea>> {
        self.move_to(area)?.try_convert_to()
    }
}

impl NodePath<ConcreteArea> {
    pub fn get_path_to_feature_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(FEATURE_ROOT)
    }
    pub fn get_path_to_product_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(PRODUCT_ROOT)
    }
    pub fn move_to_feature_root(self) -> Option<NodePath<FeatureRoot>> {
        self.move_to(&NormalizedPath::from(FEATURE_ROOT))?
            .try_convert_to()
    }
    pub fn move_to_product_root(self) -> Option<NodePath<ProductRoot>> {
        self.move_to(&NormalizedPath::from(PRODUCT_ROOT))?
            .try_convert_to()
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

impl<T: SymbolicNodeType> NodePath<T> {
    pub fn get_node(&self) -> &Rc<RefCell<Node>> {
        self.path.last().unwrap()
    }
    pub(in crate::core::model) fn new(
        path: Vec<Rc<RefCell<Node>>>,
        sym_head: VersionPointer,
        git: Rc<GitCLI>,
    ) -> Result<NodePath<T>, WrongNodeTypeError> {
        let last = path.last().unwrap();
        let node = last.borrow();
        if !T::is_compatible(&node) {
            return Err(WrongNodeTypeError::new())
        }
        let new = Self {
            path,
            version: sym_head,
            vcs: git,
            _phantom: PhantomData,
        };
        Ok(new)
    }
    pub fn try_convert_to<To: SymbolicNodeType>(&self) -> Result<NodePath<To>, WrongNodeTypeError> {
        NodePath::<To>::new(
            self.path.clone(),
            self.version.clone(),
            self.vcs.clone(),
        )
    }
    pub fn move_to<To: SymbolicNodeType>(
        mut self,
        path: &NormalizedPath,
    ) -> Result<NodePath<To>, NodePathError> {
        let without_version = path.strip_version();
        for p in without_version.iter_segments() {
            let node = if let Some(node) = self.get_node().borrow().get_child(p) {
                node
            } else {
                return Err(PathNotFoundError::new(path.clone()).into());
            };
            self.path.push(node);
        }
        let head = match path.get_version_appendix() {
            Some(version) => {
                if self.has_tag(version.clone()) {
                    VersionPointer::Tag(version)
                } else {
                    VersionPointer::Commit(CommitHash::new(version))
                }
            }
            None => VersionPointer::Head,
        };
        Ok(NodePath::<To>::new(self.path, head, self.vcs)?)
    }
    pub fn move_to_index<To: SymbolicNodeType>(self, index: usize) -> Result<NodePath<To>, WrongNodeTypeError> {
        let path = self.path[0..index + 1].to_vec();
        NodePath::<To>::new(path, VersionPointer::Head, self.vcs)
    }
    pub fn has_children(&self) -> bool {
        self.get_node().borrow().has_children()
    }
    pub fn iter_children_by_type<I: SymbolicNodeType>(&self) -> impl Iterator<Item = NodePath<I>> {
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
    pub fn iter_children_by_type_req<I: SymbolicNodeType>(&self) -> impl Iterator<Item = NodePath<I>> {
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
