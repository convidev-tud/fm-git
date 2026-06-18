// use std::cmp::Ordering;
// use crate::model::{ErrorState, IsConcrete, NodePath, NormalizedPath, ToNormalizedPaths, VirtualRoot};
// use crate::vcs::VCS;
// use colored::Colorize;
// use itertools::Itertools;
// use serde::{Deserialize, Serialize};
// use std::fmt::{Display, Formatter};
// use std::iter::once;
// 
// #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub enum MergeResult {
//     Base,
//     Success,
//     UpToDate,
//     Conflict,
//     Merging,
//     Aborted,
//     Error(String),
// }
// 
// impl Display for MergeResult {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let value = match self {
//             Self::Base => "".normal(),
//             Self::Success => "(Ok)".green(),
//             Self::UpToDate => "(Up To Date)".green(),
//             Self::Conflict => "(Conflict)".red(),
//             Self::Merging => "(Merging)".yellow(),
//             Self::Aborted => "(Aborted)".red(),
//             Self::Error(reason) => format!("(Error: {reason})").red(),
//         };
//         f.write_str(value.to_string().as_str())
//     }
// }
// 
// #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
// pub struct NormalizedMergeStatistic {
//     path: NormalizedPath,
//     stat: MergeResult,
// }
// 
// impl NormalizedMergeStatistic {
//     pub fn new(path: NormalizedPath, stat: MergeResult) -> Self {
//         Self { path, stat }
//     }
//     pub fn get_path(&self) -> &NormalizedPath {
//         &self.path
//     }
//     pub fn get_stat(&self) -> &MergeResult {
//         &self.stat
//     }
// }
// 
// impl ToNormalizedPaths for Vec<NormalizedMergeStatistic> {
//     fn to_normalized_paths(&self) -> Vec<NormalizedPath> {
//         self.iter().map(|s| s.get_path().clone()).collect()
//     }
// }
// 
// #[derive(Debug, PartialEq, Eq, Clone, Hash)]
// pub struct MergeStatistic<T: IsConcrete, V: VCS> {
//     path: NodePath<T, V>,
//     stat: MergeResult,
// }
// 
// impl<T: IsConcrete, V: VCS> MergeStatistic<T, V> {
//     pub fn new(path: NodePath<T, V>, stat: MergeResult) -> Self {
//         Self { path, stat }
//     }
// 
//     pub fn from_normalized(
//         stat: NormalizedMergeStatistic,
//         root: NodePath<VirtualRoot, V>,
//     ) -> Result<Self, NodePath<ErrorState, V>> {
//         let path = root.move_to(stat.get_path())?;
//         Ok(Self::new(path, stat.get_stat().clone()))
//     }
//     
//     pub fn to_normalized(&self) -> NormalizedMergeStatistic {
//         NormalizedMergeStatistic::new(
//             self.path.to_normalized_path_with_version(),
//             self.stat.clone(),
//         )
//     }
//     pub fn get_path(&self) -> &NodePath<T, V> {
//         &self.path
//     }
//     
//     pub fn get_stat(&self) -> &MergeResult {
//         &self.stat
//     }
//     
//     pub fn formatted(&self, colored: bool) -> String {
//         todo!()
//     }
// }
// 
// impl<T: IsConcrete, V: VCS> Display for MergeStatistic<T, V> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         let stat = self.get_stat().to_string();
//         if !stat.is_empty() {
//             f.write_str(
//                 format!("{} {stat}", self.get_path().formatted_with_version(true)).as_str(),
//             )
//         } else {
//             f.write_str(self.get_path().formatted_with_version(true).as_str())
//         }
//     }
// }
// 
// pub enum MergeChainPart<T: IsConcrete, V: VCS> {
//     Normal(MergeStatistic<T, V>),
//     Error(NodePath<ErrorState, V>),
// }
// 
// impl<T: IsConcrete, V: VCS> MergeChainPart<T, V> {
//     pub fn format(&self, colored: bool) -> String {
//         match self {
//             Self::Normal(stat) => stat.formatted(colored),
//             Self::Error(path) => path.formatted(colored),
//         }
//     }
// }
// 
// #[derive(Debug, PartialEq, Eq, Clone)]
// pub struct MergeChainStatistic<B, C, V>
// where
//     B: IsConcrete,
//     C: IsConcrete,
//     V: VCS,
// {
//     base: MergeStatistic<B, V>,
//     chain: Vec<MergeChainPart<C, V>>,
// }
// 
// impl<B, C, V> MergeChainStatistic<B, C, V>
// where
//     B: IsConcrete,
//     C: IsConcrete,
//     V: VCS,
// {
//     pub fn new(base: NodePath<B, V>) -> Self {
//         Self {
//             base: MergeStatistic::new(base, MergeResult::Base),
//             chain: vec![],
//         }
//     }
// 
//     pub fn from_normalized(
//         base: NodePath<B, V>,
//         stats: Vec<NormalizedMergeStatistic>,
//         root: NodePath<VirtualRoot, V>,
//     ) -> Result<Self, Self> {
//         let mut chain: Vec<MergeChainPart<C, V>> = vec![];
//         let mut error = false;
//         for stat in stats {
//             match MergeStatistic::from_normalized(stat, root.clone()) {
//                 Ok(s) => chain.push(MergeChainPart::Normal(s)),
//                 Err(s) => {
//                     chain.push(MergeChainPart::Error(s));
//                     error = true;
//                 }
//             }
//         }
//         let new = Self {
//             base: MergeStatistic::new(base, MergeResult::Base),
//             chain,
//         };
//         if error {
//             Err(new)
//         } else {
//             Ok(new)
//         }
//     }
// 
//     pub fn push(&mut self, stat: MergeStatistic<C, V>) {
//         self.chain.push(MergeChainPart::Normal(stat));
//     }
// 
//     pub fn fill(&mut self, stats: Vec<MergeStatistic<C, V>>) {
//         for stat in stats {
//             self.chain.push(MergeChainPart::Normal(stat));
//         }
//     }
// 
//     pub fn to_normalized(&self) -> Vec<NormalizedMergeStatistic> {
//         self.iter_chain().map(|s| s.to_normalized()).collect()
//     }
// 
//     pub fn insert(&mut self, index: usize, stat: MergeStatistic<C, V>) {
//         self.chain.insert(index, MergeChainPart::Normal(stat));
//     }
// 
//     pub fn remove(&mut self, index: usize) -> Result<MergeStatistic<C, V>, NodePath<ErrorState, V>> {
//         match self.chain.remove(index) {
//             MergeChainPart::Normal(s) => Ok(s),
//             MergeChainPart::Error(s) => Err(s),
//         }
//     }
// 
//     pub fn get(&self, index: usize) -> Option<Result<&MergeStatistic<C, V>, &NodePath<ErrorState, V>>> {
//         match self.chain.get(index) {
//             Some(s) => match s {
//                 MergeChainPart::Normal(s) => Some(Ok(s)),
//                 MergeChainPart::Error(s) => Some(Err(s)),
//             },
//             None => None,
//         }
//     }
// 
//     pub fn get_base(&self) -> &MergeStatistic<B, V> {
//         &self.base
//     }
// 
//     pub fn replace(&mut self, index: usize, stat: MergeStatistic<C, V>) {
//         self.remove(index);
//         self.insert(index, stat);
//     }
// 
//     pub fn get_chain(&self) -> &Vec<MergeStatistic<C, V>> {
//         &self.chain
//     }
// 
//     pub fn iter_chain(&self) -> impl Iterator<Item = &MergeStatistic<C, V>> {
//         self.chain.iter()
//     }
// 
//     pub fn get_n_success(&self) -> usize {
//         let success: Vec<&MergeStatistic<C, V>> = self
//             .iter_chain()
//             .filter(|s| s.get_stat() == &MergeResult::Success)
//             .collect();
//         success.len()
//     }
// 
//     pub fn get_n_conflict(&self) -> usize {
//         let all: Vec<&MergeStatistic<C, V>> = self
//             .iter_chain()
//             .filter(|s| s.get_stat() == &MergeResult::Conflict)
//             .collect();
//         all.len()
//     }
// 
//     pub fn get_n_merges(&self) -> usize {
//         let all: Vec<&MergeStatistic<C, V>> = self
//             .iter_chain()
//             .filter(|s| match s.get_stat() {
//                 MergeResult::Success | MergeResult::Conflict | MergeResult::Merging => true,
//                 _ => false,
//             })
//             .collect();
//         all.len()
//     }
// 
//     pub fn get_n_up_to_date(&self) -> usize {
//         let all: Vec<&MergeStatistic<C, V>> = self
//             .iter_chain()
//             .filter(|s| s.get_stat() == &MergeResult::UpToDate)
//             .collect();
//         all.len()
//     }
// 
//     pub fn get_n_errors(&self) -> usize {
//         let all: Vec<&MergeStatistic<C, V>> = self
//             .iter_chain()
//             .filter(|s| match s.get_stat() {
//                 MergeResult::Error(_) => true,
//                 _ => false,
//             })
//             .collect();
//         all.len()
//     }
// 
//     pub fn all_up_to_date(&self) -> bool {
//         if self.chain.is_empty() {
//             true
//         } else {
//             self.get_n_up_to_date() == self.chain.len()
//         }
//     }
// 
//     pub fn len(&self) -> usize {
//         self.chain.len()
//     }
// 
//     pub fn is_empty(&self) -> bool {
//         self.chain.is_empty()
//     }
// 
//     pub fn contains_conflicts(&self) -> bool {
//         self.get_n_conflict() > 0
//     }
// 
//     pub fn contains_up_to_date(&self) -> bool {
//         self.get_n_up_to_date() > 0
//     }
// 
//     pub fn contains_errors(&self) -> bool {
//         self.get_n_errors() > 0
//     }
// 
//     pub fn display_as_path(&self, colored: bool) -> String {
//         vec![&self.base]
//             .iter()
//             .map(|m| m.formatted(colored))
//             .chain(self.chain.iter().map(|stat| stat.format(colored)))
//             .join(" <- ")
//     }
// 
//     pub fn display_as_list(&self, colored: bool) -> impl Iterator<Item = String> {
//         once(&self.base)
//             .map(|m| m.formatted(colored))
//             .chain(self.chain.iter().map(|stat| format!(" <- {}", stat.format(colored))))
//     }
// }
// 
// pub struct MergeChainStatistics<B: IsConcrete, T: IsConcrete, V: VCS> {
//     statistics: Vec<MergeChainStatistic<B, T, V>>,
//     total_successes: usize,
//     total_conflicts: usize,
//     total_errors: usize,
// }
// 
// impl<B: IsConcrete, T: IsConcrete, V: VCS> MergeChainStatistics<B, T, V> {
//     pub fn new() -> Self {
//         Self {
//             statistics: vec![],
//             total_successes: 0,
//             total_conflicts: 0,
//             total_errors: 0,
//         }
//     }
//     pub fn fill_from_iter<I: Iterator<Item = MergeChainStatistic<B, T, V>>>(&mut self, statistics: I) {
//         for statistic in statistics {
//             self.push(statistic);
//         }
//     }
//     pub fn push(&mut self, statistic: MergeChainStatistic<B, T, V>) {
//         self.total_successes += statistic.get_n_success();
//         self.total_conflicts += statistic.get_n_conflict();
//         self.total_errors += statistic.get_n_errors();
//         self.statistics.push(statistic);
//     }
//     pub fn iter_all(&self) -> impl Iterator<Item = &MergeChainStatistic<B, T, V>> {
//         self.statistics.iter()
//     }
//     pub fn iter_conflicts(&self) -> impl Iterator<Item = &MergeChainStatistic<B, T, V>> {
//         self.statistics.iter().filter(|s| s.contains_conflicts())
//     }
//     pub fn iter_errors(&self) -> impl Iterator<Item = &MergeChainStatistic<B, T, V>> {
//         self.statistics.iter().filter(|s| s.contains_errors())
//     }
//     pub fn n_ok(&self) -> usize {
//         self.total_successes
//     }
//     pub fn n_conflicts(&self) -> usize {
//         self.total_conflicts
//     }
//     pub fn n_errors(&self) -> usize {
//         self.total_errors
//     }
// }
// 
// impl<B: IsConcrete, T: IsConcrete, V: VCS> FromIterator<MergeChainStatistic<B, T, V>>
//     for MergeChainStatistics<B, T, V>
// {
//     fn from_iter<I: IntoIterator<Item = MergeChainStatistic<B, T, V>>>(iter: I) -> Self {
//         let mut new = Self::new();
//         new.fill_from_iter(iter.into_iter());
//         new
//     }
// }
// 
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub enum MergeStatisticWeight {
//     Simple,
// }
// 
// impl MergeStatisticWeight {
//     pub fn get_weight(&self, statistic: &MergeResult) -> i32 {
//         match self {
//             Self::Simple => match statistic {
//                 MergeResult::Base => 0,
//                 MergeResult::UpToDate => 1,
//                 MergeResult::Success => 0,
//                 MergeResult::Conflict => -1,
//                 MergeResult::Merging => 0,
//                 MergeResult::Aborted => -10,
//                 MergeResult::Error(_) => -20,
//             },
//         }
//     }
// }
// 
// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct MergeStatisticComparator<T: IsConcrete, V: VCS> {
//     statistics: Vec<MergeStatistic<T, V>>,
//     weights: MergeStatisticWeight,
// }
// 
// impl<T: IsConcrete, V: VCS> PartialOrd for MergeStatisticComparator<T, V> {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         let my_weights = self.accumulate_weights();
//         let their_weights = other.accumulate_weights();
//         Some(my_weights.cmp(&their_weights))
//     }
// }
// 
// impl<T: IsConcrete, V: VCS> Ord for MergeStatisticComparator<T, V> {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.partial_cmp(other).unwrap()
//     }
// }
// 
// impl<T: IsConcrete, V: VCS> MergeStatisticComparator<T, V> {
//     pub fn new(weights: MergeStatisticWeight) -> Self {
//         Self {
//             statistics: vec![],
//             weights,
//         }
//     }
//     
//     pub fn push(&mut self, statistic: MergeStatistic<T, V>) {
//         self.statistics.push(statistic);
//     }
//     
//     pub fn accumulate_weights(&self) -> i32 {
//         let mut sum = 0;
//         for s in &self.statistics {
//             sum += self.weights.get_weight(s.get_stat())
//         }
//         sum
//     }
//     
//     pub fn get_lowest(&self) -> &MergeStatistic<T, V> {
//         self.statistics
//             .iter()
//             .min_by(|a, b| {
//                 self.weights
//                     .get_weight(a.get_stat())
//                     .cmp(&self.weights.get_weight(b.get_stat()))
//             })
//             .unwrap()
//     }
// }