use std::collections::HashMap;
use crate::git::tree::SymFeatureNode;
use crate::util::u8_to_string;
use std::io;
use std::process::{Command, Output};


#[derive(Clone, Debug)]
struct RawGitInterface {}
impl RawGitInterface {
    fn build_git_command(&self) -> Command {
        Command::new("git")
    }
    pub fn checkout(&self, branch: &str, create: bool) -> io::Result<Output> {
        let mut command = self.build_git_command();
        command.arg("checkout");
        if create {
            command.arg("-b");
        }
        command.arg(branch).output()
    }
    pub fn merge(&self, branches: Vec<String>) -> io::Result<Output> {
        self.build_git_command()
            .arg("merge")
            .args(branches)
            .output()
    }
    pub fn get_main_branch(&self) -> &str {
        "main"
    }
}

#[derive(Clone, Debug)]
pub struct GitInterface {
    feature_root_node: SymFeatureNode,
    feature_name_to_path: HashMap<String, Vec<String>>,
    raw_git_interface: RawGitInterface,
}
impl GitInterface {
    pub fn new() -> Self {
        let raw_interface = RawGitInterface {};
        Self {
            feature_root_node: SymFeatureNode::new(raw_interface.get_main_branch()),
            feature_name_to_path: HashMap::new(),
            raw_git_interface: raw_interface,
        }
    }
    fn build_complete_tree(&mut self) {
        let output = std::process::Command::new("git")
            .arg("branch")
            .output()
            .expect("failed to execute process");
        let all_branches: Vec<String> = u8_to_string(&output.stdout)
            .split("\n")
            .map(|raw_string| raw_string.trim().to_string())
            .collect();
        for branch in all_branches {
            let converted_branch = branch
                .replace("*", "")
                .replace("_", "")
                .trim()
                .to_string();
            let split_branch = converted_branch.split("/").collect::<Vec<&str>>();
            let feature_name = split_branch.last().unwrap().to_string();
            if !self.feature_name_to_path.contains_key(&feature_name) {
                self.feature_name_to_path.insert(feature_name.clone(), vec![converted_branch.clone()]);
            } else {
                self.feature_name_to_path.get_mut(&feature_name).unwrap().push(converted_branch.clone());
            }
            self.feature_root_node.add_children_recursive(
                split_branch
            );
        }
    }
    pub fn get_complete_tree(&mut self) -> &SymFeatureNode {
        self.build_complete_tree();
        &self.feature_root_node
    }
    pub fn get_unique_names(&mut self) -> Vec<String> {
        self.build_complete_tree();
        let mut unique: Vec<String> = Vec::new();
        self.feature_name_to_path
            .iter()
            .filter_map(|(k, v)| {
                match v.len() {
                    0 => { panic!("Must be a bug") }
                    1 => Some(vec![k.clone()]),
                    _ => None,
                }
            })
            .for_each(|e| unique.extend(e));
        unique
    }
    pub fn get_main_branch(&self) -> &str {
        self.raw_git_interface.get_main_branch()
    }
}
