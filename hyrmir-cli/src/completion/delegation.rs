use crate::completion::{FullRelativePathCompleter, NameSearchPathCompleter, NormalizedCompleter};
use hyrmir_lib::model::*;

pub struct SwitchingPathCompleter {
    reference: Normalized,
}

impl SwitchingPathCompleter {
    pub fn new(reference: Normalized) -> Self {
        Self { reference }
    }
}

impl NormalizedCompleter for SwitchingPathCompleter {
    fn complete(
        &self,
        prefix: impl AsRef<Normalized>,
        paths: impl Iterator<Item = Normalized>,
    ) -> Vec<String> {
        let prefix = prefix.as_ref();
        let prefix_path = prefix.get_path();
        if prefix_path.len() == 1 && prefix_path.first_segment() == "" {
            return NameSearchPathCompleter.complete(prefix, paths);
        }
        match prefix_path.first_segment().as_str() {
            "" | "." | ".." => {
                FullRelativePathCompleter::new(self.reference.clone()).complete(prefix, paths)
            }
            _ => NameSearchPathCompleter.complete(prefix, paths),
        }
    }
}
