use crate::model::*;
use std::error::Error;
use std::fmt::Debug;
use std::path::PathBuf;
use uuid::Uuid;

pub trait VCSError: Error {}

pub trait RevisionId: Debug + Clone + PartialEq + Eq {
    fn get_full_id(&self) -> String;
    fn get_printable_id(&self) -> String;
}

pub struct PathInfo<V: RevisionId> {
    id: Uuid,
    path: NormalizedPath,
    version: V,
}

impl<V: RevisionId> PathInfo<V> {
    pub fn new(id: Uuid, path: impl Into<NormalizedPath>, version: V) -> Self {
        Self {
            id,
            path: path.into(),
            version,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_path(&self) -> &NormalizedPath {
        &self.path
    }

    pub fn get_head(&self) -> &V {
        &self.version
    }
}

pub trait VCS: Debug {
    type VCSError: VCSError;

    type RevisionId: RevisionId;

    fn get_current_path(&self) -> Result<Option<Normalized>, Self::VCSError>;

    fn get_local_paths(&self) -> Result<Vec<PathInfo<Self::RevisionId>>, Self::VCSError>;

    fn get_revision(
        &self,
        version: impl Into<String>,
    ) -> Result<Option<Self::RevisionId>, Self::VCSError>;

    fn revision_exists_on_path(
        &self,
        path: &NormalizedPath,
        version: impl Into<String>,
    ) -> Result<bool, Self::VCSError>;

    fn iter_versions(
        &self,
        path: &NormalizedPath,
    ) -> impl Iterator<Item = Result<Self::RevisionId, Self::VCSError>>;

    fn get_status_without_current_branch(&self, colored: bool) -> Result<String, Self::VCSError>;

    fn format_status_message(
        &self,
        current_path_msg: String,
        pre_status_msg: String,
        post_status_msg: String,
        colored: bool,
    ) -> Result<String, Self::VCSError> {
        let current_path_msg = if current_path_msg.is_empty() {
            "".to_string()
        } else {
            format!("{current_path_msg}\n")
        };
        let pre_status_msg = if pre_status_msg.is_empty() {
            "".to_string()
        } else {
            format!("{pre_status_msg}\n")
        };
        let native_status = self.get_status_without_current_branch(colored)?;
        let post_status_msg = if post_status_msg.is_empty() {
            "".to_string()
        } else {
            format!("\n{post_status_msg}")
        };
        Ok(format!(
            "{current_path_msg}{pre_status_msg}{native_status}{post_status_msg}"
        ))
    }

    fn switch_to_branch(&self, id: Uuid, path: &impl ToNormalizedPath) -> Result<String, Self::VCSError>;

    fn create_branch(&self, uuid: Uuid, path: impl AsRef<NormalizedPath>) -> Result<String, Self::VCSError>;
    
    fn rename_branch(&self, uuid: Uuid, new_path: impl AsRef<NormalizedPath>) -> Result<String, Self::VCSError>;
    
    fn delete_branch(
        &self, uuid: Uuid) -> Result<String, Self::VCSError>;
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use thiserror::Error;

    #[derive(Error, Debug)]
    #[error("Test Error")]
    pub struct TestVCSError;

    impl VCSError for TestVCSError {}

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct TestRevisionId {
        id: usize,
    }

    impl TestRevisionId {
        pub fn new(id: usize) -> Self {
            Self { id }
        }
    }

    impl RevisionId for TestRevisionId {
        fn get_full_id(&self) -> String {
            todo!()
        }

        fn get_printable_id(&self) -> String {
            todo!()
        }
    }

    #[derive(Debug)]
    pub struct TestVCS {
        pub paths: Vec<String>,
    }

    impl TestVCS {
        pub fn new() -> Self {
            let paths = vec!["/main", "/main/feature/foo", "/main/feature/bar"];
            Self {
                paths: paths.into_iter().map(String::from).collect(),
            }
        }
    }

    impl VCS for TestVCS {
        type VCSError = TestVCSError;
        type RevisionId = TestRevisionId;

        fn get_current_path(&self, path: &PathBuf) -> Result<Option<Normalized>, Self::VCSError> {
            todo!()
        }

        fn get_local_paths(&self) -> Result<Vec<PathInfo<Self::RevisionId>>, Self::VCSError> {
            let mut vec = vec![];
            for i in 0..self.paths.len() {
                vec.push(PathInfo::new(
                    i,
                    self.paths[i].clone(),
                    TestRevisionId::new(i),
                ))
            }
            Ok(vec)
        }

        fn get_revision(
            &self,
            version: impl Into<String>,
        ) -> Result<Option<Self::RevisionId>, Self::VCSError> {
            todo!()
        }

        fn revision_exists_on_path(
            &self,
            path: &NormalizedPath,
            version: impl Into<String>,
        ) -> Result<bool, Self::VCSError> {
            todo!()
        }

        fn iter_versions(
            &self,
            path: &NormalizedPath,
        ) -> impl Iterator<Item = Result<Self::RevisionId, Self::VCSError>> {
            vec![].into_iter()
        }

        fn get_status_without_current_branch(
            &self,
            colored: bool,
        ) -> Result<String, Self::VCSError> {
            todo!()
        }

        fn switch_to_branch(
            &self,
            id: usize,
            path: &impl ToNormalizedPath,
            dir: &PathBuf,
        ) -> Result<String, Self::VCSError> {
            todo!()
        }
    }
}
