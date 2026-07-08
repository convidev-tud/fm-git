use crate::model::*;
use crate::repository::Repository;
use crate::vcs::{VersionId, VCS};
use itertools::Itertools;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub struct PathDoesNotExistError<V: VersionId> {
    path: DynamicView<V>,
}

impl<V: VersionId> PathDoesNotExistError<V> {
    pub fn new(path: DynamicView<V>) -> Self {
        Self { path }
    }
}

impl<V: VersionId> Display for PathDoesNotExistError<V> {
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
pub struct InvalidTypeError<V: VersionId> {
    types_possible: Vec<NodeType>,
    type_found: NodeType,
    path: DynamicView<V>,
}

impl<V: VersionId> InvalidTypeError<V> {
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

impl<V: VersionId> Display for InvalidTypeError<V> {
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
pub enum SemanticViewError<V: VersionId> {
    #[error(transparent)]
    PathDoesNotExist(#[from] PathDoesNotExistError<V>),
    #[error(transparent)]
    InvalidType(#[from] InvalidTypeError<V>),
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
    repo: &'a Repository<V>,
    _sym_marker: PhantomData<S>,
}

/// Construction and transformation
impl<'a, S: SymbolicNodeType, V: VCS> SemanticView<'a, S, V> {
    pub(crate) fn new(
        path: Vec<Rc<RefCell<Node<V::VersionId>>>>,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, S, V>, SemanticViewError<V::VersionId>> {
        let new = Self {
            path,
            repo,
            _sym_marker: PhantomData,
        };
        let new = new
            .check_path_not_existent()?
            .check_sym_type_compatibility()?;
        Ok(new)
    }

    pub fn try_convert_to<To: SymbolicNodeType>(
        self,
    ) -> Result<SemanticView<'a, To, V>, InvalidTypeError<V::VersionId>> {
        let new = SemanticView {
            path: self.path.clone(),
            repo: self.repo,
            _sym_marker: PhantomData,
        };
        let new = new.check_sym_type_compatibility()?;
        Ok(new)
    }

    pub fn convert_to_any_type(self) -> SemanticView<'a, AnyType<AnyC>, V> {
        self.try_convert_to().unwrap()
    }

    pub fn to_dynamic_view(&self) -> DynamicView<V::VersionId> {
        DynamicView::new(self.path.clone(), RevisionPointer::Head)
    }

    fn check_path_not_existent(self) -> Result<Self, PathDoesNotExistError<V::VersionId>> {
        if &self.get_real_type() == &NodeType::NonExistent {
            let path = DynamicView::new(self.path.clone(), RevisionPointer::Head);
            Err(PathDoesNotExistError::new(path))
        } else {
            Ok(self)
        }
    }

    fn check_sym_type_compatibility(self) -> Result<Self, InvalidTypeError<V::VersionId>> {
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

    pub fn get_child(
        &self,
        name: &str,
    ) -> Result<DynamicView<V::VersionId>, PathDoesNotExistError<V::VersionId>> {
        let mut path = self.path.clone();
        if let Some(child) = self.get_node().borrow().get_child(name) {
            path.push(child);
            Ok(DynamicView::new(path, RevisionPointer::Head))
        } else {
            path.push(Rc::new(RefCell::new(Node::new(
                name.to_string(),
                NodeType::NonExistent,
                None,
            ))));
            Err(PathDoesNotExistError::new(DynamicView::new(
                path,
                RevisionPointer::Head,
            )))
        }
    }

    pub fn has_children(&self) -> bool {
        self.get_node().borrow().has_children()
    }
}

/// Iterators
impl<'a, S: SymbolicNodeType, V: VCS> SemanticView<'a, S, V> {
    pub fn iter_children(&self) -> impl Iterator<Item = SemanticView<'a, AnyType<AnyC>, V>> {
        let dynamic = self.to_dynamic_view();
        dynamic
            .iter_children()
            .map(move |v| SemanticView::<AnyType<AnyC>, V>::new(
                v.get_path().clone(),
                self.repo,
            ).unwrap())
    }

    pub fn iter_children_req(&self) -> impl Iterator<Item = SemanticView<'a, AnyType<AnyC>, V>> {
        let dynamic = self.to_dynamic_view();
        dynamic
            .iter_children_req()
            .map(move |v| SemanticView::<AnyType<AnyC>, V>::new(
                v.get_path().clone(),
                self.repo,
            ).unwrap())
    }
}

/// Path pointer movement
impl<'a, S: SymbolicNodeType, V: VCS> SemanticView<'a, S, V> {
    pub fn get_at_index<To: SymbolicNodeType>(
        &self,
        index: usize,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, To, V>, SemanticViewError<V::VersionId>> {
        let path = self.path[0..index + 1].to_vec();
        Ok(SemanticView::<'a, To, V>::new(path, repo)?)
    }

    /// Moves path to a specific index of the node vector.
    pub fn move_to_index<To: SymbolicNodeType>(
        self,
        index: usize,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, To, V>, SemanticViewError<V::VersionId>> {
        self.get_at_index(index, repo)
    }

    pub fn get<To: SymbolicNodeType>(
        &self,
        path: &impl ToNormalizedPath,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, To, V>, SemanticViewError<V::VersionId>> {
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
    ) -> Result<SemanticView<'a, To, V>, SemanticViewError<V::VersionId>> {
        drop(self);
        repo.get_view(path)
    }
}

/// Display and pretty printing
impl<'a, S: SymbolicNodeType, V: VCS> SemanticView<'a, S, V> {
    // pub fn display_tree(&self, show_tags: bool) -> String {
    //     self.get_node().borrow().display_tree(show_tags)
    // }

    pub fn formatted(&self, show_type: bool, show_version: bool, colored: bool) -> String {
        self.to_dynamic_view()
            .formatted(show_type, show_version, colored)
    }
}

impl<'a, S: SymbolicNodeType, V: VCS> NodeHolder<V::VersionId> for SemanticView<'a, S, V> {
    fn get_node(&self) -> &Rc<RefCell<Node<V::VersionId>>> {
        &self.path.last().unwrap()
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

/// Reachability for virtual root
impl<'a, V: VCS> SemanticView<'a, VirtualRoot, V> {
    pub fn get_area<C: NodeClassification>(
        &self,
        area: &impl ToNormalizedPath,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, Area<C>, V>, SemanticViewError<V::VersionId>> {
        self.get(area, repo)
    }

    pub fn move_to_area<C: NodeClassification>(
        self,
        area: &impl ToNormalizedPath,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, Area<C>, V>, SemanticViewError<V::VersionId>> {
        self.move_to(area, repo)
    }
}

impl<'a, C: NodeClassification, V: VCS> SemanticView<'a, Area<C>, V> {
    pub fn get_path_to_feature_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(FEATURE_ROOT)
    }

    pub fn get_path_to_product_root(&self) -> NormalizedPath {
        self.to_normalized_path() + NormalizedPath::from(PRODUCT_ROOT)
    }

    pub fn move_to_feature_root(
        self,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, FeatureRoot, V>, SemanticViewError<V::VersionId>> {
        Ok(self.move_to(&NormalizedPath::from(FEATURE_ROOT), repo)?)
    }

    pub fn move_to_product_root(
        self,
        repo: &'a Repository<V>,
    ) -> Result<SemanticView<'a, ProductRoot, V>, SemanticViewError<V::VersionId>> {
        Ok(self.move_to(&NormalizedPath::from(PRODUCT_ROOT), repo)?)
    }
}

impl<'a, S: IsConcrete, V: VCS> SemanticView<'a, S, V> {
    pub fn get_id(&self) -> usize {
        self.get_node().borrow().get_branch_info().unwrap().get_id()
    }

    pub fn assert_revision(
        &self,
        revision: impl Into<String>,
    ) -> Result<V::VersionId, RevisionError<V::VersionId, V::VCSError>> {
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
    ) -> Result<RevisionView<'a, S, Rev, V>, RevisionError<V::VersionId, V::VCSError>>
    {
        RevisionView::<'a, S, Rev, V>::new(self, revision)
    }
}

impl<'a, T: UnderArea, V: VCS> SemanticView<'a, T, V> {
    pub fn get_area<C: NodeClassification>(
        &self,
        repo: &'a Repository<V>,
    ) -> SemanticView<'a, Area<C>, V> {
        self.get_at_index(1, repo).unwrap()
    }

    pub fn move_to_area<C: NodeClassification>(
        self,
        repo: &'a Repository<V>,
    ) -> SemanticView<'a, Area<C>, V> {
        self.move_to_index(1, repo).unwrap()
    }
}

pub struct FilterByType<T: SymbolicNodeType> {
    _marker: PhantomData<T>,
}

impl<T: SymbolicNodeType> FilterByType<T> {
    pub fn filter<S, V>(view: SemanticView<S, V>) -> Option<SemanticView<T, V>>
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
