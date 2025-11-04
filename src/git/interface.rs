use crate::git::error::{GitError, GitInterfaceError};
use crate::git::model::*;
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
    model: TreeDataModel,
    raw_git_interface: RawGitInterface,
}
impl GitInterface {
    pub fn new() -> Self {
        let raw_interface = RawGitInterface {};
        let mut interface = Self {
            model: TreeDataModel::new(),
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
            if !branch.is_empty() {
                self.model
                    .insert_qualified_path(QualifiedPath::from(branch));
            }
        }
        Ok(())
    }
    pub fn get_model(&self) -> &TreeDataModel {
        &self.model
    }
    pub fn get_current_branch(&self) -> Result<String, GitError> {
        Ok(u8_to_string(
            &self
                .raw_git_interface
                .run(vec!["branch", "--show-current"])?
                .stdout,
        ))
    }
    pub fn get_current_qualified_path(&self) -> Result<QualifiedPath, GitError> {
        Ok(QualifiedPath::from(self.get_current_branch()?))
    }
    pub fn get_current_area(&self) -> Result<QualifiedPath, GitError> {
        let current_branch = self.get_current_qualified_path()?;
        Ok(QualifiedPath::from(current_branch.first().unwrap().clone()))
    }
    pub fn checkout(&self, path: &QualifiedPath, create: bool) -> Result<Output, GitError> {
        if create {
            Ok(self
                .raw_git_interface
                .run(vec!["checkout", "-b", path.to_git_branch().as_str()])?)
        } else {
            if !self.model.has_branch(&path) {
                return Err(GitError::GitInterface(GitInterfaceError::new(
                    format!("Cannot checkout branch {}: does not exist", path).as_str(),
                )));
            }
            Ok(self
                .raw_git_interface
                .run(vec!["checkout", path.to_git_branch().as_str()])?)
        }
    }
    pub fn merge(&self, paths: &Vec<QualifiedPath>) -> Result<Output, GitError> {
        let mut base = vec!["merge"];
        let new_paths: Vec<String> = paths.iter().map(|s| s.to_git_branch()).collect();
        let converted_paths: Vec<&str> = new_paths.iter().map(|p| p.as_str()).collect();
        base.extend(converted_paths);
        Ok(self.raw_git_interface.run(base)?)
    }
}
