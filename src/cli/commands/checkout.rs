use crate::cli::util::get_argument_value;
use crate::cli::*;
use clap::{Arg, ArgAction, ArgMatches, Command};
use std::error::Error;

#[derive(Clone, Debug)]
pub struct CheckoutCommand;
impl CommandDefinition for CheckoutCommand {
    fn build_command(&self) -> Command {
        Command::new("checkout")
            .about("Switch branches")
            .disable_help_subcommand(true)
            .arg(Arg::new("branch"))
            .arg(
                Arg::new("new-feature")
                    .short('f')
                    .action(ArgAction::SetTrue)
                    .help(
                        "Creates a new feature branch as the child of the currently checked-out branch and checks it out. \
                        If checked-out on default, the new feature will be a root feature. \
                        Fails if checked-out on a product or working branch."
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
        let branch_any_name = get_argument_value::<String>("branch", args);
        let new_feature = get_argument_value::<bool>("new-feature", args);
        let result = context.git.checkout(branch_any_name.as_str(), new_feature)?;
        context.log_from_output(&result);
        Ok(())
    }
}
