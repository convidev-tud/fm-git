use crate::cli::completion::CompletionHelper;
use crate::cli::*;
use crate::git::model::*;
use clap::{Arg, Command};
use std::error::Error;

fn add_feature(feature: QualifiedPath, context: &CommandContext) -> Result<(), Box<dyn Error>> {
    let node_path = context.git.get_current_node_path()?;
    let current_path = match node_path.concretize() {
        NodePathType::Area(path) => path.get_path_to_feature_root(),
        NodePathType::Feature(path) => path.get_qualified_path(),
        _ => {
            return Err(Box::new(CommandError::new(
                "Cannot create feature: Current branch is not a feature or area branch",
            )));
        }
    };
    let target_path = current_path + feature;
    let output = context.git.create_branch(&target_path)?;
    context.log_from_output(&output);
    context.log_to_stdout(format!(
        "Created new feature {}",
        target_path.trim_n_left(1)
    ));
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
        Some(path) => {
            context.log_to_stdout(path.display_tree());
        }
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
                return Ok(());
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
        let current_branch = context.git.get_current_node_path()?;
        let maybe_concrete_path = match current_branch.concretize() {
            NodePathType::Area(path) => match path.to_feature_root() {
                Some(path) => Some(path.to_any_type()),
                None => None,
            },
            NodePathType::FeatureRoot(path) => Some(path.to_any_type()),
            NodePathType::Feature(path) => Some(path.to_any_type()),
            _ => None,
        };
        if maybe_concrete_path.is_none() {
            return Ok(vec![]);
        }
        let path = maybe_concrete_path.unwrap();
        let result = match completion_helper.currently_editing() {
            Some(arg) => match arg.get_id().as_str() {
                "feature" => {
                    let total = path.get_child_paths_by_branch();
                    if total.is_empty() {
                        return Ok(vec![]);
                    }
                    let has_branch = total.get(&true).unwrap();
                    let has_no_branch = total.get(&false).unwrap();
                    let has_branch_completion =
                        completion_helper.complete_qualified_path_stepwise(has_branch, false);
                    let has_no_branch_completion =
                        completion_helper.complete_qualified_path_stepwise(has_no_branch, false);
                    let mut result = has_branch_completion
                        .into_iter()
                        .map(|path| {
                            if !path.ends_with("/") {
                                path + "/"
                            } else {
                                path
                            }
                        })
                        .collect::<Vec<String>>();
                    result.extend(has_no_branch_completion);
                    result
                }
                "delete" => completion_helper.complete_qualified_path_stepwise(
                    &path.get_child_paths_by_branch().get(&true).unwrap(),
                    false,
                ),
                _ => {
                    vec![]
                }
            },
            None => {
                vec![]
            }
        };
        Ok(result)
    }
}
