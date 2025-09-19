use crate::command::*;
use clap::{Arg, ArgMatches, Command};


#[derive(Clone, Debug)]
pub struct CompletionCommand {}

impl CommandDefinition for CompletionCommand {
    fn build_command(&self) -> Command {
        Command::new("__completion")
            .hide(true)
            .arg(Arg::new("command").raw(true))
    }
    fn run_command<'a>(
        &self,
        args: &ArgMatches,
        state: CommandState<'a>,
    ) -> CommandState<'a> {
        let mut to_complete = args
            .get_many::<String>("command")
            .unwrap()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        if to_complete.is_empty() { return state }
        let maybe_last_child = state.command_map.find_last_child_recursive(
            &mut to_complete
        );
        let last_item = if to_complete.is_empty() { None } else { Some(<&str>::clone(to_complete.last().unwrap())) };
        match maybe_last_child {
            Some(last_child) => {
                if last_item.is_some() {
                    let unwrapped_last_item = last_item.unwrap();
                    let subcommands = last_child.find_children_by_prefix(unwrapped_last_item);
                    for subcommand in subcommands {
                        if subcommand.command.get_name() != "__completion" {
                            state.log_to_stdout(subcommand.command.get_name().to_string())
                        }
                    }
                    if unwrapped_last_item.starts_with("-") {
                        for arg in last_child.command.get_arguments() {
                            if !unwrapped_last_item.starts_with("--") {
                                let short = arg.get_short();
                                if short.is_some() {
                                    let mut s = "-".to_string();
                                    s.push_str(short.unwrap().to_string().as_str());
                                    state.log_to_stdout(s)
                                }
                            }
                            let long = arg.get_long();
                            if long.is_some() {
                                let mut s = "--".to_string();
                                s.push_str(long.unwrap());
                                state.log_to_stdout(s)
                            }
                        }
                    }
                }
                match last_child.command_definition.shell_complete(
                    last_item,
                    state.clone(),
                ) {
                    Some(completed_string) => { state.log_to_stdout(completed_string) }
                    _ => {}
                }
            }
            None => {}
        }
        state
    }
}
