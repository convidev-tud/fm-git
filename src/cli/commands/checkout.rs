use crate::cli::util::get_argument_value;
use crate::cli::*;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::error::Error;

#[derive(Clone, Debug)]
pub struct CheckoutCommand;
impl CommandDefinition for CheckoutCommand {
    fn build_command(&self) -> Command {
        Command::new("checkout")
            .about("Checkout a branch")
            .disable_help_subcommand(true)
            .arg(Arg::new("branch"))
            .arg(
                Arg::new("create")
                    .short('b')
                    .action(ArgAction::SetTrue)
                    .help(
                        "Creates a new feature branch as the child of the currently checked-out branch. \
                        If the current branch is main/master, the new feature will be a root feature. \
                        Fails if the current branch is a product."
                    ),
            )
    }
}
impl CommandInterface for CheckoutCommand {
    fn run_command(
        &self,
        args: &ArgMatches,
        _current: &CommandMap,
        context: &mut CommandContext,
    ) -> Result<(), Box<dyn Error>> {
        let branch = get_argument_value::<String>("branch", args);
        let to_create = get_argument_value::<bool>("create", args);
        let result = context.git.checkout(branch.as_str(), to_create)?;
        context.log_from_output(&result);
        Ok(())
    }
}
