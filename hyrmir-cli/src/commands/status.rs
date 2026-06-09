use crate::def::*;
use clap::Command;
use colored::Colorize;
use std::error::Error;
use hyrmir_lib::derivation::DerivationState;
use hyrmir_lib::model::*;
use hyrmir_lib::vcs::VCS;
use hyrmir_lib::workspace::Workspace;
use crate::CommandLogger;

#[derive(Clone, Debug)]
pub struct StatusCommand<V: VCS>;

impl<V: VCS> CommandDefinition<V> for StatusCommand<V> {
    fn build_command(&self) -> Command {
        Command::new("status")
            .about("Show the working tree status")
            .disable_help_subcommand(true)
    }
}

impl<V: VCS> CommandInterface<V> for StatusCommand<V> {
    fn run_command(
        &self,
        workspace: &mut Workspace<V>,
        logger: &mut CommandLogger,
        context: &mut CommandContext<V>
    ) -> Result<(), Box<dyn Error>> {
        let current_path = workspace.get_virtual_root().move_to_current::<AnyNode>()?;
        let first_line = format!(
            "On {} branch {}",
            current_path.get_real_type().get_formatted_name(),
            current_path.to_string().blue()
        );
        if let Ok(product) = current_path.try_convert_to::<Product<Concrete>>() {
            let state = product.get_derivation_data()?;
            match state.get_state() {
                DerivationState::None => logger.info("No derivation in progress"),
                DerivationState::InProgress => {
                    logger.info("Derivation in progress");
                    if !state.get_missing().is_empty() {
                        let manager =
                            DerivationManager::new(&product, &context.git, &logger)?;
                        let missing = manager.get_pending_chain()?.unwrap();
                        logger.info("\nFeatures remaining:");
                        for info in missing.display_as_list() {
                            logger.info(format!("  {info}"))
                        }
                    }
                }
            }
        };
        logger
            .info(format!("{maybe_new_line}{no_first_line}"));
        Ok(())
    }
}
