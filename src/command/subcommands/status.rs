use crate::command::*;
use clap::builder::Str;
use clap::Command;

#[derive(Clone)]
pub struct StatusCommand {
    name: Str,
}

impl StatusCommand {
    pub fn new() -> Self { Self { name: "status".into()} }
}

impl CommandDefinition for StatusCommand {
    fn get_name(&self) -> String {
        self.name.clone().into()
    }
    fn build_command(&self) -> Command {
        Command::new(self.name.clone())
            .about("Shows details of a run")
            .after_help("More detail")
    }
    fn run_command(&self) -> CommandResult {
        let output = std::process::Command::new("git")
            .args(["status", "--porcelain=1"])
            .output()
            .expect("failed to execute process");
        CommandResult::from_u8(&output.stdout, &output.stderr)
    }
}
