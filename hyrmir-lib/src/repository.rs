use crate::model::*;
use crate::vcs::*;
use crate::workspace::*;
use indextree::{Arena, Node, NodeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use itertools::Itertools;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MalformedModelVCSError<VE: VCSError> {
    #[error(transparent)]
    MalformedModel(#[from] MalformedModelError),
    #[error(transparent)]
    VCS(#[from] VE),
}

type ScanError<VE> = MalformedModelVCSError<VE>;

pub struct RepositoryLoader<V: VCS> {
    repository: Repository<V>,
}

impl<V: VCS> RepositoryLoader<V> {
    pub fn new(vcs: V) -> Self {
        Self {
            repository: Repository::new(vcs),
        }
    }

    pub fn load_repo(&mut self) -> Result<&mut Repository<V>, ScanError<V::VCSError>> {
        self.repository.scan_repository()?;
        Ok(&mut self.repository)
    }
}

#[derive(Debug)]
pub struct Repository<V: VCS> {
    arena: Arena<RefCell<NodeData<V>>>,
    root_id: NodeId,
    vcs_id_to_node_id: HashMap<usize, NodeId>,
    vcs: V,
}

impl<V: VCS> Repository<V> {
    fn add_node(
        &mut self,
        parent_id: NodeId,
        name: impl Into<String>,
        branch_info: Option<BranchInfo<V>>,
    ) -> Result<NodeId, MalformedModelError> {
        let name = name.into();
        let parent = self.arena[parent_id].get();
        let node_type = parent
            .borrow()
            .get_type()
            .decide_next_type(&name, branch_info.is_some())?;
        let new_node = NodeData::new(name.clone(), node_type, branch_info);
        let new_id = self.arena.new_node(RefCell::new(new_node));
        parent_id.append(new_id, &mut self.arena);
        let mut parent_mut = self.arena[parent_id].get_mut().borrow_mut();
        parent_mut.add_child(new_id, name);
        Ok(new_id)
    }

    fn update_node(
        &self,
        id: NodeId,
        new_name: impl Into<String>,
        branch_info: Option<BranchInfo<V>>,
    ) -> Result<NodeId, MalformedModelError> {
        let new_name = new_name.into();
        let mut node = self.arena[id].get().borrow_mut();
        let mut parent = self.arena[id.parent(&self.arena).unwrap()]
            .get()
            .borrow_mut();
        let old_name = node.get_name();
        let new_type = parent
            .get_type()
            .decide_next_type(&new_name, branch_info.is_some())?;
        parent.remove_child(&old_name);
        parent.add_child(id, &new_name);
        node.update_name(new_name);
        node.update_type(new_type);
        node.update_branch_info(branch_info);
        Ok(id)
    }

    fn insert_path(
        &mut self,
        path: &NormalizedPath,
        branch_info: BranchInfo<V>,
    ) -> Result<NodeId, MalformedModelError> {
        let mut current = self.root_id;
        let mut current_node = self.arena[current].get();
        for (index, p) in path.iter_all_segments().enumerate() {
            let borrowed = current_node.borrow();
            if index == path.len() - 1 {
                let id = match borrowed.get_child(p) {
                    Some(child) => {
                        let child = *child;
                        drop(borrowed);
                        self.update_node(child, p, Some(branch_info))?
                    }
                    None => {
                        drop(borrowed);
                        self.add_node(current, p, Some(branch_info))?
                    }
                };
                return Ok(id);
            } else {
                let next = match borrowed.get_child(p) {
                    Some(id) => *id,
                    None => {
                        drop(borrowed);
                        self.add_node(current, p, None)?
                    }
                };
                current = next;
                current_node = self.arena[current].get();
            }
        }
        Ok(current)
    }

    fn scan_repository(&mut self) -> Result<(), ScanError<V::VCSError>> {
        for path_info in self.vcs.get_local_paths()? {
            let path = path_info.get_path();
            let p = if path.is_absolute() {
                &path.strip_n_left(1)
            } else {
                panic!("Paths must be absolute when loaded into repository")
            };
            let vcs_id = path_info.get_id();
            let head = path_info.get_head();
            let info = BranchInfo::new(vcs_id, head.clone());
            let id = self.insert_path(p, info)?;
            self.vcs_id_to_node_id.insert(vcs_id, id);
        }
        Ok(())
    }

    pub(crate) fn get_root_id(&self) -> NodeId {
        self.root_id
    }

    pub(crate) fn get_root_node(&self) -> &Node<RefCell<NodeData<V>>> {
        self.arena.get(self.root_id).unwrap()
    }

    pub fn new(vcs: V) -> Self {
        let root = NodeData::new("".to_string(), NodeType::VirtualRoot, None);
        let mut arena = Arena::new();
        let root_id = arena.new_node(RefCell::new(root));
        Self {
            arena,
            root_id,
            vcs_id_to_node_id: HashMap::new(),
            vcs,
        }
    }

    pub fn get_arena(&self) -> &Arena<RefCell<NodeData<V>>> {
        &self.arena
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node<RefCell<NodeData<V>>>> {
        self.arena.get(id)
    }

    pub fn get_vcs(&self) -> &V {
        &self.vcs
    }

    pub fn root_view(&self) -> StructureView<VirtualRoot, Shared, V> {
        StructureView::<VirtualRoot, Shared, V>::new(self.root_id, &self).unwrap()
    }

    pub fn get_workspace<S: IsConcrete>(
        &'_ self,
        path: PathBuf,
    ) -> Result<Workspace<'_, S, Rev, Shared, V>, GetWorkSpaceError<V, V::VCSError>> {
        Workspace::new(path, self)
    }
}

#[cfg(test)]
pub mod test_utils {
    use crate::vcs::test_utils::TestVCS;
    use super::*;
    
    pub fn prepare_repo() -> Repository<TestVCS> {
        let mut repo = Repository::new(TestVCS::new());
        repo.scan_repository().unwrap();
        repo
    }
}
