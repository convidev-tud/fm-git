use std::fmt::{Display, Formatter};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use crate::model::{IsConcrete, NodePath, NormalizedPath, ToNormalizedPaths};
use crate::vcs::VCS;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MergeResult {
    Base,
    Success,
    UpToDate,
    Conflict,
    Merging,
    Aborted,
    Error(String),
}

impl Display for MergeResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Base => "".normal(),
            Self::Success => "(Ok)".green(),
            Self::UpToDate => "(Up To Date)".green(),
            Self::Conflict => "(Conflict)".red(),
            Self::Merging => "(Merging)".yellow(),
            Self::Aborted => "(Aborted)".red(),
            Self::Error(reason) => format!("(Error: {reason})").red(),
        };
        f.write_str(value.to_string().as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NormalizedMergeStatistic {
    path: NormalizedPath,
    stat: MergeResult,
}

impl NormalizedMergeStatistic {
    pub fn new(path: NormalizedPath, stat: MergeResult) -> Self {
        Self { path, stat }
    }
    pub fn get_path(&self) -> &NormalizedPath {
        &self.path
    }
    pub fn get_stat(&self) -> &MergeResult {
        &self.stat
    }
}

impl ToNormalizedPaths for Vec<NormalizedMergeStatistic> {
    fn to_normalized_paths(&self) -> Vec<NormalizedPath> {
        self.iter().map(|s| s.get_path().clone()).collect()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct MergeStatistic<T: IsConcrete, V: VCS> {
    path: NodePath<T, V>,
    stat: MergeResult,
}

impl<T: IsConcrete, V: VCS> MergeStatistic<T, V> {
    pub fn new(path: NodePath<T, V>, stat: MergeResult) -> Self {
        Self { path, stat }
    }
    pub fn from_normalized(
        stat: NormalizedMergeStatistic,
        git: &GitInterface,
    ) -> Result<Self, PathAssertionError> {
        let path = git.assert_path::<T>(stat.get_path())?;
        Ok(Self::new(path, stat.get_stat().clone()))
    }
    pub fn to_normalized(&self) -> NormalizedMergeStatistic {
        NormalizedMergeStatistic::new(
            self.path.to_normalized_path_with_version(),
            self.stat.clone(),
        )
    }
    pub fn get_path(&self) -> &NodePath<T> {
        &self.path
    }
    pub fn get_stat(&self) -> &MergeResult {
        &self.stat
    }
}

impl<T: IsConcrete, V: VCS> Display for MergeStatistic<T, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let stat = self.get_stat().to_string();
        if !stat.is_empty() {
            f.write_str(
                format!("{} {stat}", self.get_path().formatted_with_version(true)).as_str(),
            )
        } else {
            f.write_str(self.get_path().formatted_with_version(true).as_str())
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MergeChainStatistic<B, C, V>
where
    B: IsConcrete,
    C: IsConcrete,
    V: VCS,
{
    base: MergeStatistic<B, V>,
    chain: Vec<MergeStatistic<C, V>>,
}

impl<B: IsConcrete, C: IsConcrete> MergeChainStatistic<B, C> {
    pub fn new(base: NodePath<B>) -> Self {
        Self {
            base: MergeStatistic::new(base, MergeResult::Base),
            chain: vec![],
        }
    }
    pub fn push(&mut self, stat: MergeStatistic<C>) {
        self.chain.push(stat);
    }
    pub fn fill(&mut self, stats: Vec<MergeStatistic<C>>) {
        for stat in stats {
            self.chain.push(stat)
        }
    }
    pub fn fill_from_normalized(
        &mut self,
        stats: Vec<NormalizedMergeStatistic>,
        git: &GitInterface,
    ) -> Result<(), PathAssertionError> {
        for stat in stats {
            self.push(MergeStatistic::from_normalized(stat, git)?)
        }
        Ok(())
    }
    pub fn to_normalized(&self) -> Vec<NormalizedMergeStatistic> {
        self.iter_chain().map(|s| s.to_normalized()).collect()
    }
    pub fn insert(&mut self, index: usize, stat: MergeStatistic<C>) {
        self.chain.insert(index, stat);
    }
    pub fn remove(&mut self, index: usize) -> MergeStatistic<C> {
        let statistic = self.chain.remove(index);
        statistic
    }
    pub fn get(&self, index: usize) -> Option<&MergeStatistic<C>> {
        self.chain.get(index)
    }
    pub fn get_base(&self) -> &MergeStatistic<B> {
        &self.base
    }
    pub fn replace(&mut self, index: usize, stat: MergeStatistic<C>) {
        self.remove(index);
        self.insert(index, stat);
    }
    pub fn get_chain(&self) -> &Vec<MergeStatistic<C>> {
        &self.chain
    }
    pub fn iter_chain(&self) -> impl Iterator<Item = &MergeStatistic<C>> {
        self.chain.iter()
    }
    pub fn get_n_success(&self) -> usize {
        let success: Vec<&MergeStatistic<C>> = self
            .iter_chain()
            .filter(|s| s.get_stat() == &MergeResult::Success)
            .collect();
        success.len()
    }
    pub fn get_n_conflict(&self) -> usize {
        let all: Vec<&MergeStatistic<C>> = self
            .iter_chain()
            .filter(|s| s.get_stat() == &MergeResult::Conflict)
            .collect();
        all.len()
    }
    pub fn get_n_merges(&self) -> usize {
        let all: Vec<&MergeStatistic<C>> = self
            .iter_chain()
            .filter(|s| match s.get_stat() {
                MergeResult::Success | MergeResult::Conflict | MergeResult::Merging => true,
                _ => false,
            })
            .collect();
        all.len()
    }
    pub fn get_n_up_to_date(&self) -> usize {
        let all: Vec<&MergeStatistic<C>> = self
            .iter_chain()
            .filter(|s| s.get_stat() == &MergeResult::UpToDate)
            .collect();
        all.len()
    }
    pub fn get_n_errors(&self) -> usize {
        let all: Vec<&MergeStatistic<C>> = self
            .iter_chain()
            .filter(|s| match s.get_stat() {
                MergeResult::Error(_) => true,
                _ => false,
            })
            .collect();
        all.len()
    }
    pub fn all_up_to_date(&self) -> bool {
        if self.chain.is_empty() {
            true
        } else {
            self.get_n_up_to_date() == self.chain.len()
        }
    }
    pub fn len(&self) -> usize {
        self.chain.len()
    }
    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }
    pub fn contains_conflicts(&self) -> bool {
        self.get_n_conflict() > 0
    }
    pub fn contains_up_to_date(&self) -> bool {
        self.get_n_up_to_date() > 0
    }
    pub fn contains_errors(&self) -> bool {
        self.get_n_errors() > 0
    }
    pub fn display_as_path(&self) -> String {
        vec![&self.base]
            .iter()
            .map(|m| m.to_string())
            .chain(self.chain.iter().map(|stat| stat.to_string()))
            .join(" <- ")
    }
    pub fn display_as_list(&self) -> impl Iterator<Item = String> {
        once(&self.base)
            .map(|m| m.to_string())
            .chain(self.chain.iter().map(|stat| format!(" <- {}", stat)))
    }
}

pub struct MergeChainStatistics<B: IsGitObject, T: IsGitObject> {
    statistics: Vec<MergeChainStatistic<B, T>>,
    total_successes: usize,
    total_conflicts: usize,
    total_errors: usize,
}

impl<B: IsGitObject, T: IsGitObject> MergeChainStatistics<B, T> {
    pub fn new() -> Self {
        Self {
            statistics: vec![],
            total_successes: 0,
            total_conflicts: 0,
            total_errors: 0,
        }
    }
    pub fn fill_from_iter<I: Iterator<Item = MergeChainStatistic<B, T>>>(&mut self, statistics: I) {
        for statistic in statistics {
            self.push(statistic);
        }
    }
    pub fn push(&mut self, statistic: MergeChainStatistic<B, T>) {
        self.total_successes += statistic.get_n_success();
        self.total_conflicts += statistic.get_n_conflict();
        self.total_errors += statistic.get_n_errors();
        self.statistics.push(statistic);
    }
    pub fn iter_all(&self) -> impl Iterator<Item = &MergeChainStatistic<B, T>> {
        self.statistics.iter()
    }
    pub fn iter_conflicts(&self) -> impl Iterator<Item = &MergeChainStatistic<B, T>> {
        self.statistics.iter().filter(|s| s.contains_conflicts())
    }
    pub fn iter_errors(&self) -> impl Iterator<Item = &MergeChainStatistic<B, T>> {
        self.statistics.iter().filter(|s| s.contains_errors())
    }
    pub fn n_ok(&self) -> usize {
        self.total_successes
    }
    pub fn n_conflicts(&self) -> usize {
        self.total_conflicts
    }
    pub fn n_errors(&self) -> usize {
        self.total_errors
    }
}

impl<B: IsGitObject, T: IsGitObject> FromIterator<MergeChainStatistic<B, T>>
    for MergeChainStatistics<B, T>
{
    fn from_iter<I: IntoIterator<Item = MergeChainStatistic<B, T>>>(iter: I) -> Self {
        let mut new = Self::new();
        new.fill_from_iter(iter.into_iter());
        new
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MergeStatisticWeight {
    Simple,
}

impl MergeStatisticWeight {
    pub fn get_weight(&self, statistic: &MergeResult) -> i32 {
        match self {
            Self::Simple => match statistic {
                MergeResult::Base => 0,
                MergeResult::UpToDate => 1,
                MergeResult::Success => 0,
                MergeResult::Conflict => -1,
                MergeResult::Merging => 0,
                MergeResult::Aborted => -10,
                MergeResult::Error(_) => -20,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MergeStatisticComparator<T: IsGitObject> {
    statistics: Vec<MergeStatistic<T>>,
    weights: MergeStatisticWeight,
}

impl<T: IsGitObject> PartialOrd for MergeStatisticComparator<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let my_weights = self.accumulate_weights();
        let their_weights = other.accumulate_weights();
        Some(my_weights.cmp(&their_weights))
    }
}

impl<T: IsGitObject> Ord for MergeStatisticComparator<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl<T: IsGitObject> MergeStatisticComparator<T> {
    pub fn new(weights: MergeStatisticWeight) -> Self {
        Self {
            statistics: vec![],
            weights,
        }
    }
    pub fn push(&mut self, statistic: MergeStatistic<T>) {
        self.statistics.push(statistic);
    }
    pub fn accumulate_weights(&self) -> i32 {
        let mut sum = 0;
        for s in &self.statistics {
            sum += self.weights.get_weight(s.get_stat())
        }
        sum
    }
    pub fn get_lowest(&self) -> &MergeStatistic<T> {
        self.statistics
            .iter()
            .min_by(|a, b| {
                self.weights
                    .get_weight(a.get_stat())
                    .cmp(&self.weights.get_weight(b.get_stat()))
            })
            .unwrap()
    }
}