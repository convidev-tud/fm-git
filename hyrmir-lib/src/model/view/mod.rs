mod revision;
mod structure;
mod frozen;

use std::cell::RefCell;
use std::fmt::Debug;
use indextree::Node;
pub use frozen::*;
pub use revision::*;
pub use structure::*;
use crate::model::{NodeData, NormalizedPath, ToNormalizedPath};
use crate::vcs::VCS;
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
        true
    }
}

#[derive(Debug)]
pub struct Locked;

impl AccessMode for Locked {
    fn lock() -> bool {
        false
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