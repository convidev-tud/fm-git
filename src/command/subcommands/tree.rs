use crate::command::*;
use clap::{ArgMatches, Command};
use termtree::Tree;

#[derive(Clone, Debug)]
pub struct TreeCommand {}

impl CommandDefinition for TreeCommand {
    fn build_command(&self) -> Command {
        Command::new("tree")
            .about("Displays the tree structure")
    }
    fn run_command(
        &self,
        args: &ArgMatches,
        state: CommandState,
    ) -> CommandState {
        let mut tree: Tree<String> = Tree::new("root".into());
        tree.push(Tree::new("files".into()));
        tree.push(Tree::new("files2".into()));
        state.log_to_stdout(tree.to_string());
        state
    }
}
