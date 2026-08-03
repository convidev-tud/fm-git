use crate::CommandLogger;
use crate::def::*;
use clap::Command;
use hyrmir_lib::model::*;
use hyrmir_lib::repository::RepositoryLoader;
use hyrmir_lib::vcs::VCS;
use std::error::Error;
use std::marker::PhantomData;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct StatusCommand<V: VCS + 'static>(PhantomData<V>);

impl<V: VCS> StatusCommand<V> {
    pub fn new() -> Self {
        Self {
            0: PhantomData,
        }
    }
}

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
        loader: &mut RepositoryLoader<V>,
        logger: &mut CommandLogger,
        _context: &CommandContext<V>,
    ) -> Result<(), Box<dyn Error>> {
        let repo = loader.load_repo()?;
        let path = PathBuf::from(".");
        let workspace = repo.get_workspace::<AnyType<Concrete>>(path)?;
        let current = workspace.get_current_view();
        let current_msg = format!("Viewing {}", current.formatted(true, true, true),);
        let status = workspace.status(current_msg, "", "", true)?;
        logger.info(status);
        Ok(())
    }
}
