//! Reading the per-task results for both systems out of a results directory.
//!
//! `tsr task run` writes `<out>/results.jsonl` (see `tsr_batch::results`). The
//! baseline reads that file, splits it by system (`record` and `rules`), and
//! pairs the two systems by task id.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tsr_batch::results::{read_results, Outcome, TaskResult};

use crate::EvalError;

/// The two systems the PRD compares (B9: system 1 record, system 2 rules).
pub const SYSTEM_RECORD: &str = "record";
/// System 2: the Tier 0 rules controller.
pub const SYSTEM_RULES: &str = "rules";

/// The results for one system, keyed by task id.
#[derive(Debug, Clone, Default)]
pub struct SystemResults {
    /// The system name (`record` or `rules`).
    pub system: String,
    /// Task id -> result, in task-id order (BTreeMap keeps the report stable).
    pub by_task: BTreeMap<String, TaskResult>,
}

impl SystemResults {
    /// The number of tasks this system attempted.
    pub fn len(&self) -> usize {
        self.by_task.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_task.is_empty()
    }
}

/// Both systems' results, read from one results directory.
#[derive(Debug, Clone)]
pub struct ResultsDir {
    /// The directory the results were read from.
    pub dir: PathBuf,
    /// System 1 (record).
    pub record: SystemResults,
    /// System 2 (rules).
    pub rules: SystemResults,
}

impl ResultsDir {
    /// Read `<dir>/results.jsonl` and split it into the two systems.
    ///
    /// A result whose `system` is neither `record` nor `rules` is an error: the
    /// baseline compares exactly those two systems (PRD B9).
    pub fn read(dir: &Path) -> Result<Self, EvalError> {
        let path = dir.join("results.jsonl");
        if !path.exists() {
            return Err(EvalError::MissingResults(path));
        }
        let rows = read_results(&path).map_err(|e| EvalError::Io {
            path: path.clone(),
            detail: e.to_string(),
        })?;

        let mut record = SystemResults {
            system: SYSTEM_RECORD.to_string(),
            by_task: BTreeMap::new(),
        };
        let mut rules = SystemResults {
            system: SYSTEM_RULES.to_string(),
            by_task: BTreeMap::new(),
        };

        for row in rows {
            let slot = match row.system.as_str() {
                SYSTEM_RECORD => &mut record,
                SYSTEM_RULES => &mut rules,
                other => {
                    return Err(EvalError::UnknownSystem {
                        system: other.to_string(),
                        path: path.clone(),
                    })
                }
            };
            if slot.by_task.insert(row.task.clone(), row).is_some() {
                return Err(EvalError::DuplicateTask {
                    system: slot.system.clone(),
                    task: slot.by_task.keys().next_back().cloned().unwrap_or_default(),
                    path: path.clone(),
                });
            }
        }

        Ok(Self {
            dir: dir.to_path_buf(),
            record,
            rules,
        })
    }

    /// The task ids present in both systems, in sorted order. Pairing by task
    /// (the default) uses exactly this set.
    pub fn paired_task_ids(&self) -> Vec<String> {
        self.record
            .by_task
            .keys()
            .filter(|t| self.rules.by_task.contains_key(*t))
            .cloned()
            .collect()
    }

    /// Task ids present in only one system, as `(task, systems)` pairs, sorted.
    pub fn unpaired_task_ids(&self) -> Vec<(String, Vec<&'static str>)> {
        let mut out = Vec::new();
        for t in self.record.by_task.keys() {
            if !self.rules.by_task.contains_key(t) {
                out.push((t.clone(), vec![SYSTEM_RECORD]));
            }
        }
        for t in self.rules.by_task.keys() {
            if !self.record.by_task.contains_key(t) {
                out.push((t.clone(), vec![SYSTEM_RULES]));
            }
        }
        out.sort();
        out
    }
}

/// Whether a result counts as a completion: the oracle accepted it.
///
/// The oracle is "the flipped tests pass and the full suite does not regress"
/// (PRD B9). `Outcome::Accepted` is exactly that verdict; a result whose
/// outcome is `accepted` but whose oracle verdict is absent or negative is
/// inconsistent and is treated as not accepted.
pub fn is_accepted(r: &TaskResult) -> bool {
    if r.outcome != Outcome::Accepted {
        return false;
    }
    match &r.oracle {
        Some(v) => v.accepted,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_batch::results::{write_results, OracleVerdict};

    fn row(task: &str, system: &str, accepted: bool) -> TaskResult {
        TaskResult {
            task: task.to_string(),
            system: system.to_string(),
            outcome: if accepted {
                Outcome::Accepted
            } else {
                Outcome::Rejected
            },
            generator_tokens: 100,
            tool_minutes: 1,
            wall_clock_ms: 1000,
            family: "f".into(),
            lineage: "repo-a".into(),
            state_root_at_start: "aa".into(),
            state_root_at_end: "bb".into(),
            denials: vec![],
            oracle: Some(OracleVerdict {
                flipped_tests_pass: accepted,
                suite_regressed: false,
                accepted,
                exit_code: 0,
            }),
        }
    }

    #[test]
    fn pairs_by_task_and_reports_unpaired() {
        let dir = tempfile::tempdir().unwrap();
        let rows = vec![
            row("t1", "record", true),
            row("t1", "rules", true),
            row("t2", "record", false),
            row("t3", "rules", true),
        ];
        write_results(&dir.path().join("results.jsonl"), &rows).unwrap();
        let rd = ResultsDir::read(dir.path()).unwrap();
        assert_eq!(rd.paired_task_ids(), vec!["t1".to_string()]);
        assert_eq!(
            rd.unpaired_task_ids(),
            vec![
                ("t2".to_string(), vec!["record"]),
                ("t3".to_string(), vec!["rules"]),
            ]
        );
    }

    #[test]
    fn accepted_requires_the_oracle_verdict() {
        let mut r = row("t1", "record", true);
        assert!(is_accepted(&r));
        r.oracle = None;
        assert!(!is_accepted(&r));
    }
}
