use crate::cli::completion::CompletionHelper;
use crate::cli::*;
use crate::git::model::{NodeType, QualifiedPath};
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
        let target_product_name = context.arg_helper.get_argument_value::<String>("product");
        let current_area = context.git.get_current_area()?;
        let product_root = context
            .git
            .get_model()
            .get_qualified_path_to_product_root(&current_area);
        let target_path = product_root + QualifiedPath::from(target_product_name);

        let feature_root = context
            .git
            .get_model()
            .get_qualified_path_to_feature_root(&current_area);
        let all_features = context
            .arg_helper
            .get_argument_values::<String>("features")
            .into_iter()
            .map(|e| feature_root.clone() + QualifiedPath::from(e))
            .collect::<Vec<_>>();

        context.git.checkout(&current_area, false)?;
        context.git.checkout(&target_path, true)?;
        let output = context.git.merge(&all_features)?;
        context.log_from_output(&output);
        Ok(())
    }
    fn shell_complete(
        &self,
        completion_helper: CompletionHelper,
        context: &mut CommandContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let area = context.git.get_current_area()?;
        let feature_root = context
            .git
            .get_model()
            .get_qualified_path_to_feature_root(&area);
        let maybe_feature_root_node = context.git.get_model().get_node_path(&feature_root);
        if maybe_feature_root_node.is_none() {
            return Ok(vec![]);
        }
        let path_to_feature_root = maybe_feature_root_node.unwrap();
        let feature_root = path_to_feature_root.last();
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
                    .filter(|s| s.to_string().starts_with(last))
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                Ok(completion
                    .iter()
                    .filter(|s| completion.len() < 2 || !appendix.contains(&s.as_str()))
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>())
            }
            _ => Ok(vec![]),
        }
    }
}
