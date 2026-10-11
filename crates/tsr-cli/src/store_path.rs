//! Where the store lives and how a task is named.
//!
//! The store is one SQLite file. Its path comes from `TSR_DB` when set, else `.tsr/tsr.db` under
//! the current directory. A task id is content-addressed from the Claude Code `session_id`, so the
//! same session always names the same task and a different session never collides with it.

use std::path::{Path, PathBuf};
use tsr_domain::{Hash, TaskId};
use tsr_store::Store;

/// The default store path, relative to the working directory.
pub const DEFAULT_DB: &str = ".tsr/tsr.db";

/// The store path: `TSR_DB` if set, else [`DEFAULT_DB`].
pub fn db_path() -> PathBuf {
    match std::env::var_os("TSR_DB") {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => PathBuf::from(DEFAULT_DB),
    }
}

/// Open the store, creating its parent directory if needed.
pub fn open_store() -> anyhow::Result<Store> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(Store::open(&path)?)
}

/// Open the store at an explicit path (used by the self-test harness).
pub fn open_store_at(path: &Path) -> anyhow::Result<Store> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(Store::open(path)?)
}

/// The task id for a Claude Code session: blake3 over the session id.
pub fn task_for_session(session_id: &str) -> TaskId {
    TaskId(Hash::of(&[session_id.as_bytes()]))
}

/// The task id for an explicit task name (used by `tsr task open --name`).
pub fn task_for_name(name: &str) -> TaskId {
    TaskId(Hash::of(&[b"task:", name.as_bytes()]))
}
