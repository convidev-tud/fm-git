use crate::completion::NormalizedCompleter;
use hyrmir_lib::model::{Normalize, Normalized, NormalizedRevision};
use std::collections::HashSet;

pub struct FullRelativePathCompleter {
    reference: Normalized,
}

impl FullRelativePathCompleter {
    pub fn new(reference: Normalized) -> Self {
        Self { reference }
    }

    fn transform_and_filter_path(
        &self,
        prefix: &Normalized,
        paths: impl Iterator<Item = Normalized>,
    ) -> impl Iterator<Item = Normalized> {
        let prefix_path = prefix.get_path();
        let current_position = self.reference.get_path().clone() + &prefix_path;
        let current_index = current_position.len() - 1;
        paths.filter_map(move |normalized| {
            match (prefix.get_revision(), normalized.get_revision()) {
                (NormalizedRevision::Revision(rev1), NormalizedRevision::Revision(rev2)) => {
                    if !rev2.starts_with(rev1) {
                        return None;
                    }
                }
                (NormalizedRevision::Revision(_), NormalizedRevision::None) => return None,
                _ => {}
            }
            let path = normalized.get_path();
            if !path.starts_with(&current_position) {
                return None;
            }
            if path.len() <= current_index {
                return None;
            }
            let new_path = path.strip_n_left(current_index);
            if prefix_path.len() == 1 {
                Some(Normalized::new(new_path, normalized.get_revision().clone()))
            } else {
                let new_path = prefix_path.strip_until_n_right(prefix_path.len() - 1) + &new_path;
                Some(Normalized::new(new_path, normalized.get_revision().clone()))
            }
        })
    }
}

impl NormalizedCompleter for FullRelativePathCompleter {
    fn complete(
        &self,
        prefix: impl AsRef<Normalized>,
        paths: impl Iterator<Item = Normalized>,
    ) -> Vec<String> {
        let prefix = prefix.as_ref();
        let prefix_path = prefix.get_path();
        let transformed_prefix = match prefix_path.last_segment().as_str() {
            "." | ".." => prefix_path.as_dir(),
            _ => prefix_path.clone(),
        };
        let filtered: Vec<Normalized> = self
            .transform_and_filter_path(
                &Normalized::new(transformed_prefix.clone(), prefix.get_revision().clone()),
                paths,
            )
            .collect();
        match filtered.len() {
            0 => vec![],
            1 => vec![filtered[0].to_string()],
            _ => {
                let current_index = transformed_prefix.len();
                let all = filtered
                    .iter()
                    .map(|normalized| {
                        let path = normalized.get_path();
                        let to_index = path.strip_until_n_right(current_index);
                        let to_return = if path.len() == current_index {
                            to_index
                        } else {
                            to_index.as_dir()
                        };
                        Normalized::new(to_return, normalized.get_revision().clone()).to_string()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::completion::test_utils::setup_normalized;

    #[test]
    fn test_full_path_completion_direct_from_root() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("".normalize());
        let mut completion = completer.complete("".normalize(), paths.into_iter());
        completion.sort();
        assert_eq!(completion, vec!["bar", "foo", "foo/", "foo:1.0", "foo:2.0"]);
    }

    #[test]
    fn test_full_path_completion_direct_root_prefix_1() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("".normalize());
        let mut completion = completer.complete("/f".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["/foo", "/foo/", "/foo:1.0", "/foo:2.0"]);
    }

    #[test]
    fn test_full_path_completion_direct_root_prefix_2() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("".normalize());
        let mut completion = completer.complete("/".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(
            completion,
            vec!["/bar", "/foo", "/foo/", "/foo:1.0", "/foo:2.0"]
        );
    }

    #[test]
    fn test_full_path_completion_current_path() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete("".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["abc", "abc/", "bar/"]);
    }

    #[test]
    fn test_full_path_completion_current_path_prefix() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete("a".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["abc", "abc/"]);
    }

    #[test]
    fn test_full_path_completion_current_path_prefix_forward() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete("b".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["bar/baz1", "bar/baz2"]);
    }

    #[test]
    fn test_full_path_completion_relative_current() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete(".".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["./abc", "./abc/", "./bar/"]);
    }

    #[test]
    fn test_full_path_completion_relative_current_slash() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete("./".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["./abc", "./abc/", "./bar/"]);
    }

    #[test]
    fn test_full_path_completion_relative_current_slash_and_prefix() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete("./a".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["./abc", "./abc/"]);
    }

    #[test]
    fn test_full_path_completion_relative_current_slash_and_prefix_forward() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete("./b".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["./bar/baz1", "./bar/baz2"]);
    }

    #[test]
    fn test_full_path_completion_relative_previous() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete("..".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(
            completion,
            vec!["../bar", "../foo", "../foo/", "../foo:1.0", "../foo:2.0"]
        );
    }

    #[test]
    fn test_full_path_completion_relative_previous_slash() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete("../".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(
            completion,
            vec!["../bar", "../foo", "../foo/", "../foo:1.0", "../foo:2.0"]
        );
    }

    #[test]
    fn test_full_path_completion_relative_previous_slash_prefix() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete("../foo".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(
            completion,
            vec!["../foo", "../foo/", "../foo:1.0", "../foo:2.0"]
        );
    }

    #[test]
    fn test_full_path_completion_relative_previous_slash_prefix_trailing_slash() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion = completer.complete("../foo/".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["../foo/abc", "../foo/abc/", "../foo/bar/"]);
    }

    #[test]
    fn test_full_path_completion_relative_previous_mixed() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("/foo".normalize());
        let mut completion =
            completer.complete("abc/../../".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(
            completion,
            vec![
                "abc/../../bar",
                "abc/../../foo",
                "abc/../../foo/",
                "abc/../../foo:1.0",
                "abc/../../foo:2.0"
            ]
        );
    }

    #[test]
    fn test_full_path_completion_direct_full_revision() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("".normalize());
        let mut completion = completer.complete("foo:1.0".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["foo:1.0"]);
    }

    #[test]
    fn test_full_path_completion_direct_completion_partial_revision() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("".normalize());
        let mut completion = completer.complete("foo:1".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["foo:1.0"]);
    }

    #[test]
    fn test_full_path_completion_direct_completion_empty_string_revision() {
        let paths = setup_normalized();
        let completer = FullRelativePathCompleter::new("".normalize());
        let mut completion = completer.complete("foo:".normalize(), paths.clone().into_iter());
        completion.sort();
        assert_eq!(completion, vec!["foo:1.0", "foo:2.0"]);
    }
}
