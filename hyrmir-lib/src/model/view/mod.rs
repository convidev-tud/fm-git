mod frozen;
mod revision;
mod structure;

use crate::model::{NodeData, NormalizedPath, ToNormalizedPath};
use crate::vcs::VCS;
pub use frozen::*;
use indextree::Node;
pub use revision::*;
use std::cell::RefCell;
use std::fmt::Debug;
pub use structure::*;
/*
    ####################
        Access Modes
    ####################
*/

pub trait AccessMode: Debug {
    fn lock() -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct Shared;

impl AccessMode for Shared {
    fn lock() -> bool {
        false
    }
}

#[derive(Debug)]
pub struct Locked;

impl AccessMode for Locked {
    fn lock() -> bool {
        true
    }
}

impl<V: VCS> ToNormalizedPath for Vec<&Node<RefCell<NodeData<V>>>> {
    fn to_normalized_path(&self) -> NormalizedPath {
        let mut path = NormalizedPath::new();
        for p in self.iter() {
            path.push(p.get().borrow().get_name());
        }
        path
    }
}
