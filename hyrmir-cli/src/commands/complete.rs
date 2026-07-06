use crate::completion::CompletionHelper;
use crate::{CommandContext, CommandDefinition, CommandInterface, CommandLogger};
use clap::{Arg, ArgAction, Command};
use hyrmir_lib::repository::RepositoryLoader;
use hyrmir_lib::vcs::VCS;
use std::error::Error;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct HiddenCompletionCommand<V: VCS> {
    _marker: PhantomData<V>,
}

impl<V: VCS> HiddenCompletionCommand<V> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<V: VCS> CommandDefinition<V> for HiddenCompletionCommand<V> {
    fn build_command(&self) -> Command {
        Command::new("__completion")
            .hide(true)
            .arg(Arg::new("cli").raw(true))
            .arg(Arg::new("index").short('i'))
            .disable_help_subcommand(true)
    }
}

impl<V: VCS> CommandInterface<V> for HiddenCompletionCommand<V> {
    fn run_command(
        &self,
        loader: &mut RepositoryLoader<V>,
        _logger: &mut CommandLogger,
        context: &CommandContext<V>,
    ) -> Result<(), Box<dyn Error>> {
        let arg_helper = context.get_arg_helper();
        let cursor_index: usize = arg_helper
            .get_argument_value::<String>("index")
            .unwrap()
            .parse()?;
        let matches = arg_helper.get_argument_values::<String>("cli").unwrap();
        let mut to_complete = matches.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        if to_complete.is_empty() {
            return Ok(());
        }
        to_complete = to_complete[..cursor_index + 1].to_vec();
        let root = context.get_root_command();
        let matches = root
            .clap_command
            .clone()
            .ignore_errors(true)
            .get_matches_from(to_complete.clone());
        let maybe_last_child = root.find_current_child(&matches);
        let last_item = <&str>::clone(to_complete.last().unwrap());
        match maybe_last_child {
            Some(last_child) => {
                let completion = last_child
                    .command
                    .shell_complete(
                        loader,
                        CompletionHelper::new(&root.clap_command, to_complete),
                        context,
                    )?
                    .iter()
                    .filter(|c| c.starts_with(last_item))
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                match completion.len() {
                    0 => {}
                    _ => {
                        println!("{}", completion.join(" "));
                        return Ok(());
                    }
                }
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
                        println!("{}", subcommand)
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
                                    println!("{}", s)
                                }
                            }
                        }
                        let long = arg.get_long();
                        if long.is_some() {
                            let mut s = "--".to_string();
                            s.push_str(long.unwrap());
                            if s.starts_with(last_item) {
                                println!("{}", s)
                            }
                        }
                    }
                }
            }
            None => {}
        }
        Ok(())
    }
}
