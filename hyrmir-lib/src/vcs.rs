use std::error::Error;
use std::fmt::Debug;
use crate::model::NormalizedPath;

pub struct PathInfo {
    id: usize,
    path: NormalizedPath,
}

impl PathInfo {
    pub fn new(id: usize, path: NormalizedPath) -> Self {
        Self { id, path }
    }
    
    pub fn get_id(&self) -> usize {
        self.id
    }
    
    pub fn get_path(&self) -> &NormalizedPath {
        &self.path
    }
    
}

pub trait VCSError: Error {}

pub trait VersionObject: Debug {
    type VersionError: Error;
    
    fn get_metadata(&self, key: String) -> Result<String, Self::VersionError>;
}

pub trait VCS: Debug + Clone + PartialEq + Eq {
    type VCSError: VCSError;
    
    type VersionObject: VersionObject;

    fn get_current_path(&self) -> Result<PathInfo, Self::VCSError>;

    fn iter_concrete_paths(&self) -> impl Iterator<Item = Result<PathInfo, Self::VCSError>>;

    fn get_version(&self, version: &String) -> Result<Self::VersionObject, Self::VCSError>;

    fn version_exists_on_path(&self, path: &NormalizedPath, version: &String) -> Result<bool, Self::VCSError>;

    fn iter_versions(&self, path: &NormalizedPath) -> impl Iterator<Item=Result<Self::VersionObject, Self::VCSError>>;

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