use clap::{Arg, ArgAction};

pub fn make_show_tags() -> Arg {
    Arg::new("show_tags")
        .long("show-tags")
        .action(ArgAction::SetTrue)
        .help("Also show tags")
}

pub fn make_delete(force: bool) -> Arg {
    let short = if force { 'D' } else { 'd' };
    Arg::new("delete").short(short)
}
