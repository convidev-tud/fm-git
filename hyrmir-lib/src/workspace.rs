use crate::model::*;
use crate::repository::Repository;
use crate::vcs::{VCS, VCSError};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Clone, Debug)]
pub enum GetWorkSpaceError<V: VCS, VE: VCSError> {
    #[error(transparent)]
    View(#[from] SemanticViewError<V>),
    #[error("There is no workspace attached at this path.")]
    NoWorkspace,
    #[error(transparent)]
    VCS(#[from] VE),
}

#[derive(Debug)]
pub struct Workspace<'a, S, R, M, V>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
    V: VCS,
{
    current_view: RevisionView<'a, S, R, M, V>,
    path: PathBuf,
}

impl<'a, S, R, M, V> Workspace<'a, S, R, M, V>
where
    S: IsConcrete,
    R: RevPointer,
    M: AccessMode,
    V: VCS,
{
    pub fn get_current_view(&self) -> &RevisionView<'a, S, R, M, V> {
        &self.current_view
    }

    pub fn mut_get_current_view(&mut self) -> &mut RevisionView<'a, S, R, M, V> {
        &mut self.current_view
    }

    pub fn get_vcs(&self) -> &V {
        &self.get_current_view().get_structure_view().get_vcs()
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

    pub fn switch_to<T, P>(
        self,
        view: RevisionView<'a, T, P, M, V>,
    ) -> Result<Workspace<'a, T, P, M, V>, V::VCSError>
    where
        T: IsConcrete,
        P: RevPointer,
        M: AccessMode,
    {
        let structure = view.get_structure_view();
        let id = structure.get_vcs_id();
        self.get_vcs().switch_to_branch(id, structure, &self.path)?;
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

impl<'a, S, V> Workspace<'a, S, Rev, Shared, V>
where
    S: IsConcrete,
    V: VCS,
{
    pub(crate) fn new(
        path: PathBuf,
        repository: &'a Repository<V>,
    ) -> Result<Self, GetWorkSpaceError<V, V::VCSError>> {
        if let Some(current) = repository.get_vcs().get_current_path(&path)? {
            let current_semantic_view = repository
                .root_view()
                .move_to(current.get_path(), repository)?;
            let current_view = match current.get_revision() {
                NormalizedRevision::None => current_semantic_view.head().convert_to_rev(),
                NormalizedRevision::Revision(revision) => {
                    current_semantic_view.rev(revision).unwrap()
                }
            };
            Ok(Workspace::<S, Rev, Shared, V> { current_view, path })
        } else {
            Err(GetWorkSpaceError::NoWorkspace)
        }
    }
}
