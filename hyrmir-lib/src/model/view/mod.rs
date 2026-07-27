use std::cell::RefCell;
use std::rc::Rc;

mod revision;
mod structure;
mod dynamic;

use crate::model::*;
use crate::vcs::{VersionId, VCS};
pub use dynamic::*;
pub use revision::*;
pub use structure::*;

impl<V: VCS> ToNormalizedPath for Vec<Rc<RefCell<NodeData<V>>>> {
    fn to_normalized_path(&self) -> NormalizedPath {
        let mut path = NormalizedPath::new();
        for p in self.iter() {
            path.push(p.borrow().get_name());
        }
        path
    }
}

pub trait NodeHolder<V: VCS> {
    fn get_node(&self) -> &Rc<RefCell<NodeData<V>>>;

    fn get_real_type(&self) -> NodeType {
        self.get_node().borrow().get_type().clone()
    }
}
