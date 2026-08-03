use crate::completion::NormalizedPathCompleter;
use hyrmir_lib::model::*;

pub struct NameSearchPathCompleter {
    reference_path: NormalizedPath,
}

impl NameSearchPathCompleter {
    pub fn new(reference_path: NormalizedPath) -> Self {
        if reference_path.is_empty() {
            panic!("Reference path must not be empty")
        }
        Self { reference_path }
    }
}

impl NormalizedPathCompleter for NameSearchPathCompleter {
    fn transform_and_filter_path(
        &self,
        prefix: &NormalizedPath,
        paths: impl Iterator<Item=NormalizedPath>
    ) -> impl Iterator<Item=NormalizedPath> {
        collect_paths_by_name(paths)
            .into_iter()
            .filter_map(|(name, paths)| {
                if name.starts_with(&prefix.to_string()) {
                    match paths.len() {
                        0 => None,
                        _ => Some(name.to_normalized_path()),
                    }
                } else {
                    None
                }
            })
    }

    fn complete(&self, prefix: NormalizedPath, paths: impl Iterator<Item=NormalizedPath>) -> Vec<String> {
        self
            .transform_and_filter_path(&prefix, paths)
            .map(|p| p.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let completer = NameSearchPathCompleter::new(NormalizedPath::from(""));
        let paths = vec![
            NormalizedPath::from("/main"),
            NormalizedPath::from("/main/feature/foo"),
            NormalizedPath::from("/foo/feature/main"),
        ];
        let result = completer.complete(
            NormalizedPath::from("m"),
            paths.into_iter()
        );
        println!("{:?}", result)
    }
}