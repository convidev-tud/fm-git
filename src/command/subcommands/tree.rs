use crate::command::*;
use clap::builder::Str;
use clap::Command;
use termtree::Tree;

#[derive(Clone)]
pub struct TreeCommand {
    name: Str,
}

impl TreeCommand {
    pub fn new() -> Self { Self { name: "tree".into() } }
}

impl CommandDefinition for TreeCommand {
    fn get_name(&self) -> String {
        self.name.clone().into()
    }
    fn build_command(&self) -> Command {
        Command::new(self.name.clone())
            .about("Displays the tree structure")
    }
    fn run_command(&self) -> CommandResult {
        let mut tree: Tree<String> = Tree::new("root".into());
        tree.push(Tree::new("files".into()));
        tree.push(Tree::new("files2".into()));
        CommandResult {
            stdout: tree.to_string(),
            stderr: "".into(),
        }
    }
}
