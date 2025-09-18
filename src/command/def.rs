use clap::Command;

pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
}
impl CommandResult {
    pub fn from_u8(stdout: &Vec<u8>, stderr: &Vec<u8>) -> CommandResult {
        Self {
            stdout: String::from(std::str::from_utf8(stdout).unwrap()),
            stderr: String::from(std::str::from_utf8(stderr).unwrap()),
        }
    }
    pub fn print(&self) {
        if self.stdout.len() > 0 {
            println!("{}", self.stdout.trim_end())
        }
        if self.stderr.len() > 0 {
            println!("{}", self.stderr.trim_end())
        }
    }
}
pub trait CommandDefinition {
    fn get_name(&self) -> String;
    fn build_command(&self) -> Command;
    fn run_command(&self) -> CommandResult;
}