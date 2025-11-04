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
            .arg(Arg::new("branch"))
    }
}
impl CommandInterface for CheckoutCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let branch_name = context.arg_helper.get_argument_value::<String>("branch");
        let result = context
            .git
            .checkout(&QualifiedPath::from(branch_name), false)?;
        context.log_from_output(&result);
        Ok(())
    }
    fn shell_complete(
        &self,
        completion_helper: CompletionHelper,
        context: &mut CommandContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let appendix = completion_helper.get_appendix();
        let last = appendix[appendix.len() - 1];
        let current = completion_helper.currently_editing();
        if current.is_none() {
            return Ok(vec![]);
        }
        match current.unwrap().as_str() {
            "branch" => Ok(context
                .git
                .get_model()
                .iter_qualified_paths_with_branches()
                .filter(|s| s.to_string().starts_with(last))
                .map(|s| s.to_string())
                .collect::<Vec<String>>()),
            _ => Ok(vec![]),
        }
    }
}
