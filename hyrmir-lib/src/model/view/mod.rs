use thiserror::Error;

mod structural;
mod revision;
mod fuzzy;



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
