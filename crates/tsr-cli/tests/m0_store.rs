//! M0 lane-01 defect repair (r6): store discovery from a subdirectory, `tsr task status`'s event
//! count, and the `tsr forecast --event` candidate check.
//!
//! Every test drives the *built binary* (`CARGO_BIN_EXE_tsr`) in a scratch directory under `target/`
//! (never `/tmp`, which this host prunes). The store-discovery tests run `tsr` from a subdirectory
//! with `TSR_DB` unset, so they exercise the real ancestor walk.

use std::path::{Path, PathBuf};
use std::process::Command;

use tsr_store::Store;

/// The `tsr` binary cargo built for this test.
fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

/// A scratch directory under the target dir (never `/tmp`, which this host prunes).
///
/// Note: the workspace root itself carries a `.tsr/tsr.db` (a gitignored runtime artifact), so a
/// scratch dir under `target/` always has a store in an ancestor. Tests that need *no* ancestor
/// store use [`scratch_isolated`] instead.
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/m0-store")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// A scratch directory with no `.tsr/tsr.db` in any ancestor, so the store-discovery rule's
/// "create one in place" branch can be exercised. `/tmp` is the temp dir the brief allows for
/// scratch; it is not the build cache, so pruning it between runs is harmless.
fn scratch_isolated(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("m0-store").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    // Guard the test's premise: no ancestor may already hold a store.
    let mut d: Option<&Path> = Some(&dir);
    while let Some(cur) = d {
        assert!(
            !cur.join(".tsr/tsr.db").is_file(),
            "scratch_isolated needs no ancestor store, but {} has one",
            cur.display()
        );
        d = cur.parent();
    }
    dir
}

/// Run `tsr <args…>` in `cwd` with `TSR_DB` unset (so store discovery is exercised) and no
/// `TSR_SESSION`, returning the output.
fn tsr_in(cwd: &Path, args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new(tsr_bin());
    cmd.args(args).current_dir(cwd);
    cmd.env_remove("TSR_DB");
    cmd.env_remove("TSR_SESSION");
    cmd.output().expect("run tsr")
}

/// Run `tsr <args…>` in `cwd` with an explicit `TSR_DB` and `TSR_SESSION`.
fn tsr_with_db(cwd: &Path, db: &Path, session: &str, args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new(tsr_bin());
    cmd.args(args)
        .current_dir(cwd)
        .env("TSR_DB", db)
        .env("TSR_SESSION", session);
    cmd.output().expect("run tsr")
}

/// The kind tags of a task's events, in sequence order.
fn kinds(store: &Store, task: &tsr_domain::TaskId) -> Vec<String> {
    store
        .by_task(task)
        .expect("by_task")
        .into_iter()
        .map(|(_, ev)| {
            serde_json::to_value(&ev.kind).expect("serialize")["kind"]
                .as_str()
                .expect("kind tag")
                .to_string()
        })
        .collect()
}

/// Count the `tsr.db` files under `root` (recursively), so a test can assert that no *second* store
/// was created in a subdirectory.
fn count_stores(root: &Path) -> usize {
    let mut n = 0;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.file_name().is_some_and(|n| n == "tsr.db") {
                n += 1;
            }
        }
    }
    n
}

/// Defect 1: with `TSR_DB` unset, `tsr forecast` run from a subdirectory uses the store in the
/// nearest ancestor (the root's `.tsr/tsr.db`), not a fresh one in the subdirectory.
#[test]
fn forecast_from_a_subdirectory_uses_the_root_store() {
    let root = scratch("forecast-subdir");
    let sub = root.join("a/b");
    std::fs::create_dir_all(&sub).expect("subdir");

    // Seed the store at the scratch root explicitly (the workspace root also carries a store, so an
    // unset `TSR_DB` here would find *that* one). The session file is written beside the store.
    let root_db = root.join(".tsr/tsr.db");
    let open = tsr_with_db(
        &root,
        &root_db,
        "root-sess",
        &["task", "open", "--name", "root-task"],
    );
    assert!(
        open.status.success(),
        "task open in the root: stderr={}",
        String::from_utf8_lossy(&open.stderr)
    );
    assert!(root_db.exists(), "the root store exists after task open");
    // The session file lives beside the store; `tsr forecast` from the subdirectory finds it by the
    // same ancestor walk that finds the store.
    std::fs::write(root.join(".tsr/session"), "root-sess").expect("write session file");

    // From the subdirectory, `tsr forecast --kind … --args …` must find the root store. It needs a
    // session: the root's `.tsr/session` is found by the ancestor walk.
    let out = tsr_in(
        &sub,
        &[
            "forecast",
            "--kind",
            "run_approved_test",
            "--args",
            "cargo test",
            "--p",
            "0.5",
        ],
    );
    assert!(
        out.status.success(),
        "forecast from a subdirectory must find the root store: code={:?} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        count_stores(&root),
        1,
        "exactly one store exists after a forecast from a subdirectory"
    );
    assert!(
        !sub.join(".tsr/tsr.db").exists(),
        "no second store was created in the subdirectory"
    );
}

/// Defect 1: a fresh directory with no store anywhere up to the filesystem root creates one in
/// place (in the current directory).
#[test]
fn a_fresh_directory_creates_a_store_in_place() {
    let root = scratch_isolated("fresh-dir");
    let sub = root.join("fresh");
    std::fs::create_dir_all(&sub).expect("fresh dir");

    let out = tsr_in(&sub, &["task", "open", "--name", "fresh-task"]);
    assert!(
        out.status.success(),
        "task open in a fresh directory: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        sub.join(".tsr/tsr.db").exists(),
        "a fresh directory with no store creates one in place"
    );
    assert_eq!(count_stores(&root), 1, "exactly one store was created");
}

/// Defect 1: `tsr exec` resolves the store by the ancestor walk, and the read-only mount uses the
/// *resolved* store path. Two cases:
///  - the store sits inside the workspace (the workspace is the store's own directory): it is
///    mounted read-only;
///  - `tsr exec` runs from a subdirectory: the store is still found (no second store is created),
///    and because it now lies outside the subdirectory workspace it is not mounted.
#[test]
fn exec_resolves_the_store_and_mounts_it_read_only_when_inside_the_workspace() {
    let root = scratch("exec-subdir");
    let sub = root.join("deep/sub");
    std::fs::create_dir_all(&sub).expect("subdir");

    // A fake runtime whose `info` succeeds and whose `run` exits 0, recording its argv.
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).expect("bin");
    let docker = bin.join("docker");
    let argv_log = root.join("argv.log");
    let script = format!(
        "#!/bin/sh\ncase \"$1\" in\n  --version) exit 0 ;;\n  info) exit 0 ;;\n  run) printf '%s\\n' \"$@\" > {log}; exit 0 ;;\n  *) exit 0 ;;\nesac\n",
        log = argv_log.display()
    );
    std::fs::write(&docker, script).expect("write fake docker");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&docker, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake docker");
    }
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    // Seed the store at the scratch root explicitly (the workspace root also carries a store).
    let root_db = root.join(".tsr/tsr.db");
    let open = tsr_with_db(
        &root,
        &root_db,
        "exec-sess",
        &["task", "open", "--name", "exec-root"],
    );
    assert!(open.status.success(), "task open: {:?}", open.status.code());
    std::fs::write(root.join(".tsr/session"), "exec-sess").expect("write session file");

    // Case 1: run from the root. The workspace is the root, the store is `<root>/.tsr` inside it, so
    // the store's directory is mounted read-only.
    let mut cmd = Command::new(tsr_bin());
    cmd.args(["exec", "--", "true"])
        .current_dir(&root)
        .env("PATH", &path)
        .env("TSR_SESSION", "exec-sess")
        .env_remove("TSR_DB");
    let out = cmd.output().expect("run tsr exec");
    assert!(
        out.status.success(),
        "exec from the root: code={:?} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let argv = std::fs::read_to_string(&argv_log).expect("the fake runtime recorded its argv");
    let root_canon = std::fs::canonicalize(&root).expect("canonicalize root");
    let store_dir = root_canon.join(".tsr");
    let want = format!(
        "type=bind,src={d},dst={d},readonly",
        d = store_dir.display()
    );
    assert!(
        argv.lines().any(|l| l == want),
        "the store's directory is mounted read-only: want {want:?} in argv:\n{argv}"
    );

    // Case 2: run from a subdirectory. The store is still found by the ancestor walk (no second
    // store), but it now lies outside the subdirectory workspace, so it is not mounted.
    let _ = std::fs::remove_file(&argv_log);
    let mut cmd = Command::new(tsr_bin());
    cmd.args(["exec", "--", "true"])
        .current_dir(&sub)
        .env("PATH", &path)
        .env("TSR_SESSION", "exec-sess")
        .env_remove("TSR_DB");
    let out = cmd.output().expect("run tsr exec");
    assert!(
        out.status.success(),
        "exec from a subdirectory: code={:?} stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        count_stores(&root),
        1,
        "exec from a subdirectory created no second store"
    );
    assert!(
        !sub.join(".tsr/tsr.db").exists(),
        "no store was created in the subdirectory"
    );
}

/// Defect 2: `tsr task status` prints the task's *own* event count, not the store-wide highest
/// sequence number. Two tasks are interleaved so the two numbers differ.
#[test]
fn task_status_counts_the_tasks_own_events_with_two_interleaved_tasks() {
    let dir = scratch("status-count");
    let db = dir.join(".tsr/tsr.db");
    std::fs::create_dir_all(db.parent().expect("parent")).expect("store dir");

    // Two tasks, opened and then each given a second event, interleaved so the store-wide sequence
    // number of the first task's last event is larger than its own event count.
    let a = tsr_with_db(&dir, &db, "sess-a", &["task", "open", "--name", "task-a"]);
    assert!(a.status.success(), "open a: {:?}", a.status.code());
    let b = tsr_with_db(&dir, &db, "sess-b", &["task", "open", "--name", "task-b"]);
    assert!(b.status.success(), "open b: {:?}", b.status.code());

    // Give task A one more event (a forecast), so A has 3 events while the store holds 5.
    let f = tsr_with_db(
        &dir,
        &db,
        "sess-a",
        &[
            "forecast",
            "--task",
            "task-a",
            "--kind",
            "run_approved_test",
            "--args",
            "cargo test",
            "--p",
            "0.5",
        ],
    );
    assert!(f.status.success(), "forecast a: {:?}", f.status.code());

    let store = Store::open(&db).expect("open store");
    let task_a = tsr_cli::store_path::task_for_name("task-a");
    let own = store.by_task(&task_a).expect("by_task").len();
    let latest = store.latest_seq(&task_a).expect("latest_seq").unwrap_or(0);
    assert!(own >= 3, "task A has at least three events, got {own}");
    assert!(
        latest as usize > own,
        "the store-wide latest_seq ({latest}) is larger than A's own count ({own}), so the two \
         numbers differ and the test can tell them apart"
    );

    let out = tsr_with_db(&dir, &db, "sess-a", &["task", "status", "task-a"]);
    assert!(out.status.success(), "status: {:?}", out.status.code());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains(&format!("{own} events")),
        "task status prints the task's own event count ({own}): stdout={stdout:?}"
    );
    assert!(
        !stdout.contains(&format!("{latest} events")),
        "task status does not print the store-wide latest_seq ({latest}): stdout={stdout:?}"
    );
}

/// Defect 3: `tsr forecast --event <id>` refuses an id that is not a candidate proposed in the
/// session's task — exit 2, naming the id, and recording nothing.
#[test]
fn forecast_event_refuses_an_id_that_is_not_a_proposed_candidate() {
    let dir = scratch("forecast-unknown");
    let db = dir.join(".tsr/tsr.db");
    std::fs::create_dir_all(db.parent().expect("parent")).expect("store dir");

    let open = tsr_with_db(&dir, &db, "sess-u", &["task", "open", "--name", "task-u"]);
    assert!(open.status.success(), "open: {:?}", open.status.code());

    // A well-formed 64-hex id that was never proposed in this task.
    let unknown = "ab".repeat(32);
    let out = tsr_with_db(
        &dir,
        &db,
        "sess-u",
        &[
            "forecast", "--task", "task-u", "--event", &unknown, "--p", "0.5",
        ],
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "an unproposed candidate id -> exit 2: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(&unknown),
        "the refusal names the id: stderr={stderr:?}"
    );

    let store = Store::open(&db).expect("open store");
    let task = tsr_cli::store_path::task_for_name("task-u");
    let tags = kinds(&store, &task);
    assert!(
        !tags.contains(&"forecast".to_string()),
        "nothing was recorded for an unproposed id: {tags:?}"
    );
}

/// Defect 3: a forecast for a candidate that has already run is allowed — it re-arms it (the r5c
/// rule). The candidate is proposed, forecast, committed by the PreToolUse hook, dispatched, and then
/// forecast again. The task is the *session* task (created by the hook's `SessionStart`), so the
/// forecasts and `tsr exec --candidate` all resolve the same task.
#[test]
fn forecast_event_allows_a_candidate_that_has_already_run() {
    let dir = scratch("forecast-rearm");
    let db = dir.join(".tsr/tsr.db");
    std::fs::create_dir_all(db.parent().expect("parent")).expect("store dir");

    // A fake container runtime: `info` answers (a daemon is up) and `run` succeeds.
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).expect("bin");
    let docker = bin.join("docker");
    std::fs::write(
        &docker,
        "#!/bin/sh\ncase \"$1\" in\n  --version) exit 0 ;;\n  info) exit 0 ;;\n  run) exit 0 ;;\n  *) exit 0 ;;\nesac\n",
    )
    .expect("write fake docker");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&docker, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake docker");
    }
    let tsr_dir = tsr_bin().parent().expect("tsr parent").to_path_buf();
    let path = format!(
        "{}:{}:{}",
        tsr_dir.display(),
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    // The hook's SessionStart opens the session task and writes `<dir>/.tsr/session`.
    let start = serde_json::json!({
        "session_id": "sess-r",
        "hook_event_name": "SessionStart",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    let hook = |stage: &str, payload: &str| -> std::process::Output {
        let mut cmd = Command::new(tsr_bin());
        cmd.args(["hook", "--stage", stage])
            .current_dir(&dir)
            .env("PATH", &path)
            .env("TSR_DB", &db)
            .env("TSR_SESSION", "sess-r")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = cmd.spawn().expect("spawn tsr hook");
        {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .expect("stdin")
                .write_all(payload.as_bytes())
                .expect("write payload");
        }
        child.wait_with_output().expect("tsr hook output")
    };
    let out = hook("SessionStart", &start);
    assert!(out.status.success(), "SessionStart exits 0: {out:?}");

    // Propose a candidate through the `--kind/--args` path; it prints the candidate id. `TSR_SESSION`
    // (no `--task`) resolves the session task, the same one `tsr exec --candidate` will use.
    let f1 = tsr_with_db(
        &dir,
        &db,
        "sess-r",
        &[
            "forecast",
            "--kind",
            "run_approved_test",
            "--args",
            "cargo test",
            "--p",
            "0.5",
        ],
    );
    assert!(
        f1.status.success(),
        "first forecast: {:?}",
        f1.status.code()
    );
    let cid = String::from_utf8_lossy(&f1.stdout).trim().to_string();
    assert_eq!(cid.len(), 64, "the forecast printed a 64-hex candidate id");

    // The agent's Bash command: `tsr exec --candidate <id> -- true`. PreToolUse commits the candidate.
    let command = format!("tsr exec --candidate {cid} -- true");
    let input = serde_json::json!({ "command": command });
    let pre = serde_json::json!({
        "session_id": "sess-r",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": input,
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    let out = hook("PreToolUse", &pre);
    assert!(
        out.status.success(),
        "PreToolUse exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Run the command for real, as the shell would, with the fake runtime on PATH.
    let mut bash = Command::new("bash");
    bash.arg("-c")
        .arg(&command)
        .current_dir(&dir)
        .env("PATH", &path)
        .env("TSR_DB", &db)
        .env("TSR_SESSION", "sess-r");
    let exec = bash.output().expect("run bash -c");
    assert!(
        exec.status.success(),
        "`{command}` exits 0: {}",
        String::from_utf8_lossy(&exec.stderr)
    );

    let post = serde_json::json!({
        "session_id": "sess-r",
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "tool_input": input,
        "tool_response": { "stdout": "ok", "stderr": "", "exit_code": 0 },
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    let out = hook("PostToolUse", &post);
    assert!(out.status.success(), "PostToolUse exits 0: {out:?}");

    let store = Store::open(&db).expect("open store");
    let task = tsr_cli::store_path::task_for_session("sess-r");
    let tags = kinds(&store, &task);
    assert!(
        tags.contains(&"dispatched".to_string()),
        "the candidate was dispatched: {tags:?}"
    );

    // A second forecast for the same (already-run) candidate is allowed: it re-arms it.
    let f2 = tsr_with_db(
        &dir,
        &db,
        "sess-r",
        &["forecast", "--event", &cid, "--p", "0.6"],
    );
    assert!(
        f2.status.success(),
        "a forecast for an already-run candidate is allowed: code={:?} stderr={}",
        f2.status.code(),
        String::from_utf8_lossy(&f2.stderr)
    );
    let store = Store::open(&db).expect("reopen store");
    let tags = kinds(&store, &task);
    assert_eq!(
        tags.iter().filter(|t| *t == "forecast").count(),
        2,
        "the re-arming forecast was recorded: {tags:?}"
    );
}
