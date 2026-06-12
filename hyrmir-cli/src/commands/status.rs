use crate::def::*;
use crate::CommandLogger;
use clap::Command;
use hyrmir_lib::model::node_path::*;
use hyrmir_lib::model::*;
use hyrmir_lib::repository::Repository;
use hyrmir_lib::vcs::VCS;
use std::error::Error;

#[derive(Clone, Debug)]
pub struct StatusCommand<V: VCS>;

impl<V: VCS> CommandDefinition<V> for StatusCommand<V> {
    fn build_command(&self) -> Command {
        Command::new("status")
            .about("Show the status of the current workspace")
            .disable_help_subcommand(true)
    }
}

impl<V: VCS> CommandInterface<V> for StatusCommand<V> {
    fn run_command(
        &self,
        repository: &mut Repository<V>,
        logger: &mut CommandLogger,
        _context: &CommandContext<V>,
    ) -> Result<(), Box<dyn Error>> {
        let workspace = repository.get_workspace()?;
        // let status = workspace.format_status_msg(true);
        // logger.info(status);
        Ok(())
    }
}
