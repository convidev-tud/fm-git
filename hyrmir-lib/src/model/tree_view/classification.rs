use crate::model::{NodeClassification, TreeView, NormalizedPath, SymbolicNodeType};
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
impl<T: SymbolicNodeType<Classification=Concrete>> IsConcrete for T {}

/// Denotes that a [SymbolicNodeType] is abstract (without associated artifact).
///
/// Is automatically implemented if [Abstract] is used as parameter.
pub trait IsAbstract {}
impl<T: SymbolicNodeType<Classification=Abstract>> IsAbstract for T {}

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

// impl<S: ValidNodeType<Classification=Concrete>, V: VCS> NodePath<S, V> {
//     pub fn get_qualified_object(&self) -> String {
//         match &self.get_sym_type().get_version() {
//             VersionPointer::Head => self.get_object(),
//             VersionPointer::Version(_) => self.get_object(),
//         }
//     }
//     pub fn get_head(&self) -> CommitHash {
//         self.get_metadata().get_head().unwrap().clone()
//     }
//     pub fn get_version(&self) -> &VersionPointer {
//         &self.version
//     }
//     pub fn update_version(&mut self, head: VersionPointer) {
//         self.version = head;
//     }
//     pub fn to_normalized_path_with_version(&self) -> NormalizedPath {
//         let mut path = self.to_normalized_path();
//         path.set_version_appendix(Some(self.get_object()));
//         path
//     }
// }
