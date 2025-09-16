use clap::Command;

pub trait CommandDefinition: Clone {
    fn get_name(&self) -> String;
    fn build_command(&self) -> Command;
    fn run_command(&self);
}