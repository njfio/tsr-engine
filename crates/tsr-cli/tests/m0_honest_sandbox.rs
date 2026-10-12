//! M0 honest sandbox (repair item 3, PRD §B10): a sandboxed run that did not happen is never
//! recorded, and nothing inside the sandbox may write the event store.
//!
//! Every test here drives the *built binary* with a **fake `docker`** on `PATH` — no real container
//! runtime is needed, and none is used:
//!
//! * a fake whose `--version` succeeds but whose `info` fails is *not* a reachable runtime: `tsr exec`
//!   exits 3 and the store gains no event (the probe asks the daemon, not the binary);
//! * a fake whose `info` succeeds but whose `run` exits 125 (the runtime's own "container did not
//!   start") records `Unresolved` naming the runtime error — never `Executed` — and exits 3;
//! * a fake whose `run` exits 1 (the command's own failure) records `Executed { exit_code: 1 }`;
//! * the selftest with the first fake exits 3 and writes nothing anywhere;
//! * the selftest never touches `fixtures/m0` without `--update-frozen-fixture`;
//! * `container_argv` mounts the store's directory read-only inside the container whenever it lies
//!   inside the workspace, after the workspace mount, and not when it lies outside.

use std::path::{Path, PathBuf};
use std::process::Command;

use tsr_cli::exec::{self, Runtime};
use tsr_domain::EventKind;
use tsr_store::Store;

/// The `tsr` binary cargo built for this test.
fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

/// A scratch directory under the target dir (never `/tmp`, which this host prunes).
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/m0-honest")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// Write a fake `docker` script into `dir/bin` and return the directory to prepend to `PATH`.
///
/// The script answers `--version` and `info` as the caller asks, and `run` with `run_exit`. It never
/// starts a container: it is a shell script, so the test needs no runtime.
fn fake_docker(dir: &Path, version_ok: bool, info_ok: bool, run_exit: i32) -> PathBuf {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).expect("fake bin dir");
    let script = format!(
        r#"#!/bin/sh
case "$1" in
  --version) {version} ;;
  info) {info} ;;
  run) echo "fake docker run: container did not start" >&2; exit {run_exit} ;;
  *) exit 0 ;;
esac
"#,
        version = if version_ok { "exit 0" } else { "exit 1" },
        info = if info_ok { "exit 0" } else { "exit 1" },
        run_exit = run_exit,
    );
    let path = bin.join("docker");
    std::fs::write(&path, script).expect("write fake docker");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake docker");
    }
    bin
}

/// A `PATH` with the fake runtime's directory first and the real `PATH` after it.
fn path_with(bin: &Path) -> String {
    let real = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", bin.display(), real)
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

/// Run `tsr exec -- <argv>` in `cwd` with the fake runtime on `PATH` and `TSR_DB`/`TSR_SESSION` set.
fn run_exec(
    cwd: &Path,
    bin: &Path,
    db: &Path,
    session: &str,
    argv: &[&str],
) -> std::process::Output {
    let mut cmd = Command::new(tsr_bin());
    cmd.arg("exec").arg("--").args(argv);
    cmd.current_dir(cwd)
        .env("PATH", path_with(bin))
        .env("TSR_DB", db)
        .env("TSR_SESSION", session);
    cmd.output().expect("run tsr exec")
}

/// A fake whose `--version` succeeds but whose `info` fails is not a reachable runtime: `tsr exec`
/// exits 3 and records nothing. This is the defect the repair names: the probe used to ask the
/// binary, so a host with the docker CLI and no daemon recorded a run that never happened.
#[test]
fn exec_with_a_docker_cli_but_no_daemon_exits_3_and_records_nothing() {
    let dir = scratch("no-daemon");
    let bin = fake_docker(&dir, true, false, 0);
    let db = dir.join("tsr.db");
    let session = "m0-no-daemon";

    let out = run_exec(&dir, &bin, &db, session, &["cargo", "test"]);

    assert_eq!(
        out.status.code(),
        Some(exec::EXIT_NO_RUNTIME),
        "a docker CLI with no daemon → exit 3: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let store = Store::open(&db).expect("open store");
    let task = tsr_cli::store_path::task_for_session(session);
    assert_eq!(
        store.by_task(&task).expect("by_task").len(),
        0,
        "an unreachable runtime records nothing at all"
    );
}

/// The runtime is reachable but the container does not start (`run` exits 125): the dispatch is
/// recorded `Unresolved` with a reason naming the runtime error, never `Executed`, and `tsr exec`
/// exits 3.
#[test]
fn exec_records_unresolved_when_the_container_does_not_start() {
    let dir = scratch("run-125");
    let bin = fake_docker(&dir, true, true, 125);
    let db = dir.join("tsr.db");
    let session = "m0-run-125";

    let out = run_exec(&dir, &bin, &db, session, &["cargo", "test"]);

    assert_eq!(
        out.status.code(),
        Some(exec::EXIT_NO_RUNTIME),
        "a container that does not start → exit 3: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let store = Store::open(&db).expect("open store");
    let task = tsr_cli::store_path::task_for_session(session);
    let events = store.by_task(&task).expect("by_task");
    let tags = kinds(&store, &task);
    assert!(
        tags.contains(&"unresolved".to_string()),
        "a container that does not start is Unresolved: {tags:?}"
    );
    assert!(
        !tags.contains(&"executed".to_string()),
        "a container that does not start is never Executed: {tags:?}"
    );
    let reason = events
        .iter()
        .find_map(|(_, ev)| match &ev.kind {
            EventKind::Unresolved { reason, .. } => Some(reason.clone()),
            _ => None,
        })
        .expect("an Unresolved event");
    assert!(
        reason.contains("125") || reason.contains("did not start"),
        "the Unresolved reason names the runtime error: {reason:?}"
    );
}

/// The container starts and the command inside it fails (`run` exits 1): that exit code is the
/// command's own, so it is recorded `Executed { exit_code: 1 }` and `tsr exec` exits 1.
#[test]
fn exec_records_executed_with_the_commands_own_exit_code() {
    let dir = scratch("run-1");
    let bin = fake_docker(&dir, true, true, 1);
    let db = dir.join("tsr.db");
    let session = "m0-run-1";

    let out = run_exec(&dir, &bin, &db, session, &["cargo", "test"]);

    assert_eq!(
        out.status.code(),
        Some(1),
        "the command's own failure is passed through: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    let store = Store::open(&db).expect("open store");
    let task = tsr_cli::store_path::task_for_session(session);
    let tags = kinds(&store, &task);
    assert!(
        tags.contains(&"executed".to_string()),
        "a command that ran and failed is Executed: {tags:?}"
    );
    let exit_code = store
        .by_task(&task)
        .expect("by_task")
        .into_iter()
        .find_map(|(_, ev)| match ev.kind {
            EventKind::Executed { exit_code, .. } => exit_code,
            _ => None,
        })
        .expect("an Executed event");
    assert_eq!(exit_code, 1, "the command's own exit code is recorded");
}

/// The selftest with a docker CLI but no daemon exits 3 and writes nothing anywhere — not the
/// fixture, not a store, not a session file.
#[test]
fn selftest_with_a_docker_cli_but_no_daemon_exits_3_and_writes_nothing() {
    let dir = scratch("selftest-no-daemon");
    let bin = fake_docker(&dir, true, false, 0);
    let out_dir = dir.join("out");

    let out = Command::new(tsr_bin())
        .args(["hook", "--selftest", "--out"])
        .arg(&out_dir)
        .current_dir(&dir)
        .env("PATH", path_with(&bin))
        .output()
        .expect("run tsr hook --selftest");

    assert_eq!(
        out.status.code(),
        Some(exec::EXIT_NO_RUNTIME),
        "no daemon → exit 3: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out_dir.exists()
            || std::fs::read_dir(&out_dir)
                .expect("read out dir")
                .next()
                .is_none(),
        "the selftest wrote something into --out without a daemon"
    );
    assert!(
        !dir.join("fixtures/m0/session.events.jsonl").exists(),
        "the selftest must never fabricate fixtures/m0 without --update-frozen-fixture"
    );
}

/// The selftest never touches `fixtures/m0` unless `--update-frozen-fixture` is given: with a
/// reachable fake runtime and `--out`, the fixture directory is untouched and the output lands in
/// the named directory.
#[test]
fn selftest_writes_to_out_and_never_touches_fixtures_m0_without_the_flag() {
    let dir = scratch("selftest-out");
    let bin = fake_docker(&dir, true, true, 0);
    let out_dir = dir.join("out");
    // A sentinel in fixtures/m0 that must survive byte-for-byte.
    let fixture_dir = dir.join("fixtures/m0");
    std::fs::create_dir_all(&fixture_dir).expect("fixture dir");
    let sentinel = fixture_dir.join("session.events.jsonl");
    std::fs::write(&sentinel, b"sentinel\n").expect("write sentinel");

    let out = Command::new(tsr_bin())
        .args(["hook", "--selftest", "--out"])
        .arg(&out_dir)
        .current_dir(&dir)
        .env("PATH", path_with(&bin))
        .output()
        .expect("run tsr hook --selftest");

    assert!(
        out.status.success(),
        "the selftest with a reachable runtime succeeds: status={:?} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        std::fs::read(&sentinel).expect("read sentinel"),
        b"sentinel\n",
        "the selftest must not touch fixtures/m0 without --update-frozen-fixture"
    );
    assert!(
        out_dir.join("session.events.jsonl").exists(),
        "the selftest writes its trace into --out"
    );
}

/// `container_argv` mounts the store's directory read-only inside the container whenever it lies
/// inside the workspace, after the workspace mount.
#[test]
fn container_argv_mounts_the_store_directory_read_only_when_inside_the_workspace() {
    let ws = Path::new("/work");
    let argv = vec!["cargo".to_string(), "test".to_string()];

    // The default store directory `<workspace>/.tsr`.
    let v = exec::container_argv(Runtime::Docker, ws, &argv);
    let ws_mount = "type=bind,src=/work,dst=/work";
    let store_mount = "type=bind,src=/work/.tsr,dst=/work/.tsr,readonly";
    let ws_at = v
        .iter()
        .position(|a| a.as_str() == ws_mount)
        .expect("workspace mount");
    let store_at = v
        .iter()
        .position(|a| a.as_str() == store_mount)
        .expect("read-only store mount");
    assert!(
        store_at > ws_at,
        "the read-only store mount comes after the workspace mount"
    );
    // A `TSR_DB` inside the workspace: its parent is mounted read-only.
    let v = exec::container_argv_with_store(
        Runtime::Docker,
        ws,
        &argv,
        Some(Path::new("/work/sub/tsr.db")),
    );
    assert!(
        v.iter()
            .any(|a| a == "type=bind,src=/work/sub,dst=/work/sub,readonly"),
        "a TSR_DB inside the workspace mounts its parent read-only: {v:?}"
    );
}

/// A `TSR_DB` outside the workspace is not mounted: the container has no path to it.
#[test]
fn container_argv_does_not_mount_a_store_outside_the_workspace() {
    let ws = Path::new("/work");
    let argv = vec!["cargo".to_string(), "test".to_string()];
    let v = exec::container_argv_with_store(
        Runtime::Docker,
        ws,
        &argv,
        Some(Path::new("/elsewhere/tsr.db")),
    );
    assert!(
        !v.iter().any(|a| a.contains("/elsewhere")),
        "a store outside the workspace is not mounted: {v:?}"
    );
}
