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
