use std::cell::RefCell;
use crate::model::{AnyCls, InvalidNodeTypeError, Node, NodePath, NodeType, SymbolicNodeType, ValidNodeType, VersionPointer};
use crate::vcs::VCS;
use std::error::Error;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodePathError {
    #[error(transparent)]
    InvalidType(#[from] InvalidNodeTypeError),
    #[error("")]
    InvalidVersion(VersionPointer),
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
        vcs: Rc<RefCell<V>>,
        error: NodePathError,
    ) -> Self {
        Self {
            path,
            sym_type: ErrorState { error },
            vcs,
        }
    }
}

impl<V: VCS> Error for NodePath<ErrorState, V> {}