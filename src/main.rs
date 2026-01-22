use tangl::cli::{ArgSource, CommandRepository, TangleCommand};
use tangl::git::interface::GitPath;

fn main() {
    let command_repository =
        CommandRepository::new(Box::new(TangleCommand {}), GitPath::CurrentDirectory);
    command_repository.execute(ArgSource::CLI);
}
