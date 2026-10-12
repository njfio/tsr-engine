//! The corpus task as the batch runner reads it (PRD §B9).
//!
//! `tsr corpus mine` and `tsr corpus mutants` write `tasks.jsonl`; this module reads it back. A
//! task is either a *mined* task (a red→green flip in the repository's history) or a *mutant* task
//! (a killed mutant). Both carry the oracle as an [`OutcomeDef`], which is where the batch runner
//! takes the task's criteria from.

use std::path::Path;

use serde::{Deserialize, Serialize};
use tsr_domain::OutcomeDef;

/// Why reading a task manifest failed.
#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("bad task line {line}: {source}")]
    Parse {
        line: usize,
        source: serde_json::Error,
    },
    /// The task id is not a safe path component (D1): it must match `[A-Za-z0-9._-]{1,128}` and not
    /// be `.` or `..`, because the batch runner joins it into `out/state/<id>` and `out/copies/<id>`.
    #[error("bad task id {id:?} on line {line}: must match [A-Za-z0-9._-]{{1,128}} and not be `.` or `..`")]
    InvalidId { line: usize, id: String },
}

/// Which corpus produced a task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskKind {
    /// A red→green flip mined from the first-parent history.
    Mined,
    /// A killed mutant.
    Mutant,
}

/// One corpus task, as the batch runner needs it.
///
/// The two corpora write slightly different fields; this type is the union the runner reads. A
/// mined task has `parent`/`commit`/`parent_snapshot`/`flipped_tests`/`hidden_diff`; a mutant task
/// has `mutated_snapshot`/`failing_tests`/`reversal_diff`. `snapshot` and `failing_tests` are the
/// normalized view the runner uses, so the driver and the oracle do not care which corpus wrote it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorpusTask {
    /// The task id: blake3 of (repo, parent, commit) for a mined task, or of the mutant.
    pub id: String,
    /// The repository the task came from.
    pub repo: String,
    /// The parent revision (mined tasks only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// The commit revision (mined tasks only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// The parent snapshot digest (mined tasks only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_snapshot: Option<String>,
    /// The mutated snapshot digest (mutant tasks only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutated_snapshot: Option<String>,
    /// The base snapshot a mutant's copy is materialised from: the tree id of the repository HEAD
    /// the mutant was generated against (mutant tasks only, D4). `mutated_snapshot` is only a
    /// descriptor digest and names nothing materialisable, so the copy is this base tree with the
    /// mutant's own diff applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_snapshot: Option<String>,
    /// The tests that flipped red→green (mined tasks only).
    #[serde(default)]
    pub flipped_tests: Vec<String>,
    /// The tests the mutant fails (mutant tasks only).
    #[serde(default)]
    pub failing_tests: Vec<String>,
    /// The CAS digest of the hidden reference diff (mined tasks only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hidden_diff: Option<String>,
    /// The CAS digest of the known reversal diff (mutant tasks only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reversal_diff: Option<String>,
    /// The CAS digest of the mutation diff (mutant tasks only, D4): the diff cargo-mutants applied
    /// to the base tree. The disposable copy is the base snapshot with this applied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mutation_diff: Option<String>,
    /// The family: the crate name, or `(operator, crate)` for a mutant.
    pub family: String,
    /// The lineage: the repository.
    pub lineage: String,
    /// When the task was mined.
    pub timestamp: String,
    /// The oracle, as an outcome definition.
    pub outcome: OutcomeDef,
}

impl CorpusTask {
    /// The snapshot the disposable copy is provisioned at.
    ///
    /// A mined task provisions at its parent tree id. A mutant task provisions at its base tree id
    /// (D4): `mutated_snapshot` is a descriptor digest that names nothing materialisable, so the
    /// copy is the base tree with the mutant's diff applied afterwards.
    pub fn snapshot(&self) -> &str {
        self.parent_snapshot
            .as_deref()
            .or(self.base_snapshot.as_deref())
            .or(self.mutated_snapshot.as_deref())
            .unwrap_or("")
    }

    /// The tests the oracle runs first: the flipped tests, or the mutant's failing tests.
    pub fn failing_tests(&self) -> Vec<String> {
        if !self.flipped_tests.is_empty() {
            self.flipped_tests.clone()
        } else {
            self.failing_tests.clone()
        }
    }

    /// Which corpus wrote this task.
    pub fn kind(&self) -> TaskKind {
        if self.mutated_snapshot.is_some() {
            TaskKind::Mutant
        } else {
            TaskKind::Mined
        }
    }

    /// The hidden diff's CAS digest, whichever corpus wrote it.
    pub fn hidden_diff(&self) -> Option<&str> {
        self.hidden_diff
            .as_deref()
            .or(self.reversal_diff.as_deref())
    }
}

/// Read a `tasks.jsonl` manifest.
///
/// The task id is untrusted: the batch runner joins it into `out/state/<id>` and `out/copies/<id>`,
/// so an absolute id would replace the base and a `..` id would escape `--out` (D1). Every id is
/// validated here, where the manifest is parsed, before any path is built from it.
pub fn read_tasks(path: &Path) -> Result<Vec<CorpusTask>, TaskError> {
    let text = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let task: CorpusTask = serde_json::from_str(line).map_err(|source| TaskError::Parse {
            line: i + 1,
            source,
        })?;
        if !is_safe_task_id(&task.id) {
            return Err(TaskError::InvalidId {
                line: i + 1,
                id: task.id,
            });
        }
        out.push(task);
    }
    Ok(out)
}

/// Whether `id` is a safe path component: `[A-Za-z0-9._-]{1,128}`, and not `.` or `..`.
///
/// The batch runner builds `out/state/<id>` and `out/copies/<id>` from this id, so anything that
/// could name a path outside `--out` — an absolute path, a separator, a `..` component — is
/// refused (D1).
pub fn is_safe_task_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 128 || id == "." || id == ".." {
        return false;
    }
    id.bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Write a `tasks.jsonl` manifest.
pub fn write_tasks(path: &Path, tasks: &[CorpusTask]) -> Result<(), TaskError> {
    let mut buf = String::new();
    for t in tasks {
        buf.push_str(&serde_json::to_string(t).expect("serializable"));
        buf.push('\n');
    }
    std::fs::write(path, buf)?;
    Ok(())
}
