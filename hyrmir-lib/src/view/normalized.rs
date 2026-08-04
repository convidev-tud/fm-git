use colored::Colorize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Index};
use thiserror::Error;

const PATH_SEPARATOR: char = '/';
const REVISION_SEPARATOR: char = ':';

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum NormalizedRevision {
    Head,
    Revision(String),
}

impl Display for NormalizedRevision {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Head => f.write_str(""),
            Self::Revision(revision) => f.write_str(revision),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, Ord, PartialOrd)]
pub struct NormalizedPath {
    path: Vec<String>,
}

impl NormalizedPath {
    fn new_empty() -> Self {
        Self { path: vec![] }
    }

    pub fn new() -> Self {
        Self { path: vec!["".to_string()] }
    }

    pub fn from_iter(path: impl Iterator<Item=String>) -> Self {
        let mut new = Self { path: vec![] };
        for p in path {
            new.path.push(p);
        }
        new
    }

    pub fn push(&mut self, path: impl Into<String>) {
        let qualified_str = path.into();
        for split in qualified_str.trim().split(PATH_SEPARATOR) {
            self.path.push(split.to_lowercase());
        }
    }

    pub fn iter_all_segments(&self) -> impl Iterator<Item = &String> {
        self.path.iter()
    }

    pub fn iter_segments(&self, l: usize, r: usize) -> impl Iterator<Item = &String> {
        self.path[l..r].iter()
    }

    pub fn first_segment(&self) -> &String {
        self.path.first().unwrap()
    }

    pub fn last_segment(&self) -> &String {
        self.path.last().unwrap()
    }

    pub fn strip_n(&self, n_left: usize, n_right: usize) -> NormalizedPath {
        let l_removed = self.path[n_left..].to_vec();
        let first = &l_removed[0];
        let mut r_removed = l_removed[0..n_right].to_vec();
        if r_removed.is_empty() {
            match first.as_str() {
                "" => r_removed.push("".to_string()),
                _ => r_removed.push(".".to_string()),
            }
        }
        NormalizedPath::from_iter(r_removed.into_iter())
    }

    pub fn strip_n_left(&self, n: usize) -> NormalizedPath {
        self.strip_n(n, self.path.len())
    }

    pub fn strip_until_n_right(&self, n: usize) -> NormalizedPath {
        self.strip_n(0, n)
    }
    pub fn trim_whitespaces(&self) -> NormalizedPath {
        let mut new_path = self.path.clone();
        match new_path.first() {
            Some(value) => {
                if value == "" {
                    new_path.remove(0);
                }
            }
            None => {}
        }
        match new_path.last() {
            Some(value) => {
                if value == "" {
                    new_path.remove(new_path.len() - 1);
                }
            }
            None => {}
        }
        NormalizedPath::from_iter(new_path.into_iter())
    }

    pub fn replace<S: Into<String>>(&self, index: usize, value: S) -> NormalizedPath {
        let mut new_path = self.path.clone();
        new_path.insert(index, value.into());
        NormalizedPath::from_iter(new_path.into_iter())
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = NormalizedPath> {
        self.iter_all_segments()
            .map(|s| NormalizedPath::from(s.clone()))
    }

    pub fn get(&self, index: usize) -> Option<NormalizedPath> {
        Some(NormalizedPath::from(self.path.get(index)?.clone()))
    }

    pub fn starts_with(&self, prefix: impl ToNormalizedPath) -> bool {
        self.to_string().starts_with(&prefix.to_normalized_path().to_string())
    }

    pub fn last_is(&self, suffix: impl ToNormalizedPath) -> bool {
        self.last_segment() == suffix.to_normalized_path().last_segment()
    }

    pub fn len(&self) -> usize {
        self.path.len()
    }

    pub fn as_dir(&self) -> NormalizedPath {
        let mut new_path = self.path.clone();
        new_path.push("".to_string());
        NormalizedPath::from_iter(new_path.into_iter())
    }

    pub fn as_absolute(&self) -> NormalizedPath {
        let mut new_path = self.path.clone();
        new_path.insert(0, "".to_string());
        NormalizedPath::from_iter(new_path.into_iter())
    }

    pub fn is_dir(&self) -> bool {
        self.path.len() > 1 && self.last_segment() == &"".to_string()
    }

    pub fn is_absolute(&self) -> bool {
        self.path.len() > 0 && self.first_segment() == &"".to_string()
    }

    pub fn formatted(&self, colored: bool) -> String {
        let base = if colored {
            self.to_string().blue().to_string()
        } else {
            self.to_string()
        };
        base
    }
}

impl<T: Into<String>> From<T> for NormalizedPath {
    fn from(value: T) -> Self {
        let mut qualified_path = Self::new_empty();
        qualified_path.push(value);
        qualified_path
    }
}

impl<T: ToString> PartialEq<T> for NormalizedPath {
    fn eq(&self, other: &T) -> bool {
        self.to_string() == *other.to_string()
    }

    fn ne(&self, other: &T) -> bool {
        self.to_string() != *other.to_string()
    }
}

impl Display for NormalizedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.path.join("/").as_str())
    }
}

impl Index<usize> for NormalizedPath {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        &self.path[index]
    }
}

impl Add for NormalizedPath {
    type Output = NormalizedPath;

    fn add(self, rhs: Self) -> Self::Output {
        // always greater then 0
        assert!(!self.is_empty());
        let mut next_index = self.len();
        let mut new_path = self;
        if new_path.is_dir() && new_path.len() > 1 {
            new_path = new_path.strip_until_n_right(new_path.len() - 1);
        }
        for (i, part) in rhs.iter_all_segments().enumerate() {
            match part.as_str() {
                "." => {}
                ".." => {
                    new_path = new_path.strip_until_n_right(next_index - 1);
                    next_index = new_path.len();
                }
                "" => {
                    if i == 0 && rhs.len() > 1 {
                        return rhs.clone();
                    } else if i == rhs.len() - 1 || new_path.is_empty() {
                        new_path.push(part.to_string())
                    }
                }
                _ => {
                    new_path.push(part.to_string());
                    next_index += 1;
                }
            }
        }
        new_path
    }
}

#[derive(Error, Clone, Debug)]
pub struct NormalizeError {
    msg: String,
}

impl NormalizeError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

impl Display for NormalizeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg.as_str())
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Normalized {
    path: NormalizedPath,
    revision: NormalizedRevision,
}

impl Normalized {
    pub fn new(path: NormalizedPath, revision: NormalizedRevision) -> Self {
        Self { path, revision }
    }

    pub fn from_string(value: &String) -> Result<Self, NormalizeError> {
        let split = value.split(REVISION_SEPARATOR).collect::<Vec<&str>>();
        if split.len() > 2 {
            return Err(NormalizeError::new(format!(
                "Cannot normalize '{value}': input is malformed"
            )));
        }
        let path = NormalizedPath::from(split[0].to_string());
        let revision = if split.len() == 2 {
            NormalizedRevision::Revision(split[1].to_string())
        } else {
            NormalizedRevision::Head
        };
        Ok(Normalized::new(path, revision))
    }

    pub fn get_path(&self) -> &NormalizedPath {
        &self.path
    }

    pub fn get_revision(&self) -> &NormalizedRevision {
        &self.revision
    }
}

impl Display for Normalized {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match &self.revision {
            NormalizedRevision::Head => self.path.to_string(),
            NormalizedRevision::Revision(revision) => {
                format!("{}:{revision}", self.path,)
            }
        };
        f.write_str(&s)
    }
}

/*
    ##########
    # Traits #
    ##########
*/

pub trait Normalize {
    fn normalize(&self) -> Normalized;
}

pub trait FailableNormalize {
    fn normalize(&self) -> Result<Normalized, NormalizeError>;
}

impl<T: ToString> FailableNormalize for T {
    fn normalize(&self) -> Result<Normalized, NormalizeError> {
        Normalized::from_string(&self.to_string())
    }
}

pub trait ToNormalizedPath {
    fn to_normalized_path(&self) -> NormalizedPath;
}

impl<T: ToString> ToNormalizedPath for T {
    fn to_normalized_path(&self) -> NormalizedPath {
        NormalizedPath::from(self.to_string())
    }
}

/*
    ############################
    # Transformers and Filters #
    ############################
*/

pub fn collect_paths_by_name(paths: impl Iterator<Item=NormalizedPath>) -> HashMap<String, Vec<NormalizedPath>> {
    let mut names = HashMap::new();
    for path in paths {
        let name = path.last_segment();
        if !names.contains_key(name) {
            names.insert(name.to_string(), vec![path]);
        } else {
            let paths_vec = names.get_mut(name).unwrap();
            // removes duplicates
            if !paths_vec.contains(&path) {
                paths_vec.push(path);
            }
        }
    };
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_path_from_qualified() {
        assert_eq!(NormalizedPath::from("foo/bar").path, vec!["foo", "bar"]);
        assert_eq!(
            NormalizedPath::from("/foo/bar").path,
            vec!["", "foo", "bar"]
        );
        assert_eq!(NormalizedPath::from("/foo/bar").to_string(), "/foo/bar");
        assert_eq!(NormalizedPath::from("foo/").path, vec!["foo", ""]);
        assert_eq!(NormalizedPath::from("/").path, vec!["", ""]);
    }

    #[test]
    fn test_normalized_path_add_empty() {
        let l = NormalizedPath::new();
        let r = NormalizedPath::from("foo/bar");
        assert_eq!(l + r, NormalizedPath::from("/foo/bar"));

        let l = NormalizedPath::new();
        let r = NormalizedPath::from("/foo/bar");
        assert_eq!(l + r, NormalizedPath::from("/foo/bar"));
    }

    #[test]
    fn test_normalized_path_add_absolute() {
        let l = NormalizedPath::from("foo");
        let r = NormalizedPath::from("bar/baz");
        assert_eq!(l + r, NormalizedPath::from("foo/bar/baz"));

        let l = NormalizedPath::from("");
        let r = NormalizedPath::from("bar/baz");
        assert_eq!((l + r).path, vec!["", "bar", "baz"]);

        let l = NormalizedPath::from("foo/");
        let r = NormalizedPath::from("bar/baz");
        assert_eq!(l + r, NormalizedPath::from("foo/bar/baz"));
    }

    #[test]
    fn test_normalized_path_add_relative() {
        let l = NormalizedPath::from("foo");
        let r = NormalizedPath::from("..");
        assert_eq!(l + r, ".");

        let l = NormalizedPath::from("foo");
        let r = NormalizedPath::from("./bar");
        assert_eq!(l + r, "foo/bar");

        let l = NormalizedPath::from("foo");
        let r = NormalizedPath::from("./");
        assert_eq!(l + r, "foo/");

        let l = NormalizedPath::from("foo");
        let r = NormalizedPath::from("../bar");
        assert_eq!(l + r, "./bar");

        let l = NormalizedPath::from("foo/bar");
        let r = NormalizedPath::from("../baz");
        assert_eq!(l + r, "foo/baz");

        let l = NormalizedPath::from("foo/bar");
        let r = NormalizedPath::from("../../baz");
        assert_eq!(l + r, "./baz");

        let l = NormalizedPath::from("foo/bar");
        let r = NormalizedPath::from("../../../../../../baz");
        assert_eq!(l + r, "./baz");

        let l = NormalizedPath::from("foo/bar");
        let r = NormalizedPath::from("baz/../baz/../baz/../baz");
        assert_eq!(l + r, "foo/bar/baz");

        let l = NormalizedPath::from("foo/bar");
        let r = NormalizedPath::from("../baz/../baz/../baz");
        assert_eq!(l + r, "foo/baz");
    }

    #[test]
    fn test_normalized_path_add_whitespaces() {
        let l = NormalizedPath::from("foo");
        let r = NormalizedPath::from("");
        assert_eq!(l + r, NormalizedPath::from("foo/"));

        let l = NormalizedPath::from("foo");
        let r = NormalizedPath::from("/bar/baz");
        assert_eq!(l + r, NormalizedPath::from("/bar/baz"));
    }

    #[test]
    fn test_normalized_path_trim() {
        let path = NormalizedPath::from("foo/bar");
        assert_eq!(path.strip_n(0, path.len() - 1).path, vec!["foo"]);
    }

    #[test]
    fn test_normalized_path_as_absolute() {
        let path = NormalizedPath::from("foo/bar");
        let absolute = path.as_absolute();
        assert!(absolute.is_absolute());
        assert_eq!(absolute, "/foo/bar");
    }
}
