use crate::model::{NodeClassification, SymbolicNodeType};

/// Defines a [SymbolicNodeType] as concrete (with associated artifact).
///
/// The trait [IsConcrete] is automatically implemented if this is used as parameter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Concrete {
    version_pointer: VersionPointer,
}

/// Defines a [SymbolicNodeType] as abstract (without associated artifact).
///
/// The trait [IsAbstract] is automatically implemented if this is used as parameter.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Abstract;

/// Placeholder if a concretized classification ([Concrete] or [Abstract]) does not matter or is impossible.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct AnyClassification;

/// Denotes that a [SymbolicNodeType] is abstract (without associated artifact).
///
/// Is automatically implemented if [Abstract] is used as parameter.
pub trait IsAbstract {}
impl<T: SymbolicNodeType<Classification=Abstract>> IsAbstract for T {}

impl NodeClassification for Concrete {}
impl NodeClassification for Abstract {}
impl NodeClassification for AnyClassification {}