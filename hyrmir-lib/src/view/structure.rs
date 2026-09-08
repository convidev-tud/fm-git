use crate::model::*;
use crate::repository::Repository;
use crate::vcs::VCS;
use indextree::{Arena, Node, NodeId};
use itertools::Itertools;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use thiserror::Error;
use uuid::Uuid;
// ##########
// # Errors #
// ##########

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
pub enum StructureViewError<V: VCS> {
    #[error(transparent)]
    PathDoesNotExist(#[from] PathDoesNotExistError<V>),
    #[error(transparent)]
    InvalidType(#[from] InvalidTypeError<V>),
}

// ###################
// # Main Definition #
// ###################

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
    _access_mode: PhantomData<M>,
    _type_marker: PhantomData<S>,
}

impl<'a, S, M, V> StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    fn get_arena(&self) -> &Arena<RefCell<NodeData<V>>> {
        self.get_repo().get_arena()
    }

    fn iter_path_towards_root(&self) -> impl Iterator<Item = NodeId> {
        self.get_node_id().ancestors(self.get_arena())
    }

    fn get_id_path(&self) -> Vec<NodeId> {
        let mut v = self.iter_path_towards_root().collect::<Vec<NodeId>>();
        v.reverse();
        v
    }

    fn get_node_path(&self) -> Vec<&Node<RefCell<NodeData<V>>>> {
        self.get_id_path()
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

    fn path_to_frozen(&self) -> Vec<FrozenNode> {
        let mut path = self.iter_path_towards_root().collect_vec();
        path.reverse();
        path.iter()
            .map(|id| {
                let node = self.repo.get_node(*id).unwrap().get().borrow();
                FrozenNode::new(node.get_name().clone(), node.get_type().clone())
            })
            .collect()
    }

    fn check_path_not_existent(&self) -> Result<(), PathDoesNotExistError<V>> {
        if &self.get_real_type() == &NodeType::NonExistent {
            let path = FrozenView::new(self.path_to_frozen(), RevisionPointer::None);
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
        node.reference_structure_view();
        if node.is_structure_locked() {
            drop(node);
            panic!(
                "Cannot create structure view for '{}': node is locked",
                self.to_normalized_path(),
            )
        }
        if M::lock() {
            let referenced = node.structure_views_referenced() - 1;
            if referenced > 0 {
                drop(node);
                panic!(
                    "Cannot lock node for structure view '{}': there are {referenced} other structure views referencing it",
                    self.to_normalized_path(),
                )
            }
            node.lock_structure();
        }
    }

    fn convert_access<M2: AccessMode>(self) -> StructureView<'a, S, M2, V> {
        let id = self.id;
        let repo = self.repo;
        drop(self);
        StructureView::new(id, repo).unwrap()
    }

    pub(crate) fn new(id: NodeId, repo: &'a Repository<V>) -> Result<Self, StructureViewError<V>> {
        let new = Self {
            id,
            repo,
            _access_mode: PhantomData,
            _type_marker: PhantomData,
        };
        new.assert_lock();
        new.check_path_not_existent()?;
        new.check_sym_type_compatibility()?;
        Ok(new)
    }

    pub(crate) fn get_node(&self) -> &Node<RefCell<NodeData<V>>> {
        self.repo.get_node(self.get_node_id()).unwrap()
    }

    pub(crate) fn get_repo(&self) -> &Repository<V> {
        self.repo
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
            _access_mode: PhantomData,
            _type_marker: PhantomData,
        };
        new.check_sym_type_compatibility()?;
        Ok(new)
    }

    pub fn convert_to_any_type(self) -> StructureView<'a, AnyType<AnyC>, M, V> {
        self.try_convert_to().unwrap()
    }

    pub fn to_frozen_view(&self, revision: RevisionPointer<V>) -> FrozenView<V> {
        FrozenView::new(self.path_to_frozen(), revision)
    }

    pub fn iter_children(
        &self,
        repo: &'a Repository<V>,
    ) -> impl Iterator<Item = StructureView<'a, AnyType<AnyC>, Read, V>> {
        let id = self.get_node_id();
        id.children(self.get_repo().get_arena())
            .map(|child| StructureView::new(child, repo).unwrap())
    }

    pub fn iter_children_req(
        &self,
        repo: &'a Repository<V>,
    ) -> impl Iterator<Item = StructureView<AnyType<AnyC>, Read, V>> {
        let id = self.get_node_id();
        id.descendants(self.get_repo().get_arena())
            .skip(1)
            .map(|child| StructureView::new(child, repo).unwrap())
    }

    /// Moves path to a specific index on the node vector.
    pub fn move_to_index<To: SymbolicNodeType>(
        self,
        index: usize,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, To, M, V>, StructureViewError<V>> {
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
        path: impl AsRef<NormalizedPath>,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, To, M, V>, StructureViewError<V>> {
        fn make_error_node(name: String) -> FrozenNode {
            FrozenNode::new(name, NodeType::NonExistent)
        }

        // set path to absolute from root
        let path = self.to_normalized_path() + path.as_ref();
        let root = self.get_root_id();
        let mut current = Some(root);
        let mut id_vec: Vec<NodeId> = vec![root];
        let mut error_nodes: Vec<FrozenNode> = vec![];
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
                    FrozenNode::new(node.get_name().clone(), node.get_type().clone())
                })
                .collect::<Vec<_>>();
            nodes.extend(error_nodes);
            Err(PathDoesNotExistError::new(FrozenView::new(nodes, RevisionPointer::None)).into())
        }
    }

    fn move_to_guaranteed_type<To: SymbolicNodeType>(
        self,
        path: impl AsRef<NormalizedPath>,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, To, M, V>, PathDoesNotExistError<V>> {
        match self.move_to::<To>(path, repo) {
            Ok(v) => Ok(v),
            Err(StructureViewError::PathDoesNotExist(e)) => Err(e),
            _ => unreachable!(),
        }
    }

    pub fn formatted(&self, show_type: bool, colored: bool) -> String {
        self.to_frozen_view(RevisionPointer::None)
            .formatted(show_type, false, colored)
    }
}

impl<'a, S, V> StructureView<'a, S, Read, V>
where
    S: SymbolicNodeType,
    V: VCS,
{
    pub(crate) fn private_clone(&self) -> Self {
        Self::new(self.id, self.repo).unwrap()
    }

    pub fn clone(&self, repo: &'a Repository<V>) -> Self {
        Self::new(self.id, repo).unwrap()
    }
}

// ############################
// # Specific Implementations #
// ############################

impl<'a, S, M, V> StructureView<'a, S, M, V>
where
    S: IsConcrete,
    M: AccessMode,
    V: VCS,
{
    pub fn get_vcs_id(&self) -> Uuid {
        self.get_node()
            .get()
            .borrow()
            .get_head()
            .unwrap()
            .get_id()
    }
}

impl<'a, S, V> StructureView<'a, S, Read, V>
where
    S: IsConcrete,
    V: VCS,
{
    pub fn lock(self) -> StructureView<'a, S, ReadWrite, V> {
        self.convert_access()
    }

    pub fn head(self) -> RevisionView<'a, S, Head, Read, V> {
        RevisionView::<'a, S, Head, Read, V>::new(self)
    }

    pub fn rev(
        self,
        revision: impl Into<String>,
    ) -> Result<RevisionView<'a, S, Rev, Read, V>, RevisionError<V, V::VCSError>> {
        RevisionView::<'a, S, Rev, Read, V>::new(self, revision)
    }
}

impl<'a, S, V> StructureView<'a, S, ReadWrite, V>
where
    S: IsConcrete,
    V: VCS,
{
    pub fn unlock(self) -> StructureView<'a, S, Read, V> {
        self.convert_access()
    }
}

impl<'a, T, M, V> StructureView<'a, T, M, V>
where
    T: UnderChannel,
    M: AccessMode,
    V: VCS,
{
    pub fn move_to_parent_channel<C: NodeClassification>(
        self,
        repo: &'a Repository<V>,
    ) -> StructureView<'a, Channel<C>, M, V> {
        self.move_to_index(1, repo).unwrap()
    }
}

impl<'a, M: AccessMode, V: VCS> StructureView<'a, VirtualRoot, M, V> {
    pub fn move_to_channel<C: NodeClassification>(
        self,
        channel: impl AsRef<NormalizedPath>,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, Channel<C>, M, V>, StructureViewError<V>> {
        self.move_to(channel, repo)
    }
}

impl<'a, C, M, V> StructureView<'a, Channel<C>, M, V>
where
    C: NodeClassification,
    M: AccessMode,
    V: VCS,
{
    pub fn get_path_to_feature_root(&self) -> NormalizedPath {
        self.to_normalized_path() + &FEATURE_ROOT.to_normalized_path()
    }

    pub fn get_path_to_product_root(&self) -> NormalizedPath {
        self.to_normalized_path() + &PRODUCT_ROOT.to_normalized_path()
    }

    pub fn move_to_feature_root(
        self,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, FeatureRoot, M, V>, PathDoesNotExistError<V>> {
        let path = self.get_path_to_feature_root();
        self.move_to_guaranteed_type(&path, repo)
    }

    pub fn move_to_product_root(
        self,
        repo: &'a Repository<V>,
    ) -> Result<StructureView<'a, ProductRoot, M, V>, PathDoesNotExistError<V>> {
        let path = self.get_path_to_product_root();
        self.move_to_guaranteed_type(&path, repo)
    }
}

impl<'a, S, V> StructureView<'a, S, ReadWrite, V>
where
    S: CanBecomeConcrete,
    V: VCS,
{
    pub fn create_branch<T, R, M>(
        self,
        revision_view: &RevisionView<T, R, M, V>,
    ) -> StructureView<'a, S::Target, ReadWrite, V>
    where
        T: CanCreate<S::Target>,
        R: RevPointer,
        M: AccessMode,
    {
        self.get_vcs()
    }
}

impl<'a, S, V> StructureView<'a, S, ReadWrite, V>
where
    S: CanBecomeAbstract,
    V: VCS,
{
    pub fn delete_branch(self) -> StructureView<'a, S::Target, ReadWrite, V> {
        todo!()
    }
}

// #########################
// # Trait Implementations #
// #########################

impl<'a, S, M, V> ToNormalizedPath for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    fn to_normalized_path(&self) -> NormalizedPath {
        NormalizedPath::from_iter(
            self.get_node_path()
                .iter()
                .map(|n| n.get().borrow().get_name().clone()),
        )
    }
}

impl<'a, S, M, V> Display for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_normalized_path().to_string())
    }
}

impl<'a, S, M, V, T> PartialEq<T> for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
    T: ToNormalizedPath,
{
    fn eq(&self, other: &T) -> bool {
        self.to_normalized_path() == other.to_normalized_path()
    }
}

impl<'a, S, M, V> Eq for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
}

impl<'a, S, M, V> PartialOrd for StructureView<'a, S, M, V>
where
    S: SymbolicNodeType,
    M: AccessMode,
    V: VCS,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.to_normalized_path()
            .partial_cmp(&other.to_normalized_path())
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

// ############################
// # Transformers and Filters #
// ############################

pub struct FilterByType<T: SymbolicNodeType>(PhantomData<T>);

impl<T: SymbolicNodeType> FilterByType<T> {
    pub fn filter<S, M, V>(view: StructureView<S, M, V>) -> Option<StructureView<T, M, V>>
    where
        S: SymbolicNodeType,
        M: AccessMode,
        V: VCS,
    {
        match view.try_convert_to::<T>() {
            Ok(view) => Some(view),
            Err(_) => None,
        }
    }
}

// #########
// # Tests #
// #########

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::test_utils::prepare_repo;

    #[test]
    fn test_structure_view_move_without_errors() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let feature = root
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert_eq!(feature, "/main/feature/foo");
    }

    #[test]
    fn test_structure_view_move_wrong_path() {
        let repo = prepare_repo();
        let root = repo.root_view();
        match root.move_to::<Feature<Concrete>>("/main/nothing".normalize(), &repo) {
            Err(error) => match error {
                StructureViewError::PathDoesNotExist(error) => {
                    assert_eq!(error.path, "/main/nothing")
                }
                _ => panic!("Wrong error variant!"),
            },
            Ok(_) => panic!("Should have returned an error"),
        }
    }

    #[test]
    fn test_structure_view_move_wrong_type() {
        let repo = prepare_repo();
        let root = repo.root_view();
        match root.move_to::<Product<Concrete>>("/main/feature/foo".normalize(), &repo) {
            Err(error) => match error {
                StructureViewError::InvalidType(error) => {
                    assert_eq!(error.path, "/main/feature/foo")
                }
                _ => panic!("Wrong error variant!"),
            },
            Ok(_) => panic!("Should have returned an error"),
        }
    }

    #[test]
    #[should_panic(
        expected = "Cannot create structure view for '/main/feature/foo': node is locked"
    )]
    fn test_structure_view_lock_and_create_second() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let view = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        let _locked = view.lock();
        root.move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
    }

    #[test]
    #[should_panic(
        expected = "Cannot lock node for structure view '/main/feature/foo': there are 1 other structure views referencing it"
    )]
    fn test_structure_view_lock_while_other_exists() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let _view1 = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        let _view2 = root
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap()
            .lock();
    }

    #[test]
    fn test_structure_view_lock_and_unlock() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let view1 = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap()
            .lock();
        assert!(view1.get_node().get().borrow().is_structure_locked());
        let view1 = view1.unlock();
        assert!(!view1.get_node().get().borrow().is_structure_locked());
        let view2 = root
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert_eq!(view1, view2);
    }

    #[test]
    fn test_structure_view_count_increment() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let view1 = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert_eq!(
            view1.get_node().get().borrow().structure_views_referenced(),
            1
        );
        let view2 = root
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert_eq!(
            view1.get_node().get().borrow().structure_views_referenced(),
            2
        );
        assert_eq!(
            view2.get_node().get().borrow().structure_views_referenced(),
            2
        );
    }

    #[test]
    fn test_structure_view_count_decrement() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let view1 = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert_eq!(
            view1.get_node().get().borrow().structure_views_referenced(),
            1
        );
        let view2 = root
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert_eq!(
            view1.get_node().get().borrow().structure_views_referenced(),
            2
        );
        drop(view2);
        assert_eq!(
            view1.get_node().get().borrow().structure_views_referenced(),
            1
        );
    }

    #[test]
    fn test_structure_view_unlock_on_drop() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let view1 = root
            .clone(&repo)
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap()
            .lock();
        assert!(view1.get_node().get().borrow().is_structure_locked());
        drop(view1);
        let view2 = root
            .move_to::<Feature<Concrete>>("/main/feature/foo".normalize(), &repo)
            .unwrap();
        assert!(!view2.get_node().get().borrow().is_structure_locked());
    }

    #[test]
    fn test_structure_view_iterate_children() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let children = root
            .iter_children(&repo)
            .map(|p| p.to_string())
            .collect::<Vec<_>>();
        assert_eq!(children, vec!["/main"],)
    }

    #[test]
    fn test_structure_view_iterate_children_req() {
        let repo = prepare_repo();
        let root = repo.root_view();
        let children = root
            .iter_children_req(&repo)
            .map(|p| p.to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            children,
            vec![
                "/main",
                "/main/feature",
                "/main/feature/foo",
                "/main/feature/bar",
            ]
        )
    }
}
