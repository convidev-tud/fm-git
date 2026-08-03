use crate::model::*;
use crate::repository::Repository;
use crate::vcs::VCS;
use indextree::{Arena, Node, NodeId};
use itertools::Itertools;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use thiserror::Error;

/*
    ##############
        Errors
    ##############
*/

#[derive(Error, Clone, Debug)]
pub struct PathDoesNotExistError<V: VCS> {
    path: FrozenView<V>,
}

impl<V: VCS> PathDoesNotExistError<V> {
    pub fn new(path: FrozenView<V>) -> Self {
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
    path: FrozenView<V>,
}

impl<V: VCS> InvalidTypeError<V> {
    pub fn new(types_possible: Vec<NodeType>, type_found: NodeType, path: FrozenView<V>) -> Self {
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

    pub fn path(&self) -> &FrozenView<V> {
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

/*
    #######################
        Main Definition
    #######################
*/

/// Semantic view onto the path model.
///
/// This struct symbolizes a path in the tree model and is the primary interface to interact with the underlying VCS repository.
/// Its capabilities are defined by:
/// - the type of node it points to ([SymbolicNodeType] parameter),
/// - the VCS implementation ([VCS] parameter).
#[derive(Debug)]
pub struct StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    id: NodeId,
    repo: &'a Repository<V>,
    _mode_marker: PhantomData<M>,
    _type_marker: PhantomData<S>,
}

impl<'a, S, M, V> StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    fn get_repo(&self) -> &Repository<V> {
        self.repo
    }

    fn get_arena(&self) -> &Arena<RefCell<NodeData<V>>> {
        self.get_repo().get_arena()
    }

    fn iter_path(&self) -> impl Iterator<Item=NodeId> {
        self.get_node_id().ancestors(self.get_arena())
    }

    fn get_id_path(&self) -> Vec<NodeId> {
        let mut v = self.iter_path().collect::<Vec<NodeId>>();
        v.reverse();
        v
    }

    fn get_node_path(&self) -> Vec<&Node<RefCell<NodeData<V>>>> {
        self
            .get_id_path()
            .iter()
            .map(|id| self.get_arena().get(*id).unwrap())
            .collect()
    }

    fn get_root_id(&self) -> NodeId {
        self.repo.get_root_id()
    }

    fn get_root_node(&self) -> &Node<RefCell<NodeData<V>>> {
        self.repo.get_root_node()
    }

    fn path_to_static(&self) -> Vec<StaticNode> {
        let mut path = self.iter_path().collect_vec();
        path.reverse();
        path
            .iter()
            .map(|id| {
                let node = self.repo.get_node(*id).unwrap().get().borrow();
                StaticNode::new(node.get_name().clone(), node.get_type().clone())
            })
            .collect()
    }

    fn check_path_not_existent(&self) -> Result<(), PathDoesNotExistError<V>> {
        if &self.get_real_type() == &NodeType::NonExistent {
            let path = FrozenView::new(self.path_to_static(), RevisionPointer::None);
            Err(PathDoesNotExistError::new(path))
        } else {
            Ok(())
        }
    }

    fn check_sym_type_compatibility(&self) -> Result<(), InvalidTypeError<V>> {
        if !S::compatible().contains(&self.get_real_type()) {
            let real_type = self.get_real_type();
            Err(InvalidTypeError::new(
                S::compatible(),
                real_type,
                self.to_frozen_view(RevisionPointer::None),
            ))
        } else {
            Ok(())
        }
    }

    fn assert_lock(&self) {
        let mut node = self.get_node().get().borrow_mut();
        if node.is_structure_locked() {
            drop(node);
            panic!(
                "Cannot create structure view for for '{}': node is locked",
                self.to_normalized_path(),
            )
        }
        if M::lock() {
            if node.structure_views_referenced() > 0 {
                let referenced = node.structure_views_referenced();
                drop(node);
                panic!(
                    "Cannot lock node for structure view '{}': there are {referenced} other structure views referencing it",
                    self.to_normalized_path(),
                )
            }
            node.lock_structure();
        }
        node.reference_structure_view();
    }

    pub(crate) fn new(
        id: NodeId,
        repo: &'a Repository<V>,
    ) -> Result<Self, SemanticViewError<V>> {
        let new = Self {
            id,
            repo,
            _mode_marker: PhantomData,
            _type_marker: PhantomData,
        };
        new.check_path_not_existent()?;
        new.check_sym_type_compatibility()?;
        new.assert_lock();
        Ok(new)
    }

    pub(crate) fn get_node(&self) -> &Node<RefCell<NodeData<V>>> {
        self.repo.get_node(self.get_node_id()).unwrap()
    }

    pub fn get_node_id(&self) -> NodeId {
        self.id
    }

    pub fn get_real_type(&self) -> NodeType {
        self.get_node().get().borrow().get_type().clone()
    }

    pub fn get_vcs(&self) -> &V {
        self.get_repo().get_vcs()
    }

    pub fn try_convert_to<To: SymbolicNodeType>(
        self,
    ) -> Result<StructureView<'a, To, M, V>, InvalidTypeError<V>> {
        let new = StructureView {
            id: self.id,
            repo: self.repo,
            _mode_marker: PhantomData,
            _type_marker: PhantomData,
        };
        new.check_sym_type_compatibility()?;
        Ok(new)
    }

    pub fn convert_to_any_type(self) -> StructureView<'a, AnyType<AnyC>, M, V> {
        self.try_convert_to().unwrap()
    }

    pub fn to_frozen_view(&self, revision: RevisionPointer<V>) -> FrozenView<V> {
        FrozenView::new(self.path_to_static(), revision)
    }

    pub fn iter_children(
        &self,
        repo: &'a Repository<V>,
    ) -> impl Iterator<Item = StructureView<'a, AnyType<AnyC>, Shared, V>> {
        let id = self.get_node_id();
        id
            .children(self.get_repo().get_arena())
            .map(|child| {
                StructureView::new(child, repo).unwrap()
            })
    }

    pub fn iter_children_req(
        &self,
        repo: &'a Repository<V>,
    ) -> impl Iterator<Item = StructureView<AnyType<AnyC>, Shared, V>> {
        let id = self.get_node_id();
        id
            .descendants(self.get_repo().get_arena())
            .map(|child| {
                StructureView::new(child, repo).unwrap()
            })
    }

    /// Moves path to a specific index on the node vector.
    pub fn move_to_index<To: SymbolicNodeType>(
        self,
        index: usize,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, To, M, V>, SemanticViewError<V>> {
        let path = self.get_id_path();
        Ok(StructureView::<To, M, V>::new(path[index], repo)?)
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
    ) -> Result<StructureView<'a, To, M, V>, SemanticViewError<V>> {
        fn make_error_node(name: String) -> StaticNode {
            StaticNode::new(
                name,
                NodeType::NonExistent,
            )
        }

        // set path to absolute from root
        let path = self.to_normalized_path() + path.to_normalized_path();
        let root = self.get_root_id();
        let mut current = Some(root);
        let mut id_vec: Vec<NodeId> = vec![root];
        let mut error_nodes: Vec<StaticNode> = vec![];
        // iter path
        for p in path.iter_segments(1, path.len()) {
            match current {
                Some(id) => {
                    let borrowed = repo.get_node(id).unwrap().get().borrow();
                    match borrowed.get_child(p) {
                        Some(child) => {
                            id_vec.push(*child);
                            current = Some(*child);
                        }
                        None => {
                            error_nodes.push(make_error_node(p.clone()));
                            current = None;
                        }
                    }
                }
                None => {
                    error_nodes.push(make_error_node(p.clone()));
                }
            }
        }
        if let Some(current) = current {
            StructureView::new(current, repo)
        } else {
            let mut nodes = id_vec
                .iter()
                .map(|id| {
                    let node = self.repo.get_node(*id).unwrap().get().borrow();
                    StaticNode::new(
                        node.get_name().clone(),
                        node.get_type().clone(),
                    )
                })
                .collect::<Vec<_>>();
            nodes.extend(error_nodes);
            Err(PathDoesNotExistError::new(FrozenView::new(nodes, RevisionPointer::None)).into())
        }
    }

    fn move_to_guaranteed_type<To: SymbolicNodeType>(
        self,
        path: &impl ToNormalizedPath,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, To, M, V>, PathDoesNotExistError<V>> {
        match self.move_to::<To>(path, repo) {
            Ok(v) => Ok(v),
            Err(SemanticViewError::PathDoesNotExist(e)) => Err(e),
            _ => unreachable!(),
        }
    }

    pub fn formatted(&self, show_type: bool, colored: bool) -> String {
        self.to_frozen_view(RevisionPointer::None)
            .formatted(show_type, false, colored)
    }
}

impl<'a, S, V> StructureView<'a, S, Shared, V>
where
    S: SymbolicNodeType,
    V: VCS,
{
    pub fn clone(&self, repo: &'a Repository<V>) -> Self {
        Self::new(self.id, repo).unwrap()
    }
}

/*
    #######################################
        Important trait implementations
    #######################################
*/

impl<'a, S, M, V> ToNormalizedPath for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    fn to_normalized_path(&self) -> NormalizedPath {
        self.get_node_path().to_normalized_path()
    }
}

impl<'a, S, M, V> PartialEq for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    fn eq(&self, other: &Self) -> bool {
        self.to_normalized_path() == other.to_normalized_path()
    }
}

impl<'a, S, M, V> Eq for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{}

impl<'a, S, M, V> PartialOrd for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_normalized_path().partial_cmp(&other.to_normalized_path())
    }
}

impl<'a, S, M, V> Ord for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

impl<'a, S, M, V> Drop for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    fn drop(&mut self) {
        let mut node = self.get_node().get().borrow_mut();
        if M::lock() {
            node.unlock_structure()
        }
        node.dereference_structure_view()
    }
}

/*
    ############################
    # Specific Implementations #
    ############################
*/

impl<'a, S, M, V> StructureView<'a, S, M, V>
where
    S: IsConcrete,
    M: AccessMode,
    V: VCS,
{
    pub fn get_vcs_id(&self) -> usize {
        self.get_node().get().borrow().get_branch_info().unwrap().get_id()
    }
}

impl<'a, S, V> StructureView<'a, S, Shared, V>
where
    S: IsConcrete,
    V: VCS,
{
    pub fn lock(self) -> StructureView<'a, S, Locked, V> {
        let id = self.id;
        let repo = self.repo;
        drop(self);
        StructureView::new(id, repo).unwrap()
    }
    
    pub fn to_head_rev(self) -> RevisionView<'a, S, Head, M, V> {
        RevisionView::<'a, S, Head, V>::new(self)
    }

    pub fn to_rev(
        self,
        revision: impl Into<String>,
    ) -> Result<RevisionView<'a, S, Rev, M, V>, RevisionError<V, V::VCSError>> {
        RevisionView::<'a, S, Rev, V>::new(self, revision)
    }
}

impl<'a, S, V> StructureView<'a, S, Locked, V>
where
    S: IsConcrete,
    V: VCS,
{
    pub fn unlock(self) -> StructureView<'a, S, Shared, V> {
        let id = self.id;
        let repo = self.repo;
        drop(self);
        StructureView::new(id, repo).unwrap()
    }
}

impl<'a, T, M, V> StructureView<'a, T, M, V>
where
    T: UnderArea,
    M: AccessMode,
    V: VCS,
{
    pub fn move_to_area<C: NodeClassification>(
        self,
        repo: &'a Repository<V>,
    ) -> StructureView<'a, Area<C>, M, V> {
        self.move_to_index(1, repo).unwrap()
    }
}

impl<'a, M: AccessMode, V: VCS> StructureView<'a, VirtualRoot, M, V> {
    pub fn move_to_area<C: NodeClassification>(
        self,
        area: &impl ToNormalizedPath,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, Area<C>, M, V>, SemanticViewError<V>> {
        self.move_to(area, repo)
    }
}

impl<'a, C, M, V> StructureView<'a, Area<C>, M, V>
where
    C: NodeClassification,
    M: AccessMode,
    V: VCS,
{
    pub fn get_path_to_feature_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(FEATURE_ROOT)
    }

    pub fn get_path_to_product_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(PRODUCT_ROOT)
    }

    pub fn move_to_feature_root(
        self,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, FeatureRoot, M, V>, PathDoesNotExistError<V>> {
        let path = self.get_path_to_feature_root().to_normalized_path();
        self.move_to_guaranteed_type(&path, repo)
    }

    pub fn move_to_product_root(
        self,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, ProductRoot, M, V>, PathDoesNotExistError<V>> {
        let path = self.get_path_to_feature_root().to_normalized_path();
        self.move_to_guaranteed_type(&path, repo)
    }
}

/*
    ##########################
    # Filter implementations #
    ##########################
*/

pub struct FilterByType<T: SymbolicNodeType>(PhantomData<T>);

impl<T: SymbolicNodeType> FilterByType<T> {
    pub fn filter<S, M, V>(view: StructureView<S, M, V>) -> Option<StructureView<T, M, V>>
    where
        S: SymbolicNodeType,
        M: AccessMode,
        V: VCS
    {
        match view.try_convert_to::<T>() {
            Ok(view) => Some(view),
            Err(_) => None,
        }
    }
}
