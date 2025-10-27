use crate::cli::completion::CompletionHelper;
use crate::cli::*;
use clap::{Arg, ArgAction, Command};
use std::error::Error;

#[derive(Clone, Debug)]
pub struct DeriveCommand {}

impl CommandDefinition for DeriveCommand {
    fn build_command(&self) -> Command {
        Command::new("derive")
            .about("Derive a product")
            .disable_help_subcommand(true)
            .arg(Arg::new("features").action(ArgAction::Append))
            .arg(
                Arg::new("name")
                    .long("name")
                    .required(true)
                    .help("Specifies the name of the resulting product branch"),
            )
    }
}

impl CommandInterface for DeriveCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        // let all_targets = get_argument_values::<String>(args, "features");
        // let target_branch = get_argument_value(args, "name");
        // let main = { context.git.get_main_branch() };
        // context.git.checkout(main, false);
        // context.log_to_stdout(format!("Creating product branch {}", target_branch).to_string());
        // context.git.checkout(target_branch.as_str(), true);
        // let output = context.git.merge(all_targets);
        // match output {
        //     Ok(output) => { context.log_from_u8(&output.stdout, &output.stderr) },
        //     Err(output) => { context.log_to_stderr(output.to_string()); },
        // }
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
            "features" => {
                let completion = context
                    .git
                    .get_model()
                    .get_short_feature_names()
                    .into_iter()
                    .filter(|s| s.starts_with(last))
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                Ok(completion
                    .iter()
                    .filter(|s| completion.len() < 2 || !appendix.contains(&s.as_str()))
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>())
            }
            _ => {
                return Ok(vec![]);
            }
        }
    }
}
