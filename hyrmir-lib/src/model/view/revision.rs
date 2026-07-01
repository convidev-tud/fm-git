use std::cell::RefCell;
use std::rc::Rc;
use crate::model::{AnyC, AnyType, InvalidTypeError, Node, NodeHolder, NodeType, NormalizedPath, PathDoesNotExistError, SymbolicNodeType, ToNormalizedPath, TreeViewError, VersionPointer};
use crate::repository::Repository;
use crate::vcs::VCS;

/// Semantic view onto the path model.
///
/// This struct symbolizes a path in the tree model and is the primary interface to interact with the underlying VCS repository.
/// Its capabilities are defined by:
/// - the type of node it points to ([SymbolicNodeType] parameter),
/// - the VCS implementation ([VCS] parameter).
#[derive(Debug)]
pub struct RevisionView<'a, S: SymbolicNodeType, V: VCS> {
    path: Vec<Rc<RefCell<Node<V::VersionId>>>>,
    sym_type: S,
    repo: &'a Repository<V>,
}

/// Construction and transformation
impl<'a, S: SymbolicNodeType, V: VCS> RevisionView<'a, S, V> {
    pub fn test(&mut self) {}

    pub(crate) fn new(
        path: Vec<Rc<RefCell<Node<V::VersionId>>>>,
        repo: &'a Repository<V>,
    ) -> Result<RevisionView<'a, S, V>, TreeViewError<V::VersionId>> {
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
    ) -> Result<RevisionView<'a, To, V>, InvalidTypeError> {
        let new = RevisionView {
            path: self.path.clone(),
            repo: self.repo,
            sym_type: To::new(),
        };
        new.lock_node();
        let new = new.check_sym_type_compatibility()?;
        Ok(new)
    }

    pub fn convert_to_any(self) -> RevisionView<'a, AnyType<AnyC>, V> {
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
impl<'a, S: SymbolicNodeType, V: VCS> RevisionView<'a, S, V> {
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
impl<'a, S: SymbolicNodeType, V: VCS> RevisionView<'a, S, V> {
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
impl<'a, S: SymbolicNodeType, V: VCS> RevisionView<'a, S, V> {
    pub fn get_at_index<To: SymbolicNodeType>(
        &self,
        index: usize,
        repo: &'a Repository<V>,
    ) -> Result<RevisionView<'a, To, V>, TreeViewError<V::VersionId>> {
        let path = self.path[0..index + 1].to_vec();
        Ok(RevisionView::<'a, To, V>::new(path, repo)?)
    }

    /// Moves path to a specific index of the node vector.
    pub fn move_to_index<To: SymbolicNodeType>(
        self,
        index: usize,
        repo: &'a Repository<V>,
    ) -> Result<RevisionView<'a, To, V>, TreeViewError<V::VersionId>> {
        self.get_at_index(index, repo)
    }

    pub fn get<To: SymbolicNodeType>(
        &self,
        path: &impl ToNormalizedPath,
        repo: &'a Repository<V>,
    ) -> Result<RevisionView<'a, To, V>, TreeViewError<V::VersionId>> {
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
    ) -> Result<RevisionView<'a, To, V>, TreeViewError<V::VersionId>> {
        drop(self);
        repo.get_view(path)
    }
}

/// Display and pretty printing
impl<'a, S: SymbolicNodeType, V: VCS> RevisionView<'a, S, V> {
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

impl<'a, S: SymbolicNodeType, V: VCS> NodeHolder<V::VersionId> for RevisionView<'a, S, V> {
    fn get_node(&self) -> &Rc<RefCell<Node<V::VersionId>>> {
        &self.path.last().unwrap()
    }
}

impl<'a, S: SymbolicNodeType, V: VCS> Drop for RevisionView<'a, S, V> {
    fn drop(&mut self) {
        self.get_node().borrow_mut().unlock()
    }
}

impl<'a, T: SymbolicNodeType, V: VCS> ToNormalizedPath for RevisionView<'a, T, V> {
    fn to_normalized_path(&self) -> NormalizedPath {
        self.path.to_normalized_path()
    }
}

impl<'a, S: SymbolicNodeType, V: VCS> PartialEq for RevisionView<'a, S, V> {
    fn eq(&self, other: &Self) -> bool {
        self.to_normalized_path() == other.to_normalized_path()
    }
}

impl<'a, S: SymbolicNodeType, V: VCS> Eq for RevisionView<'a, S, V> {}