use crate::model::*;
use std::error::Error;
use std::fmt::Debug;
use std::path::Path;

pub trait VCSError: Error {}

pub trait RevisionId: Debug + Clone + PartialEq + Eq {
    fn get_full_id(&self) -> String;
    fn get_printable_id(&self) -> String;
}

pub struct BranchInfo<V: RevisionId> {
    branch: String,
    head: V,
}

impl<V: RevisionId> BranchInfo<V> {
    pub fn new(branch: impl Into<String>, head: V) -> Self {
        Self {
            branch: branch.into(),
            head,
        }
    }

    pub fn get_branch(&self) -> &String {
        &self.branch
    }

    pub fn get_head(&self) -> &V {
        &self.head
    }
}

pub trait VCS: Debug {
    type VCSError: VCSError;

    type RevisionId: RevisionId;

    fn get_local_branches(&self) -> Result<Vec<BranchInfo<Self::RevisionId>>, Self::VCSError>;

    fn get_current_branch(&self) -> Result<Option<String>, Self::VCSError>;

    fn revision_exists_on_branch(
        &self,
        branch: impl AsRef<str>,
        revision: impl AsRef<str>,
    ) -> Result<bool, Self::VCSError>;

    fn get_revision(
        &self,
        revision: impl AsRef<str>,
    ) -> Result<Option<Self::RevisionId>, Self::VCSError>;

    fn read_file_from_revision(
        &self,
        file: impl AsRef<Path>,
        revision: impl AsRef<str>,
    ) -> Result<String, Self::VCSError>;

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

    fn switch_to_branch(&self, branch: impl AsRef<str>) -> Result<String, Self::VCSError>;

    fn create_branch(&self, branch: impl AsRef<str>) -> Result<String, Self::VCSError>;
    
    fn rename_branch(&self, old: impl AsRef<str>, new: impl AsRef<str>) -> Result<String, Self::VCSError>;
    
    fn delete_branch(&self, branch: impl AsRef<str>) -> Result<String, Self::VCSError>;
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

        fn get_current_branch(&self) -> Result<Option<String>, Self::VCSError> {
            todo!()
        }

        fn get_local_branches(&self) -> Result<Vec<BranchInfo<Self::RevisionId>>, Self::VCSError> {
            let mut vec = vec![];
            for i in 0..self.paths.len() {
                vec.push(BranchInfo::new(
                    self.paths[i].clone(),
                    TestRevisionId::new(i),
                ))
            }
            Ok(vec)
        }

        fn get_revision(
            &self,
            version: impl AsRef<str>,
        ) -> Result<Option<Self::RevisionId>, Self::VCSError> {
            todo!()
        }

        fn revision_exists_on_branch(
            &self,
            branch: impl AsRef<str>,
            version: impl Into<String>,
        ) -> Result<bool, Self::VCSError> {
            todo!()
        }

        fn get_status_without_current_branch(
            &self,
            colored: bool,
        ) -> Result<String, Self::VCSError> {
            todo!()
        }

        fn switch_to_branch(&self, branch: impl AsRef<str>) -> Result<String, Self::VCSError> {
            todo!()
        }

        fn create_branch(&self, branch: impl AsRef<str>) -> Result<String, Self::VCSError> {
            todo!()
        }

        fn rename_branch(&self, old: impl AsRef<str>, new: impl AsRef<str>) -> Result<String, Self::VCSError> {
            todo!()
        }

        fn delete_branch(&self, branch: impl AsRef<str>) -> Result<String, Self::VCSError> {
            todo!()
        }
    }
}
