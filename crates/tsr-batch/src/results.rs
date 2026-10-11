//! The per-task result record (PRD §B9, §B11 M3).
//!
//! One JSON object per task per system, written to `<out>/results.jsonl`. The fields are the ones
//! the PRD names: task, system, outcome, generator tokens, tool minutes, wall clock, family,
//! lineage, and the state root at start. `tsr eval baseline` reads this file.

use serde::{Deserialize, Serialize};

/// How a task ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// The oracle passed: the flipped tests pass and the suite did not regress.
    Accepted,
    /// The oracle failed: the task was attempted but did not reach the oracle.
    Rejected,
    /// The driver could not run (no Claude Code, no container runtime).
    Unavailable,
    /// The task was not attempted (a budget stop before any action).
    NotAttempted,
}

impl Outcome {
    /// The lowercase word the results carry.
    pub fn as_str(self) -> &'static str {
        match self {
            Outcome::Accepted => "accepted",
            Outcome::Rejected => "rejected",
            Outcome::Unavailable => "unavailable",
            Outcome::NotAttempted => "not_attempted",
        }
    }
}

/// One task's result under one system.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskResult {
    /// The corpus task id.
    pub task: String,
    /// The system: `record` or `rules`.
    pub system: String,
    /// How the task ended.
    pub outcome: Outcome,
    /// Generator tokens spent.
    pub generator_tokens: u64,
    /// Tool minutes spent.
    pub tool_minutes: u32,
    /// Wall-clock milliseconds spent.
    pub wall_clock_ms: u64,
    /// The task's family.
    pub family: String,
    /// The task's lineage (the repository).
    pub lineage: String,
    /// The store's head root when the session opened (B9, state isolation).
    pub state_root_at_start: String,
    /// The store's head root when the session closed.
    pub state_root_at_end: String,
    /// Gated actions the gate denied, with the gate's reason.
    pub denials: Vec<super::driver::Denial>,
    /// The oracle's verdict, when it ran.
    pub oracle: Option<OracleVerdict>,
}

/// The oracle's verdict for one task.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OracleVerdict {
    /// Whether the flipped tests pass.
    pub flipped_tests_pass: bool,
    /// Whether the full suite regressed.
    pub suite_regressed: bool,
    /// The oracle's overall verdict: `flipped_tests_pass && !suite_regressed`.
    pub accepted: bool,
    /// The exit code the oracle run reported.
    pub exit_code: i32,
}

/// Write results as JSONL, one per line, in order.
pub fn write_results(path: &std::path::Path, results: &[TaskResult]) -> std::io::Result<()> {
    let mut buf = String::new();
    for r in results {
        buf.push_str(&serde_json::to_string(r).expect("serializable"));
        buf.push('\n');
    }
    std::fs::write(path, buf)
}

/// Read results from a JSONL file.
pub fn read_results(path: &std::path::Path) -> std::io::Result<Vec<TaskResult>> {
    let text = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<TaskResult>(line) {
            Ok(r) => out.push(r),
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("bad result line: {e}"),
                ))
            }
        }
    }
    Ok(out)
}
