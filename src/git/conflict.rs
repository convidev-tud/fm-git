use crate::git::error::GitError;
use crate::git::interface::GitInterface;
use crate::model::QualifiedPath;
use colored::Colorize;
use std::fmt::Display;

#[derive(Debug)]
pub enum ConflictStatistic {
    OK((QualifiedPath, QualifiedPath)),
    CONFLICT((QualifiedPath, QualifiedPath)),
    ERROR((QualifiedPath, QualifiedPath), GitError),
}

impl From<ConflictStatistic> for String {
    fn from(value: ConflictStatistic) -> Self {
        match value {
            ConflictStatistic::OK((l, r)) => {
                format!("Merge {} and {} ", l, r) + "OK".green().to_string().as_str()
            }
            ConflictStatistic::CONFLICT((l, r)) => {
                format!("Merge {} and {} ", l, r) + "CONFLICT".red().to_string().as_str()
            }
            ConflictStatistic::ERROR((l, r), error) => {
                format!("Merge {} and {} ", l, r) + "ERROR".green().to_string().as_str()
            }
        }
    }
}

impl Display for ConflictStatistic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}

pub struct ConflictStatistics {
    ok: Vec<ConflictStatistic>,
    conflict: Vec<ConflictStatistic>,
    error: Vec<ConflictStatistic>,
}

impl ConflictStatistics {
    pub fn new() -> Self {
        Self {
            ok: vec![],
            conflict: vec![],
            error: vec![],
        }
    }
    pub fn from<T: Iterator<Item = ConflictStatistic>>(statistics: T) -> Self {
        let mut new = Self::new();
        for statistic in statistics {
            new.push(statistic);
        }
        new
    }
    pub fn push(&mut self, statistic: ConflictStatistic) {
        match statistic {
            ConflictStatistic::OK(_) => self.ok.push(statistic),
            ConflictStatistic::CONFLICT(_) => self.conflict.push(statistic),
            ConflictStatistic::ERROR(_, _) => self.error.push(statistic),
        }
    }
    pub fn iter_ok(&self) -> impl Iterator<Item = &ConflictStatistic> {
        self.ok.iter()
    }
    pub fn n_ok(&self) -> usize {
        self.ok.len()
    }
    pub fn n_conflict(&self) -> usize {
        self.conflict.len()
    }
    pub fn n_errors(&self) -> usize {
        self.error.len()
    }
}

impl FromIterator<ConflictStatistic> for ConflictStatistics {
    fn from_iter<T: IntoIterator<Item = ConflictStatistic>>(iter: T) -> Self {
        Self::from(iter.into_iter())
    }
}

pub struct ConflictChecker<'a> {
    interface: &'a GitInterface,
}

impl<'a> ConflictChecker<'a> {
    pub fn new(interface: &'a GitInterface) -> Self {
        Self { interface }
    }

    pub fn check(
        &self,
        paths: &Vec<QualifiedPath>,
    ) -> Result<impl Iterator<Item = ConflictStatistic>, GitError> {
        let current_area = self.interface.get_current_area()?;
        self.interface
            .checkout(&current_area.get_qualified_path())?;

        let mut feature_combinations: Vec<(&QualifiedPath, &QualifiedPath)> = Vec::new();
        for (i, path) in paths.iter().enumerate() {
            for part in paths[i + 1..].iter() {
                feature_combinations.push((path, part));
            }
        }

        let iterator = feature_combinations.into_iter().map(|(l, r)| {
            let statistic = self.check_two(l.clone(), r.clone());
            match statistic {
                Ok(stat) => match stat {
                    true => ConflictStatistic::OK((l.clone(), r.clone())),
                    false => ConflictStatistic::CONFLICT((l.clone(), r.clone())),
                },
                Err(e) => ConflictStatistic::ERROR((l.clone(), r.clone()), e),
            }
        });
        Ok(iterator)
    }

    fn check_two(&self, l: QualifiedPath, r: QualifiedPath) -> Result<bool, GitError> {
        let current_area = self.interface.get_current_area()?;
        let temporary = QualifiedPath::from("tmp");
        self.interface.create_branch_no_mut(&temporary)?;
        self.interface.checkout_raw(&temporary)?;
        let success = self.interface.merge(&vec![l, r])?.status.success();
        if !success {
            self.interface.abort_merge()?;
        }
        self.interface
            .checkout(&current_area.get_qualified_path())?;
        self.interface.delete_branch(&temporary)?;
        Ok(success)
    }
}
