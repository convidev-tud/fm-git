use crate::completion::*;
use crate::{CommandContext, CommandDefinition, CommandInterface, CommandLogger};
use clap::{Arg, Command};
use colored::Colorize;
use hyrmir_lib::model::*;
use hyrmir_lib::repository::RepositoryLoader;
use hyrmir_lib::vcs::VCS;
use std::error::Error;
use std::marker::PhantomData;
use std::path::PathBuf;

const VIEW: &str = "view";
const PATH: &str = "path";

#[derive(Clone, Debug)]
pub struct ViewCommand<V: VCS + 'static>(PhantomData<V>);

impl<V: VCS> ViewCommand<V> {
    pub fn new() -> Self {
        Self { 0: PhantomData }
    }
}

impl<V: VCS> CommandDefinition<V> for ViewCommand<V> {
    fn build_command(&self) -> Command {
        Command::new(VIEW)
            .about("Switch views")
            .disable_help_subcommand(true)
            .arg(Arg::new(PATH).required(true))
    }
}

impl<V: VCS> CommandInterface<V> for ViewCommand<V> {
    fn run_command(
        &self,
        loader: &mut RepositoryLoader<V>,
        logger: &mut CommandLogger,
        context: &CommandContext<V>,
    ) -> Result<(), Box<dyn Error>> {
        // parameters
        let parsed_target = context
            .get_arg_helper()
            .get_argument_value::<String>(PATH)
            .unwrap()
            .normalize()?;

        // repo allocations
        let repo = loader.load_repo()?;
        let path = PathBuf::from(".");
        let workspace = repo.get_workspace::<AnyType<Concrete>>(path)?;
        let current = workspace.get_current_view().get_structure_view();

        let root = repo.root_view();
        let found_path = get_path_from_name(
            parsed_target.get_path(),
            root
                .iter_children_req(repo)
                .filter_map(FilterByType::<AnyType<Concrete>>::filter)
                .map(|p| p.to_normalized_path())
        )?;
        let target_path = current.to_normalized_path() + found_path;

        if current.to_normalized_path() == target_path {
            logger.info(format!(
                "Already viewing {}",
                target_path.to_string().blue(),
            ));
            return Ok(());
        }

        let target = match repo
            .root_view()
            .move_to::<AnyType<Concrete>>(&target_path, repo)
        {
            Ok(path) => path,
            Err(error) => {
                return match error {
                    SemanticViewError::PathDoesNotExist(path) => Err(path.into()),
                    SemanticViewError::InvalidType(_) => Err(format!(
                        "Cannot view {}: target does not have an associated artifact",
                        target_path.to_string().blue()
                    )
                    .into()),
                };
            }
        };
        let workspace = workspace.switch_to(target.head())?;
        let new_current = workspace.get_current_view();
        let msg = format!("Now viewing {}", new_current.formatted(true, true, true),);
        let status = workspace.status(msg, "", "", true)?;
        logger.info(status);
        Ok(())
    }

    fn shell_complete(
        &self,
        loader: &mut RepositoryLoader<V>,
        completion_helper: CompletionHelper,
        _context: &CommandContext<V>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let maybe_editing = completion_helper.currently_editing();
        if maybe_editing.is_none() {
            return Ok(vec![]);
        }
        let repo = loader.load_repo()?;
        let current = repo
            .get_workspace::<AnyType<Concrete>>(PathBuf::from("."))?
            .get_current_view()
            .get_structure_view()
            .to_normalized_path();
        let root = repo.root_view();
        let all_branches = root
            .iter_children_req(repo)
            .filter_map(FilterByType::<AnyType<Concrete>>::filter)
            .map(|p| p.to_normalized_path());
        let result = match maybe_editing.unwrap().get_id().as_str() {
            PATH => {
                let strategy = DelegatingPathCompleter::new(current);
                completion_helper.complete_normalized_paths(&strategy, all_branches)
            },
            _ => vec![],
        };
        Ok(result)
    }
}
