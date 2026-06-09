use crate::model::*;
use crate::vcs::VCS;
use colored::Colorize;
use itertools::Itertools;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use thiserror::Error;
use crate::derivation::DerivationData;

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

#[derive(Clone, Debug)]
pub struct NodePath<S: SymbolicNodeType, V: VCS> {
    path: Vec<Rc<RefCell<Node>>>,
    sym_type: S,
    vcs: Rc<RefCell<V>>,
}

impl<S: SymbolicNodeType, V: VCS> NodePath<S, V> {
    fn get_vcs(&self) -> &Rc<RefCell<V>> {
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

impl<V: VCS> NodePath<VirtualRoot, V> {
    pub fn scan_repository(&self) -> Result<(), WrongNodeTypeError> {
        let mut node = self.get_node().borrow_mut();
        match node.get_type() {
            NodeType::VirtualRoot(mut data) => {
                if data.repo_scanned() {
                    return Ok(());
                }
                let vcs = self.vcs.borrow();
                for path in vcs.iter_concrete_paths() {
                    let p = if path.is_absolute() {
                        path.strip_n_left(1)
                    } else { path };
                    node.insert_path(&p, true)?;
                }
                data.set_repo_scanned()
            }
            _ => unreachable!(),
        }
        Ok(())
    }
    
    pub fn move_to_area<C: NodeClassification>(self, area: &NormalizedPath) -> Result<NodePath<Area<C>, V>, PathNotFoundError> {
        match self.move_to(area) {
            Ok(node) => Ok(node),
            Err(error) => match error {
                NodePathError::NotFound(e) => Err(e),
                _ => unreachable!(),
            }
        }
    }
}

impl<V: VCS> NodePath<Product<Concrete>, V> {
    pub fn get_derivation_data(&self) -> Result<DerivationData, dyn Error> {
        todo!()
    }
}

impl<S: HasFeatureChildren, V: VCS> NodePath<S, V> {
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

impl<T: IsUnderArea> NodePath<T> {
    pub fn move_to_area(self) -> NodePath<Area> {
        self.move_to_index(1).unwrap()
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
