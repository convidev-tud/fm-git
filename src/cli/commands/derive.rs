use crate::cli::completion::CompletionHelper;
use crate::cli::*;
use crate::git::model::QualifiedPath;
use clap::{Arg, ArgAction, Command};
use std::error::Error;

#[derive(Clone, Debug)]
pub struct DeriveCommand {}

impl CommandDefinition for DeriveCommand {
    fn build_command(&self) -> Command {
        Command::new("derive")
            .about("Derive a product")
            .disable_help_subcommand(true)
            .arg(Arg::new("features").action(ArgAction::Append))
            .arg(
                Arg::new("product")
                    .short('p')
                    .required(true)
                    .help("Specifies the name of the resulting product branch"),
            )
    }
}

impl CommandInterface for DeriveCommand {
    fn run_command(&self, context: &mut CommandContext) -> Result<(), Box<dyn Error>> {
        let target_product_name = context.arg_helper.get_argument_value::<String>("product").unwrap();
        let current_area = context.git.get_current_area()?;
        let feature_root = context.git.get_current_feature_root()?;
        let product_root = context.git.get_current_product_root()?;
        let target_path = product_root + QualifiedPath::from(target_product_name);

        let all_features = context
            .arg_helper
            .get_argument_values::<String>("features")
            .into_iter()
            .map(|e| feature_root.clone() + QualifiedPath::from(e))
            .collect::<Vec<_>>();

        context.git.checkout(&current_area)?;
        context.git.checkout(&target_path)?;
        let output = context.git.merge(&all_features)?;
        context.log_from_output(&output);
        Ok(())
    }
    fn shell_complete(
        &self,
        completion_helper: CompletionHelper,
        context: &mut CommandContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let model = context.git.get_model();
        let feature_root = context.git.get_current_feature_root()?;
        let maybe_feature_root_node = model.get_node_path(&feature_root);
        if maybe_feature_root_node.is_none() {
            return Ok(vec![]);
        }
        let feature_root = maybe_feature_root_node.unwrap().get_node();
        let current = completion_helper.currently_editing();
        let result = match current {
            Some(value) => match value.get_id().as_str() {
                "features" => completion_helper.complete_qualified_path_stepwise(
                    &feature_root.get_child_paths_by_branch(true),
                    true,
                ),
                _ => vec![],
            },
            None => vec![],
        };
        Ok(result)
    }
}
