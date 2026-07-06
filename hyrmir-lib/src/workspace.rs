use crate::model::*;
use crate::repository::Repository;
use crate::vcs::{VCS, VCSError, VersionId};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum GetWorkSpaceError<V: VersionId, VE: VCSError> {
    #[error(transparent)]
    SemanticView(#[from] SemanticViewError<V>),
    #[error("There is no workspace attached at this path.")]
    NoWorkspace,
    #[error(transparent)]
    VCS(#[from] VE),
}

pub enum WorkspaceKind<'a, S: IsConcrete, V: VCS> {
    Head(Workspace<'a, S, Head, V>),
    Rev(Workspace<'a, S, Rev<V::VersionId>, V>),
}

impl<'a, S: IsConcrete, V: VCS> WorkspaceKind<'a, S, V> {
    pub fn get(
        path: PathBuf,
        repository: &'a Repository<V>,
    ) -> Result<WorkspaceKind<'a, S, V>, GetWorkSpaceError<V::VersionId, V::VCSError>> {
        if let Some(current) = repository.get_vcs().get_current_path(&path)? {
            let current_semantic_view = repository.get_view(current.get_path())?;
            match current.get_revision() {
                NormalizedRevision::Head => {
                    let current_view = current_semantic_view.head();
                    let new = Workspace::<S, Head, V> { current_view, path };
                    Ok(WorkspaceKind::Head(new))
                }
                NormalizedRevision::Revision(revision) => {
                    let current_view = current_semantic_view.rev(revision).unwrap();
                    let new = Workspace::<S, Rev<V::VersionId>, V> { current_view, path };
                    Ok(WorkspaceKind::Rev(new))
                }
            }
        } else {
            Err(GetWorkSpaceError::NoWorkspace)
        }
    }
}

#[derive(Debug)]
pub struct Workspace<'a, S: IsConcrete, R: RevPointer, V: VCS> {
    current_view: RevisionView<'a, S, R, V>,
    path: PathBuf,
}

/// Base implementation
impl<'a, S, R, V> Workspace<'a, S, R, V>
where
    S: IsConcrete,
    R: RevPointer,
    V: VCS,
{
    pub fn new() {
        todo!()
    }
}

/// VCS commands
impl<'a, S, R, V> Workspace<'a, S, R, V>
where
    S: IsConcrete,
    R: RevPointer,
    V: VCS,
{
    pub fn get_current_view(&self) -> &RevisionView<'a, S, R, V> {
        &self.current_view
    }

    pub fn get_vcs(&self) -> &V {
        &self.get_current_view().get_semantic_view().get_vcs()
    }

    pub fn mut_get_current_view(&mut self) -> &mut RevisionView<'a, S, R, V> {
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

    pub fn switch_to<T: IsConcrete, P: RevPointer>(
        self,
        view: RevisionView<'a, T, P, V>,
    ) -> Result<Workspace<'a, T, P, V>, V::VCSError> {
        let semantic = view.get_semantic_view();
        let id = semantic.get_id();
        self.get_vcs().switch_to_branch(id, semantic, &self.path)?;
        let new = Workspace {
            current_view: view,
            path: self.path,
        };
        Ok(new)
    }

    pub fn commit(&self) {
        todo!()
    }
}
