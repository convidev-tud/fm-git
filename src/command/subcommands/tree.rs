use crate::command::*;
use clap::{ArgMatches, Command};
use std::collections::HashMap;
use termtree::Tree;

fn extend_tree_from_vec_rec(tree: Tree<String>, path: Vec<String>) -> Tree<String> {
    if path.len() == 0 { return tree; }
    let mut work_tree = tree.clone();
    let maybe_subtree = tree.leaves.iter().find(|x| x.root == path[0]);
    let subtree = if maybe_subtree.is_some() {
        work_tree.leaves = work_tree.leaves.into_iter().filter(|x| x.root != path[0]).collect();
        maybe_subtree.unwrap()
    } else {
        &Tree::new(path[0].clone())
    };
    work_tree.push(extend_tree_from_vec_rec(subtree.clone(), path[1..].to_vec()));
    work_tree
}

#[derive(Clone, Debug)]
pub struct TreeCommand {}

impl CommandDefinition for TreeCommand {
    fn build_command(&self) -> Command {
        Command::new("tree")
            .about("Displays the tree structure")
            .disable_help_subcommand(true)
    }
}

impl CommandInterface for TreeCommand {
    fn run_command(
        &self,
        _args: &ArgMatches,
        state: &mut CommandContext,
    ) {
        let mut all_branches = state.git.get_all_branches()
            .iter()
            .map(|branch| {
                branch
                    .replace("_", "")
                    .split("/")
                    .into_iter()
                    .map(|e| e.trim().to_string())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        all_branches.sort_by(|a, b| a.len().cmp(&b.len()));
        let mut trees: HashMap<String, Tree<String>> = HashMap::new();
        for branch in all_branches {
            match branch.len() {
                0 => panic!("branch name cannot be empty"),
                _ => {
                    if !trees.contains_key(&branch[0]) { trees.insert(branch[0].clone(), Tree::new(branch[0].clone())); }
                    trees.insert(branch[0].clone(), extend_tree_from_vec_rec(trees.get(&branch[0]).unwrap().clone(), branch[1..].to_vec()));
                }
            }
        }
        for tree in trees.values() {
            state.log_to_stdout(tree.to_string().trim().to_string());
        }
    }
}
