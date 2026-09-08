use crate::view::ColorFormat;
use colored::Colorize;
use itertools::Itertools;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Index};
use thiserror::Error;

const MODEL_REVISION_PREFIX: char = '@';
const PATH_SEPARATOR: char = '/';
const DIMENSION_SEPARATOR: char = ':';

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

// ####################
// # Main Definitions #
// ####################

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Normalized {
    dimensions: NormalizedDimensionPath,
    revision: NormalizedRevision,
}

impl Normalized {
    pub fn new(dimensions: NormalizedDimensionPath, revision: NormalizedRevision) -> Self {
        Self { dimensions, revision }
    }

    pub fn try_parse(value: impl AsRef<str>) -> Result<Self, NormalizeError> {
        let value = value.as_ref();
        let split = value.split(DIMENSION_SEPARATOR).collect::<Vec<&str>>();

        let mut revision_path: Option<&str> = None;
        let mut model_path: Option<&str> = None;
        let mut revision: Option<&str> = None;
        for (index, element) in split.iter().enumerate() {
            match (index, element) {
                (0, e) => {
                    if e.starts_with(MODEL_REVISION_PREFIX) {
                        revision_path = Some(e);
                    }
                    else {
                        model_path = Some(e);
                    }
                },
                (1, e) => {
                    if e.starts_with(MODEL_REVISION_PREFIX) {
                        return Err(NormalizeError::new("@ at wrong position"))
                    } else if revision_path.is_some() {
                        model_path = Some(e);
                    } else {
                        revision = Some(e);
                    }
                }
                (2, e) => {
                    if e.starts_with(MODEL_REVISION_PREFIX) {
                        return Err(NormalizeError::new("@ at wrong position"))
                    } else {
                        revision = Some(e);
                    }
                }
                (_, _) => {
                    return Err(NormalizeError::new("mismatched number of dimensions"))
                }
            }
        }

        let path = match (revision_path, model_path) {
            (Some(mr), Some(mp)) => NormalizedDimensionPath::RevisionAndModelPath(
                NormalizedPath::new(mr),
                NormalizedPath::new(mp),
            ),
            (Some(mr), None) => NormalizedDimensionPath::RevisionPath(
                NormalizedPath::new(mr),
            ),
            (None, Some(mp)) => NormalizedDimensionPath::ModelPath(
                NormalizedPath::new(mp),
            ),
            (_, _) => unreachable!(),
        };
        let revision = if let Some(revision) = revision {
            NormalizedRevision::Revision(revision.to_string())
        } else {
            NormalizedRevision::None
        };
        Ok(Normalized::new(path, revision))
    }

    pub fn get_path(&self) -> &NormalizedDimensionPath {
        &self.dimensions
    }

    pub fn get_revision(&self) -> &NormalizedRevision {
        &self.revision
    }

    pub fn extract(self) -> (NormalizedDimensionPath, NormalizedRevision) {
        (self.dimensions, self.revision)
    }
}

impl<T: AsRef<str>> From<T> for Normalized {
    fn from(value: T) -> Self {
        Normalized::try_parse(value).unwrap()
    }
}

impl From<NormalizedPath> for Normalized {
    fn from(value: NormalizedPath) -> Self {
        Self::new(value, NormalizedRevision::None)
    }
}

impl AsRef<NormalizedPath> for Normalized {
    fn as_ref(&self) -> &NormalizedPath {
        self.get_path()
    }
}

impl AsRef<Normalized> for Normalized {
    fn as_ref(&self) -> &Normalized {
        self
    }
}

impl<T: AsRef<str>> PartialEq<T> for Normalized {
    fn eq(&self, other: &T) -> bool {
        self.to_string() == other.as_ref()
    }
}

impl Display for Normalized {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match &self.revision {
            NormalizedRevision::None => self.dimensions.to_string(),
            NormalizedRevision::Revision(revision) => {
                format!("{}:{revision}", self.dimensions)
            }
        };
        f.write_str(&s)
    }
}

impl ColorFormat for Normalized {
    fn formatted(&self, colored: bool) -> String {
        let path = self.get_path().formatted(colored);
        let result = match &self.revision {
            NormalizedRevision::None => path,
            NormalizedRevision::Revision(revision) => {
                format!(
                    "{path}{}{}",
                    DIMENSION_SEPARATOR.to_string().yellow(),
                    revision.yellow()
                )
            }
        };
        result
    }
}

impl Add<&Normalized> for Normalized {
    type Output = Normalized;

    fn add(self, rhs: &Normalized) -> Self::Output {
        let new_path = self.get_path().clone() + rhs.get_path();
        let revision = rhs.get_revision().clone();
        Normalized::new(new_path, revision)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub enum NormalizedRevision {
    None,
    Revision(String),
}

impl Display for NormalizedRevision {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => f.write_str(""),
            Self::Revision(revision) => f.write_str(revision),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum NormalizedDimensionPath {
    RevisionPath(NormalizedPath),
    ModelPath(NormalizedPath),
    RevisionAndModelPath(NormalizedPath, NormalizedPath),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
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

    pub fn from_iter(path: impl Iterator<Item = String>) -> Self {
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

    pub fn starts_with(&self, prefix: impl AsRef<NormalizedPath>) -> bool {
        self.to_string()
            .starts_with(prefix.as_ref().to_string().as_str())
    }

    pub fn last_is(&self, suffix: impl AsRef<NormalizedPath>) -> bool {
        self.last_segment() == suffix.as_ref().last_segment()
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
}

impl<T: AsRef<str>> From<T> for NormalizedPath {
    fn from(value: T) -> Self {
        value.to_normalized_path()
    }
}

impl AsRef<NormalizedPath> for NormalizedPath {
    fn as_ref(&self) -> &NormalizedPath {
        self
    }
}

impl<T: AsRef<str>> PartialEq<T> for NormalizedPath {
    fn eq(&self, other: &T) -> bool {
        self.to_string() == other.as_ref()
    }
}

impl Display for NormalizedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.path.join("/").as_str())
    }
}

impl ColorFormat for NormalizedPath {
    fn formatted(&self, colored: bool) -> String {
        let base = if colored {
            self.to_string().blue().to_string()
        } else {
            self.to_string()
        };
        base
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

impl Normalize for Normalized {
    fn try_normalize(&self) -> Result<Normalized, NormalizeError> {
        Ok(self.clone())
    }
}

pub trait ToNormalizedPath {
    fn to_normalized_path(&self) -> NormalizedPath;
}

impl<T: Normalize> ToNormalizedPath for T {
    fn to_normalized_path(&self) -> NormalizedPath {
        self.normalize().extract().0
    }
}

pub trait GetPathName {
    fn get_path_name(&self) -> String;
}

impl<T: GetPathName> GetPathName for &T {
    fn get_path_name(&self) -> String {
        (*self).get_path_name()
    }
}

impl GetPathName for NormalizedPath {
    fn get_path_name(&self) -> String {
        self.last_segment().to_string()
    }
}

impl GetPathName for Normalized {
    fn get_path_name(&self) -> String {
        let path_name = self.get_path().get_path_name();
        match self.get_revision() {
            NormalizedRevision::None => path_name,
            NormalizedRevision::Revision(revision) => {
                format!("{}{}{}", path_name, DIMENSION_SEPARATOR, revision)
            }
        }
    }
}

// ###########
// # Utility #
// ###########

pub fn collect_by_name<T: GetPathName + Eq>(
    paths: impl Iterator<Item = T>,
) -> HashMap<String, Vec<T>> {
    let mut names = HashMap::new();
    for path in paths {
        let name = path.get_path_name();
        if !names.contains_key(&name) {
            names.insert(name.to_string(), vec![path]);
        } else {
            let paths_vec = names.get_mut(&name).unwrap();
            // removes duplicates
            if !paths_vec.contains(&path) {
                paths_vec.push(path);
            }
        }
    }
    names
}

#[derive(Error, Clone, Debug)]
pub enum NameSearchError<T: ColorFormat> {
    NoneFound(T),
    MultipleFound(T, Vec<T>),
}

impl<T: ColorFormat> Display for NameSearchError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::NoneFound(path) => {
                format!("There is no path with the name '{}'", path.formatted(true),)
            }
            Self::MultipleFound(path, paths) => format!(
                "Multiple paths exist with the name '{}':\n  {}",
                path.formatted(true),
                paths.iter().map(|p| p.formatted(true)).join("\n  ")
            ),
        };
        f.write_str(error.as_str())
    }
}

pub fn get_path_from_name<T: GetPathName + ColorFormat + Eq + Clone>(
    path: T,
    search_space: impl Iterator<Item = T>,
) -> Result<T, NameSearchError<T>> {
    match path.get_path_name().as_str() {
        "" | "." | ".." => Ok(path),
        _ => {
            let mut name_to_paths = collect_by_name(search_space);
            if let Some(paths) = name_to_paths.remove(path.get_path_name().as_str()) {
                match paths.len() {
                    0 => Err(NameSearchError::NoneFound(path)),
                    1 => Ok(paths[0].clone()),
                    _ => Err(NameSearchError::MultipleFound(path, paths)),
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
    fn test_normalize_relative() {
        assert_eq!("foo/bar".normalize().get_path().path, vec!["foo", "bar"]);
    }

    #[test]
    fn test_normalize_absolute() {
        assert_eq!("/foo/bar".normalize().get_path().path, vec!["", "foo", "bar"]);
    }

    #[test]
    fn test_normalize_relative_dir() {
        assert_eq!("foo/".normalize().get_path().path, vec!["foo", ""]);
    }

    #[test]
    fn test_normalize_root() {
        assert_eq!("/".normalize().get_path().path, vec!["", ""]);
    }

    #[test]
    fn test_normalize_revision() {
        let normalized = "foo/bar:1.0".normalize();
        assert_eq!(normalized.get_path().path, vec!["foo", "bar"]);
        assert_eq!(
            normalized.get_revision(),
            &NormalizedRevision::Revision("1.0".to_string())
        );
    }

    #[test]
    fn test_normalized_path_add() {
        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("bar/baz");
        assert_eq!(l + &r, NormalizedPath::new("foo/bar/baz"));
    }

    #[test]
    fn test_normalized_path_add_to_absolute() {
        let l = NormalizedPath::new("");
        let r = NormalizedPath::new("bar/baz");
        assert_eq!((l + &r).path, vec!["", "bar", "baz"]);
    }

    #[test]
    fn test_normalized_path_add_empty_string() {
        let l = NormalizedPath::new("bar/baz");
        let r = NormalizedPath::new("");
        assert_eq!((l + &r).path, vec!["bar", "baz", ""]);
    }

    #[test]
    fn test_normalized_path_add_slash() {
        let l = NormalizedPath::new("bar/baz");
        let r = NormalizedPath::new("/");
        assert_eq!((l + &r).path, vec!["", ""]);
    }

    #[test]
    fn test_normalized_path_add_absolute() {
        let l = NormalizedPath::new("bar/baz");
        let r = NormalizedPath::new("/foo");
        assert_eq!((l + &r).path, vec!["", "foo"]);
    }

    #[test]
    fn test_normalized_path_add_to_dir() {
        let l = NormalizedPath::new("foo/");
        let r = NormalizedPath::new("bar/baz");
        assert_eq!(l + &r, NormalizedPath::new("foo/bar/baz"));
    }

    #[test]
    fn test_normalized_path_add_on_up() {
        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("..");
        assert_eq!(l + &r, ".");
    }

    #[test]
    fn test_normalized_path_add_in_current_dir() {
        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("./bar");
        assert_eq!(l + &r, "foo/bar");
    }

    #[test]
    fn test_normalized_path_add_current_dir() {
        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("./");
        assert_eq!(l + &r, "foo/");
    }

    #[test]
    fn test_normalized_path_add_sibling_replace() {
        let l = NormalizedPath::new("foo");
        let r = NormalizedPath::new("../bar");
        assert_eq!(l + &r, "./bar");
    }

    #[test]
    fn test_normalized_path_add_sibling_replace_multiple_levels() {
        let l = NormalizedPath::new("foo/bar");
        let r = NormalizedPath::new("../../baz");
        assert_eq!(l + &r, "./baz");

        let l = NormalizedPath::new("foo/bar");
        let r = NormalizedPath::new("../../../../../../baz");
        assert_eq!(l + &r, "./baz");
    }

    #[test]
    fn test_normalized_path_add_sibling_common_root() {
        let l = NormalizedPath::new("foo/bar");
        let r = NormalizedPath::new("../baz");
        assert_eq!(l + &r, "foo/baz");
    }

    #[test]
    fn test_normalized_path_add_mixed_relative1() {
        let l = NormalizedPath::new("foo/bar");
        let r = NormalizedPath::new("baz/../baz/../baz/../baz");
        assert_eq!(l + &r, "foo/bar/baz");
    }

    #[test]
    fn test_normalized_path_add_mixed_relative2() {
        let l = NormalizedPath::new("foo/bar");
        let r = NormalizedPath::new("../baz/../baz/../baz");
        assert_eq!(l + &r, "foo/baz");
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

    #[test]
    fn test_normalized_add() {
        let l = "foo/bar:1.0".normalize();
        let r = "baz:2.0".normalize();
        assert_eq!(l + &r, "foo/bar/baz:2.0");
    }

    #[test]
    fn test_collect_normalized_paths_by_name() {
        let paths = vec![
            "foo".to_normalized_path(),
            "foo/bar".to_normalized_path(),
            "bar/baz".to_normalized_path(),
        ];
        let collected = collect_by_name(paths.iter());
        let foo = collected.get("foo").unwrap();
        let bar = collected.get("bar").unwrap();
        let baz = collected.get("baz").unwrap();
        assert!(foo.len() == 1 && bar.len() == 1 && baz.len() == 1);
        assert!(foo.contains(&&"foo".to_normalized_path()));
        assert!(bar.contains(&&"foo/bar".to_normalized_path()));
        assert!(baz.contains(&&"bar/baz".to_normalized_path()));
    }

    #[test]
    fn test_collect_normalized_by_name() {
        let paths = vec!["foo".normalize(), "foo:1.0".normalize()];
        let collected = collect_by_name(paths.iter());
        let foo = collected.get("foo").unwrap();
        let foo_1 = collected.get("foo:1.0").unwrap();
        assert!(foo.len() == 1 && foo_1.len() == 1);
        assert!(foo.contains(&&"foo".normalize()));
        assert!(foo_1.contains(&&"foo:1.0".normalize()));
    }
}
