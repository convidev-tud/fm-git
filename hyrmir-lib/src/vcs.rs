use std::fmt::Debug;
use crate::model::NormalizedPath;

pub trait VCS: Debug {
    fn get_current_path(&self) -> NormalizedPath;
    
    fn iter_concrete_paths(&self) -> impl Iterator<Item = NormalizedPath>;

    fn status(&self, colored: bool) -> String;

    fn format_status(
        &self,
        current_path_msg: String,
        extra_msg: String,
        colored: bool,
    ) -> String;
}
