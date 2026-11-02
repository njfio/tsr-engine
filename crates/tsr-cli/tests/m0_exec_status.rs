//! CLI status and ledger agreement. Real checks require the already-installed image;
//! they never pull it. No-runtime/no-session checks isolate only the child PATH.
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tsr_cli::{exec, store_path};
use tsr_domain::EventKind;

const IMAGE: &str = "sha256:cff6dcceb8b9d203a818b7ff104b89a6aaa353f8576850ca4f58903539417945";
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/m0-exec-status")
        .join(format!("{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
fn command(dir: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tsr"));
    cmd.current_dir(dir)
        .env("TSR_DB", dir.join("tsr.db"))
        .env_remove("TSR_SESSION")
        .env("TSR_EXEC_IMAGE", IMAGE);
    cmd
}
fn open(dir: &Path) {
    let mut child = command(dir)
        .args(["hook", "--stage", "SessionStart"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"session_id":"status-test","hook_event_name":"SessionStart"}"#)
        .unwrap();
    assert!(child.wait_with_output().unwrap().status.success());
}
fn kinds(dir: &Path) -> Vec<EventKind> {
    store_path::open_store_at(&dir.join("tsr.db"))
        .unwrap()
        .by_task(&store_path::task_for_session("status-test"))
        .unwrap()
        .into_iter()
        .map(|(_, e)| e.kind)
        .collect()
}
fn run(dir: &Path, args: &[&str]) -> Output {
    command(dir)
        .args(["exec", "--"])
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn refusal_statuses_remain_two_and_three_without_execution() {
    let dir = scratch("refusal");
    let empty = dir.join("empty-path");
    std::fs::create_dir_all(&empty).unwrap();
    let out = command(&dir)
        .args(["exec", "--", "command"])
        .env("PATH", &empty)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no session"));
    assert!(kinds(&dir).is_empty());
    open(&dir);
    let out = command(&dir)
        .args(["exec", "--", "command"])
        .env("PATH", &empty)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("no container runtime"));
    assert!(matches!(
        kinds(&dir).as_slice(),
        [EventKind::TaskOpened { .. }]
    ));
}
#[test]
fn real_command_statuses_and_startup_uncertainty_match_ledger() {
    if exec::Runtime::detect().is_none() {
        eprintln!("no runtime; real CLI status checks not executed");
        return;
    }
    for (name, args, expected) in [
        ("zero", vec!["sh", "-c", "exit 0"], 0),
        ("failure", vec!["sh", "-c", "exit 7"], 7),
        ("command127", vec!["sh", "-c", "exit 127"], 127),
        ("startup", vec!["tsr-no-such-executable"], 1),
    ] {
        let dir = scratch(name);
        open(&dir);
        let out = run(&dir, &args);
        assert_eq!(out.status.code(), Some(expected), "{name}: {out:?}");
        let events = kinds(&dir);
        if name == "startup" {
            assert!(String::from_utf8_lossy(&out.stderr).contains("execution unresolved"));
            assert!(events
                .iter()
                .any(|e| matches!(e, EventKind::Unresolved { .. })));
            assert!(!events
                .iter()
                .any(|e| matches!(e, EventKind::Executed { .. })));
        } else {
            assert!(events.iter().any(|e| matches!(e, EventKind::Executed { exit_code: Some(code), .. } if *code == expected)));
            assert!(!events
                .iter()
                .any(|e| matches!(e, EventKind::Unresolved { .. })));
        }
    }
}
