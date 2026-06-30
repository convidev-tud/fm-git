use crate::model::{NodeClassification, PathView, SymbolicNodeType};
use crate::vcs::VCS;

/// Defines a compatible [SymbolicNodeType] as concrete (with associated artifact).
///
/// The trait [IsConcrete] is automatically implemented.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Concrete;

/// Defines a [SymbolicNodeType] as abstract (without associated artifact).
///
/// The trait [IsAbstract] is automatically implemented if this is used as parameter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Abstract;

/// Placeholder if a concretized classification ([Concrete] or [Abstract]) does not matter or is impossible.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyCls;

/// Denotes that a [SymbolicNodeType] is concrete (with associated artifact).
///
/// Is automatically implemented if the type uses [Concrete] as parameter.
pub trait IsConcrete: SymbolicNodeType {}
impl<T: SymbolicNodeType<Classification = Concrete>> IsConcrete for T {}

/// Denotes that a [SymbolicNodeType] is abstract (without associated artifact).
///
/// Is automatically implemented if [Abstract] is used as parameter.
pub trait IsAbstract {}
impl<T: SymbolicNodeType<Classification = Abstract>> IsAbstract for T {}

impl NodeClassification for Concrete {
    fn requires_artifact() -> Option<bool> {
        Some(true)
    }
}

impl NodeClassification for Abstract {
    fn requires_artifact() -> Option<bool> {
        Some(false)
    }
}

impl NodeClassification for AnyCls {
    fn requires_artifact() -> Option<bool> {
        None
    }
}

impl<'a, S: IsConcrete, V: VCS> PathView<'a, S, V> {
    pub fn get_id(&self) -> usize {
        self.get_node().borrow().get_branch_info().unwrap().get_id()
    }
}
