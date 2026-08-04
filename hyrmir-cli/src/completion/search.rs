use crate::completion::NormalizedPathCompleter;
use hyrmir_lib::model::*;

pub struct NameSearchPathCompleter;

impl NormalizedPathCompleter for NameSearchPathCompleter {
    fn complete(&self, prefix: impl ToNormalizedPath, paths: impl Iterator<Item=NormalizedPath>) -> Vec<String> {
        collect_paths_by_name(paths)
            .into_iter()
            .filter_map(|(name, paths)| {
                if name.starts_with(&prefix.to_normalized_path().to_string()) {
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