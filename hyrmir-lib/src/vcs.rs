pub trait VCS {
    fn halt_on_conflict(&self) -> bool { true }

    fn view(&self);

    fn iter_history(&self);

    fn commit(&self);

    fn merge(&self);

    fn simulate_merge(&self);

    fn apply_tag(&self);
}
