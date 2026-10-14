//! Where the store lives and how a task is named.
//!
//! The store is one SQLite file. Its path comes from `TSR_DB` when set, else the discovery rule in
//! [`db_path`] (the nearest `.tsr/tsr.db` from the current directory up to the git work tree's top,
//! else one created at that top; outside a work tree, the current directory only). A task id is
//! content-addressed from the Claude Code `session_id`, so the same session always names the same
//! task and a different session never collides with it.

use std::path::{Path, PathBuf};
use tsr_domain::{Hash, TaskId};
use tsr_store::Store;

/// The default store path, relative to the working directory.
pub const DEFAULT_DB: &str = ".tsr/tsr.db";

/// The store path: `TSR_DB` if set, else the store discovery rule below.
///
/// **Operator decision (r6b): discovery never leaves the project.** With `TSR_DB` unset:
/// - if the current directory is inside a git work tree (a directory containing `.git`, file or
///   directory), search for `.tsr/tsr.db` from the current directory up to and including that work
///   tree's top, and create `.tsr/tsr.db` at the top when none is found;
/// - outside a git work tree, use `.tsr/tsr.db` in the current directory only (no walk).
///
/// The walk is what makes `tsr forecast` / `tsr exec` run from a subdirectory find the store the
/// session was opened in, instead of creating a second store there. It is bounded by the work tree
/// so an unrelated ancestor store (`~/.tsr/tsr.db`, `/tmp/.tsr/tsr.db`) is never used silently: two
/// projects under one parent keep separate stores. The session file and the exec lock follow the
/// store's directory, so they are found by the same rule.
pub fn db_path() -> PathBuf {
    match std::env::var_os("TSR_DB") {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => discover_db_path(),
    }
}

/// The store path for an explicit starting directory, instead of the process's current directory.
///
/// `TSR_DB` is honoured exactly as in [`db_path`]; with it unset, discovery walks from `start`
/// (r6 review defect 5: an in-process caller that passes a workspace must be judged against that
/// workspace, not against wherever the process happens to be).
pub fn db_path_for(start: &Path) -> PathBuf {
    match std::env::var_os("TSR_DB") {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => discover_db_path_from(start),
    }
}

/// The nearest `.tsr/tsr.db` from the current directory up to the work tree's top, else
/// `<top>/.tsr/tsr.db` (the work tree's top, or the current directory outside a work tree).
///
/// The walk stops at the work tree's top (the highest ancestor containing `.git`); when no store
/// exists anywhere up to it, the store is created at the top (the caller's `open_store` creates the
/// parent). Outside a git work tree there is no walk: the store is `<cwd>/.tsr/tsr.db`.
fn discover_db_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    discover_db_path_from(&cwd)
}

/// [`discover_db_path`] from an explicit starting directory (r6 review defect 5).
fn discover_db_path_from(cwd: &Path) -> PathBuf {
    let top = work_tree_top(cwd);
    let mut dir: Option<&Path> = Some(cwd);
    while let Some(d) = dir {
        let candidate = d.join(DEFAULT_DB);
        if candidate.is_file() {
            return candidate;
        }
        if d == top {
            break;
        }
        dir = d.parent();
    }
    top.join(DEFAULT_DB)
}

/// The top of the git work tree containing `dir`: the highest ancestor of `dir` (including `dir`)
/// that contains a `.git` entry (a directory or a file, as a linked work tree or submodule has).
///
/// When no ancestor contains `.git`, `dir` itself is returned, so the caller's walk is a single
/// step and the store is created in the current directory (the documented limit outside a work
/// tree).
fn work_tree_top(dir: &Path) -> PathBuf {
    let mut top: Option<PathBuf> = None;
    let mut cur: Option<&Path> = Some(dir);
    while let Some(d) = cur {
        if d.join(".git").exists() {
            top = Some(d.to_path_buf());
        }
        cur = d.parent();
    }
    top.unwrap_or_else(|| dir.to_path_buf())
}

/// Resolve a store path to an absolute, symlink-resolved path, so two spellings of the same store
/// compare equal and a symlink is followed.
///
/// The rule (repair item 3, review (5)):
/// 1. a relative `db` is joined to `cwd`;
/// 2. `.` and `..` are normalised lexically (no filesystem access);
/// 3. the **longest existing ancestor** is canonicalised — so symlinks resolve — and the
///    non-existent remainder is re-appended, so a store directory that does not exist yet still
///    resolves to a stable absolute path.
///
/// `cwd` is expected to be absolute (the CLI passes `current_dir()`); a relative `cwd` is left as
/// given, which only matters for a caller that never touches the filesystem.
pub fn resolve_store_path(db: &Path, cwd: &Path) -> PathBuf {
    let joined = if db.is_absolute() {
        db.to_path_buf()
    } else {
        cwd.join(db)
    };
    resolve_lexical_and_canonical(&joined)
}

/// Canonicalise a workspace path the same way [`resolve_store_path`] resolves a store path, so the
/// workspace used in the containment checks is the real (symlink-resolved) directory.
pub fn resolve_workspace(ws: &Path) -> PathBuf {
    resolve_lexical_and_canonical(ws)
}

/// Resolve `.`/`..` and symlinks component by component: walk the path, and for each component
/// canonicalise the accumulated prefix when it exists, so a symlink is followed **before** a
/// following `..` is applied. A `..` after a symlink must pop the symlink's *target*, not the link
/// name: `link/../x` with `link → deep/sub` resolves to `deep/x`, not to the link's parent.
fn resolve_lexical_and_canonical(p: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::Prefix(_) | Component::RootDir => out.push(comp.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                // Resolve the accumulated prefix first, so a `..` pops the symlink's target.
                if let Ok(real) = std::fs::canonicalize(&out) {
                    out = real;
                }
                out.pop();
            }
            Component::Normal(c) => {
                out.push(c);
                // Follow a symlink as soon as the component exists, so a later `..` sees the target.
                if let Ok(real) = std::fs::canonicalize(&out) {
                    out = real;
                }
            }
        }
    }
    out
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

/// The directory that holds the session file: the store's directory (`TSR_DB`'s parent, else the
/// resolved store's parent — the work tree's top, or the current directory outside one). The
/// session belongs to the store, so a store at a custom path keeps its session file beside it —
/// which is also what makes the tests hermetic.
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
/// store's directory (or its nearest ancestor up to the work tree's top, so a tool run from a
/// subdirectory still finds it — but an unrelated ancestor above the work tree is never used).
pub fn current_session() -> Option<String> {
    if let Some(s) = std::env::var_os(SESSION_ENV) {
        let s = s.to_string_lossy().into_owned();
        if !s.is_empty() {
            return Some(s);
        }
    }
    let start = session_dir();
    // Bound the walk by the same work tree the store discovery uses, so a session file above the
    // work tree (e.g. `~/.tsr/session`) is never read.
    let top = work_tree_top(&start);
    let mut dir: Option<&Path> = Some(&start);
    while let Some(d) = dir {
        if let Ok(text) = std::fs::read_to_string(session_file_in(d)) {
            let text = text.trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
        if d == top {
            break;
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
/// 3. the `.tsr/session` file in the store's directory or its nearest ancestor up to the work
///    tree's top.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch directory under the target dir (never `/tmp`, which this host prunes).
    fn scratch(name: &str) -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/store-path-unit")
            .join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    #[cfg(unix)]
    fn symlink(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).expect("symlink");
    }

    /// A relative store path is joined to the current directory.
    #[test]
    fn relative_store_path_is_joined_to_cwd() {
        let cwd = scratch("rel-cwd");
        let got = resolve_store_path(Path::new("data/tsr.db"), &cwd);
        let want = std::fs::canonicalize(&cwd)
            .expect("canonicalize cwd")
            .join("data/tsr.db");
        assert_eq!(got, want);
    }

    /// `.` and `..` are normalised lexically before the path is used.
    #[test]
    fn dot_and_dotdot_are_normalised() {
        let cwd = scratch("dotdot");
        let got = resolve_store_path(Path::new("./.tsr/../.tsr/tsr.db"), &cwd);
        let want = std::fs::canonicalize(&cwd)
            .expect("canonicalize cwd")
            .join(".tsr/tsr.db");
        assert_eq!(got, want);
    }

    /// The longest existing ancestor is canonicalised, so a symlinked directory resolves to its
    /// target and the non-existent remainder is re-appended.
    #[cfg(unix)]
    #[test]
    fn longest_existing_ancestor_is_canonicalised() {
        let root = scratch("canon");
        let real = root.join("real");
        std::fs::create_dir_all(&real).expect("real dir");
        let link = root.join("link");
        symlink(&real, &link);
        // `link` exists and points at `real`; `sub/tsr.db` does not exist yet.
        let got = resolve_store_path(&link.join("sub/tsr.db"), &root);
        let real_canon = std::fs::canonicalize(&real).expect("canonicalize real");
        assert_eq!(got, real_canon.join("sub/tsr.db"));
    }

    /// A workspace path is canonicalised the same way, so a symlinked workspace compares equal to
    /// its target.
    #[cfg(unix)]
    #[test]
    fn workspace_is_canonicalised() {
        let root = scratch("ws-canon");
        let real = root.join("real");
        std::fs::create_dir_all(&real).expect("real dir");
        let link = root.join("link");
        symlink(&real, &link);
        assert_eq!(
            resolve_workspace(&link),
            std::fs::canonicalize(&real).expect("canonicalize real")
        );
    }

    /// A scratch directory under the system temp dir, with no `.git` in any ancestor, so
    /// [`work_tree_top`]'s walk is not bounded by the workspace's own work tree.
    fn scratch_outside_worktree(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("store-path-unit").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let mut d: Option<&Path> = Some(&dir);
        while let Some(cur) = d {
            assert!(
                !cur.join(".git").exists(),
                "scratch_outside_worktree needs no ancestor .git, but {} has one",
                cur.display()
            );
            d = cur.parent();
        }
        dir
    }

    /// The work tree's top is the highest ancestor containing `.git` (a directory or a file).
    #[test]
    fn work_tree_top_is_the_highest_ancestor_with_git() {
        let root = scratch_outside_worktree("work-tree-top");
        let top = root.join("proj");
        let sub = top.join("a/b");
        std::fs::create_dir_all(&sub).expect("subdir");
        std::fs::create_dir_all(top.join(".git")).expect(".git dir");
        assert_eq!(work_tree_top(&sub), top, "the top is the .git directory");
        assert_eq!(work_tree_top(&top), top, "the top is itself a work tree");
    }

    /// A `.git` **file** (a linked work tree or submodule) also marks the top.
    #[test]
    fn work_tree_top_accepts_a_git_file() {
        let root = scratch_outside_worktree("work-tree-gitfile");
        let top = root.join("proj");
        let sub = top.join("a");
        std::fs::create_dir_all(&sub).expect("subdir");
        std::fs::write(top.join(".git"), "gitdir: /elsewhere\n").expect(".git file");
        assert_eq!(work_tree_top(&sub), top);
    }

    /// Outside a work tree there is no walk: the top is the directory itself.
    #[test]
    fn work_tree_top_outside_a_work_tree_is_the_directory() {
        let root = scratch_outside_worktree("work-tree-none");
        let sub = root.join("a/b");
        std::fs::create_dir_all(&sub).expect("subdir");
        assert_eq!(work_tree_top(&sub), sub);
    }

    /// A `..` after a symlink pops the symlink's **target**, not the link name (review item 8):
    /// `link → <W>/deep/sub`, so `link/../x/tsr.db` resolves to `<W>/deep/x/tsr.db`.
    #[cfg(unix)]
    #[test]
    fn dotdot_after_a_symlink_pops_the_target() {
        let root = scratch("symlink-dotdot");
        let deep_sub = root.join("deep/sub");
        std::fs::create_dir_all(&deep_sub).expect("deep/sub dir");
        let link = root.join("link");
        symlink(&deep_sub, &link);
        let got = resolve_store_path(&link.join("../x/tsr.db"), &root);
        let want = std::fs::canonicalize(&root)
            .expect("canonicalize root")
            .join("deep/x/tsr.db");
        assert_eq!(got, want);
    }
}
