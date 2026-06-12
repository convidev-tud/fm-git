mod git;

use hyrmir_cli::{ArgSource, CommandLogger, EntryPoint};
use crate::git::Git;

fn main() {
    let logger = CommandLogger::new();
    let mut entry = EntryPoint::new("tangl", Git::new(), logger);
    entry.execute(ArgSource::CLI)
}