use hyrmir_lib::model::*;
use hyrmir_lib::vcs::{PathInfo, RevisionId, VCS, VCSError};
use std::fmt::{Display, Formatter};
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Output};
use thiserror::Error;

#[derive(Error, Debug)]
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

impl Display for GitCommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.git_output)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitHash {
    hash: String,
}

impl CommitHash {
    fn new(id: impl Into<String>) -> Self {
        Self { hash: id.into() }
    }
}

impl RevisionId for CommitHash {
    fn get_full_id(&self) -> String {
        self.hash.clone()
    }

    fn get_printable_id(&self) -> String {
        if &self.hash.len() > &8 {
            self.hash[..8].to_string()
        } else {
            self.hash.clone()
        }
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Git {
    git_cli: GitCLI,
}

impl Git {
    pub fn new() -> Self {
        Self {
            git_cli: GitCLI::new(GitPath::CurrentDirectory),
        }
    }

    fn split_branch<'a>(&self, branch: &'a str) -> (&'a str, &'a str) {
        branch.rsplit_once('.').unwrap()
    }
}

impl VCS for Git {
    type VCSError = GitError;
    type RevisionId = CommitHash;

    fn get_current_path(&self, _: &PathBuf) -> Result<Option<Normalized>, Self::VCSError> {
        let command = vec!["branch", "--show-current"];
        let out = self.git_cli.run_attached(&command)?;
        let path_string = output_to_result(out, &command)?;
        if !path_string.is_empty() {
            let (path, _) = self.split_branch(&path_string);
            Ok(Some(Normalized::new(
                NormalizedPath::from_git_branch(path).as_absolute(),
                NormalizedRevision::None,
            )))
        } else {
            Ok(None)
        }
    }

    fn get_local_paths(&self) -> Result<Vec<PathInfo<Self::RevisionId>>, Self::VCSError> {
        let branch_command = vec!["branch", "--format=%(refname:short) %(objectname)"];
        let branch_output = self.git_cli.run_attached(&branch_command)?;
        let all_branches: Vec<PathInfo<Self::RevisionId>> = String::from_utf8(branch_output.stdout)
            .unwrap()
            .trim()
            .split("\n")
            .map(|raw_string| {
                let split = raw_string.split(" ").collect::<Vec<&str>>();
                let path_segment = split[0].to_string();
                let hash = split[1].to_string();
                let (path, id) = self.split_branch(&path_segment);
                PathInfo::new(
                    id.parse().unwrap(),
                    NormalizedPath::from_git_branch(path).as_absolute(),
                    CommitHash::new(hash),
                )
            })
            .collect();
        Ok(all_branches)
    }

    fn get_revision(
        &self,
        version: impl Into<String>,
    ) -> Result<Option<Self::RevisionId>, Self::VCSError> {
        todo!()
    }

    fn revision_exists_on_path(
        &self,
        path: &NormalizedPath,
        version: impl Into<String>,
    ) -> Result<bool, Self::VCSError> {
        todo!()
    }

    fn iter_versions(
        &self,
        path: &NormalizedPath,
    ) -> impl Iterator<Item = Result<Self::RevisionId, Self::VCSError>> {
        vec![].into_iter()
    }

    fn get_status_without_current_branch(&self, colored: bool) -> Result<String, Self::VCSError> {
        let command = vec!["status"];
        let out = self.git_cli.run_attached(&command)?;
        let original = output_to_result(out, &command)?;
        let status = original.split("\n").collect::<Vec<_>>()[1..]
            .to_vec()
            .join("\n")
            .trim()
            .to_string();
        Ok(status)
    }

    fn switch_to_branch(
        &self,
        id: usize,
        path: &impl ToNormalizedPath,
        _: &PathBuf,
    ) -> Result<String, Self::VCSError> {
        let path = path.to_normalized_path();
        let branch = path.to_git_branch(id);
        let command = vec!["checkout", branch.as_str()];
        let out = self.git_cli.run_attached(&command)?;
        Ok(output_to_result(out, &command)?)
    }
}

trait GitBranch {
    fn from_git_branch(branch: &str) -> Self;
    fn to_git_branch(&self, id: usize) -> String;
}

impl GitBranch for NormalizedPath {
    fn from_git_branch(branch: &str) -> Self {
        let new = branch.replace(".", "");
        new.to_normalized_path()
    }

    fn to_git_branch(&self, id: usize) -> String {
        let trimmed_path = self.trim_whitespaces();
        let path = trimmed_path
            .iter_all_segments()
            .map(|x| x.to_string() + ".")
            .collect::<Vec<String>>()
            .join("/");
        path + id.to_string().as_str()
    }
}
