mod helper;
mod full;
mod search;
mod delegation;

pub use helper::*;
pub use full::*;
pub use search::*;
pub use delegation::*;
use hyrmir_lib::model::{NormalizedPath, ToNormalizedPath};

pub trait NormalizedPathCompleter {
    fn complete(
        &self,
        prefix: impl ToNormalizedPath,
        paths: impl Iterator<Item = NormalizedPath>,
    ) -> Vec<String>;
}

#[cfg(test)]
pub mod test_utils {
    use hyrmir_lib::model::NormalizedPath;

    pub fn setup_qualified_paths() -> Vec<NormalizedPath> {
        vec![
            NormalizedPath::from("/foo"),
            NormalizedPath::from("/foo/bar/baz1"),
            NormalizedPath::from("/foo/bar/baz2"),
            NormalizedPath::from("/foo/abc/def"),
            NormalizedPath::from("/foo/abc"),
            NormalizedPath::from("/bar"),
        ]
    }
}
