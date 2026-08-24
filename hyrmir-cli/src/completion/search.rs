use crate::completion::NormalizedCompleter;
use hyrmir_lib::model::*;

pub struct NameSearchPathCompleter;

impl NormalizedCompleter for NameSearchPathCompleter {
    fn complete(
        &self,
        prefix: impl AsRef<Normalized>,
        paths: impl Iterator<Item = Normalized>,
    ) -> Vec<String> {
        collect_by_name(paths)
            .into_iter()
            .filter_map(|(name, paths)| {
                if name.starts_with(&prefix.as_ref().to_string()) {
                    match paths.len() {
                        0 => None,
                        _ => Some(name.to_normalized_path()),
                    }
                } else {
                    None
                }
            })
            .map(|p| p.to_string())
            .collect()
    }
}
