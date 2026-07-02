use crate::model::*;
use crate::repository::Repository;
use crate::vcs::{VCS, VCSError, VersionId};
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum WorkSpaceError<V: VersionId, VE: VCSError> {
    #[error(transparent)]
    TreeView(#[from] TreeViewError<V>),
    #[error("There is no workspace attached to this repository.")]
    NoWorkspace,
    #[error(transparent)]
    VCS(#[from] VE),
}

#[derive(Debug)]
pub struct Workspace<'a, S: IsConcrete, V: VCS> {
    current_view: RevisionView<'a, S, V>,
    repository: &'a Repository<V>,
}

/// Base implementation
impl<'a, S: IsConcrete, V: VCS> Workspace<'a, S, V> {
    pub fn new(
        repository: &'a Repository<V>,
    ) -> Result<Self, WorkSpaceError<V::VersionId, V::VCSError>> {
        if let Some(current) = repository.get_vcs().get_current_path()? {
            let current_path = repository.get_view(&current)?;
            Ok(Self {
                current_view: current_path,
                repository,
            })
        } else {
            Err(WorkSpaceError::NoWorkspace)
        }
    }
}

/// VCS commands
impl<'a, S: IsConcrete, V: VCS> Workspace<'a, S, V> {
    pub fn get_vcs(&self) -> &V {
        &self.repository.get_vcs()
    }

    pub fn get_current_view(&self) -> &RevisionView<'a, S, V> {
        &self.current_view
    }

    pub fn mut_get_current_view(&mut self) -> &mut RevisionView<'a, S, V> {
        &mut self.current_view
    }

    pub fn status(
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

    pub fn switch_to<T: IsConcrete>(
        self,
        path: RevisionView<'a, T, V>,
    ) -> Result<Workspace<'a, T, V>, V::VCSError> {
        let id = path.get_id();
        self.repository.get_vcs().switch_to_branch(id, &path)?;
        let new = Workspace {
            current_view: path,
            repository: self.repository,
        };
        Ok(new)
    }

    pub fn commit(&self) {
        todo!()
    }
}
