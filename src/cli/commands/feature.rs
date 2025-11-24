use crate::cli::completion::*;
use crate::cli::*;
use crate::model::*;
use clap::{Arg, Command};
use std::error::Error;

fn add_feature(feature: QualifiedPath, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
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
        target_path.strip_n_left(1)
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
fn print_feature_tree(context: &CommandContext, show_tags: bool) -> Result<(), Box<dyn Error>> {
    let area = context.git.get_current_area()?;
    match area.to_feature_root() {
        Some(path) => {
            context.log_to_stdout(path.display_tree(show_tags));
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
            .arg(make_show_tags())
    }
}
impl CommandInterface for FeatureCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let maybe_feature_name = context.arg_helper.get_argument_value::<String>("feature");
        let maybe_delete = context.arg_helper.get_argument_value::<String>("delete");
        let show_tags = context.arg_helper.get_argument_value::<bool>("show_tags").unwrap();
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
                print_feature_tree(context, show_tags)?;
            }
        }
        Ok(())
    }
    fn shell_complete(
        &self,
        completion_helper: CompletionHelper,
        context: &mut CommandContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let result = match completion_helper.currently_editing() {
            Some(arg) => match arg.get_id().as_str() {
                "feature" => {
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
                    let total = path.get_child_paths_by_branch();
                    if total.is_empty() {
                        return Ok(vec![]);
                    }
                    let has_branch = total.get(&true).unwrap();
                    let has_no_branch = total.get(&false).unwrap();
                    let has_branch_completion = completion_helper.complete_qualified_path(
                        AbsolutePathCompletion,
                        has_branch,
                        false,
                    );
                    let has_no_branch_completion = completion_helper.complete_qualified_path(
                        AbsolutePathCompletion,
                        has_no_branch,
                        false,
                    );
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
                "delete" => {
                    let maybe_feature_root = context.git.get_current_area()?.to_feature_root();
                    match maybe_feature_root {
                        Some(path) => completion_helper.complete_qualified_path(
                            AbsolutePathCompletion,
                            path.get_child_paths_by_branch().get(&true).unwrap(),
                            false,
                        ),
                        None => {
                            vec![]
                        }
                    }
                }
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
