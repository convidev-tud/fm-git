use crate::completion::CompletionHelper;
use crate::{CommandContext, CommandDefinition, CommandInterface, CommandLogger};
use clap::{Arg, ArgAction, Command};
use hyrmir_lib::model::*;
use hyrmir_lib::repository::RepositoryLoader;
use hyrmir_lib::vcs::VCS;
use hyrmir_lib::workspace::WorkspaceKind;
use std::error::Error;
use std::marker::PhantomData;
use std::path::PathBuf;

pub const VERBOSE: &str = "verbose";
pub const SHOW_TAGS: &str = "show_tags";
pub const ADD: &str = "add";
pub const PATH_TO_ADD: &str = "path";

pub fn show_tags() -> Arg {
    Arg::new(SHOW_TAGS)
        .long("show-tags")
        .action(ArgAction::SetTrue)
        .help("Also show tags")
}

pub fn verbose() -> Arg {
    Arg::new(VERBOSE)
        .short('v')
        .long("verbose")
        .action(ArgAction::Count)
        .help(
            "Set verbosity of output. \
            Verbosity increases with number of occurrences.",
        )
}

pub fn format_command_help<S: Into<String>>(command: S) -> String {
    format!("\"{}\"", command.into())
}

#[derive(Clone, Debug)]
pub struct AddPathCommand<V: VCS + 'static> {
    short_help: String,
    _vcs: PhantomData<V>,
}

impl<V: VCS> AddPathCommand<V> {
    pub fn new(short_help: impl Into<String>) -> Self {
        Self {
            short_help: short_help.into(),
            _vcs: PhantomData
        }
    }
}

impl<V: VCS> CommandDefinition<V> for AddPathCommand<V> {
    fn build_command(&self) -> Command {
        let help = self.short_help.clone();
        Command::new(ADD)
            .about(help)
            .disable_help_subcommand(true)
            .arg(Arg::new(PATH_TO_ADD).required(true))
    }
}

impl<V: VCS> CommandInterface<V> for AddPathCommand<V> {
    fn run_command(
        &self,
        loader: &mut RepositoryLoader<V>,
        logger: &mut CommandLogger,
        context: &CommandContext<V>,
    ) -> Result<(), Box<dyn Error>> {
        // parameters
        let parsed_target = context
            .get_arg_helper()
            .get_argument_value::<String>(PATH_TO_ADD)
            .unwrap()
            .normalize()?;
        if let NormalizedRevision::Revision(_) = parsed_target.get_revision() {
            return Err("Explicit versioning does not make sense during path creation".into())
        }
        let target_path = parsed_target.get_path();

        // repo allocations
        let repo = loader.load_repo()?;
        let path = PathBuf::from(".");
        let workspace = repo.get_workspace::<AnyType<Concrete>>(path)?;
        let current = match &workspace {
            WorkspaceKind::Head(w) => w.get_current_view().get_semantic_view(),
            WorkspaceKind::Rev(w) => w.get_current_view().get_semantic_view(),
        };
        let target_path = current.to_normalized_path() + parsed_target.get_path().clone();


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
        let root = repo.get_virtual_root_view();
        let all_branches = root
            .iter_children_req()
            .filter_map(FilterByType::<AnyType<Concrete>>::filter)
            .map(|p| p.to_normalized_path());
        let result = match maybe_editing.unwrap().get_id().as_str() {
            PATH => completion_helper.complete_normalized_paths(current, all_branches),
            _ => vec![],
        };
        Ok(result)
    }
}
