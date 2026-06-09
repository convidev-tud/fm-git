use crate::model::*;
use crate::vcs::VCS;
use std::cell::RefCell;
use std::rc::Rc;


pub struct Workspace<V: VCS> {
    virtual_root: Rc<RefCell<Node>>,
    vcs: Rc<RefCell<V>>,
}

impl<V: VCS> Workspace<V> {
    fn get_vcs(&self) -> &Rc<RefCell<V>> {
        &self.vcs
    }

    pub fn new(virtual_root: Rc<RefCell<Node>>, vcs: Rc<RefCell<V>>) -> Self {
        Self { virtual_root, vcs }
    }

    pub fn get_virtual_root(&self) -> NodePath<VirtualRoot, V> {
        NodePath::new(vec![self.virtual_root.clone()], self.get_vcs().clone()).unwrap()
    }
    
    pub fn status(&self, colored: bool) -> String {
        self.get_vcs().borrow().status(colored)
    }

    pub fn format_status(
        &self,
        current_path_msg: String,
        extra_msg: String,
        colored: bool,
    ) -> String {
        self.get_vcs().borrow().format_status(
            current_path_msg,
            extra_msg,
            colored,
        )
    }

    pub fn commit(&self) { todo!() }
}
