use crate::model::QualifiedPath;
use std::collections::HashSet;

pub trait PathCompletion {
    fn transform_and_filter_path(
        &self,
        prefix: &QualifiedPath,
        paths: &Vec<QualifiedPath>,
    ) -> Vec<QualifiedPath>;
    fn complete(&self, prefix: &QualifiedPath, paths: &Vec<QualifiedPath>) -> Vec<String> {
        let filtered = self.transform_and_filter_path(prefix, paths);
        match filtered.len() {
            0 => vec![],
            1 => vec![filtered[0].to_string()],
            _ => {
                let current_index = prefix.len();
                let all = filtered
                    .iter()
                    .map(|path| {
                        let to_index = path.strip_n_right(current_index);
                        if path.len() == current_index {
                            to_index.to_string()
                        } else {
                            to_index.to_string() + "/"
                        }
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

pub struct AbsolutePathCompletion;
impl PathCompletion for AbsolutePathCompletion {
    fn transform_and_filter_path(
        &self,
        prefix: &QualifiedPath,
        paths: &Vec<QualifiedPath>,
    ) -> Vec<QualifiedPath> {
        paths
            .iter()
            .filter_map(|path| {
                if path.starts_with(prefix) {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

pub struct RelativePathCompletion {
    current_path: QualifiedPath,
}
impl RelativePathCompletion {
    pub fn new(current_path: QualifiedPath) -> Self {
        Self { current_path }
    }
}
impl PathCompletion for RelativePathCompletion {
    fn transform_and_filter_path(
        &self,
        prefix: &QualifiedPath,
        paths: &Vec<QualifiedPath>,
    ) -> Vec<QualifiedPath> {
        let transformed_prefix = if prefix.last().is_some() {
            match prefix.last().unwrap().as_str() {
                "." | ".." => prefix.clone() + QualifiedPath::from(""),
                _ => prefix.clone(),
            }
        } else {
            prefix.clone()
        };
        let current_position = self.current_path.clone() + transformed_prefix.clone();
        let current_index = current_position.len() - 1;
        paths
            .iter()
            .filter_map(|path| {
                if !path.starts_with(&current_position) {
                    return None;
                }
                if path.len() <= current_index {
                    return None;
                }
                let new_path = transformed_prefix.strip_n_right(transformed_prefix.len() - 1)
                    + path.strip_n_left(current_index);
                Some(new_path)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_qualified_paths() -> Vec<QualifiedPath> {
        vec![
            QualifiedPath::from("foo"),
            QualifiedPath::from("foo/bar/baz1"),
            QualifiedPath::from("foo/bar/baz2"),
            QualifiedPath::from("foo/abc/def"),
            QualifiedPath::from("foo/abc"),
        ]
    }

    #[test]
    fn test_relative_path_completion_from_virtual_root() {
        let mut paths = setup_qualified_paths()
            .into_iter()
            .map(|path| QualifiedPath::from("") + path)
            .collect::<Vec<QualifiedPath>>();
        paths.push(QualifiedPath::from("/bar"));
        let completion = RelativePathCompletion::new(QualifiedPath::new());

        let mut direct = completion.complete(&QualifiedPath::from(""), &paths);
        direct.sort();
        assert_eq!(
            direct,
            vec![
                "/bar",
                "/foo",
                "/foo/abc",
                "/foo/abc/def",
                "/foo/bar/baz1",
                "/foo/bar/baz2"
            ]
        );

        let mut prefixed1 = completion.complete(&QualifiedPath::from("/f"), &paths);
        prefixed1.sort();
        assert_eq!(prefixed1, vec!["/foo", "/foo/"]);
    }

    #[test]
    fn test_relative_path_completion_relative_identifier_current_path() {
        let paths = setup_qualified_paths();
        let completion = RelativePathCompletion::new(QualifiedPath::from("foo"));

        let mut direct = completion.complete(&QualifiedPath::from("."), &paths);
        direct.sort();
        assert_eq!(
            direct,
            vec!["./abc", "./abc/def", "./bar/baz1", "./bar/baz2"]
        );

        let mut direct_with_slash = completion.complete(&QualifiedPath::from("./"), &paths);
        direct_with_slash.sort();
        assert_eq!(direct_with_slash, vec!["./abc", "./abc/", "./bar/"]);

        let mut prefixed = completion.complete(&QualifiedPath::from("./a"), &paths);
        prefixed.sort();
        assert_eq!(prefixed, vec!["./abc", "./abc/"]);

        let mut consecutive = completion.complete(&QualifiedPath::from("./b"), &paths);
        consecutive.sort();
        assert_eq!(consecutive, vec!["./bar/baz1", "./bar/baz2"]);
    }

    #[test]
    fn test_relative_path_completion_relative_identifier_previous_path() {
        let paths = setup_qualified_paths();
        let completion = RelativePathCompletion::new(QualifiedPath::from("foo"));

        let mut direct = completion.complete(&QualifiedPath::from("../"), &paths);
        direct.sort();
        assert_eq!(direct, vec!["../foo", "../foo/"]);

        let mut consecutive = completion.complete(&QualifiedPath::from("../foo/"), &paths);
        consecutive.sort();
        assert_eq!(
            consecutive,
            vec!["../foo/abc", "../foo/abc/", "../foo/bar/"]
        );

        let mut previous_of_previous =
            completion.complete(&QualifiedPath::from("abc/../../"), &paths);
        previous_of_previous.sort();
        assert_eq!(
            previous_of_previous,
            vec!["abc/../../foo", "abc/../../foo/"]
        );
    }

    #[test]
    fn test_relative_path_completion_current_path() {
        let paths = setup_qualified_paths();
        let completion = RelativePathCompletion::new(QualifiedPath::from("foo"));

        let mut direct = completion.complete(&QualifiedPath::from(""), &paths);
        direct.sort();
        assert_eq!(direct, vec!["abc", "abc/", "bar/"]);

        let mut prefixed = completion.complete(&QualifiedPath::from("a"), &paths);
        prefixed.sort();
        assert_eq!(prefixed, vec!["abc", "abc/"]);

        let mut consecutive = completion.complete(&QualifiedPath::from("b"), &paths);
        consecutive.sort();
        assert_eq!(consecutive, vec!["bar/baz1", "bar/baz2"]);
    }
}
