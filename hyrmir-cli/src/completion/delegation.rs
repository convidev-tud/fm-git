use crate::completion::{FullRelativePathCompleter, NameSearchPathCompleter, NormalizedPathCompleter};
use hyrmir_lib::model::*;

pub struct DelegatingPathCompleter {
    reference_path: NormalizedPath,
}

impl DelegatingPathCompleter {
    pub fn new(reference_path: NormalizedPath) -> Self {
        if reference_path.is_empty() {
            panic!("Reference path must not be empty")
        }
        Self { reference_path }
    }
}

impl NormalizedPathCompleter for DelegatingPathCompleter {
    fn complete(&self, prefix: impl ToNormalizedPath, paths: impl Iterator<Item=NormalizedPath>) -> Vec<String> {
        let prefix = prefix.to_normalized_path();
        if prefix.len() == 1 && prefix.first_segment() == "" {
            return NameSearchPathCompleter.complete(prefix, paths)
        }
        match prefix.first_segment().as_str() {
            "" | "." | ".." => {
                FullRelativePathCompleter::new(self.reference_path.clone())
                    .complete(prefix, paths)
            },
            _ => NameSearchPathCompleter.complete(prefix, paths),
        }
    }
}