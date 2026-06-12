use crate::model::WrongNodeTypeError;
use crate::model::*;
use crate::vcs::VCS;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use thiserror::Error;
use crate::workspace::Workspace;

#[derive(Error, Debug)]
pub enum ScanError<V: VCS> {
    #[error(transparent)]
    WrongType(#[from] WrongNodeTypeError),
    #[error("ScanError::VCS")]
    VCS(#[source] V::VCSError),
}

#[derive(Debug)]
pub struct Repository<V: VCS> {
    virtual_root: Rc<RefCell<Node>>,
    vcs: Rc<RefCell<V>>,
    repo_scanned: RefCell<bool>,
}

impl<V: VCS> Repository<V> {
    fn scan_repository(&self) -> Result<(), ScanError<V>> {
        let mut root = self.virtual_root.borrow_mut();
        let vcs = self.vcs.borrow();
        for path in vcs.iter_concrete_paths() {
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
            vcs: Rc::new(RefCell::new(vcs)),
            repo_scanned: RefCell::new(false),
        }
    }

    pub fn get_virtual_root(&self) -> Result<NodePath<VirtualRoot, V>, ScanError<V>> {
        let scanned = self.repo_scanned.borrow().deref();
        if !scanned {
            self.scan_repository()?
        }
        NodePath::new(vec![self.virtual_root.clone()], self.vcs.clone())?.into()
    }
    
    pub fn get_workspace(&self) -> Result<Workspace<V>, ScanError<V>> {
        let root = self.get_virtual_root()?;
        Workspace::new(root, self.vcs.clone()).into()
    }
}
