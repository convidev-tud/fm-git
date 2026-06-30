mod git;

use crate::git::Git;
use hyrmir_cli::{ArgSource, CommandLogger, EntryPoint, PrintingLogger};
use hyrmir_lib::repository::RepositoryLoader;
use log::{LevelFilter, set_logger, set_max_level};

fn main() {
    set_logger(&PrintingLogger).unwrap();
    set_max_level(LevelFilter::Info);
    let mut logger = CommandLogger::new();
    let mut loader = RepositoryLoader::new(Git::new());
    let entry = EntryPoint::new("tangl");
    entry.execute(&mut loader, &mut logger, ArgSource::CLI)
}
