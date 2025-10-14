use crate::util::u8_to_string;
use std::collections::HashMap;
use std::path::Path;

#[derive(Clone, Debug)]
struct FeatureNode {
    children: Vec<FeatureNode>,
    git_branch: Option<String>,
}
impl FeatureNode {}

#[derive(Clone, Debug)]
struct FeatureTree {
    qualified_path_to_node: HashMap<String, FeatureNode>,
    root_nodes: Vec<String>,
}
impl FeatureTree {
    pub fn new() -> Self {
        Self {
            qualified_path_to_node: HashMap::new(),
            root_nodes: Vec::new(),
        }
    }
    pub fn insert(&mut self, branch: String) {
        let qualified_name = branch.replace("_", "");
        let qualified_path = Path::new(&qualified_name);
        let subbranches = branch.split("/").collect::<Vec<&str>>();
        let mut current_parent = qualified_path;
        let mut all_parent_paths: Vec<&Path> = subbranches
            .iter()
            .map(|_| {
                current_parent = current_parent.parent().unwrap();
                current_parent
            })
            .collect();
        all_parent_paths.reverse();


        if !self.qualified_path_to_node.contains_key(&qualified_name) {
            self.qualified_path_to_node.insert(
                qualified_name.clone(),
                FeatureNode {
                    children: vec![],
                    git_branch: Some(branch.clone()),
                },
            );
        }
        let maybe_parent = qualified_path.parent();
        if maybe_parent.is_some() {

        }
    }
}

#[derive(Clone, Debug)]
pub struct GitInterface {
    feature_tree: FeatureTree,
}

impl GitInterface {
    pub fn new() -> Self {
        Self {
            feature_tree: FeatureTree::new(),
        }
    }
    pub fn get_all_branches(&self) -> Vec<String> {
        let output = std::process::Command::new("git")
            .arg("branch")
            .output()
            .expect("failed to execute process");
        u8_to_string(&output.stdout)
            .replace("*", "")
            .split("\n")
            .map(|raw_string| raw_string.trim().to_string())
            .collect()
    }
    fn build_feature_tree(&mut self) {
        let output = std::process::Command::new("git")
            .arg("branch")
            .output()
            .expect("failed to execute process");
        let all_branches: Vec<String> = u8_to_string(&output.stdout)
            .split("\n")
            .map(|raw_string| raw_string.trim().to_string())
            .collect();
        for branch in all_branches {
            self.feature_tree.insert(branch);
        }
    }
}
