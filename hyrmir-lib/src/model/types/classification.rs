use crate::model::*;
use crate::vcs::VCS;



impl<'a, S: IsConcrete, V: VCS> SemanticView<'a, S, V> {
    pub fn get_id(&self) -> usize {
        self.get_node().borrow().get_branch_info().unwrap().get_id()
    }
}
