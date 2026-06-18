use crate::model::{Node, NodePath, NodeType, SymbolicNodeType, VersionPointer};
use crate::vcs::VCS;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, Hash)]
pub struct InvalidSymTypeError {
    types_possible: Vec<NodeType>,
    type_found: NodeType,
}

impl InvalidSymTypeError {
    pub fn new(types_possible: Vec<NodeType>, type_found: NodeType) -> Self {
        Self {
            types_possible,
            type_found,
        }
    }
}

impl Display for InvalidSymTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Error, Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodePathError {
    #[error(transparent)]
    InvalidSymType(#[from] InvalidSymTypeError),
    #[error("")]
    VersionNotSupported,
    #[error("")]
    VersionNotOnPath,
    #[error("")]
    DoesNotExist,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ErrorState {
    error: NodePathError,
}

impl SymbolicNodeType for ErrorState {}

impl<V: VCS> NodePath<ErrorState, V> {
    pub(crate) fn new(
        path: Vec<Rc<RefCell<Node>>>,
        vcs: V,
        version: VersionPointer,
        error: NodePathError,
    ) -> Self {
        Self {
            path,
            vcs,
            sym_type: ErrorState { error },
            version_pointer: version,
        }
    }
}

impl<V: VCS> Display for NodePath<ErrorState, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<V: VCS> Error for NodePath<ErrorState, V> {}