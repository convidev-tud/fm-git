use crate::cli::completion::*;
use crate::cli::*;
use crate::model::QualifiedPath;
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
        let branch = context
            .arg_helper
            .get_argument_value::<String>("branch")
            .unwrap();
        let full_target = if branch.starts_with("/") {
            QualifiedPath::from(branch).strip_n_left(1)
        } else {
            context.git.get_current_qualified_path()? + QualifiedPath::from(branch)
        };
        let result = context.git.checkout(&full_target)?;
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
            "branch" => match completion_helper.get_last() {
                Some(last) => {
                    if last.starts_with("/") {
                        completion_helper.complete_qualified_paths(
                            QualifiedPath::new(),
                            all_branches
                                .iter()
                                .map(|path| QualifiedPath::from("") + path.clone()),
                            false,
                        )
                    } else {
                        completion_helper.complete_qualified_paths(
                            context.git.get_current_qualified_path()?,
                            all_branches.iter().map(|path| path.clone()),
                            false,
                        )
                    }
                }
                None => vec![],
            },
            _ => vec![],
        };
        Ok(result)
    }
}
