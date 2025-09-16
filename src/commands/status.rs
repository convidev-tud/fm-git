use clap::Command;
use crate::commands::*;

#[derive(Clone)]
pub struct Status;

impl CommandDefinition for Status {
    fn get_name(&self) -> String {
        "status".into()
    }
    fn build_command(&self) -> Command {
        Command::new("status")
            .about("Shows details of a run")
            .after_help("More detail")
    }
    fn run_command(&self) {
        println!("run status command");
    }
}
