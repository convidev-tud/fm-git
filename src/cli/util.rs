use clap::{Arg, ArgAction, ArgMatches, Command};
use std::collections::HashMap;

pub fn get_argument_value<T: Clone + Send + Sync + 'static>(id: &str, args: &ArgMatches) -> T {
    args.get_one::<T>(id).unwrap().clone()
}
pub fn get_argument_values<T: Clone + Send + Sync + 'static>(
    id: &str,
    args: &ArgMatches,
) -> Vec<T> {
    args.get_many::<T>(id)
        .unwrap()
        .map(|s| s.clone())
        .collect::<Vec<_>>()
}
/// Returns if the passed target is the currently one edited on the console.
///
/// Examples:
/// ```bash
/// mytool foo // foo is edited
/// mytool foo bar // foo is edited, if curser remains on bar
/// mytool foo bar abc // foo is not edited
/// ```
pub fn currently_editing(command: &Command, appendix: &Vec<&str>) -> Option<String> {
    let mut current_option: Option<&Arg> = None;
    let mut current_option_start: usize = 0;
    let mut positionals = command.get_positionals();
    let mut current_positional: Option<&Arg> = None;
    let mut current_positional_start: usize = 0;
    // check if the last arg is still edited
    fn is_last_option(
        index: usize,
        current_option: Option<&Arg>,
        current_option_start: usize,
    ) -> bool {
        if current_option.is_none() {
            return false;
        }
        match current_option.unwrap().get_action() {
            ArgAction::Set => current_option_start == index - 1,
            ArgAction::Append => current_option_start < index,
            _ => false,
        }
    }
    fn is_last_positional(
        index: usize,
        current_positional: Option<&Arg>,
        current_positional_start: usize,
    ) -> bool {
        if current_positional.is_none() {
            return false;
        }
        match current_positional.unwrap().get_action() {
            ArgAction::Set => current_positional_start == index,
            ArgAction::Append => current_positional_start <= index,
            _ => false,
        }
    }
    // match appendix index to argument
    let cmd_to_index: HashMap<usize, &Arg> = appendix
        .iter()
        .enumerate()
        .filter_map(|(index, element)| {
            if element.to_string() == command.get_name() {
                return None;
            }
            // checks if the current one is an option name
            let found_option = command.get_opts().find(|o| {
                let found_short = match o.get_short() {
                    Some(short) => {
                        ("-".to_string() + short.to_string().as_str()) == element.to_string()
                    }
                    None => false,
                };
                let found_long = match o.get_long() {
                    Some(long) => ("--".to_string() + long) == element.to_string(),
                    None => false,
                };
                found_short || found_long
            });
            let maybe_option: Option<(usize, &Arg)> = match found_option {
                // if currently an option, save the index
                Some(option) => {
                    current_option = Some(option);
                    current_option_start = index;
                    return None;
                }
                // if not, check if the last option is still edited
                None => {
                    if is_last_option(index, current_option, current_option_start) {
                        Some((index, current_option.unwrap()))
                    } else {
                        None
                    }
                }
            };
            if maybe_option.is_some() {
                return Some(maybe_option.unwrap());
            }
            // if no optional, move on to positionals
            if is_last_positional(index, current_positional, current_positional_start) {
                return Some((index, current_positional.unwrap()));
            }
            match positionals.next() {
                Some(positional) => {
                    current_positional_start = index;
                    current_positional = Some(positional);
                    Some((index, positional))
                }
                None => None,
            }
        })
        .collect();
    Some(
        cmd_to_index
            .get(&(appendix.len() - 1))?
            .get_id()
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Command;

    fn setup_test_command() -> Command {
        Command::new("mytool")
            .arg(Arg::new("option1").long("option1").short('a'))
            .arg(
                Arg::new("option2")
                    .long("option2")
                    .short('b')
                    .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("pos1"))
            .arg(Arg::new("pos2").action(ArgAction::Append))
    }

    #[test]
    fn test_currently_editing_emtpy() {
        let cmd = setup_test_command();
        let appendix = vec!["mytool"];
        assert_eq!(currently_editing(&cmd, &appendix), None);
    }
    #[test]
    fn test_currently_editing_one_option() {
        let cmd = setup_test_command();
        let appendix = vec!["mytool", "--option1", ""];
        assert_eq!(
            currently_editing(&cmd, &appendix),
            Some("option1".to_string())
        );
    }
    #[test]
    fn test_currently_editing_one_option_one_positional() {
        let cmd = setup_test_command();
        let appendix = vec!["mytool", "--option1", "abc", ""];
        assert_eq!(currently_editing(&cmd, &appendix), Some("pos1".to_string()));
    }
    #[test]
    fn test_currently_editing_one_positional() {
        let cmd = setup_test_command();
        let appendix = vec!["mytool", "abc"];
        assert_eq!(currently_editing(&cmd, &appendix), Some("pos1".to_string()));
    }
    #[test]
    fn test_currently_editing_append() {
        let cmd = setup_test_command();
        let appendix = vec!["mytool", "abc", "a", "b", "c", "d"];
        assert_eq!(currently_editing(&cmd, &appendix), Some("pos2".to_string()));
    }
    #[test]
    fn test_currently_boolean() {
        let cmd = setup_test_command();
        let appendix = vec!["mytool", "-b", ""];
        assert_eq!(currently_editing(&cmd, &appendix), Some("pos1".to_string()));
    }
}
