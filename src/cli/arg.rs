use clap::ArgMatches;

#[derive(Debug)]
pub struct ArgHelper<'a> {
    args: &'a ArgMatches,
}
impl<'a> ArgHelper<'a> {
    pub fn new(matches: &'a ArgMatches) -> Self {
        Self { args: matches }
    }
    pub fn get_matches(&self) -> &ArgMatches {
        &self.args
    }
    pub fn get_argument_value<T: Clone + Send + Sync + 'static>(&self, id: &str) -> T {
        self.args.get_one::<T>(id).unwrap().clone()
    }
    pub fn get_argument_values<T: Clone + Send + Sync + 'static>(&self, id: &str) -> Vec<T> {
        self.args
            .get_many::<T>(id)
            .unwrap()
            .map(|s| s.clone())
            .collect::<Vec<_>>()
    }
}
