use clap::{ArgMatches, Command};
use std::fmt::Debug;

#[derive(Debug)]
pub struct CommandMap {
    pub command: Command,
    pub command_definition: Box<dyn CommandDefinition>,
    pub children: Vec<CommandMap>,
}

impl CommandMap {
    pub fn new(command: Box<dyn CommandDefinition>) -> CommandMap {
        let mut children: Vec<CommandMap> = Vec::new();
        let clap_command = command.build_command().subcommands(
            command
                .get_subcommands()
                .into_iter()
                .map(|c| {
                    let sub_command = c.build_command();
                    children.push(CommandMap::new(c));
                    sub_command
                })
                .collect::<Vec<Command>>(),
        );
        CommandMap {
            command: clap_command,
            command_definition: command,
            children,
        }
    }
    pub fn find_child(&self, name: &str) -> Option<&CommandMap> {
        self.children.iter().find(|child| {child.command.get_name() == name})
    }
    pub fn find_last_child_recursive(&self, names: &mut Vec<&str>) -> Option<&CommandMap> {
        if names.is_empty() { return None; }
        if names.len() >= 1 && self.command.get_name() == *names.first().unwrap() {
            if names.len() == 1 { return Some(self) }
            names.remove(0);
            let maybe_child = self.find_child(names.first().unwrap());
            if maybe_child.is_some() {
                let maybe_final = maybe_child.unwrap().find_last_child_recursive(names);
                if maybe_final.is_some() {
                    maybe_final
                } else { Some (self) }
            } else { Some(self) }
        } else { None }
    }
    pub fn find_children_by_prefix(&self, prefix: &str) -> Vec<&CommandMap> {
        self.children.iter()
            .filter(|child| {child.command.get_name().starts_with(prefix)})
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct CommandState<'a> {
    pub command_map: &'a CommandMap,
}

impl CommandState<'_> {
    pub fn new(command_map: &'_ CommandMap) -> CommandState<'_> {
        CommandState { command_map }
    }
    pub fn log_from_u8(&self, stdout: &Vec<u8>, stderr: &Vec<u8>) {
        self.log_to_stdout(String::from(std::str::from_utf8(stdout).unwrap()));
        self.log_to_stderr(String::from(std::str::from_utf8(stderr).unwrap()));
    }
    pub fn log_to_stdout(&self, stdout: String) {
        if stdout.len() > 0 {
            println!("{}", stdout.trim_end())
        }
    }
    pub fn log_to_stderr(&self, stderr: String) {
        if stderr.len() > 0 {
            println!("{}", stderr.trim_end())
        }
    }
}

pub trait CommandDefinition: Debug {
    fn build_command(&self) -> Command;
    fn get_subcommands(&self) -> Vec<Box<dyn CommandDefinition>> {
        Vec::new()
    }
    fn run_command<'a>(&self, _args: &ArgMatches, state: CommandState<'a>) -> CommandState<'a> {
        state
    }
    fn shell_complete(&self, _appendix: Option<&str>, _state: CommandState) -> Option<String> {
        None
    }
}
