use crate::cli::completion::CompletionHelper;
use crate::cli::*;
use crate::git::model::QualifiedPath;
use clap::{Arg, Command};
use std::error::Error;

#[derive(Clone, Debug)]
pub struct CheckoutCommand;
impl CommandDefinition for CheckoutCommand {
    fn build_command(&self) -> Command {
        Command::new("checkout")
            .about("Switch branches")
            .disable_help_subcommand(true)
            .arg(Arg::new("branch").required(true))
    }
}
impl CommandInterface for CheckoutCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let branch_name = context
            .arg_helper
            .get_argument_value::<String>("branch")
            .unwrap();
        let result = context.git.checkout(&QualifiedPath::from(branch_name))?;
        context.log_from_output(&result);
        Ok(())
    }
    fn shell_complete(
        &self,
        completion_helper: CompletionHelper,
        context: &mut CommandContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let maybe_editing = completion_helper.currently_editing();
        if maybe_editing.is_none() {
            return Ok(vec![]);
        }
        let all_branches = context.git.get_model().get_qualified_paths_with_branches();
        let result = match maybe_editing.unwrap().get_id().as_str() {
            "branch" => completion_helper.complete_qualified_path_stepwise(all_branches, false),
            _ => vec![],
        };
        Ok(result)
    }
}
