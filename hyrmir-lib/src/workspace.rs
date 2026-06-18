use crate::model::*;
use crate::vcs::VCS;

pub struct Workspace<V: VCS> {
    virtual_root: NodePath<VirtualRoot, V>,
    vcs: V,
}

/// Base implementation
impl<V: VCS> Workspace<V> {
    pub fn new(virtual_root: NodePath<VirtualRoot, V>, vcs: V) -> Self {
        Self { virtual_root, vcs }
    }
}

/// VCS commands
impl<V: VCS> Workspace<V> {
    pub fn get_vcs(&self) -> &V {
        &self.vcs
    }

    pub fn get_virtual_root(&self) -> &NodePath<VirtualRoot, V> {
        &self.virtual_root
    }

    pub fn get_current_path<T: IsConcrete>(&self) -> Result<NodePath<T, V>, VCSPathError<V, V::VCSError>> {
        let current = self.get_vcs().get_current_path()?;
        let root = self.get_virtual_root().clone();
        Ok(root.move_to::<T>(&current)?)
    }

    pub fn format_status_msg(
        &self,
        current_path_message: impl Into<String>,
        pre_status: impl Into<String>,
        post_status: impl Into<String>,
        colored: bool,
    ) -> Result<String, V::VCSError> {
        let status =self
            .get_vcs()
            .format_status_message(
                current_path_message.into(),
                pre_status.into(),
                post_status.into(),
                colored,
            )?;
        Ok(status)
    }

    pub fn commit(&self) { todo!() }
    
    pub fn view(&self) { todo!() }
}
