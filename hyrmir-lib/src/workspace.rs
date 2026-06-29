use crate::model::*;
use crate::repository::Repository;
use crate::vcs::VCS;

pub struct Workspace<'a, V: VCS> {
    repository: &'a Repository<V>,
}

/// Base implementation
impl<'a, V: VCS> Workspace<'a, V> {
    pub fn new(repository: &'a Repository<V>) -> Self {
        Self { repository }
    }
}

/// VCS commands
impl<'a, V: VCS> Workspace<'a, V> {
    pub fn get_vcs(&self) -> &V {
        &self.repository.get_vcs()
    }

    pub fn get_current_path_view<T: IsConcrete>(&self) -> Result<TreeView<T, V>, TreeViewError<V::VersionId>> {
        let current = self.get_vcs().get_current_path()?.get_path();
        Ok(self.repository.get_view(&current)?)
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
    
    pub fn switch_to(&self) { todo!() }
}
