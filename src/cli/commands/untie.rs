use crate::cli::*;
use crate::model::{Commit, NodePathType};
use clap::Command;
use std::error::Error;

#[derive(Clone, Debug)]
pub struct UntieCommand;

impl CommandDefinition for UntieCommand {
    fn build_command(&self) -> Command {
        Command::new("untie")
            .about("Untie commit from product and merge back into feature")
            .disable_help_subcommand(true)
    }
}

impl CommandInterface for UntieCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let current = match context.git.get_current_node_path()?.concretize() {
            NodePathType::Product(path) => path,
            _ => {
                return Err("Not on product branch".into());
            }
        };
        let commit_history = context
            .git
            .get_commit_history(&current.get_qualified_path())?;
        let derivation_commits = commit_history
            .into_iter()
            .filter(|commit| commit.message().contains("DERIVATION FINISHED"))
            .collect::<Vec<Commit>>();
        
        Ok(())
    }
}
