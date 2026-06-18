use crate::arg::ArgHelper;
use crate::commands::*;
use crate::completion::CompletionHelper;
use crate::logging::CommandLogger;
use crate::*;
use clap::{Arg, ArgMatches, Command};
use hyrmir_lib::importer::ImportFormat;
use hyrmir_lib::repository::Repository;
use hyrmir_lib::vcs::VCS;
use log::LevelFilter;
use std::error::Error;
use std::marker::PhantomData;

const IMPORT_FORMAT: &str = "import_format";

#[derive(Clone, Debug)]
pub struct RootCommand<V: VCS> {
    name: String,
    _phantom: PhantomData<V>,
}

impl<V: VCS> RootCommand<V> {
    pub fn new(name: String) -> Self {
        Self { name, _phantom: PhantomData }
    }
}

impl<V: VCS + 'static> CommandDefinition<V> for RootCommand<V> {
    fn build_command(&self) -> Command {
        Command::new("tangl")
            .arg_required_else_help(true)
            .arg(
                Arg::new(IMPORT_FORMAT)
                    .short('f')
                    .long("import-format")
                    .default_value("waffle")
                    .help("Specify file import format for all commands"),
            )
    }

    fn get_subcommands(&self) -> Vec<Box<dyn CommandImpl<V>>> {
        vec![
            Box::new(StatusCommand::new()),
            // Box::new(LSCommand),
            // Box::new(DeriveCommand),
            // Box::new(CheckCommand),
            // Box::new(CheckoutCommand),
            // Box::new(InitCommand),
            // Box::new(CloneCommand),
            // Box::new(FeatureCommand),
            // Box::new(ProductCommand),
            // Box::new(TagCommand),
            // Box::new(SpreadCommand),
            // Box::new(UntieCommand),
            // Box::new(CommitCommand),
            // Box::new(HiddenCompletionCommand),
        ]
    }
}

impl<V: VCS> CommandInterface<V> for RootCommand<V> {
    fn run_command(
        &self,
        _repository: &mut Repository<V>,
        _logger: &mut CommandLogger,
        context: &CommandContext<V>,
    ) -> Result<(), Box<dyn Error>> {
        // let format = context
        //     .arg_helper
        //     .get_argument_value::<String>(crate::def::IMPORT_FORMAT)
        //     .unwrap();
        // context.import_format = ImportFormat::from(format);
        Ok(())
    }

    fn shell_complete(
        &self, _repository: &mut Repository<V>,
        completion_helper: CompletionHelper,
        _context: &CommandContext<V>,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        match completion_helper.currently_editing() {
            Some(value) => match value.get_id().as_str() {
                "format" => Ok(vec!["waffle".to_string(), "uvl".to_string()]),
                _ => Ok(vec![]),
            },
            None => Ok(vec![]),
        }
    }
}

pub enum ArgSource<'a> {
    CLI,
    SUPPLIED(Vec<&'a str>),
}

pub struct EntryPoint<V: VCS> {
    command_map: CommandMap<V>,
}
impl<V: VCS + 'static> EntryPoint<V> {
    pub fn new(name: &str) -> Self {
        Self {
            command_map: CommandMap::new(Box::new(RootCommand::new(name.to_string()))),
        }
    }

    fn execute_recursive<'a>(
        &self,
        repository: &mut Repository<V>,
        logger: &mut CommandLogger,
        context: CommandContext<V>,
    ) -> Result<(), Box<dyn Error>> {
        let arg_helper = context.get_arg_helper();
        // if arg_helper.has_arg(VERBOSE) {
        //     match arg_helper.get_count(VERBOSE) {
        //         0 => log::set_max_level(LevelFilter::Info),
        //         1 => log::set_max_level(LevelFilter::Debug),
        //         _ => log::set_max_level(LevelFilter::Trace),
        //     }
        // } else {
        //     log::set_max_level(LevelFilter::Info)
        // }
        log::set_max_level(LevelFilter::Info);
        let current = context.get_current_command();
        match current.command.run_command(repository, logger, &context) {
            Ok(_) => {}
            Err(err) => return Err(err),
        };
        match arg_helper.get_matches().subcommand() {
            Some((sub, sub_args)) => {
                if let Some(child) = current.find_child(sub) {
                    let new_context = CommandContext::<V>::new(
                        child,
                        context.get_root_command(),
                        ArgHelper::new(sub_args.clone()),
                        context.get_import_format().clone(),
                    );
                    self.execute_recursive(repository, logger, new_context)
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    pub fn execute(&self, repository: &mut Repository<V>, logger: &mut CommandLogger, arg_source: ArgSource) {
        let context = self.build_context(arg_source, ImportFormat::Waffle);
        match self.execute_recursive(repository, logger, context) {
            Ok(_) => {}
            Err(err) => { logger.error(err.to_string()); }
        }
    }

    fn build_context(
        &self,
        arg_source: ArgSource,
        import_format: ImportFormat,
    ) -> CommandContext<V> {
        let args: ArgMatches = match arg_source {
            ArgSource::CLI => self.command_map.clap_command.clone().get_matches(),
            ArgSource::SUPPLIED(supplied) => self
                .command_map
                .clap_command
                .clone()
                .get_matches_from(supplied),
        };
        CommandContext::new(
            &self.command_map,
            &self.command_map,
            ArgHelper::new(args),
            import_format,
        )
    }
}
