use hyrmir_lib::model::NormalizedPath;
use hyrmir_lib::vcs::{VCS, VCSError, VersionId};
use std::cell::RefCell;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output};
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Git command error")]
pub struct GitCommandError {
    git_output: String,
    msg: String,
}
impl GitCommandError {
    pub fn new<S1: Into<String>, S2: Into<String>>(git_output: S1, msg: S2) -> GitCommandError {
        GitCommandError {
            git_output: git_output.into(),
            msg: msg.into(),
        }
    }
    pub fn get_git_output(&self) -> &String {
        &self.git_output
    }
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Git(#[from] GitCommandError),
}

impl VCSError for GitError {}

#[derive(Debug)]
pub struct Commit;

impl VersionId for Commit {
    type VersionError = GitError;

    fn get_metadata(&self, key: String) -> Result<String, Self::VersionError> {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GitPath {
    CurrentDirectory,
    CustomDirectory(PathBuf),
}

fn output_to_result(output: Output, command: &Vec<&str>) -> Result<String, GitCommandError> {
    let stdout = String::from_utf8(output.stdout).unwrap().trim().to_string();
    let stderr = String::from_utf8(output.stderr).unwrap().trim().to_string();
    let message = format!("{}\n{}", stdout, stderr).trim().to_string();
    if output.status.success() {
        Ok(message)
    } else {
        let code = output.status.code().unwrap();
        let git_command = command.join(" ");
        let error = format!(
            "fatal: Command 'git {}' returned with exit code {}:\n",
            git_command, code
        );
        Err(GitCommandError::new(message, error))
    }
}

fn status_to_result(status: ExitStatus, command: &Vec<&str>) -> Result<(), GitCommandError> {
    if status.success() {
        Ok(())
    } else {
        let code = status.code().unwrap();
        let git_command = command.join(" ");
        let error = format!(
            "fatal: Command 'git {}' returned with exit code {}:\n",
            git_command, code
        );
        Err(GitCommandError::new("", error))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct GitCLI {
    path: GitPath,
    colored: bool,
}
impl GitCLI {
    pub fn in_current_directory() -> Self {
        Self::new(GitPath::CurrentDirectory)
    }
    pub fn in_custom_directory(path: PathBuf) -> Self {
        Self::new(GitPath::CustomDirectory(path))
    }
    pub fn new(path: GitPath) -> Self {
        Self {
            path,
            colored: false,
        }
    }
    pub fn colored(&mut self, colored: bool) {
        self.colored = colored;
    }
    pub fn prepare_command(&self, args: &Vec<&str>) -> Vec<String> {
        let mut arguments: Vec<String> = vec![];
        match self.path {
            GitPath::CurrentDirectory => {}
            GitPath::CustomDirectory(ref path) => {
                arguments.push(format!("--git-dir={}/.git", path.to_str().unwrap()));
                arguments.push(format!("--work-tree={}", path.to_str().unwrap()));
            }
        }
        if self.colored {
            arguments.push("-c".to_string());
            arguments.push("color.ui=always".to_string());
        }
        arguments.extend(args.into_iter().map(|arg| arg.to_string()));
        arguments
    }
    pub fn run_attached(&self, args: &Vec<&str>) -> io::Result<Output> {
        let mut base = Command::new("git");
        let arguments = self.prepare_command(args);
        base.args(arguments).output()
    }
    pub fn run_detached(&self, args: &Vec<&str>) -> io::Result<ExitStatus> {
        let mut base = Command::new("git");
        let arguments = self.prepare_command(args);
        base.args(arguments).status()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Git {
    git_cli: Rc<RefCell<GitCLI>>,
}

impl Git {
    pub fn new(git_cli: Rc<RefCell<GitCLI>>) -> Self {
        Self { git_cli }
    }
}

impl VCS for Git {
    type VCSError = GitError;
    type VersionId = Commit;

    fn get_current_path(&self) -> Result<NormalizedPath, Self::VCSError> {
        todo!()
    }

    fn get_version(&self, version: &String) -> Result<Self::VersionId, Self::VCSError> {
        todo!()
    }

    fn version_exists_on_path(
        &self,
        path: &NormalizedPath,
        version: &String,
    ) -> Result<bool, Self::VCSError> {
        todo!()
    }

    fn iter_concrete_paths(&self) -> impl Iterator<Item = Result<NormalizedPath, Self::VCSError>> {
        vec![].into_iter()
    }

    fn iter_versions(
        &self,
        path: &NormalizedPath,
    ) -> impl Iterator<Item = Result<Self::VersionId, Self::VCSError>> {
        vec![].into_iter()
    }

    fn format_status_message(
        &self,
        current_path_msg: String,
        pre_status: String,
        post_status: String,
        colored: bool,
    ) -> Result<String, Self::VCSError> {
        let command = vec!["status"];
        let out = self.git_cli.borrow().run_attached(&command)?;
        let original = output_to_result(out, &command)?;
        let no_first_line = original.split("\n").collect::<Vec<_>>()[1..]
            .to_vec()
            .join("\n")
            .trim()
            .to_string();
        Ok(format!(
            "{current_path_msg}\n{pre_status}\n{no_first_line}\n{post_status}"
        ))
    }
}
