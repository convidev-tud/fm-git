use std::error::Error;
use crate::def::*;
use crate::CommandLogger;
use clap::Command;
use hyrmir_lib::model::*;
use hyrmir_lib::repository::RepositoryLoader;
use hyrmir_lib::vcs::VCS;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct StatusCommand<V: VCS + 'static> {
    _phantom: PhantomData<V>,
}

impl<V: VCS> StatusCommand<V> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
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
        let workspace = repo.get_workspace::<AnyType<Concrete>>()?;
        let current = workspace.get_current_view();
        let current_msg = format!("Viewing {}", current.get_semantic_view().formatted(true, true, true),);
        let status = workspace.status(current_msg, "", "", true)?;
        logger.info(status);
        Ok(())
    }
}
