use crate::cli::{ArgHelper, CommandContext, CommandImpl, CommandMap};
use crate::git::interface::GitInterface;
use std::ffi::OsString;

pub struct CommandRepository {
    command_map: CommandMap,
}
impl CommandRepository {
    pub fn new(command: Box<dyn CommandImpl>) -> Self {
        Self {
            command_map: CommandMap::new(command),
        }
    }
    fn execute_recursive(&self, context: &mut CommandContext) {
        let current = context.current_command;
        match current.command.run_command(context) {
            Ok(_) => {}
            Err(err) => context.log_to_stderr(err.to_string()),
        };
        match context.arg_helper.get_matches().subcommand() {
            Some((sub, sub_args)) => {
                if let Some(child) = current.find_child(sub) {
                    self.execute_recursive(&mut CommandContext::new(
                        child,
                        context.root_command,
                        context.git,
                        ArgHelper::new(sub_args),
                    ))
                } else {
                    let ext_args: Vec<_> = sub_args.get_many::<OsString>("").unwrap().collect();
                    std::process::Command::new("git")
                        .arg(sub)
                        .args(ext_args)
                        .status()
                        .expect("failed to execute git");
                }
            }
            _ => {}
        }
    }
    pub fn execute(&self) {
        self.execute_recursive(&mut CommandContext::new(
            &self.command_map,
            &self.command_map,
            &mut GitInterface::new(),
            ArgHelper::new(&self.command_map.clap_command.clone().get_matches()),
        ));
    }
}
