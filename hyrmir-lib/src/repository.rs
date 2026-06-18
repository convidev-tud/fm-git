use crate::model::*;
use crate::vcs::{VCSError, VCS};
use crate::workspace::Workspace;
use std::cell::RefCell;
use std::error::Error;
use std::ops::Deref;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScanError<V: VCSError> {
    #[error(transparent)]
    MalformedModel(#[from] MalformedModelError),
    #[error(transparent)]
    VCS(#[from] V),
}

#[derive(Debug)]
pub struct Repository<V: VCS> {
    virtual_root: Rc<RefCell<Node>>,
    vcs: V,
    repo_scanned: RefCell<bool>,
}

impl<V: VCS> Repository<V> {
    fn scan_repository(&self) -> Result<(), ScanError<V::VCSError>> {
        let mut root = self.virtual_root.borrow_mut();
        for path in self.get_vcs().iter_concrete_paths() {
            let unwrapped = path?;
            let p = if unwrapped.is_absolute() {
                unwrapped.strip_n_left(1)
            } else { unwrapped };
            root.insert_path(&p, true)?;
        }
        self.repo_scanned.replace(true);
        Ok(())
    }

    pub fn new(vcs: V) -> Self {
        let root = Node::new(
            "".to_string(),
            NodeType::VirtualRoot,
        );
        Self {
            virtual_root: Rc::new(RefCell::new(root)),
            vcs,
            repo_scanned: RefCell::new(false),
        }
    }

    pub fn get_vcs(&self) -> &V {
        &self.vcs
    }

    pub fn get_virtual_root(&self) -> Result<NodePath<VirtualRoot, V>, ScanError<V::VCSError>> {
        let scanned = self.repo_scanned.borrow().clone();
        if !scanned {
            self.scan_repository()?
        }
        Ok(NodePath::<VirtualRoot, V>::new(vec![self.virtual_root.clone()], self.vcs.clone(), None).unwrap())
    }
    
    pub fn get_workspace(&self) -> Result<Workspace<V>, ScanError<V::VCSError>> {
        let root = self.get_virtual_root()?;
        Ok(Workspace::new(root, self.vcs.clone()))
    }
}
