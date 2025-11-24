use clap::{Arg, ArgAction};

pub fn make_show_tags() -> Arg {
    Arg::new("show_tags")
        .long("show-tags")
        .action(ArgAction::SetTrue)
        .help("Also how tags")
}