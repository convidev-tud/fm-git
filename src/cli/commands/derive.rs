use crate::cli::*;
use clap::{Arg, ArgAction, ArgMatches, Command};
use crate::cli::commands::util::{get_argument_value, get_argument_values};

#[derive(Clone, Debug)]
pub struct DeriveCommand {}

impl CommandDefinition for DeriveCommand {
    fn build_command(&self) -> Command {
        Command::new("derive")
            .about("Derive a product")
            .disable_help_subcommand(true)
            .arg(Arg::new("features").action(ArgAction::Append))
            .arg(Arg::new("name").long("name").required(true).help("Specifies the name of the resulting product branch"))
    }
}

impl CommandInterface for DeriveCommand {
    fn run_command(
        &self,
        args: &ArgMatches,
        context: &mut CommandContext,
    ) {
        let all_targets = get_argument_values(args, "features");
        let target_branch = get_argument_value(args, "name");
        let main = { context.git.get_main_branch() };
        context.git.checkout(main, false);
        context.log_to_stdout(format!("Creating product branch {}", target_branch).to_string());
        context.git.checkout(target_branch.as_str(), true);
        let output = context.git.merge(all_targets);
        match output {
            Ok(output) => { context.log_from_u8(&output.stdout, &output.stderr) },
            Err(output) => { context.log_to_stderr(output.to_string()); },
        }
    }
}
