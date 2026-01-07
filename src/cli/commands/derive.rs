use crate::cli::completion::*;
use crate::cli::*;
use crate::git::interface::ConflictStatistics;
use crate::model::QualifiedPath;
use clap::{Arg, ArgAction, Command};
use petgraph::algo::maximal_cliques;
use petgraph::graph::UnGraph;
use std::collections::HashMap;
use std::error::Error;
use colored::Colorize;

fn map_paths_to_id(paths: &Vec<QualifiedPath>) -> (HashMap<usize, QualifiedPath>, HashMap<QualifiedPath, usize>) {
    let mut id_to_path: HashMap<usize, QualifiedPath> = HashMap::new();
    let mut path_to_id: HashMap<QualifiedPath, usize> = HashMap::new();
    let mut i = 0;
    for path in paths.iter() {
        id_to_path.insert(i, path.clone());
        path_to_id.insert(path.clone(), i);
        i += 1;
    }
    (id_to_path, path_to_id)
}

fn build_edges(conflict_data: &Vec<ConflictStatistics>, path_to_id: &HashMap<QualifiedPath, usize>) -> Vec<(u32, u32)> {
    conflict_data
        .iter()
        .filter_map(|element| {
            if !element.has_conflict() {
                let left = path_to_id.get(&element.branches().0).unwrap().clone() as u32;
                let right = path_to_id.get(&element.branches().1).unwrap().clone() as u32;
                Some((left, right))
            } else { None }
        })
        .collect()
}

fn get_max_clique(graph: &UnGraph<usize, ()>) -> Vec<usize> {
    let cliques = maximal_cliques(graph);
    let mut max_clique: Vec<usize> = Vec::new();
    for clique in cliques.iter() {
        if clique.len() > max_clique.len() {
            max_clique = clique.iter().map(|e|e.index()).collect();
        }
    };
    max_clique
}

fn clique_to_paths(clique: Vec<usize>, id_to_path: &HashMap<usize, QualifiedPath>) -> Vec<QualifiedPath> {
    let mut paths: Vec<QualifiedPath> = Vec::new();
    for path in clique {
        paths.push(id_to_path.get(&path).unwrap().clone());
    }
    paths
}

fn make_no_conflict_log() -> String {
    "without conflicts".green().to_string()
}

fn make_conflict_log() -> String {
    "will produce conflicts".red().to_string()
}

#[derive(Clone, Debug)]
pub struct DeriveCommand;

impl CommandDefinition for DeriveCommand {
    fn build_command(&self) -> Command {
        Command::new("derive")
            .about("Derive a product")
            .disable_help_subcommand(true)
            .arg(
                Arg::new("features")
                    .action(ArgAction::Append)
                    .required(true),
            )
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
        let target_product_name = context
            .arg_helper
            .get_argument_value::<String>("product")
            .unwrap();
        let current_area = context.git.get_current_area()?;
        let target_path =
            current_area.get_path_to_product_root() + QualifiedPath::from(target_product_name);

        let all_features = context
            .arg_helper
            .get_argument_values::<String>("features")
            .unwrap()
            .into_iter()
            .map(|e| current_area.get_path_to_feature_root() + QualifiedPath::from(e))
            .collect::<Vec<_>>();

        context.log_to_stdout("Checking for conflicts");
        let (id_to_path, path_to_id) = map_paths_to_id(&all_features);
        let conflicts = context.git.check_for_conflicts(&all_features)?;
        let edges = build_edges(&conflicts, &path_to_id);
        let graph = UnGraph::<usize, ()>::from_edges(&edges);
        let max_clique = get_max_clique(&graph);
        let mergeable_features = clique_to_paths(max_clique, &id_to_path);
        if mergeable_features.len() == all_features.len() {
            let area_path = current_area.get_qualified_path();
            drop(current_area);
            context.git.checkout(&area_path)?;
            context.git.create_branch(&target_path)?;
            context.git.checkout(&target_path)?;
            context.git.merge(&all_features)?;
            context.log_to_stdout("Derivation finished ".to_string() + make_no_conflict_log().as_str() + ".");
        } else {
            context.log_to_stdout(format!("Can merge {} features ", mergeable_features.len()) + make_no_conflict_log().as_str() + ".");
            context.log_to_stdout(format!("{} features ", all_features.len() - mergeable_features.len()) + make_conflict_log().as_str() + ".");
            context.log_to_stdout("A partial derivation will be performed with all conflict-free features.")
        }
        Ok(())
    }
    fn shell_complete(
        &self,
        completion_helper: CompletionHelper,
        context: &mut CommandContext,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let maybe_feature_root = context.git.get_current_area()?.to_feature_root();
        if maybe_feature_root.is_none() {
            return Ok(vec![]);
        }
        let feature_root = maybe_feature_root.unwrap();
        let current = completion_helper.currently_editing();
        let result = match current {
            Some(value) => match value.get_id().as_str() {
                "features" => completion_helper.complete_qualified_path(
                    AbsolutePathCompletion,
                    &feature_root.get_child_paths_by_branch().get(&true).unwrap(),
                    true,
                ),
                _ => vec![],
            },
            None => vec![],
        };
        Ok(result)
    }
}
