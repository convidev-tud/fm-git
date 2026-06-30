use crate::model::*;
use crate::vcs::{VCS, VCSError, VersionId};
use crate::workspace::{WorkSpaceError, Workspace};
use std::cell::RefCell;
use std::collections::HashMap;
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
        let no_version = path.strip_version();
        let mut new_node_vec = vec![self.virtual_root.clone()];
        for p in no_version.iter_segments(1, path.len()) {
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

    pub fn get_virtual_root_view(&self) -> PathView<VirtualRoot, V> {
        PathView::<VirtualRoot, V>::new(vec![self.virtual_root.clone()], &self).unwrap()
    }

    pub fn get_view<S: SymbolicNodeType>(
        &self,
        path: &NormalizedPath,
    ) -> Result<PathView<S, V>, TreeViewError<V::VersionId>> {
        let node_vec = self.get_node_vec(path);
        Ok(PathView::new(node_vec, &self)?)
    }

    pub fn get_path(&self, path: &NormalizedPath) -> Result<NodePath<V::VersionId>, V::VCSError> {
        let node_vec = self.get_node_vec(path);
        let version = match path.get_version_appendix() {
            Some(version) => {
                let version_id = if self
                    .get_vcs()
                    .version_exists_on_path(&node_vec.to_normalized_path(), &version)?
                {
                    let version_id = self.get_vcs().get_version(&version)?.unwrap();
                    let mut node = node_vec.last().unwrap().borrow_mut();
                    node.mut_get_branch_info().unwrap().insert_version(version_id.clone());
                    version_id
                } else {
                    V::VersionId::new(version)
                };
                VersionPointer::Version(version_id)
            }
            None => VersionPointer::Default,
        };
        Ok(NodePath::new(node_vec, version))
    }

    pub fn get_path_by_id(&self, id: usize) -> Option<&NormalizedPath> {
        self.id_to_path.get(&id)
    }

    pub fn get_workspace<S: IsConcrete>(
        &self,
    ) -> Result<Workspace<S, V>, WorkSpaceError<V::VersionId, V::VCSError>> {
        Workspace::new(&self)
    }
}
