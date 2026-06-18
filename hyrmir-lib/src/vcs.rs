use std::error::Error;
use std::fmt::Debug;
use crate::model::NormalizedPath;

pub trait VCSError: Error {}

pub trait VersionObject: Debug {
    type VersionError: Error;
    
    fn get_metadata(&self, key: String) -> Result<String, Self::VersionError>;
}

pub trait VCS: Debug + Clone + PartialEq + Eq {
    type VCSError: VCSError;
    
    type VersionObject: VersionObject;

    fn get_current_path(&self) -> Result<NormalizedPath, Self::VCSError>;

    fn get_version(&self, version: &String) -> Result<Self::VersionObject, Self::VCSError>;

    fn version_exists_on_path(&self, path: &NormalizedPath, version: &String) -> Result<bool, Self::VCSError>;
    
    fn iter_concrete_paths(&self) -> impl Iterator<Item = Result<NormalizedPath, Self::VCSError>>;

    fn iter_versions(&self, path: &NormalizedPath) -> impl Iterator<Item=Result<Self::VersionObject, Self::VCSError>>;

    fn format_status_message(
        &self,
        current_path_msg: String,
        pre_status: String,
        post_status: String,
        colored: bool,
    ) -> Result<String, Self::VCSError>;
}