use crate::model::*;
use crate::repository::Repository;
use crate::vcs::VCS;
use itertools::Itertools;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub struct PathDoesNotExistError<V: VCS> {
    path: DynamicView<V>,
}

impl<V: VCS> PathDoesNotExistError<V> {
    pub fn new(path: DynamicView<V>) -> Self {
        Self { path }
    }
}

impl<V: VCS> Display for PathDoesNotExistError<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            format!(
                "Path '{}' does not exist",
                self.path.formatted(true, true, true)
            )
            .as_str(),
        )
    }
}

#[derive(Error, Clone, Debug)]
pub struct InvalidTypeError<V: VCS> {
    types_possible: Vec<NodeType>,
    type_found: NodeType,
    path: DynamicView<V>,
}

impl<V: VCS> InvalidTypeError<V> {
    pub fn new(types_possible: Vec<NodeType>, type_found: NodeType, path: DynamicView<V>) -> Self {
        Self {
            types_possible,
            type_found,
            path,
        }
    }

    pub fn types_possible(&self) -> &Vec<NodeType> {
        &self.types_possible
    }

    pub fn type_found(&self) -> &NodeType {
        &self.type_found
    }

    pub fn path(&self) -> &DynamicView<V> {
        &self.path
    }
}

impl<V: VCS> Display for InvalidTypeError<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let expected = self
            .types_possible
            .iter()
            .map(|t| t.get_formatted_name())
            .collect::<Vec<_>>()
            .join(", ");
        f.write_str(
            format!(
                "Path '{}' has invalid type\n\
            Found type {} but expected one of the following:\n  {expected}",
                self.path.formatted(false, false, true),
                self.type_found.get_formatted_name(),
            )
            .as_str(),
        )
    }
}

#[derive(Error, Clone, Debug)]
pub enum SemanticViewError<V: VCS> {
    #[error(transparent)]
    PathDoesNotExist(#[from] PathDoesNotExistError<V>),
    #[error(transparent)]
    InvalidType(#[from] InvalidTypeError<V>),
}

pub trait AccessMode: Debug {
    type V: VCS;

    fn get_repo(&self) -> &Repository<Self::V>;
}

#[derive(Clone, Copy, Debug)]
pub struct R<'a, V: VCS> {
    repo: &'a Repository<V>
}

impl<'a, V: VCS> AccessMode for R<'a, V> {
    type V = V;

    fn get_repo(&self) -> &'a Repository<Self::V> {
        self.repo
    }
}

#[derive(Debug)]
pub struct RW<'a, V: VCS> {
    repo: &'a mut Repository<V>
}

impl<'a, V: VCS> AccessMode for RW<'a, V> {
    type V = V;

    fn get_repo(&self) -> &Repository<Self::V> {
        self.repo
    }
}

impl<'a, V: VCS> RW<'a, V> {
    pub fn get_repo_mut(&mut self) -> &mut Repository<V> {
        self.repo
    }
}

/// Semantic view onto the path model.
///
/// This struct symbolizes a path in the tree model and is the primary interface to interact with the underlying VCS repository.
/// Its capabilities are defined by:
/// - the type of node it points to ([SymbolicNodeType] parameter),
/// - the VCS implementation ([VCS] parameter).
#[derive(Debug)]
pub struct StructureView<S, M>
where
    S: SymbolicNodeType,
    M: AccessMode,
{
    path: Vec<Rc<RefCell<Node<M::V>>>>,
    mode: M,
    _sym_marker: PhantomData<S>,
}

/// Construction and transformation
impl<S: SymbolicNodeType, M: AccessMode> StructureView<S, M> {
    pub(crate) fn new(
        path: Vec<Rc<RefCell<Node<M::V>>>>,
        mode: M,
    ) -> Result<Self, SemanticViewError<M::V>> {
        let new = Self {
            path,
            mode,
            _sym_marker: PhantomData,
        };
        let new = new
            .check_path_not_existent()?
            .check_sym_type_compatibility()?;
        Ok(new)
    }

    pub fn try_convert_to<To: SymbolicNodeType>(
        self,
    ) -> Result<StructureView<To, M>, InvalidTypeError<M::V>> {
        let new = StructureView {
            path: self.path,
            mode: self.mode,
            _sym_marker: PhantomData,
        };
        let new = new.check_sym_type_compatibility()?;
        Ok(new)
    }

    pub fn convert_to_any_type(self) -> StructureView<AnyType<AnyC>, M> {
        self.try_convert_to().unwrap()
    }

    pub fn to_dynamic_view(&self) -> DynamicView<M::V> {
        DynamicView::new(self.path.clone(), RevisionPointer::Head)
    }

    fn check_path_not_existent(self) -> Result<Self, PathDoesNotExistError<M::V>> {
        if &self.get_real_type() == &NodeType::NonExistent {
            let path = DynamicView::new(self.path.clone(), RevisionPointer::Head);
            Err(PathDoesNotExistError::new(path))
        } else {
            Ok(self)
        }
    }

    fn check_sym_type_compatibility(self) -> Result<Self, InvalidTypeError<M::V>> {
        if !S::compatible().contains(&self.get_real_type()) {
            let real_type = self.get_real_type();
            Err(InvalidTypeError::new(
                S::compatible(),
                real_type,
                self.to_dynamic_view(),
            ))
        } else {
            Ok(self)
        }
    }
}

/// Getters and setters
impl<S: SymbolicNodeType, M: AccessMode> StructureView<S, M> {
    fn get_repo(&self) -> &Repository<M::V> {
        self.mode.get_repo()
    }

    fn get_root(&self) -> &Rc<RefCell<Node<M::V>>> {
        self.path.first().unwrap()
    }

    pub fn get_vcs(&self) -> &M::V {
        self.get_repo().get_vcs()
    }

    pub fn has_children(&self) -> bool {
        self.get_node().borrow().has_children()
    }
}

impl<'a, S: SymbolicNodeType, V: VCS> StructureView<S, RW<'a, V>> {
    fn get_repo_mut(&mut self) -> &mut Repository<V> {
        self.mode.get_repo_mut()
    }
}

/// Iterators
impl<'a, S: SymbolicNodeType, M: AccessMode> StructureView<S, M> {
    pub fn iter_children(&self) -> impl Iterator<Item = StructureView<AnyType<AnyC>, M>> {
        self.get_node()
            .borrow()
            .get_children()
            .into_iter()
            .map(|node| {
                let mut path = self.path.clone();
                path.push(node);
                StructureView::new(path, self.mode).unwrap()
            })
            .sorted()
    }

    pub fn iter_children_req(&self) -> impl Iterator<Item = StructureView<AnyType<AnyC>, M>> {
        self
            .iter_children()
            .flat_map(|v| {
                v.iter_children_req().collect::<Vec<_>>()
            })
    }
}

/// Path pointer movement
impl<S: SymbolicNodeType, M: AccessMode> StructureView<S, M> {
    /// Moves path to a specific index of the node vector.
    pub fn move_to_index<To: SymbolicNodeType>(
        self,
        index: usize,
    ) -> Result<StructureView<To, M>, SemanticViewError<M::V>> {
        let path = self.path[0..index + 1].to_vec();
        Ok(StructureView::<To, M>::new(path, self.mode)?)
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
    ) -> Result<StructureView<To, M>, SemanticViewError<M::V>> {
        let path = path.to_normalized_path();
        let mut new_node_vec = vec![self.get_root().clone()];
        for p in path.iter_segments(1, path.len()) {
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
        StructureView::new(new_node_vec, self.mode)
    }

    fn move_to_guaranteed_type<To: SymbolicNodeType>(
        self,
        path: &impl ToNormalizedPath,
    ) -> Result<StructureView<To, M>, PathDoesNotExistError<M::V>> {
        match self.move_to::<To>(path) {
            Ok(v) => Ok(v),
            Err(SemanticViewError::PathDoesNotExist(e)) => Err(e),
            _ => unreachable!(),
        }
    }
}

/// Display and pretty printing
impl<S: SymbolicNodeType, M: AccessMode> StructureView<S, M> {
    // pub fn display_tree(&self, show_tags: bool) -> String {
    //     self.get_node().borrow().display_tree(show_tags)
    // }

    pub fn formatted(&self, show_type: bool, show_version: bool, colored: bool) -> String {
        self.to_dynamic_view()
            .formatted(show_type, show_version, colored)
    }
}

impl<S: SymbolicNodeType, M: AccessMode> NodeHolder<M::V> for StructureView<S, M> {
    fn get_node(&self) -> &Rc<RefCell<Node<M::V>>> {
        &self.path.last().unwrap()
    }
}

impl<'a, T: SymbolicNodeType, M: AccessMode> ToNormalizedPath for StructureView<T, M> {
    fn to_normalized_path(&self) -> NormalizedPath {
        self.path.to_normalized_path()
    }
}

impl<S: SymbolicNodeType, M: AccessMode> PartialEq for StructureView<S, M> {
    fn eq(&self, other: &Self) -> bool {
        self.to_normalized_path() == other.to_normalized_path()
    }
}

impl<S: SymbolicNodeType, M: AccessMode> Eq for StructureView<S, M> {}

impl<S: SymbolicNodeType, M: AccessMode> PartialOrd for StructureView<S, M> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_normalized_path().partial_cmp(&other.to_normalized_path())
    }
}

impl<S: SymbolicNodeType, M: AccessMode> Ord for StructureView<S, M> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

/// Reachability for virtual root
impl<M: AccessMode> StructureView<VirtualRoot, M> {
    pub fn move_to_area<C: NodeClassification>(
        self,
        area: &impl ToNormalizedPath,
    ) -> Result<StructureView<Area<C>, M>, SemanticViewError<M::V>> {
        self.move_to(area)
    }
}

impl<C: NodeClassification, M: AccessMode> StructureView<Area<C>, M> {
    pub fn get_path_to_feature_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(FEATURE_ROOT)
    }

    pub fn get_path_to_product_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(PRODUCT_ROOT)
    }

    pub fn move_to_feature_root(
        self,
    ) -> Result<StructureView<FeatureRoot, M>, PathDoesNotExistError<M::V>> {
        let path = self.get_path_to_feature_root().to_normalized_path();
        self.move_to_guaranteed_type(&path)
    }

    pub fn move_to_product_root(
        self,
    ) -> Result<StructureView<ProductRoot, M>, PathDoesNotExistError<M::V>> {
        let path = self.get_path_to_feature_root().to_normalized_path();
        self.move_to_guaranteed_type(&path)
    }
}

impl<S: IsConcrete, M: AccessMode> StructureView<S, M> {
    pub fn get_id(&self) -> usize {
        self.get_node().borrow().get_branch_info().unwrap().get_id()
    }

    pub fn assert_revision(
        &self,
        revision: impl Into<String>,
    ) -> Result<<M::V as VCS>::VersionId, RevisionError<M::V, <M::V as VCS>::VCSError>> {
        let rev = revision.into();
        let vcs = self.get_vcs();
        if vcs
            .revision_exists_on_path(&self.to_normalized_path(), &rev)?
        {
            let revision = vcs.get_revision(&rev)?.unwrap();
            self.get_node()
                .borrow_mut()
                .mut_get_branch_info()
                .unwrap()
                .add_known_version(revision.clone());
            Ok(revision)
        } else {
            Err(self.to_dynamic_view().into())
        }
    }

    pub fn to_head_rev(self) -> RevisionView<'a, S, Head, V> {
        RevisionView::<'a, S, Head, V>::new(self)
    }

    pub fn to_rev(
        self,
        revision: impl Into<String>,
    ) -> Result<RevisionView<'a, S, Rev, V>, RevisionError<V::VCS, V::VCSError>>
    {
        RevisionView::<'a, S, Rev, V>::new(self, revision)
    }
}

impl<T: UnderArea, M: AccessMode> StructureView<T, M> {
    pub fn move_to_area<C: NodeClassification>(
        self,
    ) -> StructureView<Area<C>, M> {
        self.move_to_index(1).unwrap()
    }
}

pub struct FilterByType<T: SymbolicNodeType> {
    _marker: PhantomData<T>,
}

impl<T: SymbolicNodeType> FilterByType<T> {
    pub fn filter<S, V>(view: StructureView<S, V>) -> Option<StructureView<T, V>>
    where
        S: SymbolicNodeType,
        V: VCS
    {
        match view.try_convert_to::<T>() {
            Ok(view) => Some(view),
            Err(_) => None,
        }
    }
}
