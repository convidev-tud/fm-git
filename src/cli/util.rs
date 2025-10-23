use clap::ArgMatches;

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
/// Only works for arguments taking exactly one parameter.
///
/// Examples:
/// ```bash
/// mytool foo // foo is edited
/// mytool foo bar // foo is edited, if curser remains on bar
/// mytool foo bar abc // foo is not edited
/// ```
pub fn currently_editing(target: &str, appendix: &Vec<&str>) -> bool {
    let maybe_last = appendix.last();
    let last = if maybe_last.is_some() {
        maybe_last.unwrap()
    } else {
        ""
    };
    let maybe_pre_last = appendix.get(appendix.len() - 2);
    let pre_last = if maybe_pre_last.is_some() {
        maybe_pre_last.unwrap()
    } else {
        ""
    };
    "--name".starts_with(last) && last != "" || "--name".starts_with(pre_last) && pre_last != ""
}
