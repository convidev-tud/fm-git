use crate::model::SymbolicNodeType;
use crate::vcs::VCS;

pub trait WorkspaceBase {}

pub struct Workspace<S: SymbolicNodeType, V: VCS>;

