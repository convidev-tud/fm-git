use crate::arg::ArgHelper;
use crate::completion::CompletionHelper;
use crate::logging::CommandLogger;
use clap::{ArgMatches, Command};
use hyrmir_lib::importer::ImportFormat;
use hyrmir_lib::repository::RepositoryLoader;
use hyrmir_lib::vcs::VCS;
use std::error::Error;
use std::fmt::Debug;

#[derive(Debug)]
pub struct CommandMap<V: VCS> {
    pub clap_command: Command,
    pub command: Box<dyn CommandInterface<V>>,
    pub children: Vec<CommandMap<V>>,
}

impl<V: VCS> CommandMap<V> {
    pub fn new(command: Box<dyn CommandImpl<V>>) -> CommandMap<V> {
        let mut children: Vec<CommandMap<V>> = Vec::new();
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
            clap_command,
            command,
            children,
        }
    }
    pub fn find_child(&self, name: &str) -> Option<&CommandMap<V>> {
        self.children
            .iter()
            .find(|child| child.clap_command.get_name() == name)
    }
    pub fn find_current_child(&self, matches: &ArgMatches) -> Option<&CommandMap<V>> {
        match matches.subcommand() {
            Some((name, sub_matches)) => {
                let maybe_child = self.find_child(name);
                if maybe_child.is_some() {
                    let child_result = maybe_child.unwrap().find_current_child(sub_matches);
                    if child_result.is_some() {
                        child_result
                    } else {
                        Some(self)
                    }
                } else {
                    Some(self)
                }
            }
            _ => Some(self),
        }
    }
    pub fn find_children_by_prefix(&self, prefix: &str) -> Vec<&CommandMap<V>> {
        self.children
            .iter()
            .filter(|child| child.clap_command.get_name().starts_with(prefix))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct CommandContext<'a, V: VCS> {
    current_command: &'a CommandMap<V>,
    root_command: &'a CommandMap<V>,
    arg_helper: ArgHelper,
    import_format: ImportFormat,
}

impl<'a, V: VCS> CommandContext<'a, V> {
    pub fn new(
        current_command: &'a CommandMap<V>,
        root_command: &'a CommandMap<V>,
        arg_helper: ArgHelper,
        import_format: ImportFormat,
    ) -> CommandContext<'a, V> {
        CommandContext {
            current_command,
            root_command,
            arg_helper,
            import_format,
        }
    }
    
    pub fn get_current_command(&self) -> &CommandMap<V> {
        &self.current_command
    }
    
    pub fn get_root_command(&self) -> &CommandMap<V> {
        &self.root_command
    }
    
    pub fn get_arg_helper(&self) -> &ArgHelper {
        &self.arg_helper
    }
    
    pub fn get_import_format(&self) -> &ImportFormat {
        &self.import_format
    }
}

pub trait CommandDefinition<V: VCS>: Debug {
    fn build_command(&self) -> Command;
    fn get_subcommands(&self) -> Vec<Box<dyn CommandImpl<V>>> {
        Vec::new()
    }
}

pub trait CommandInterface<V: VCS>: Debug {
    fn run_command(
        &self,
        _loader: &mut RepositoryLoader<V>,
        _logger: &mut CommandLogger,
        _context: &CommandContext<V>,
    ) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn shell_complete(
        &self,
        _loader: &mut RepositoryLoader<V>,
        _completion_helper: CompletionHelper,
        _context: &CommandContext<V>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        Ok(Vec::new())
    }
}

pub trait CommandImpl<V: VCS>: CommandDefinition<V> + CommandInterface<V> {}

impl<V: VCS, T: CommandDefinition<V> + CommandInterface<V>> CommandImpl<V> for T {}
