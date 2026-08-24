mod frozen;
pub mod normalized;
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

// ################
// # Access Modes #
// ################

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

// ####################
// # Formatting Trait #
// ####################

pub trait ColorFormat {
    fn formatted(&self, colored: bool) -> String;
}

impl<T: ColorFormat> ColorFormat for &T {
    fn formatted(&self, colored: bool) -> String {
        (*self).formatted(colored)
    }
}
