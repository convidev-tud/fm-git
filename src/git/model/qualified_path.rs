use std::fmt::{Display, Formatter};
use std::ops::Add;

const SEPARATOR: char = '/';

#[derive(Clone, Debug)]
pub struct QualifiedPath {
    path: Vec<String>,
}
impl From<String> for QualifiedPath {
    fn from(value: String) -> Self {
        let qualified_str = value.replace("*", "").replace("_", "");
        let mut git_path = Self::new();
        git_path.push(qualified_str.trim().to_string());
        git_path
    }
}
impl From<&str> for QualifiedPath {
    fn from(value: &str) -> Self {
        let mut git_path = Self::new();
        git_path.push(value.to_string());
        git_path
    }
}
impl From<Vec<String>> for QualifiedPath {
    fn from(value: Vec<String>) -> Self {
        let mut path = Self::new();
        for v in value {
            path.push(v);
        }
        path
    }
}
impl PartialEq for QualifiedPath {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }

    fn ne(&self, other: &Self) -> bool {
        self.path != other.path
    }
}
impl Add for QualifiedPath {
    type Output = QualifiedPath;

    fn add(self, rhs: Self) -> Self::Output {
        let mut new_path = self.path;
        new_path.extend(rhs.path);
        Self::from(new_path)
    }
}
impl Display for QualifiedPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}
impl QualifiedPath {
    pub fn new() -> Self {
        Self { path: Vec::new() }
    }
    pub fn to_git_branch(&self) -> String {
        match self.path.len() {
            1 => self.path[0].to_string(),
            _ => {
                let mut prefix = self.path[..self.path.len() - 1]
                    .iter()
                    .map(|x| "_".to_string() + x)
                    .collect::<Vec<_>>();
                prefix.push(self.path[self.path.len() - 1].to_string());
                prefix.join("/")
            }
        }
    }
    pub fn to_string(&self) -> String {
        self.path.join("/")
    }
    pub fn push<S: Into<String>>(&mut self, path: S) {
        let split_path = path
            .into()
            .split(SEPARATOR)
            .map(|e| e.to_string())
            .collect::<Vec<String>>();
        self.path.extend(split_path);
    }
    pub fn trim_n(&self, n_left: usize, n_right: usize) -> QualifiedPath {
        QualifiedPath::from(self.path[n_left..n_right].to_vec())
    }
    pub fn trim_n_left(&self, n: usize) -> QualifiedPath {
        self.trim_n(n, self.path.len())
    }
    pub fn trim_n_right(&self, n: usize) -> QualifiedPath {
        self.trim_n(0, n)
    }
    pub fn first(&self) -> Option<&String> {
        self.path.first()
    }
    pub fn _last(&self) -> Option<&String> {
        self.path.last()
    }
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.path.iter()
    }
    pub fn get(&self, index: usize) -> Option<QualifiedPath> {
        Some(QualifiedPath::from(self.path.get(index)?.clone()))
    }
    pub fn starts_with(&self, prefix: &QualifiedPath) -> bool {
        self.to_string().starts_with(&prefix.to_string())
    }
    pub fn len(&self) -> usize {
        self.path.len()
    }
}
