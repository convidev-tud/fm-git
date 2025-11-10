use crate::cli::*;
use clap::{Arg, Command};
use std::error::Error;
use crate::git::model::{NodeType, NodeTypeInterface, QualifiedPath};

#[derive(Clone, Debug)]
pub struct FeatureCommand;
impl CommandDefinition for FeatureCommand {
    fn build_command(&self) -> Command {
        Command::new("feature")
            .about("Manage features")
            .disable_help_subcommand(true)
            .arg(Arg::new("feature").help("Creates new feature as the child of the current one. Requires to be checked out on a feature branch."))
    }
}
impl CommandInterface for FeatureCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let model = context.git.get_model();
        let maybe_feature_name = context.arg_helper.get_argument_value::<String>("feature");
        match maybe_feature_name {
            Some(feature_name) => {
                let new_feature = QualifiedPath::from(feature_name);
                let node_path = model.get_node_path(&context.git.get_current_qualified_path()?).unwrap();
                let current_path = match node_path.get_node().get_type() {
                    NodeType::Area => {
                        model.get_qualified_path_to_feature_root(&node_path.to_path())
                    }
                    NodeType::Feature => {
                        node_path.to_path()
                    }
                    _ => { return Err(Box::new(CommandError::new("Cannot create feature: Current branch is not a feature branch"))) }
                };
                context.log_to_stdout(current_path + new_feature)
            }
            None => {
                let feature_root_path = context.git.get_current_feature_root()?;
                match model.get_node_path(&feature_root_path) {
                    Some(path) => { context.log_to_stdout(path.get_node().display_tree()); }
                    None => {}
                }
            }
        }
        Ok(())
    }
    // fn shell_complete(
    //     &self,
    //     completion_helper: CompletionHelper,
    //     context: &mut CommandContext,
    // ) -> Result<Vec<String>, Box<dyn Error>> {
    //     let model = context.git.get_model();
    //     let current_branch = context.git.get_current_qualified_path()?;
    //     let maybe_path = model.get_node_path(&current_branch);
    //     match maybe_path {
    //         Some(path) => {
    //             match path.get_node().get_type() {
    //                 NodeType::Feature | NodeType::FeatureRoot {
    //
    //                 },
    //                 _ => {return Ok(vec![])}
    //             }
    //         }
    //         None => { return Ok(vec![]) }
    //     }
    //     let feature_root = context.git.get_current_feature_root()?;
    //     let maybe_feature_root_node = model.get_node_path(&feature_root);
    //     if maybe_feature_root_node.is_none() {
    //         return Ok(vec![]);
    //     }
    //     let feature_root = maybe_feature_root_node.unwrap().get_node();
    //     let current = completion_helper.currently_editing();
    //     let result = match current {
    //         Some(_) => vec![],
    //         None => {
    //             completion_helper.complete_qualified_path_stepwise(
    //                 &feature_root.get_child_paths_by_branch(false),
    //                 true,
    //             )
    //         },
    //     };
    //     Ok(result)
    // }
}
