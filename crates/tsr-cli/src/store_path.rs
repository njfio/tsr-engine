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

/// The environment variable a caller may set to name the session explicitly, so `tsr forecast` and
/// `tsr exec` resolve the same task the hook would for that session.
pub const SESSION_ENV: &str = "TSR_SESSION";

/// The directory, under the working directory, that holds the store and the session file.
pub const SESSION_DIR: &str = ".tsr";

/// The file, under [`SESSION_DIR`], that holds the session the hook last saw.
///
/// The hook cannot set the environment of the tool call Claude Code spawns, so it leaves the session
/// identity on disk instead: every hook stage writes the payload's `session_id` here, and
/// `tsr forecast` / `tsr exec` read it. That is the one channel that makes the CLI and the hook
/// agree on *which task a session is* without either guessing.
pub const SESSION_FILE: &str = "session";

/// The directory that holds the session file: the store's directory (`TSR_DB`'s parent, else
/// `<cwd>/.tsr`). The session belongs to the store, so a store at a custom path keeps its session
/// file beside it — which is also what makes the tests hermetic.
pub fn session_dir() -> PathBuf {
    let db = db_path();
    match db.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
        _ => PathBuf::from(SESSION_DIR),
    }
}

/// The session file in `dir`: `<dir>/session`.
pub fn session_file_in(dir: &Path) -> PathBuf {
    dir.join(SESSION_FILE)
}

/// Record `session_id` as the session the hook last saw, in the store's directory (mode 0600), so a
/// later `tsr forecast` / `tsr exec` resolves the same task. Best-effort: a write failure must never
/// fail the hook.
pub fn remember_session(session_id: &str) {
    let dir = session_dir();
    let _ = std::fs::create_dir_all(&dir);
    let _ = write_private(&session_file_in(&dir), session_id);
}

/// Write `contents` to `path` with mode 0600 (owner read/write only) on Unix.
fn write_private(path: &Path, contents: &str) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        f.write_all(contents.as_bytes())
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, contents)
    }
}

/// The session the hook last recorded, if any: `TSR_SESSION` first, then the session file in the
/// store's directory (or its nearest ancestor, so a tool run from a subdirectory still finds it).
pub fn current_session() -> Option<String> {
    if let Some(s) = std::env::var_os(SESSION_ENV) {
        let s = s.to_string_lossy().into_owned();
        if !s.is_empty() {
            return Some(s);
        }
    }
    let start = session_dir();
    let mut dir: Option<&Path> = Some(&start);
    while let Some(d) = dir {
        if let Ok(text) = std::fs::read_to_string(session_file_in(d)) {
            let text = text.trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
        dir = d.parent();
    }
    None
}

/// The one session-resolution rule shared by `tsr forecast` and `tsr exec`.
///
/// A session is named by its id, and its task is `task_for_session(id)` — the same id the hook
/// derives from the payload's `session_id`. The id comes from, in order:
/// 1. an explicit `--task <id>` (handled by the caller, which never reaches here);
/// 2. `TSR_SESSION`;
/// 3. the `.tsr/session` file in the current directory or its nearest ancestor.
///
/// There is no "latest task" and no `hash("default")` fallback: those are exactly the ambiguity that
/// let a `--task`-less forecast land in another session's task when two sessions were open. When no
/// session is known the caller must refuse (exit 2) rather than guess.
pub fn resolve_session_task(explicit: Option<&str>) -> anyhow::Result<TaskId> {
    if let Some(id) = explicit {
        if !id.is_empty() {
            return Ok(task_for_session(id));
        }
    }
    match current_session() {
        Some(id) => Ok(task_for_session(&id)),
        None => anyhow::bail!(
            "no session: pass --task, set TSR_SESSION, or run inside a hooked session"
        ),
    }
}

/// Resolve a user-supplied task argument to a task id, accepting every id the CLI itself prints.
///
/// `tsr task open`/`status` print a name-hash id, the hook names its task from the Claude Code
/// session id, and `tsr task export` may be handed a raw 64-hex id. This tries, in order:
/// 1. a raw 64-hex task id;
/// 2. the session-hash id (what the hook uses);
/// 3. the name-hash id (what `tsr task open --name` uses).
///
/// The first candidate that already has events in `store` wins; if none does, the name-hash id is
/// returned so the caller reports an empty task rather than an error.
pub fn resolve_task(store: &Store, arg: &str) -> TaskId {
    let mut candidates: Vec<TaskId> = Vec::new();
    if let Ok(h) = Hash::from_hex(arg) {
        candidates.push(TaskId(h));
    }
    candidates.push(task_for_session(arg));
    candidates.push(task_for_name(arg));
    for id in &candidates {
        if store.latest_seq(id).ok().flatten().is_some() {
            return *id;
        }
    }
    task_for_name(arg)
}
