use colored::Colorize;
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use thiserror::Error;

mod revision;
mod semantic;
mod r#static;

use crate::model::*;
use crate::vcs::VersionId;
pub use revision::*;
pub use semantic::*;
pub use r#static::*;

impl<V: VersionId> ToNormalizedPath for Vec<Rc<RefCell<Node<V>>>> {
    fn to_normalized_path(&self) -> NormalizedPath {
        let mut path = NormalizedPath::new();
        for p in self.iter() {
            path.push(p.borrow().get_name());
        }
        path
    }
}

pub trait NodeHolder<V: VersionId> {
    fn get_node(&self) -> &Rc<RefCell<Node<V>>>;

    fn get_real_type(&self) -> NodeType {
        self.get_node().borrow().get_type().clone()
    }
}
