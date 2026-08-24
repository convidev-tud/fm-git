mod delegation;
mod full;
mod helper;
mod search;

pub use delegation::*;
pub use full::*;
pub use helper::*;
use hyrmir_lib::model::Normalized;
pub use search::*;

pub trait NormalizedCompleter {
    fn complete(
        &self,
        prefix: impl AsRef<Normalized>,
        paths: impl Iterator<Item = Normalized>,
    ) -> Vec<String>;
}

#[cfg(test)]
pub mod test_utils {
    use hyrmir_lib::model::{Normalize, Normalized};

    pub fn setup_normalized() -> Vec<Normalized> {
        vec![
            "/foo".normalize(),
            "/foo:1.0".normalize(),
            "/foo:2.0".normalize(),
            "/foo/bar/baz1".normalize(),
            "/foo/bar/baz2".normalize(),
            "/foo/abc/def".normalize(),
            "/foo/abc".normalize(),
            "/bar".normalize(),
        ]
    }
}
