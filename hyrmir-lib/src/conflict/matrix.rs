use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use itertools::Itertools;
use crate::conflict::statistic::{MergeChainStatistic, MergeChainStatistics, MergeResult, MergeStatistic, MergeStatisticComparator, MergeStatisticWeight};

#[derive(Debug, Clone)]
pub enum CheckMode {
    Merge,
    CherryPick,
}

#[derive(Debug, Clone)]
pub struct ConflictChecker<'a> {
    git: &'a GitInterface,
    mode: CheckMode,
}

impl<'a> ConflictChecker<'a> {
    pub fn new(git: &'a GitInterface, mode: CheckMode) -> Self {
        Self { git, mode }
    }

    pub fn check_k_permutations<T: IsGitObject>(
        &self,
        paths: &Vec<NodePath<T>>,
        k: usize,
    ) -> impl Iterator<Item = Result<MergeChainStatistic<T, T>, PathAssertionError>> {
        let iterator = paths.iter().permutations(k).map(|perm| {
            let base = perm[0];
            self.check_chain(&base, &perm[1..].to_vec())
        });
        iterator
    }

    pub fn check_permutations_against_base<B: IsGitObject, T: IsGitObject>(
        &self,
        base: &NodePath<B>,
        keep: &Vec<NodePath<T>>,
        shuffle: &Vec<NodePath<T>>,
        k: usize,
    ) -> impl Iterator<Item = Result<MergeChainStatistic<B, T>, PathAssertionError>> {
        let iterator = shuffle.iter().permutations(k).map(|target| {
            let mut chain = vec![];
            chain.extend(keep.iter());
            chain.extend(target.into_iter());
            self.check_chain(base, &chain)
        });
        iterator
    }

    pub fn check_by_order<T: IsGitObject>(
        &self,
        paths: &Vec<NodePath<T>>,
    ) -> Result<MergeChainStatistic<T, T>, PathAssertionError> {
        let chain: Vec<&NodePath<T>> = paths.iter().collect();
        let base = chain[0];
        self.check_chain(base, &chain[1..].to_vec())
    }

    pub fn check_n_against_permutations<T: IsGitObject>(
        &self,
        n: &'a Vec<NodePath<T>>,
        against: &'a Vec<NodePath<T>>,
        k: &'a usize,
    ) -> impl Iterator<Item = Result<MergeChainStatistic<T, T>, PathAssertionError>> {
        // I don't know why, but k has to be borrowed here
        let iterator = n
            .iter()
            .map(|path| {
                against
                    .iter()
                    .combinations(*k)
                    .map(|mut combination| {
                        combination.push(path);
                        combination
                            .iter()
                            .permutations(*k + 1)
                            .map(|permutations| {
                                let dereferenced = permutations
                                    .iter()
                                    .map(|permutation| **permutation)
                                    .collect::<Vec<_>>();
                                self.check_chain(dereferenced[0], &dereferenced[1..].to_vec())
                            })
                            .collect::<Vec<_>>()
                    })
                    .flatten()
            })
            .flatten();
        iterator
    }

    pub fn clean_up(&mut self) {}

    fn check_chain<B: IsGitObject, C: IsGitObject>(
        &self,
        base: &NodePath<B>,
        chain: &Vec<&NodePath<C>>,
    ) -> Result<MergeChainStatistic<B, C>, PathAssertionError> {
        if chain.len() < 1 {
            panic!("Chain has to contain at least 1 path")
        }
        let mut chain_statistic = MergeChainStatistic::new(base.clone());
        let current_path = self.git.assert_current_node_path::<AnyGitObject>()?;
        self.git.checkout(base)?;
        let area = self.git.get_current_area()?;
        let temporary = area.to_normalized_path() + NormalizedPath::from("tmp");
        self.git.create_branch::<Temporary>(&temporary)?;
        self.git.checkout_raw(&temporary)?;
        let mut skip = false;
        for path in chain.to_vec().into_iter() {
            if skip {
                chain_statistic.push(MergeStatistic::new(path.clone(), MergeResult::Aborted));
            } else {
                let statistic = match self.mode {
                    CheckMode::Merge => {
                        let (statistic, _) = self.git.merge::<Temporary, C>(path.clone())?;
                        if statistic.contains_conflicts() {
                            self.git.abort_merge()?;
                            skip = true;
                        }
                        statistic
                    }
                    CheckMode::CherryPick => {
                        let (statistic, _) =
                            self.git.cherry_pick::<Temporary, C>(path.clone(), false)?;
                        if statistic.contains_conflicts() || statistic.contains_up_to_date() {
                            self.git.abort_cherry_pick()?;
                            skip = true;
                        }
                        statistic
                    }
                };
                chain_statistic.push(statistic.get(0).unwrap().clone());
            }
        }
        self.git.checkout(&current_path)?;
        self.git.delete_branch_no_mut(&temporary)?;
        Ok(chain_statistic)
    }
}

#[derive(Debug, Clone)]
pub struct Conflict2DMatrix {
    matrix: HashMap<
        NodePath<AnyGitObject>,
        HashMap<NodePath<AnyGitObject>, MergeStatistic<AnyGitObject>>,
    >,
}

impl Display for Conflict2DMatrix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for (k, v) in &self.matrix {
            result += format!("Key: {}\n", k.formatted_with_version(true)).as_str();
            for (_, value) in v {
                result += format!("  Value: {}\n", value).as_str();
            }
        }
        f.write_str(result.as_str())
    }
}

impl Conflict2DMatrix {
    pub fn new(statistics: &MergeChainStatistics<AnyGitObject, AnyGitObject>) -> Self {
        let mut matrix: HashMap<
            NodePath<AnyGitObject>,
            HashMap<NodePath<AnyGitObject>, MergeStatistic<AnyGitObject>>,
        > = HashMap::new();
        for chain in statistics.iter_all() {
            if chain.len() > 1 {
                panic!("Matrix only supports 2 dimensions")
            }
            let base = chain.get_base();
            let second = chain.get(0).unwrap();
            if !matrix.contains_key(base.get_path()) {
                matrix.insert(base.get_path().clone(), HashMap::new());
            }
            matrix
                .get_mut(base.get_path())
                .unwrap()
                .insert(second.get_path().clone(), second.clone());
        }
        Self { matrix }
    }

    pub fn predict_conflicts<B: IsGitObject, C: IsGitObject>(
        &self,
        base: &NodePath<B>,
        order: &Vec<NodePath<C>>,
    ) -> Option<MergeChainStatistic<B, C>> {
        let start = base.try_convert_to()?;
        let mut final_path: Vec<(
            NodePath<AnyGitObject>,
            MergeStatisticComparator<AnyGitObject>,
        )> = vec![];
        for path in order.iter() {
            let converted = path.try_convert_to()?;
            let votes = self.calculate_votes(&start, &final_path, &vec![converted.clone()]);
            let vote = votes.get(&converted).unwrap();
            final_path.push((converted, vote.clone()));
        }
        self.statistics_from_votes(base.clone(), &final_path)
    }

    pub fn estimate_best_path<B: IsGitObject, C: IsGitObject>(
        &self,
        base_path: &NodePath<B>,
    ) -> Option<MergeChainStatistic<B, C>> {
        let mut missing: Vec<NodePath<AnyGitObject>> = self.matrix.keys().cloned().collect();
        let start = base_path.try_convert_to()?;
        missing.retain(|k| k != &start);
        if missing.len() == 0 {
            panic!("Path estimation does only work if there are more keys than the base itself.")
        }
        let mut final_path: Vec<(
            NodePath<AnyGitObject>,
            MergeStatisticComparator<AnyGitObject>,
        )> = vec![];
        while missing.len() > 0 {
            let votes = Self::reverse_votes(self.calculate_votes(&start, &final_path, &missing));
            let max_vote = votes.keys().max().unwrap();
            let max_candidates = &votes[&max_vote];
            let winner = match max_candidates.len() {
                0 => {
                    panic!("Empty candidates should not be possible")
                }
                1 => max_candidates[0].clone(),
                _ => {
                    let start = max_candidates[0].clone();
                    let compatibility = self.calculate_forward_compatibility(&start, &missing);
                    let mut highest_compatible = (start, compatibility);
                    for candidate in max_candidates[1..].iter() {
                        let compatibility =
                            self.calculate_forward_compatibility(&candidate, &missing);
                        if compatibility > highest_compatible.1 {
                            highest_compatible = (candidate.clone(), compatibility);
                        }
                    }
                    highest_compatible.0
                }
            };
            let index: usize = missing
                .iter()
                .enumerate()
                .find_map(|(index, e)| if e == &winner { Some(index) } else { None })
                .unwrap();
            missing.remove(index);
            final_path.push((winner, max_vote.clone()));
        }
        self.statistics_from_votes(base_path.clone(), &final_path)
    }

    fn calculate_forward_compatibility(
        &self,
        element: &NodePath<AnyGitObject>,
        missing: &Vec<NodePath<AnyGitObject>>,
    ) -> MergeStatisticComparator<AnyGitObject> {
        let row = &self.matrix[element];
        let mut statistics = MergeStatisticComparator::new(MergeStatisticWeight::Simple);
        for statistic in row.iter().filter_map(|(k, v)| {
            if missing.contains(k) && k != element {
                Some(v.clone())
            } else {
                None
            }
        }) {
            statistics.push(statistic)
        }
        statistics
    }

    fn calculate_votes(
        &self,
        base: &NodePath<AnyGitObject>,
        voters: &Vec<(
            NodePath<AnyGitObject>,
            MergeStatisticComparator<AnyGitObject>,
        )>,
        targets: &Vec<NodePath<AnyGitObject>>,
    ) -> HashMap<NodePath<AnyGitObject>, MergeStatisticComparator<AnyGitObject>> {
        let mut votes: HashMap<NodePath<AnyGitObject>, MergeStatisticComparator<AnyGitObject>> =
            HashMap::new();
        for candidate in targets.iter() {
            let mut statistics = MergeStatisticComparator::new(MergeStatisticWeight::Simple);
            let by_base = self.matrix[base].get(candidate).unwrap();
            if voters.is_empty() {
                statistics.push(by_base.clone());
            } else {
                let mut include_base = true;
                for (voter, their_votes) in voters.iter() {
                    let opinion_of_base = self.matrix[base].get(voter).unwrap();
                    if opinion_of_base.get_stat() == &MergeResult::Success {
                        include_base = false;
                    }
                    if their_votes.get_lowest().get_stat() != &MergeResult::UpToDate {
                        let statistic = self.matrix[voter].get(candidate).unwrap();
                        statistics.push(statistic.clone());
                    }
                }
                if include_base {
                    statistics.push(by_base.clone());
                }
            }
            votes.insert(candidate.clone(), statistics);
        }
        votes
    }

    fn reverse_votes(
        votes: HashMap<NodePath<AnyGitObject>, MergeStatisticComparator<AnyGitObject>>,
    ) -> HashMap<MergeStatisticComparator<AnyGitObject>, Vec<NodePath<AnyGitObject>>> {
        let mut reversed: HashMap<
            MergeStatisticComparator<AnyGitObject>,
            Vec<NodePath<AnyGitObject>>,
        > = HashMap::new();
        for (path, vote) in votes.iter() {
            if reversed.contains_key(vote) {
                reversed.get_mut(vote).unwrap().push(path.clone());
            } else {
                reversed.insert(vote.clone(), vec![path.clone()]);
            }
        }
        reversed
    }

    fn statistics_from_votes<B: IsGitObject, C: IsGitObject>(
        &self,
        base: NodePath<B>,
        votes: &Vec<(
            NodePath<AnyGitObject>,
            MergeStatisticComparator<AnyGitObject>,
        )>,
    ) -> Option<MergeChainStatistic<B, C>> {
        let mut chain_statistic = MergeChainStatistic::new(base);
        for (_, vote) in votes.iter() {
            let lowest = vote.get_lowest();
            chain_statistic.push(MergeStatistic::new(
                lowest.get_path().try_convert_to()?,
                lowest.get_stat().clone(),
            ));
        }
        Some(chain_statistic)
    }
}

pub struct ConflictAnalyzer<'a> {
    checker: ConflictChecker<'a>,
    logger: &'a TanglLogger,
}

impl<'a> ConflictAnalyzer<'a> {
    pub fn new(checker: ConflictChecker<'a>, logger: &'a TanglLogger) -> Self {
        Self { checker, logger }
    }

    pub fn calculate_2d_heuristics_matrix_with_merge_base(
        &mut self,
        paths: &Vec<NodePath<AnyGitObject>>,
        base: &NodePath<AnyGitObject>,
    ) -> Result<Conflict2DMatrix, PathAssertionError> {
        let mut statistics = MergeChainStatistics::new();

        let mut successful_with_base: Vec<NodePath<AnyGitObject>> = vec![];
        let mut conflicting_with_base: Vec<NodePath<AnyGitObject>> = vec![];
        self.logger.debug("Checking against base pairwise");
        for s in self
            .checker
            .check_permutations_against_base(base, &vec![], paths, 1)
        {
            let result = s?;
            self.logger.debug(result.display_as_path());
            statistics.push(result.clone());
            let path = result.get(0).unwrap().get_path().clone();
            if result.contains_conflicts() {
                conflicting_with_base.push(path);
            } else {
                successful_with_base.push(path);
            }
        }

        self.logger.debug("Checking with successful against base");
        for successful in successful_with_base.iter() {
            for with_base in self.checker.check_permutations_against_base(
                base,
                &vec![successful.clone()],
                paths,
                1,
            ) {
                let result = with_base?;
                self.logger.debug(result.display_as_path());
                let second = result.get(0).unwrap();
                let third = result.get(1).unwrap();
                let mut new = MergeChainStatistic::new(second.get_path().clone());
                new.push(third.clone());
                statistics.push(new);
            }
        }

        self.logger.debug("Checking conflicting");
        for conflicting in conflicting_with_base.iter() {
            for with_base in
                self.checker
                    .check_permutations_against_base(conflicting, &vec![], paths, 1)
            {
                let result = with_base?;
                self.logger.debug(result.display_as_path());
                statistics.push(result);
            }
        }
        self.checker.clean_up();
        let matrix = Conflict2DMatrix::new(&statistics);
        Ok(matrix)
    }
}
