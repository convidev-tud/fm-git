use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;

#[derive(Debug, Clone)]
pub struct GitInterfaceError {
    msg: String,
}
impl GitInterfaceError {
    pub fn new(msg: &str) -> GitInterfaceError {
        GitInterfaceError {
            msg: msg.to_string(),
        }
    }
}
impl Display for GitInterfaceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for GitInterfaceError {}
#[derive(Debug)]
pub enum GitError {
    Io(io::Error),
    GitInterface(GitInterfaceError),
}
impl Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::Io(err) => err.fmt(f),
            GitError::GitInterface(err) => err.fmt(f),
        }
    }
}
impl Error for GitError {}
impl From<io::Error> for GitError {
    fn from(err: io::Error) -> GitError {
        GitError::Io(err)
    }
}
