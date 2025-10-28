use crate::git::error::{GitError, GitInterfaceError};
use crate::git::model::{BranchDataModel, SymPath};
use crate::util::u8_to_string;
use std::io;
use std::process::{Command, Output};

#[derive(Clone, Debug)]
struct RawGitInterface;
impl RawGitInterface {
    fn build_git_command(&self) -> Command {
        Command::new("git")
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
            model: BranchDataModel::new(),
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
    pub fn get_current_qualified_branch_name(&self) -> Result<String, GitError> {
        Ok(self.model.transform_to_qualified_path(u8_to_string(
            &self
                .raw_git_interface
                .run(vec!["branch", "--show-current"])?
                .stdout,
        )))
    }
    pub fn get_current_path(&'_ self) -> Result<SymPath<'_>, GitError> {
        Ok(self
            .model
            .get_global_root()
            .get_path(self.get_current_qualified_branch_name()?)
            .unwrap())
    }
    pub fn get_current_area(&self) -> Result<String, GitError> {
        Ok(self.get_current_path()?.get_first().unwrap().get_name().clone())
    }
    pub fn checkout(&self, qualified_path: &str) -> Result<Output, GitError> {
        if !self.model.has_qualified_path(qualified_path) {
            return Err(GitError::GitInterface(GitInterfaceError::new(
                format!("Cannot checkout branch {}: does not exist", qualified_path).as_str(),
            )));
        }
        Ok(self.raw_git_interface.run(vec![
            "checkout",
            self.model.get_git_branch(qualified_path).unwrap().as_str(),
        ])?)
    }
}
