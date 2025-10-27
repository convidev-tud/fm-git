use crate::git::error::{GitError, GitInterfaceError};
use crate::git::model::BranchDataModel;
use crate::util::u8_to_string;
use std::io;
use std::process::{Command, Output};

#[derive(Clone, Debug)]
struct RawGitInterface;
impl RawGitInterface {
    fn build_git_command(&self) -> Command {
        Command::new("git")
    }
    pub fn get_default_branch(&self) -> &str {
        "main"
    }
    pub fn run(&self, args: Vec<&str>) -> io::Result<Output> {
        self.build_git_command().args(args).output()
    }
}

#[derive(Clone, Debug)]
pub struct GitInterface {
    model: BranchDataModel,
    raw_git_interface: RawGitInterface,
}
impl GitInterface {
    pub fn new() -> Self {
        let raw_interface = RawGitInterface {};
        let mut interface = Self {
            model: BranchDataModel::new(raw_interface.get_default_branch()),
            raw_git_interface: raw_interface,
        };
        match interface.update_complete_model() {
            Ok(_) => interface,
            Err(e) => panic!("{:?}", e),
        }
    }
    fn update_complete_model(&mut self) -> Result<(), io::Error> {
        let output = self.raw_git_interface.run(vec!["branch"])?;
        let all_branches: Vec<String> = u8_to_string(&output.stdout)
            .split("\n")
            .map(|raw_string| raw_string.trim().to_string())
            .collect();
        for branch in all_branches {
            self.model.insert_from_git_native_branch(&branch);
        }
        Ok(())
    }
    pub fn get_model(&self) -> &BranchDataModel {
        &self.model
    }
    pub fn checkout_global_root(&self) -> Result<Output, io::Error> {
        self.raw_git_interface.run(vec![
            "checkout",
            self.raw_git_interface.get_default_branch(),
        ])
    }
    pub fn checkout(&self, branch: &str, create: bool) -> Result<Output, GitError> {
        let maybe_qualified_path = self.model.expand_from_short(branch);
        if maybe_qualified_path.is_none() {
            return Err(GitError::GitInterface(GitInterfaceError::new(
                format!("Cannot checkout branch {}: does not exist", branch).as_str(),
            )));
        }
        Ok(self.raw_git_interface.run(vec![
            "checkout",
            self.model.get_git_branch(branch).unwrap().as_str(),
        ])?)
    }
}
