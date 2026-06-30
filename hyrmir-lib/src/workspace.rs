use crate::model::*;
use crate::repository::Repository;
use crate::vcs::{VCS, VCSError, VersionId};
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum WorkSpaceError<V: VersionId, VE: VCSError> {
    #[error(transparent)]
    TreeView(#[from] TreeViewError<V>),
    #[error(transparent)]
    VCS(#[from] VE),
}

pub struct Workspace<'a, S: IsConcrete, V: VCS> {
    current_view: TreeView<'a, S, V>,
    repository: &'a Repository<V>,
}

/// Base implementation
impl<'a, S: IsConcrete, V: VCS> Workspace<'a, S, V> {
    pub fn new(
        repository: &'a Repository<V>,
    ) -> Result<Self, WorkSpaceError<V::VersionId, V::VCSError>> {
        let current = repository.get_vcs().get_current_path()?;
        let path = current.get_path();
        let current_path = repository.get_view(&path)?;
        Ok(Self {
            current_view: current_path,
            repository,
        })
    }
}

/// VCS commands
impl<'a, S: IsConcrete, V: VCS> Workspace<'a, S, V> {
    pub fn get_vcs(&self) -> &V {
        &self.repository.get_vcs()
    }

    pub fn get_current_view(&self) -> &TreeView<'a, S, V> {
        &self.current_view
    }

    pub fn mut_get_current_view(&mut self) -> &mut TreeView<'a, S, V> {
        &mut self.current_view
    }

    pub fn format_status_msg(
        &self,
        current_path_message: impl Into<String>,
        pre_status: impl Into<String>,
        post_status: impl Into<String>,
        colored: bool,
    ) -> Result<String, V::VCSError> {
        let status = self.get_vcs().format_status_message(
            current_path_message.into(),
            pre_status.into(),
            post_status.into(),
            colored,
        )?;
        Ok(status)
    }

    pub fn commit(&self) {
        todo!()
    }

    pub fn switch_to(&self) {
        todo!()
    }
}
