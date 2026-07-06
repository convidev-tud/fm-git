use crate::model::*;
use crate::vcs::*;
use crate::workspace::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
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
    virtual_root: Rc<RefCell<Node<V::VersionId>>>,
    id_to_path: HashMap<usize, NormalizedPath>,
    vcs: V,
}

impl<V: VCS> Repository<V> {
    fn scan_repository(&mut self) -> Result<(), ScanError<V::VCSError>> {
        let mut root = self.virtual_root.borrow_mut();
        for path_info in self.vcs.get_local_paths()? {
            let path = path_info.get_path();
            let p = if path.is_absolute() {
                &path.strip_n_left(1)
            } else {
                panic!("Paths must be absolute when loaded into repository")
            };
            let id = path_info.get_id();
            let head = path_info.get_version();
            let info = BranchInfo::<V::VersionId>::new(id, head.clone());
            root.insert_path(p, Some(info))?;
            self.id_to_path.insert(id, path.clone());
        }
        Ok(())
    }

    fn get_node_vec(&self, path: &NormalizedPath) -> Vec<Rc<RefCell<Node<V::VersionId>>>> {
        let mut new_node_vec = vec![self.virtual_root.clone()];
        for p in path.iter_segments(1, path.len()) {
            let current = new_node_vec.last().unwrap();
            let node = if let Some(node) = current.borrow().get_child(p) {
                node
            } else {
                Rc::new(RefCell::new(Node::new(
                    p.clone(),
                    NodeType::NonExistent,
                    None,
                )))
            };
            new_node_vec.push(node);
        }
        new_node_vec
    }

    pub fn new(vcs: V) -> Self {
        let root = Node::new("".to_string(), NodeType::VirtualRoot, None);
        Self {
            virtual_root: Rc::new(RefCell::new(root)),
            id_to_path: HashMap::new(),
            vcs,
        }
    }

    pub fn get_vcs(&self) -> &V {
        &self.vcs
    }

    pub fn get_virtual_root_view(&self) -> SemanticView<VirtualRoot, V> {
        SemanticView::<VirtualRoot, V>::new(vec![self.virtual_root.clone()], &self).unwrap()
    }

    pub fn get_view<S: SymbolicNodeType>(
        &self,
        path: &impl ToNormalizedPath,
    ) -> Result<SemanticView<S, V>, SemanticViewError<V::VersionId>> {
        let node_vec = self.get_node_vec(&path.to_normalized_path());
        Ok(SemanticView::new(node_vec, &self)?)
    }

    pub fn get_path_by_id(&self, id: usize) -> Option<&NormalizedPath> {
        self.id_to_path.get(&id)
    }

    pub fn get_workspace<S: IsConcrete>(
        &'_ self,
        path: PathBuf,
    ) -> Result<WorkspaceKind<'_, S, V>, GetWorkSpaceError<V::VersionId, V::VCSError>> {
        WorkspaceKind::<S, V>::get(path, &self)
    }
}
