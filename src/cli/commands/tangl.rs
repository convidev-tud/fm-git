use crate::cli::*;
use clap::Command;

#[derive(Clone, Debug)]
pub struct TangleCommand {}

impl CommandDefinition for TangleCommand {
    fn build_command(&self) -> Command {
        Command::new("tangl")
            .arg_required_else_help(true)
            .allow_external_subcommands(true)
    }
    fn get_subcommands(&self) -> Vec<Box<dyn CommandImpl>> {
        vec![
            Box::new(StatusCommand),
            Box::new(TreeCommand),
            Box::new(DeriveCommand),
            Box::new(CheckCommand),
            Box::new(CheckoutCommand),
            Box::new(InitCommand),
            Box::new(FeatureCommand),
            Box::new(ProductCommand),
            Box::new(TagCommand),
            Box::new(SpreadCommand),
            Box::new(UntieCommand),
            Box::new(HiddenCompletionCommand),
        ]
    }
}

impl CommandInterface for TangleCommand {}
