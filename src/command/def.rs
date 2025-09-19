use clap::{ArgMatches, Command};
use std::fmt::Debug;

pub struct CommandState {}

impl CommandState {
    pub fn new() -> CommandState { CommandState {} }
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
    fn run_command(
        &self,
        _args: &ArgMatches,
        _state: CommandState,
    ) -> CommandState {
        CommandState::new()
    }
}