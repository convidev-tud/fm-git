use colored::Colorize;
use crate::model::{NodeClassification, NodePath, NormalizedPath, SymbolicNodeType};
use crate::vcs::VCS;

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum VersionPointer {
    Head,
    Version(String),
}

impl VersionPointer {
    fn formatted(&self, colored: bool, current_head: String) -> String {
        fn make_head_info(head: &CommitHash) -> String {
            format!("(Head -> {head})")
        }

        let info = if colored {
            match self {
                Self::Head => make_head_info(&current_head).yellow(),
                Self::Version(c) => {
                    if c == &current_head {
                        make_head_info(&current_head).yellow()
                    } else {
                        format!("({})", c.get_short_hash()).yellow()
                    }
                }
                Self::Tag(tag) => format!("({})", tag).green(),
            }
        } else {
            match self {
                Self::Head => make_head_info(&current_head).normal(),
                Self::Version(c) => {
                    if c == &current_head {
                        make_head_info(&current_head).normal()
                    } else {
                        format!("({})", c.get_short_hash()).normal()
                    }
                }
                Self::Tag(tag) => format!("({})", tag).green().normal(),
            }
        };
        info.to_string()
    }
}

/// Defines a compatible [SymbolicNodeType] as concrete (with associated artifact).
///
/// The trait [IsConcrete] is automatically implemented.
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
pub struct AnyCls;

/// Denotes that a [SymbolicNodeType] is concrete (with associated artifact).
///
/// Is automatically implemented if the type uses [Concrete] as parameter.
pub trait IsConcrete: SymbolicNodeType {
    fn get_version(&self) -> &VersionPointer;
    fn set_version(&mut self, version: VersionPointer);
}
impl<T: SymbolicNodeType<Classification=Concrete>> IsConcrete for T {
    fn get_version(&self) -> &VersionPointer {
        todo!()
    }

    fn set_version(&mut self, version: VersionPointer) {
        todo!()
    }
}

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

impl<S: SymbolicNodeType<Classification=Concrete>, V: VCS> NodePath<S, V> {
    pub fn get_qualified_object(&self) -> String {
        match &self.get_sym_type().get_version() {
            VersionPointer::Head => self.get_object(),
            VersionPointer::Version(_) => self.get_object(),
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
