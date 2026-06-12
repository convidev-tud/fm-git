use crate::model::*;
use crate::vcs::VCS;
use std::cell::RefCell;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CurrentPathError<V: VCS> {
    #[error(transparent)]
    WrongType(#[from] NodePathError),
    #[error("CurrentPathError::VCS")]
    VCS(#[source] V::VCSError),
}

pub struct Workspace<V: VCS> {
    virtual_root: NodePath<VirtualRoot, V>,
    vcs: Rc<RefCell<V>>,
}

/// Base implementation
impl<V: VCS> Workspace<V> {
    pub fn new(virtual_root: NodePath<VirtualRoot, V>, vcs: Rc<RefCell<V>>) -> Self {
        Self { virtual_root, vcs }
    }
}

/// VCS commands
impl<V: VCS> Workspace<V> {
    pub fn get_vcs(&self) -> &Rc<RefCell<V>> {
        &self.vcs
    }

    pub fn get_virtual_root(&self) -> &NodePath<VirtualRoot, V> {
        &self.virtual_root
    }

    pub fn get_current_path<T: IsConcrete>(&self) -> Result<NodePath<T, V>, CurrentPathError<V>> {
        let current = self.get_vcs().borrow().get_current_path()?;
        self.get_virtual_root().clone().move_to::<T>(&current)?.into()
    }

    pub fn format_status_msg(
        &self,
        current_path_message: String,
        extra_message: String,
        colored: bool,
    ) -> Result<String, V::VCSError> {
        self
            .get_vcs()
            .borrow()
            .format_status_message(
                current_path_message,
                extra_message,
                colored,
            )?
            .into()
    }

    pub fn commit(&self) { todo!() }
    
    pub fn view(&self) { todo!() }
}
