use crate::logging::CommandLogger;
use crate::*;
use clap::ArgMatches;
use log::LevelFilter;
use std::error::Error;
use std::ffi::OsString;
use hyrmir_lib::vcs::VCS;

const IMPORT_FORMAT: &str = "import_format";

#[derive(Clone, Debug)]
pub struct RootCommand<V: VCS>;

impl<V: VCS> CommandDefinition<V> for RootCommand<V> {
    fn build_command(&self) -> Command {
        Command::new("tangl")
            .arg_required_else_help(true)
            .arg(
                Arg::new(crate::def::IMPORT_FORMAT)
                    .short('f')
                    .long("import-format")
                    .default_value("waffle")
                    .help("Specify file import format for all commands"),
            )
    }
    fn get_subcommands(&self) -> Vec<Box<dyn CommandImpl<V>>> {
        vec![
            Box::new(StatusCommand),
            Box::new(LSCommand),
            Box::new(DeriveCommand),
            Box::new(CheckCommand),
            Box::new(CheckoutCommand),
            Box::new(InitCommand),
            Box::new(CloneCommand),
            Box::new(FeatureCommand),
            Box::new(ProductCommand),
            Box::new(TagCommand),
            Box::new(SpreadCommand),
            Box::new(UntieCommand),
            Box::new(CommitCommand),
            Box::new(HiddenCompletionCommand),
        ]
    }
}

impl<V: VCS> CommandInterface<V> for RootCommand<V> {
    fn run_command(
        &self,
        workspace: &mut Workspace<V>,
        context: &mut CommandContext<V>,
    ) -> Result<(), Box<dyn Error>> {
        let format = context
            .arg_helper
            .get_argument_value::<String>(crate::def::IMPORT_FORMAT)
            .unwrap();
        context.import_format = ImportFormat::from(format);
        Ok(())
    }

    fn shell_complete(
        &self,
        completion_helper: CompletionHelper,
        _context: &mut CommandContext,
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

pub struct CommandRepository<V: VCS> {
    command_map: CommandMap<V>,
}
impl<V: VCS> CommandRepository<V> {
    pub fn new(root_command: Box<dyn CommandImpl<V>>) -> Self {
        Self {
            command_map: CommandMap::new(root_command),
        }
    }
    fn execute_recursive<'a>(
        &self,
        mut context: CommandContext<'a>,
    ) -> Result<CommandContext<'a>, Box<dyn Error>> {
        if context.arg_helper.has_arg(VERBOSE) {
            match context.arg_helper.get_count(VERBOSE) {
                0 => log::set_max_level(LevelFilter::Info),
                1 => log::set_max_level(LevelFilter::Debug),
                _ => log::set_max_level(LevelFilter::Trace),
            }
        } else {
            log::set_max_level(LevelFilter::Info)
        }
        let current = context.current_command;
        match current.command.run_command(&mut context) {
            Ok(_) => {}
            Err(err) => return Err(err),
        };
        match context.arg_helper.get_matches().subcommand() {
            Some((sub, sub_args)) => {
                if let Some(child) = current.find_child(sub) {
                    context.current_command = child;
                    context.arg_helper = ArgHelper::new(sub_args.clone());
                    self.execute_recursive(context)
                } else {
                    let ext_args: Vec<_> = sub_args.get_many::<OsString>("").unwrap().collect();
                    std::process::Command::new("git")
                        .arg(sub)
                        .args(ext_args)
                        .status()
                        .expect("failed to execute git");
                    Ok(context)
                }
            }
            _ => Ok(context),
        }
    }
    pub fn execute(&self, arg_source: ArgSource) -> Result<(), Box<dyn Error>> {
        let context = self.build_context(arg_source, ImportFormat::Waffle);
        self.execute_recursive(context)?;
        Ok(())
    }
    pub fn build_context(
        &self,
        arg_source: ArgSource,
        import_format: ImportFormat,
    ) -> CommandContext<'_> {
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
            GitInterface::new(self.work_path.clone()),
            CommandLogger::new(),
            ArgHelper::new(args),
            import_format,
        )
    }
}
