use std::borrow::Borrow;
use colored::Colorize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Index};
use itertools::Itertools;
use thiserror::Error;

const PATH_SEPARATOR: char = '/';
const REVISION_SEPARATOR: char = ':';

// ##########
// # Errors #
// ##########

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

// ##########
// # Main Definitions #
// ##########

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Normalized {
    path: NormalizedPath,
    revision: NormalizedRevision,
}

impl Normalized {
    pub fn new(path: NormalizedPath, revision: NormalizedRevision) -> Self {
        Self { path, revision }
    }

    pub fn try_parse(value: impl AsRef<str>) -> Result<Self, NormalizeError> {
        let value = value.as_ref();
        let split = value.split(REVISION_SEPARATOR).collect::<Vec<&str>>();
        if split.len() > 2 {
            return Err(NormalizeError::new(format!(
                "Cannot normalize '{value}': input is malformed"
            )));
        }
        let path = NormalizedPath::new(split[0].to_string());
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

    pub fn extract(self) -> (NormalizedPath, NormalizedRevision) {
        (self.path, self.revision)
    }
}

impl<T: AsRef<str>> From<T> for Normalized {
    fn from(value: T) -> Self {
        Normalized::try_parse(value).unwrap()
    }
}

impl Borrow<NormalizedPath> for Normalized {
    fn borrow(&self) -> &NormalizedPath {
        self.get_path()
    }
}

impl Display for Normalized {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match &self.revision {
            NormalizedRevision::Head => self.path.to_string(),
            NormalizedRevision::Revision(revision) => {
                format!("{}:{revision}", self.path)
            }
        };
        f.write_str(&s)
    }
}

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

    fn new(value: impl AsRef<str>) -> Self {
        let mut new = Self::new_empty();
        new.push(value);
        new
    }

    pub fn from_iter(path: impl Iterator<Item=String>) -> Self {
        let mut new = Self::new_empty();
        for p in path {
            new.path.push(p);
        }
        new
    }

    pub fn push(&mut self, path: impl AsRef<str>) {
        let qualified_str = path.as_ref();
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
        let mut r_removed = self.path[n_left..n_right].to_vec();
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
            .map(|s| NormalizedPath::new(s.clone()))
    }

    pub fn get(&self, index: usize) -> Option<NormalizedPath> {
        Some(NormalizedPath::new(self.path.get(index)?.clone()))
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

impl<T: AsRef<str>> From<T> for NormalizedPath {
    fn from(value: T) -> Self {
        value.to_normalized_path()
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

impl Add<&NormalizedPath> for NormalizedPath {
    type Output = NormalizedPath;

    fn add(self, rhs: &NormalizedPath) -> Self::Output {
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

// ##########
// # Traits #
// ##########

pub trait Normalize {
    fn try_normalize(&self) -> Result<Normalized, NormalizeError>;

    fn normalize(&self) -> Normalized {
        self.try_normalize().unwrap()
    }
}

impl<T: AsRef<str>> Normalize for T {
    fn try_normalize(&self) -> Result<Normalized, NormalizeError> {
        Normalized::try_parse(self)
    }
}

pub trait ToNormalizedPath {
    fn to_normalized_path(&self) -> NormalizedPath;
}

impl<T: Normalize> ToNormalizedPath for T {
    fn to_normalized_path(&self) -> NormalizedPath {
        self
            .normalize()
            .extract()
            .0
    }
}

// ###########
// # Utility #
// ###########

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

#[derive(Error, Clone, Debug)]
pub enum NameSearchError {
    NoneFound(NormalizedPath),
    MultipleFound(NormalizedPath, Vec<NormalizedPath>),
}

impl Display for NameSearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::NoneFound(path) => format!(
                "There is no path with the name '{}'",
                path,
            ),
            Self::MultipleFound(path, paths) => format!(
                "Multiple paths exist with the name '{}':\n  {}",
                path,
                paths.iter().map(|p| p.formatted(true)).join("\n  ")
            ),
        };
        f.write_str(error.as_str())
    }
}

pub fn get_path_from_name(
    path: impl ToNormalizedPath,
    search_space: impl Iterator<Item=NormalizedPath>
) -> Result<NormalizedPath, NameSearchError> {
    let path = path.to_normalized_path();
    match path.first_segment().as_str() {
        "" | "." | ".." => Ok(path),
        _ => {
            let mut name_to_paths = collect_paths_by_name(search_space);
            if let Some(paths) = name_to_paths.remove(path.last_segment().as_str()) {
                match paths.len() {
                    0 => Err(NameSearchError::NoneFound(path)),
                    1 => Ok(paths[0].clone()),
                    _ => Err(NameSearchError::MultipleFound(path, paths))
                }
            } else {
                Err(NameSearchError::NoneFound(path))
            }
        }
    }
}

// #########
// # Tests #
// #########

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalized_path_from_qualified() {
        assert_eq!(NormalizedPath::new("foo/bar").path, vec!["foo", "bar"]);
        assert_eq!(
            NormalizedPath::new("/foo/bar").path,
            vec!["", "foo", "bar"]
        );
        assert_eq!(NormalizedPath::new("/foo/bar").to_string(), "/foo/bar");
        assert_eq!(NormalizedPath::new("foo/").path, vec!["foo", ""]);
        assert_eq!(NormalizedPath::new("/").path, vec!["", ""]);
    }

    #[test]
    fn test_normalized_path_add_empty() {
        let l = NormalizedPath::new();
        let r = NormalizedPath::new("foo/bar");
        assert_eq!(l + &r, NormalizedPath::new("/foo/bar"));

        let l = NormalizedPath::new();
        let r = NormalizedPath::new("/foo/bar");
        assert_eq!(l + r, NormalizedPath::new("/foo/bar"));
    }

    #[test]
    fn test_normalized_path_add_absolute() {
        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("bar/baz");
        assert_eq!(l + r, NormalizedPath::new("foo/bar/baz"));

        let l = NormalizedPath::new("");
        let r = NormalizedPath::new("bar/baz");
        assert_eq!((l + r).path, vec!["", "bar", "baz"]);

        let l = NormalizedPath::new("foo/");
        let r = NormalizedPath::new("bar/baz");
        assert_eq!(l + r, NormalizedPath::new("foo/bar/baz"));
    }

    #[test]
    fn test_normalized_path_add_relative() {
        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("..");
        assert_eq!(l + r, ".");

        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("./bar");
        assert_eq!(l + r, "foo/bar");

        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("./");
        assert_eq!(l + r, "foo/");

        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("../bar");
        assert_eq!(l + r, "./bar");

        let l = NormalizedPath::new("foo/bar");
        let r = NormalizedPath::new("../baz");
        assert_eq!(l + r, "foo/baz");

        let l = NormalizedPath::new("foo/bar");
        let r = NormalizedPath::new("../../baz");
        assert_eq!(l + r, "./baz");

        let l = NormalizedPath::new("foo/bar");
        let r = NormalizedPath::new("../../../../../../baz");
        assert_eq!(l + r, "./baz");

        let l = NormalizedPath::new("foo/bar");
        let r = NormalizedPath::new("baz/../baz/../baz/../baz");
        assert_eq!(l + r, "foo/bar/baz");

        let l = NormalizedPath::new("foo/bar");
        let r = NormalizedPath::new("../baz/../baz/../baz");
        assert_eq!(l + r, "foo/baz");
    }

    #[test]
    fn test_normalized_path_add_whitespaces() {
        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("");
        assert_eq!(l + r, NormalizedPath::new("foo/"));

        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("/bar/baz");
        assert_eq!(l + r, NormalizedPath::new("/bar/baz"));
    }

    #[test]
    fn test_normalized_path_trim() {
        let path = NormalizedPath::new("foo/bar");
        assert_eq!(path.strip_n(0, path.len() - 1).path, vec!["foo"]);
    }

    #[test]
    fn test_normalized_path_as_absolute() {
        let path = NormalizedPath::new("foo/bar");
        let absolute = path.as_absolute();
        assert!(absolute.is_absolute());
        assert_eq!(absolute, "/foo/bar");
    }
}
