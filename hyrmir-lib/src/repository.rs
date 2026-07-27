use crate::model::*;
use crate::vcs::*;
use crate::workspace::*;
use indextree::{Arena, NodeId};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MalformedModelVCSError<VE: VCSError> {
    #[error(transparent)]
    MalformedModel(#[from] MalformedModelError),
    #[error(transparent)]
    VCS(#[from] VE),
}

type ScanError<VE: VCSError> = MalformedModelVCSError<VE>;

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
    arena: Arena<NodeData<V>>,
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
        let node_type = parent.get_type().decide_next_type(&name, branch_info.is_some())?;
        let new_node = NodeData::new(name.clone(), node_type, branch_info);
        let new_id = self.arena.new_node(new_node);
        let parent_mut = self.arena[parent_id].get_mut();
        parent_mut.add_child(new_id, name);
        Ok(new_id)
    }

    fn update_node(
        &mut self,
        id: NodeId,
        name: impl Into<String>,
        branch_info: Option<BranchInfo<V>>
    ) -> Result<NodeId, MalformedModelError> {
        let name = name.into();
        let old_name = self.arena.get(id).unwrap().get().get_name().clone();
        let parent_id = id.parent(&self.arena).unwrap();
        let parent_data = self.arena.get_mut(parent_id).unwrap().get_mut();
        let new_type = parent_data.get_type().decide_next_type(&name, branch_info.is_some())?;
        parent_data.remove_child(&old_name);
        parent_data.add_child(id, name);
        let node_data = self.arena.get_mut(id).unwrap().get_mut();
        node_data.update_type(new_type);
        node_data.update_branch_info(branch_info);
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
            if index == path.len() - 1 {
                let id = match current_node.get_child(p)
                {
                    Some(id) => self.update_node(id.clone(), p, Some(branch_info))?,
                    None => self.add_node(current, p, Some(branch_info))?,
                };
                return Ok(id);
            } else {
                let next = match current_node.get_child(p)
                {
                    Some(id) => *id,
                    None => {
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

    pub fn new(vcs: V) -> Self {
        let root = NodeData::new("".to_string(), NodeType::VirtualRoot, None);
        let mut arena = Arena::new();
        let root_id = arena.new_node(root);
        Self {
            arena,
            root_id,
            vcs_id_to_node_id: HashMap::new(),
            vcs,
        }
    }

    pub fn get_vcs(&self) -> &V {
        &self.vcs
    }

    pub fn get_virtual_root_view(&self) -> StructureView<VirtualRoot, V> {
        StructureView::<VirtualRoot, V>::new(vec![self.virtual_root.clone()], &self).unwrap()
    }

    pub fn get_view<S: SymbolicNodeType>(
        &self,
        path: &impl ToNormalizedPath,
    ) -> Result<StructureView<S, V>, SemanticViewError<V::VersionId>> {
        let node_vec = self.get_node_vec(&path.to_normalized_path());
        Ok(StructureView::new(node_vec, &self)?)
    }

    pub fn get_path_by_id(&self, id: usize) -> Option<&NormalizedPath> {
        self.vcs_id_to_node_id.get(&id)
    }

    pub fn get_workspace<S: IsConcrete>(
        &'_ self,
        path: PathBuf,
    ) -> Result<WorkspaceKind<'_, S, V>, GetWorkSpaceError<V::VersionId, V::VCSError>> {
        WorkspaceKind::<S, V>::get(path, &self)
    }
}
