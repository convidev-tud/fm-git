use std::error::Error;
use std::fmt::Debug;
use crate::model::NormalizedPath;

pub trait VCSError: Error {}

pub trait VersionId: Debug + Clone {
    fn new(id: impl Into<String>) -> Self;
    fn get_full_id(&self) -> &String;
    fn get_printable_id(&self) -> &String;
}

pub struct PathInfo<V: VersionId> {
    id: usize,
    path: NormalizedPath,
    version: V,
}

impl<V: VersionId> PathInfo<V> {
    pub fn new(id: usize, path: NormalizedPath, version: V) -> Self {
        Self { id, path, version }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_path(&self) -> &NormalizedPath {
        &self.path
    }

    pub fn get_version(&self) -> &V {
        &self.version
    }

}

pub trait VCS: Debug + Clone + PartialEq + Eq {
    type VCSError: VCSError;
    
    type VersionId: VersionId;

    fn get_current_path(&self) -> Result<PathInfo<Self::VersionId>, Self::VCSError>;

    fn iter_concrete_paths(&self) -> impl Iterator<Item = Result<PathInfo<Self::VersionId>, Self::VCSError>>;

    fn get_version(&self, version: &str) -> Result<Option<Self::VersionId>, Self::VCSError>;

    fn version_exists_on_path(&self, path: &NormalizedPath, version: &String) -> Result<bool, Self::VCSError>;

    fn iter_versions(&self, path: &NormalizedPath) -> impl Iterator<Item=Result<Self::VersionId, Self::VCSError>>;

    fn get_status_without_current_info(&self, colored: bool) -> Result<String, Self::VCSError>;

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
        let native_status = self.get_status_without_current_info(colored)?;
        let post_status_msg = if post_status_msg.is_empty() {
            "".to_string()
        } else {
            format!("\n{post_status_msg}")
        };
        Ok(format!("{current_path_msg}{pre_status_msg}{native_status}{post_status_msg}"))
    }
}