use crate::cli::*;
use clap::{Arg, ArgAction, ArgMatches, Command};

#[derive(Clone, Debug)]
pub struct HiddenCompletionCommand {}

impl CommandDefinition for HiddenCompletionCommand {
    fn build_command(&self) -> Command {
        Command::new("__completion")
            .hide(true)
            .arg(Arg::new("cli").raw(true))
            .disable_help_subcommand(true)
    }
}

impl CommandInterface for HiddenCompletionCommand {
    fn run_command(&self, args: &ArgMatches, _current: &CommandMap, state: &mut CommandContext) {
        let mut to_complete = args
            .get_many::<String>("cli")
            .unwrap()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();
        if to_complete.is_empty() {
            return;
        }
        let maybe_last_child = state
            .command_map
            .find_last_child_recursive(&mut to_complete.clone());
        let last_item = <&str>::clone(to_complete.last().unwrap());
        match maybe_last_child {
            Some(last_child) => {
                let mut subcommands = last_child
                    .find_children_by_prefix(last_item)
                    .iter()
                    .map(|c| c.clap_command.get_name())
                    .collect::<Vec<_>>();
                if "help".starts_with(last_item)
                    && !last_child.clap_command.is_disable_help_subcommand_set()
                {
                    subcommands.push("help")
                }
                for subcommand in subcommands {
                    if subcommand != "__completion" {
                        state.log_to_stdout(subcommand)
                    }
                }
                if last_item.starts_with("-") {
                    let mut all_args: Vec<&Arg> = last_child.clap_command.get_arguments().collect();
                    let help_attr = Arg::new("help")
                        .long("help")
                        .short('h')
                        .action(ArgAction::Help);
                    if !last_child.clap_command.is_disable_help_flag_set() {
                        all_args.push(&help_attr)
                    }
                    for arg in all_args {
                        if !last_item.starts_with("--") {
                            let short = arg.get_short();
                            if short.is_some() {
                                let mut s = "-".to_string();
                                s.push_str(short.unwrap().to_string().as_str());
                                if s.starts_with(last_item) {
                                    state.log_to_stdout(s)
                                }
                            }
                        }
                        let long = arg.get_long();
                        if long.is_some() {
                            let mut s = "--".to_string();
                            s.push_str(long.unwrap());
                            if s.starts_with(last_item) {
                                state.log_to_stdout(s)
                            }
                        }
                    }
                }
                let completion =
                    last_child
                        .command
                        .shell_complete(to_complete[1..].to_vec(), last_child, state);
                match completion.len() {
                    0 => {}
                    _ => state.log_to_stdout(&*completion.join(" ")),
                }
            }
            None => {}
        }
    }
}
