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
            self.model.insert_from_git_native_branch(&branch);
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
    pub fn get_current_qualified_path(&self) -> Result<String, GitError> {
        Ok(TreeDataModel::transform_to_qualified_path(
            self.get_current_branch()?,
        ))
    }
    pub fn get_current_area_node(&self) -> Result<&Node, GitError> {
        let current_branch = self.get_current_qualified_path()?;
        let current_split_branch = current_branch.split("/").collect::<Vec<&str>>();
        let current_area_name = current_split_branch.first().unwrap();
        Ok(self.model.get_area_node(current_area_name).unwrap())
    }
    pub fn checkout<S: Into<String> + Copy>(
        &self,
        qualified_path: S,
        create: bool,
    ) -> Result<Output, GitError> {
        if create {
            Ok(self.raw_git_interface.run(vec![
                "checkout",
                "-b",
                TreeDataModel::transform_to_branch(qualified_path).as_str(),
            ])?)
        } else {
            if !self.model.has_branch(qualified_path) {
                return Err(GitError::GitInterface(GitInterfaceError::new(
                    format!(
                        "Cannot checkout branch {}: does not exist",
                        qualified_path.into()
                    )
                    .as_str(),
                )));
            }
            Ok(self.raw_git_interface.run(vec![
                "checkout",
                self.model
                    .get_node_from_qualified_path(qualified_path)
                    .unwrap()
                    .get_branch()
                    .unwrap(),
            ])?)
        }
    }
    pub fn merge(&self, paths: Vec<&str>) -> Result<Output, GitError> {
        let mut base = vec!["merge"];
        base.extend(paths);
        Ok(self.raw_git_interface.run(base)?)
    }
}
