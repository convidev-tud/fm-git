use crate::cli::*;
use crate::git::tree::SymFeatureNode;
use clap::{ArgMatches, Command};
use termtree::Tree;

fn transform_to_printable(root: &SymFeatureNode) -> Vec<Tree<String>> {
    root.iter_children()
        .map(|child| {
            let mut tree = Tree::new(child.name.clone());
            tree.leaves = transform_to_printable(child);
            tree
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct TreeCommand {}

impl CommandDefinition for TreeCommand {
    fn build_command(&self) -> Command {
        Command::new("tree")
            .about("Displays the tree structure")
            .disable_help_subcommand(true)
    }
}

impl CommandInterface for TreeCommand {
    fn run_command(&self, _args: &ArgMatches, _current: &CommandMap, state: &mut CommandContext) {
        let complete_tree = state.git.get_complete_tree();
        for tree in transform_to_printable(complete_tree) {
            state.log_to_stdout(tree.to_string().trim().to_string());
        }
    }
}
