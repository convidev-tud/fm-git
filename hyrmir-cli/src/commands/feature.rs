use crate::{CommandContext, CommandDefinition, CommandInterface, CommandLogger};
use clap::Command;
use hyrmir_lib::model::*;
use hyrmir_lib::repository::RepositoryLoader;
use hyrmir_lib::vcs::VCS;
use std::error::Error;
use std::marker::PhantomData;

const FEATURE: &str = "feature";

#[derive(Clone, Debug)]
pub struct FeatureCommand<V: VCS + 'static> {
    _vcs: PhantomData<V>,
}

impl<V: VCS> FeatureCommand<V> {
    pub fn new() -> Self {
        Self { _vcs: PhantomData }
    }
}

impl<V: VCS> CommandDefinition<V> for FeatureCommand<V> {
    fn build_command(&self) -> Command {
        Command::new(FEATURE)
            .about("Manage features")
            .disable_help_subcommand(true)
            // .arg(show_tags())
    }
}
impl<V: VCS> CommandInterface<V> for FeatureCommand<V> {
    fn run_command(
        &self,
        loader: &mut RepositoryLoader<V>,
        logger: &mut CommandLogger,
        context: &CommandContext<V>,
    ) -> Result<(), Box<dyn Error>> {
        // let maybe_feature_name = context.arg_helper.get_argument_value::<String>("feature");
        // let maybe_delete = context.arg_helper.get_argument_value::<String>("delete");
        // let show_tags = context
        //     .arg_helper
        //     .get_argument_value::<bool>("show_tags")
        //     .unwrap();
        // match maybe_delete {
        //     Some(delete) => {
        //         let current = context.git.assert_current_node_path::<AnyGitObject>()?;
        //         let to_delete = if let Some(feature) = current.try_convert_to::<Feature>() {
        //             feature.to_normalized_path() + delete.to_normalized_path()
        //         } else {
        //             context.git.get_current_area()?.get_path_to_feature_root()
        //                 + delete.to_normalized_path()
        //         };
        //         delete_path::<Feature>(&to_delete, context)?;
        //         return Ok(());
        //     }
        //     None => {}
        // }
        // match maybe_feature_name {
        //     Some(feature_name) => {
        //         add_feature(NormalizedPath::from(feature_name), context)?;
        //     }
        //     None => {
        //         print_feature_tree(context, show_tags)?;
        //     }
        // }
        Ok(())
    }
    
    // fn shell_complete(
    //     &self,
    //     completion_helper: CompletionHelper,
    //     context: &mut CommandContext,
    // ) -> Result<Vec<String>, Box<dyn Error>> {
    //     let maybe_feature_root = context.git.get_current_area()?.move_to_feature_root();
    //     if maybe_feature_root.is_none() {
    //         return Ok(vec![]);
    //     }
    //     let feature_root = maybe_feature_root.unwrap();
    //     let result = match completion_helper.currently_editing() {
    //         Some(arg) => match arg.get_id().as_str() {
    //             "delete" => {
    //                 let current = context.git.assert_current_node_path::<AnyGitObject>()?;
    //                 let reference = if let Some(feature) = current.try_convert_to::<Feature>() {
    //                     feature.to_normalized_path()
    //                 } else {
    //                     feature_root.to_normalized_path()
    //                 };
    //                 completion_helper.complete_normalized_paths(
    //                     reference,
    //                     HasBranchFilteringNodePathTransformer::new(true)
    //                         .transform(feature_root.iter_children_by_type_req())
    //                         .map(|path| path.to_normalized_path()),
    //                 )
    //             }
    //             _ => {
    //                 vec![]
    //             }
    //         },
    //         None => {
    //             vec![]
    //         }
    //     };
    //     Ok(result)
    // }
}
