use crate::{CommandContext, CommandDefinition, CommandInterface, CommandLogger};
use clap::{Arg, Command};
use colored::Colorize;
use hyrmir_lib::model::{AnyNode, Concrete, ToNormalizedPath, TreeViewError};
use hyrmir_lib::repository::RepositoryLoader;
use hyrmir_lib::vcs::VCS;
use std::error::Error;
use std::marker::PhantomData;

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
        let target_string = context
            .get_arg_helper()
            .get_argument_value::<String>(PATH)
            .unwrap();

        // repo allocations
        let repo = loader.load_repo()?;
        let workspace = repo.get_workspace::<AnyNode<Concrete>>()?;
        let current = workspace.get_current_view();

        let target_path = current.to_normalized_path() + target_string.to_normalized_path();
        let target = match repo.get_view::<AnyNode<Concrete>>(&target_path) {
            Ok(path) => path,
            Err(error) => {
                return match error {
                    TreeViewError::PathDoesNotExist(_) => Err(format!(
                        "Cannot checkout {}: path does not exist",
                        target_path.to_string().blue()
                    )
                    .into()),
                    TreeViewError::InvalidType(_) => Err(format!(
                        "Cannot checkout {}: target does not have a branch",
                        target_path.to_string().blue()
                    )
                    .into()),
                };
            }
        };

        if current == &target {
            logger.info(format!("Already on branch {}", target.formatted(true),));
        } else {
            let workspace = workspace.switch_to(target)?;
            let new_current = workspace.get_current_view();
            let msg = format!(
                "Switched to {} branch {}",
                new_current.get_real_type().get_formatted_name(),
                new_current.formatted(true),
            );
            let status = workspace.status(
                msg,
                "",
                "",
                true,
            )?;
            logger.info(status);
        }
        Ok(())
    }
    // fn shell_complete(
    //     &self,
    //     completion_helper: CompletionHelper,
    //     context: &mut CommandContext,
    // ) -> Result<Vec<String>, Box<dyn Error>> {
    //     let maybe_editing = completion_helper.currently_editing();
    //     if maybe_editing.is_none() {
    //         return Ok(vec![]);
    //     }
    //     let transformer = ByTypeFilteringNodePathTransformer::<_, AnyGitObject>::new();
    //     let root = context.git.get_virtual_root();
    //     let all_branches = transformer.transform(root.iter_children_by_type_req());
    //     let result = match maybe_editing.unwrap().get_id().as_str() {
    //         "branch" => completion_helper.complete_normalized_paths(
    //             context.git.get_current_normalized_path()?,
    //             all_branches.map(|p| p.to_normalized_path()),
    //         ),
    //         _ => vec![],
    //     };
    //     Ok(result)
    // }
}
