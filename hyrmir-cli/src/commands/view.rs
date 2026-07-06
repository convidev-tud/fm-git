use crate::completion::CompletionHelper;
use crate::{CommandContext, CommandDefinition, CommandInterface, CommandLogger};
use clap::{Arg, Command};
use colored::Colorize;
use hyrmir_lib::model::*;
use hyrmir_lib::repository::RepositoryLoader;
use hyrmir_lib::vcs::VCS;
use hyrmir_lib::workspace::WorkspaceKind;
use std::error::Error;
use std::marker::PhantomData;
use std::path::PathBuf;

const PATH: &str = "path";

#[derive(Clone, Debug)]
pub struct ViewCommand<V: VCS + 'static> {
    _vcs: PhantomData<V>,
}

impl<V: VCS> ViewCommand<V> {
    pub fn new() -> Self {
        Self { _vcs: PhantomData }
    }
}

impl<V: VCS> CommandDefinition<V> for ViewCommand<V> {
    fn build_command(&self) -> Command {
        Command::new("view")
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
        let current = match &workspace {
            WorkspaceKind::Head(w) => w.get_current_view().get_semantic_view(),
            WorkspaceKind::Rev(w) => w.get_current_view().get_semantic_view(),
        };
        let target_path = current.to_normalized_path() + parsed_target.get_path().clone();

        if current.to_normalized_path() == target_path {
            logger.info(format!(
                "Already viewing {}",
                target_path.to_string().blue(),
            ));
            return Ok(());
        }

        let target = match repo.get_view::<AnyType<Concrete>>(&target_path) {
            Ok(path) => path,
            Err(error) => {
                return match error {
                    SemanticViewError::PathDoesNotExist(path) => Err(path.into()),
                    SemanticViewError::InvalidType(_) => Err(format!(
                        "Cannot view {}: target does not have a branch",
                        target_path.to_string().blue()
                    )
                    .into()),
                };
            }
        };
        let workspace = match workspace {
            WorkspaceKind::Head(w) => w.switch_to(target.head())?,
            WorkspaceKind::Rev(w) => w.switch_to(target.head())?,
        };
        let new_current = workspace.get_current_view();
        let msg = format!(
            "Now viewing {}",
            new_current.get_semantic_view().formatted(true, true, true),
        );
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
        let current = match repo.get_workspace::<AnyType<Concrete>>(PathBuf::from("."))? {
            WorkspaceKind::Head(w) => w
                .get_current_view()
                .get_semantic_view()
                .to_normalized_path(),
            WorkspaceKind::Rev(w) => w
                .get_current_view()
                .get_semantic_view()
                .to_normalized_path(),
        };
        let root = repo.get_virtual_root_view().to_static_view();
        let all_branches = root
            .iter_children_req()
            .map(|p| p.normalize().get_path().clone());
        let result = match maybe_editing.unwrap().get_id().as_str() {
            PATH => completion_helper.complete_normalized_paths(current, all_branches),
            _ => vec![],
        };
        Ok(result)
    }
}
