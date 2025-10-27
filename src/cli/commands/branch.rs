use crate::cli::*;
use crate::git::model::SymNode;
use clap::Command;
use std::error::Error;
use termtree::Tree;

fn transform_to_printable(root: &SymNode) -> Vec<Tree<&str>> {
    root.iter_children_flat()
        .map(|child| {
            let mut tree = Tree::new(child.get_name());
            tree.leaves = transform_to_printable(child);
            tree
        })
        .collect()
}

#[derive(Clone, Debug)]
pub struct BranchCommand {}

impl CommandDefinition for BranchCommand {
    fn build_command(&self) -> Command {
        Command::new("branch")
            .about("Displays the tree structure")
            .disable_help_subcommand(true)
    }
}

impl CommandInterface for BranchCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let maybe_feature_tree = context.git.get_model().get_feature_root();
        if maybe_feature_tree.is_none() {
            return Ok(());
        }
        let feature_tree = maybe_feature_tree.unwrap();
        for tree in transform_to_printable(feature_tree) {
            context.log_to_stdout(tree.to_string().trim().to_string());
        }
        Ok(())
    }
}
