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
        let current_node_path = context.git.get_current_node_path()?;
        let tree = current_node_path.display_tree();
        context.log_to_stdout(tree);
        Ok(())
    }
}
