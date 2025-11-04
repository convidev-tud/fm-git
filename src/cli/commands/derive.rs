use crate::cli::completion::CompletionHelper;
use crate::cli::*;
use crate::git::model::{ModelConstants, NodeType, TreeDataModel};
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
        let all_targets = context.arg_helper.get_argument_values::<String>("features");
        let all_targets_str = all_targets.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        let target_product_name = context.arg_helper.get_argument_value::<String>("product");
        let current_area_node = context.git.get_current_area_node()?;
        let target_path = current_area_node.get_name().to_string()
            + "/"
            + ModelConstants::product_path_prefix().as_str()
            + target_product_name.as_str();
        context.git.checkout(current_area_node.get_name(), false)?;
        context.git.checkout(target_path.as_str(), true)?;
        let output = context.git.merge(all_targets_str)?;
        context.log_from_output(&output);
        Ok(())
    }
    fn shell_complete(
        &self,
        completion_helper: CompletionHelper,
        context: &mut CommandContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let maybe_current_feature_root = context
            .git
            .get_current_area_node()?
            .get_child(ModelConstants::feature_prefix());
        if maybe_current_feature_root.is_none() {
            return Ok(vec![]);
        }
        let feature_root = maybe_current_feature_root.unwrap();
        let feature_root_type = match feature_root.get_type() {
            NodeType::FeatureRoot(t) => t,
            _ => unreachable!(),
        };

        let appendix = completion_helper.get_appendix();
        let last = appendix[appendix.len() - 1];
        let current = completion_helper.currently_editing();
        if current.is_none() {
            return Ok(vec![]);
        }
        match current.unwrap().as_str() {
            "features" => {
                let completion = feature_root_type
                    .iter_features_with_branches()
                    .filter(|s| s.starts_with(last))
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                Ok(completion
                    .iter()
                    .filter(|s| completion.len() < 2 || !appendix.contains(&s.as_str()))
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>())
            }
            _ => {
                Ok(vec![])
            }
        }
    }
}
