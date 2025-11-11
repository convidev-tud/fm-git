use crate::cli::*;
use clap::{Arg, Command};
use std::error::Error;
use crate::cli::completion::CompletionHelper;
use crate::git::model::*;

fn add_feature(feature: QualifiedPath, context: &CommandContext) -> Result<(), Box<dyn Error>> {
    let node_path = context.git.get_current_node_path()?;
    let current_path = match node_path.concretize() {
        NodePathType::Area(path) => {
            path.get_path_to_feature_root()
        }
        NodePathType::Feature(path) => {
            path.get_qualified_path()
        }
        _ => { return Err(Box::new(CommandError::new("Cannot create feature: Current branch is not a feature branch"))) }
    };
    let output = context.git.create_branch(&(current_path + feature))?;
    context.log_from_output(&output);
    Ok(())
}
fn delete_feature(feature: QualifiedPath, context: &CommandContext) -> Result<(), Box<dyn Error>> {
    let area = context.git.get_current_area()?;
    let complete_path = area.get_path_to_feature_root() + feature;
    let output = context.git.delete_branch(&complete_path)?;
    context.log_from_output(&output);
    Ok(())
}
fn print_feature_tree(context: &CommandContext) -> Result<(), Box<dyn Error>> {
    let area = context.git.get_current_area()?;
    match area.to_feature_root() {
        Some(path) => { context.log_to_stdout(path.display_tree()); }
        None => {}
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct FeatureCommand;
impl CommandDefinition for FeatureCommand {
    fn build_command(&self) -> Command {
        Command::new("feature")
            .about("Manage features")
            .disable_help_subcommand(true)
            .arg(Arg::new("feature").help("Creates new feature as the child of the current one. Requires to be checked out on a feature branch."))
            .arg(Arg::new("delete").short('D').help("Deletes a feature branch"))
    }
}
impl CommandInterface for FeatureCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let maybe_feature_name = context.arg_helper.get_argument_value::<String>("feature");
        let maybe_delete = context.arg_helper.get_argument_value::<String>("delete");
        match maybe_delete {
            Some(delete) => {
                delete_feature(QualifiedPath::from(delete), &context)?;
                return Ok(())
            }
            None => {}
        }
        match maybe_feature_name {
            Some(feature_name) => {
                add_feature(QualifiedPath::from(feature_name), context)?;
            }
            None => {
                print_feature_tree(context)?;
            }
        }
        Ok(())
    }
    fn shell_complete(
        &self,
        completion_helper: CompletionHelper,
        context: &mut CommandContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let maybe_feature_root = context.git.get_current_area()?.to_feature_root();
        if maybe_feature_root.is_none() { return Ok(Vec::new()) }
        let feature_root = maybe_feature_root.unwrap();
        let result = match completion_helper.currently_editing() {
            Some(arg) => {
                match arg.get_id().as_str() {
                    "feature" => {
                        completion_helper.complete_qualified_path_stepwise(
                            &feature_root.get_child_paths_by_branch(false),
                            false
                        )
                    }
                    "delete" => {
                        completion_helper.complete_qualified_path_stepwise(
                            &feature_root.get_child_paths_by_branch(true),
                            false
                        )
                    }
                    _ => { vec![] }
                }
            }
            None => { vec![] }
        };
        Ok(result)
    }
}
