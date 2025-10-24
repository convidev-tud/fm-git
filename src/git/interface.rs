use crate::git::model::BranchDataModel;
use crate::util::u8_to_string;
use std::fmt::Display;
use std::io;
use std::process::{Command, Output};
use crate::git::error::{GitError, GitInterfaceError};

#[derive(Clone, Debug)]
struct RawGitInterface;
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
    pub fn branch(&self) -> io::Result<Output> {
        self.build_git_command().arg("branch").output()
    }
    pub fn get_default_branch(&self) -> &str {
        "main"
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
        Self {
            model: BranchDataModel::new(raw_interface.get_default_branch()),
            raw_git_interface: raw_interface,
        }
    }
    fn update_complete_model(&mut self) -> Result<&BranchDataModel, io::Error> {
        let output = self.raw_git_interface.branch()?;
        let all_branches: Vec<String> = u8_to_string(&output.stdout)
            .split("\n")
            .map(|raw_string| raw_string.trim().to_string())
            .collect();
        for branch in all_branches {
            self.model.insert_from_git_native_branch(&branch);
        }
        Ok(&self.model)
    }
    pub fn get_model(&mut self) -> Result<&BranchDataModel, io::Error> {
        self.update_complete_model()
    }
    pub fn checkout_global_root(&self) -> Result<Output, io::Error> {
        self.raw_git_interface
            .checkout(self.raw_git_interface.get_default_branch(), false)
    }
    pub fn checkout(&self, branch: &str, create: bool) -> Result<Output, GitError> {
        let maybe_qualified_path = self.model.expand_from_short(branch);
        if maybe_qualified_path.is_none() {
            return Err(GitError::GitInterface(GitInterfaceError::new(
                format!("Cannot checkout branch {}: does not exist", branch).as_str(),
            )));
        }
        Ok(self.raw_git_interface.checkout(
            self.model
                .get_git_branch(maybe_qualified_path.unwrap().as_str())
                .unwrap(),
            false,
        )?)
    }
}
