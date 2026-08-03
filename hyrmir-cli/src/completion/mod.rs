mod helper;
mod full;
mod search;

use std::collections::HashSet;
pub use helper::*;
pub use full::*;
pub use search::*;
use hyrmir_lib::model::NormalizedPath;

pub trait NormalizedPathCompleter {
    fn transform_and_filter_path(
        &self,
        prefix: &NormalizedPath,
        paths: impl Iterator<Item = NormalizedPath>,
    ) -> impl Iterator<Item = NormalizedPath>;

    fn complete(
        &self,
        prefix: NormalizedPath,
        paths: impl Iterator<Item = NormalizedPath>,
    ) -> Vec<String> {
        let filtered: Vec<NormalizedPath> = self
            .transform_and_filter_path(&prefix, paths)
            .collect();
        match filtered.len() {
            0 => vec![],
            1 => vec![filtered[0].to_string()],
            _ => {
                let current_index = prefix.len();
                let all = filtered
                    .iter()
                    .map(|path| {
                        let to_index = path.strip_n_right(current_index);
                        let to_return = if path.len() == current_index {
                            to_index
                        } else {
                            to_index.as_dir()
                        };
                        to_return.to_string()
                    })
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect::<Vec<String>>();
                if all.len() == 1 {
                    filtered.iter().map(|path| path.to_string()).collect()
                } else {
                    all
                }
            }
        }
    }
}
