use crate::completion::NormalizedPathCompleter;
use hyrmir_lib::model::NormalizedPath;

pub struct FullRelativePathCompleter {
    reference_path: NormalizedPath,
}

impl FullRelativePathCompleter {
    pub fn new(reference_path: NormalizedPath) -> Self {
        if reference_path.is_empty() {
            panic!("Reference path must not be empty")
        }
        Self { reference_path }
    }
}

impl NormalizedPathCompleter for FullRelativePathCompleter {
    fn transform_and_filter_path(
        &self, 
        prefix: &NormalizedPath, 
        paths: impl Iterator<Item=NormalizedPath>
    ) -> impl Iterator<Item=NormalizedPath> {
        let transformed_prefix = if prefix.last_segment().is_some() {
            match prefix.last_segment().unwrap().as_str() {
                "." | ".." => prefix.as_dir(),
                _ => prefix.clone(),
            }
        } else {
            prefix.clone()
        };
        let current_position = self.reference_path.clone() + transformed_prefix.clone();
        let current_index = current_position.len() - 1;
        paths.filter_map(move |path| {
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
    }
}

// #[cfg(test)]
// pub mod test_utils {
//     use hyrmir_lib::model::NormalizedPath;
//
//     pub fn setup_qualified_paths() -> Vec<NormalizedPath> {
//         vec![
//             NormalizedPath::from("/foo"),
//             NormalizedPath::from("/foo/bar/baz1"),
//             NormalizedPath::from("/foo/bar/baz2"),
//             NormalizedPath::from("/foo/abc/def"),
//             NormalizedPath::from("/foo/abc"),
//             NormalizedPath::from("/bar"),
//         ]
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::cli::completion::path::test_utils::setup_qualified_paths;
//
//     #[test]
//     fn test_relative_path_completion_from_virtual_root() {
//         let paths = setup_qualified_paths();
//         let completion = RelativePathCompleter::new(NormalizedPath::from(""));
//
//         let mut direct = completion.complete(NormalizedPath::from(""), paths.clone().into_iter());
//         direct.sort();
//         assert_eq!(direct, vec!["bar", "foo", "foo/",]);
//
//         let mut prefixed1 =
//             completion.complete(NormalizedPath::from("/f"), paths.clone().into_iter());
//         prefixed1.sort();
//         assert_eq!(prefixed1, vec!["/foo", "/foo/"]);
//
//         let mut prefixed2 =
//             completion.complete(NormalizedPath::from("/"), paths.clone().into_iter());
//         prefixed2.sort();
//         assert_eq!(prefixed2, vec!["/bar", "/foo", "/foo/"]);
//     }
//
//     #[test]
//     fn test_relative_path_completion_relative_identifier_current_path() {
//         let paths = setup_qualified_paths();
//         let completion = RelativePathCompleter::new(NormalizedPath::from("/foo"));
//
//         let mut direct = completion.complete(NormalizedPath::from("."), paths.clone().into_iter());
//         direct.sort();
//         assert_eq!(
//             direct,
//             vec!["./abc", "./abc/def", "./bar/baz1", "./bar/baz2"]
//         );
//
//         let mut direct_with_slash =
//             completion.complete(NormalizedPath::from("./"), paths.clone().into_iter());
//         direct_with_slash.sort();
//         assert_eq!(direct_with_slash, vec!["./abc", "./abc/", "./bar/"]);
//
//         let mut prefixed =
//             completion.complete(NormalizedPath::from("./a"), paths.clone().into_iter());
//         prefixed.sort();
//         assert_eq!(prefixed, vec!["./abc", "./abc/"]);
//
//         let mut consecutive = completion.complete(NormalizedPath::from("./b"), paths.into_iter());
//         consecutive.sort();
//         assert_eq!(consecutive, vec!["./bar/baz1", "./bar/baz2"]);
//     }
//
//     #[test]
//     fn test_relative_path_completion_relative_identifier_previous_path() {
//         let paths = setup_qualified_paths();
//         let completion = RelativePathCompleter::new(NormalizedPath::from("/foo"));
//
//         let mut direct =
//             completion.complete(NormalizedPath::from("../"), paths.clone().into_iter());
//         direct.sort();
//         assert_eq!(direct, vec!["../bar", "../foo", "../foo/"]);
//
//         let mut consecutive =
//             completion.complete(NormalizedPath::from("../foo/"), paths.clone().into_iter());
//         consecutive.sort();
//         assert_eq!(
//             consecutive,
//             vec!["../foo/abc", "../foo/abc/", "../foo/bar/"]
//         );
//
//         let mut previous_of_previous =
//             completion.complete(NormalizedPath::from("abc/../../"), paths.into_iter());
//         previous_of_previous.sort();
//         assert_eq!(
//             previous_of_previous,
//             vec!["abc/../../bar", "abc/../../foo", "abc/../../foo/"]
//         );
//     }
//
//     #[test]
//     fn test_relative_path_completion_current_path() {
//         let paths = setup_qualified_paths();
//         let completion = RelativePathCompleter::new(NormalizedPath::from("/foo"));
//
//         let mut direct = completion.complete(NormalizedPath::from(""), paths.clone().into_iter());
//         direct.sort();
//         assert_eq!(direct, vec!["abc", "abc/", "bar/"]);
//
//         let mut prefixed =
//             completion.complete(NormalizedPath::from("a"), paths.clone().into_iter());
//         prefixed.sort();
//         assert_eq!(prefixed, vec!["abc", "abc/"]);
//
//         let mut consecutive = completion.complete(NormalizedPath::from("b"), paths.into_iter());
//         consecutive.sort();
//         assert_eq!(consecutive, vec!["bar/baz1", "bar/baz2"]);
//     }
//
//     #[test]
//     #[should_panic]
//     fn test_relative_path_completion_empty_reference() {
//         RelativePathCompleter::new(NormalizedPath::new());
//     }
// }
