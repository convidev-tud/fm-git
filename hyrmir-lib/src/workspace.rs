use std::cell::RefCell;
use std::rc::Rc;
use crate::derivation::DerivationManager;
use crate::model::*;
use crate::vcs::VCS;


pub struct Workspace<S: SymbolicNodeType, V: VCS> {
    virtual_root: Rc<RefCell<Node>>,
    vcs: V,
}

impl<S: SymbolicNodeType, V: VCS> Workspace<S, V> {
    pub fn new(repo: Repository<V>) -> Workspace<S, V> {
        Workspace { repo }
    }
}

impl<V: VCS> Workspace<Feature<Concrete>, V> {
    pub fn merge<T: CanMergeWithFeature>(&self, path: &NodePath<T, V>) {
        todo!()
    }
}

impl<V: VCS> Workspace<Product<Concrete>, V> {
    pub fn derivation(&self) -> DerivationManager {
        todo!()
    }
}

impl<S: IsConcrete, V: VCS> Workspace<S, V> {
    pub fn view<NewS: IsConcrete>(&self, path: &NodePath<NewS, V>) -> Workspace<NewS, V> {
        todo!()
    }

    pub fn simulate_merge(&self) {
        todo!()
    }
}
