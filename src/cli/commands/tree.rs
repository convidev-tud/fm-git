use crate::cli::*;
use clap::Command;
use std::error::Error;

#[derive(Clone, Debug)]
pub struct TreeCommand;

impl CommandDefinition for TreeCommand {
    fn build_command(&self) -> Command {
        Command::new("tree")
            .about("Displays the tree structure")
            .disable_help_subcommand(true)
    }
}

impl CommandInterface for TreeCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let model = context.git.get_model();
        let current_branch = context.git.get_current_qualified_path()?;
        let maybe_node_path = model.get_node_path(&current_branch);
        if maybe_node_path.is_none() {
            return Ok(());
        }
        let node_path = maybe_node_path.unwrap();
        let tree = node_path.last().display_tree();
        context.log_to_stdout(tree);
        Ok(())
    }
}
