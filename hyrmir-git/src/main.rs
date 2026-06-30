mod git;

use crate::git::{Git, GitCLI, GitPath};
use hyrmir_cli::{ArgSource, CommandLogger, EntryPoint, PrintingLogger};
use hyrmir_lib::repository::Repository;
use log::{LevelFilter, set_logger, set_max_level};
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    set_logger(&PrintingLogger).unwrap();
    set_max_level(LevelFilter::Info);
    let mut logger = CommandLogger::new();
    let cli = GitCLI::new(GitPath::CurrentDirectory);
    let mut repository = Repository::new(Git::new(Rc::new(RefCell::new(cli))));
    let entry = EntryPoint::new("tangl");
    entry.execute(&mut repository, &mut logger, ArgSource::CLI)
}
