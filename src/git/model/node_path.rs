use crate::git::model::*;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct NodePathError {
    msg: String,
}
impl NodePathError {
    pub fn new<S: Into<String>>(msg: S) -> NodePathError {
        NodePathError { msg: msg.into() }
    }
}
impl Display for NodePathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}
impl Error for NodePathError {}

pub struct NodePath<'a> {
    path: Vec<Rc<&'a Node>>,
}
impl<'a> NodePath<'a> {
    pub fn new(root: &'a Node) -> NodePath<'a> {
        Self { path: vec![Rc::new(root)] }
    }
    pub fn push_next<S: Into<String>>(&mut self, name: S) -> Result<(), NodePathError> {
        let real_name = name.into();
        match self.path.last().unwrap().get_child(real_name.clone()) {
            Some(child) => {
                self.path.push(Rc::new(child));
                Ok(())
            }
            None => Err(NodePathError::new(format!(
                "Cannot push child: {} is no child of {}",
                real_name,
                self.path.last().unwrap().get_name()
            ))),
        }
    }
    pub fn push_path(&mut self, path: QualifiedPath) -> Result<(), NodePathError> {
        for p in path.iter() {
            self.push_next(p)?;
        }
        Ok(())
    }
    pub fn to_path(&self) -> QualifiedPath {
        let mut path = QualifiedPath::new();
        for p in self.path.iter() {
            path.push(p.get_name());
        }
        path
    }
    pub fn last(&self) -> &'a Node {
        self.path.last().unwrap()
    }
}
