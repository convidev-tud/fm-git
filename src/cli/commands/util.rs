use clap::ArgMatches;


pub fn get_argument_value(args: &ArgMatches, id: &str) -> String {
    args
        .get_one::<String>(id)
        .unwrap()
        .clone()
}

pub fn get_argument_values(args: &ArgMatches, id: &str) -> Vec<String> {
    args
        .get_many::<String>(id)
        .unwrap()
        .map(|s| s.clone())
        .collect::<Vec<_>>()
}