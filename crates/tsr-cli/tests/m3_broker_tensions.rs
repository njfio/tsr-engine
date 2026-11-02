//! Controller CLI wiring with inert worker/provider evidence, never live inference.
mod support;
use std::{fs, path::Path};
use tsr_domain::{Event, EventKind, Hash, Source, TaskId};

#[test]
fn reviewed_tensions_require_explicit_review_and_record_abstention() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("controller.db");
    let store = tsr_store::Store::open(&path).unwrap();
    let task = TaskId(Hash::of(&[b"cli-tensions"]));
    store
        .append(&Event::new(
            task,
            vec![],
            Source::Operator,
            EventKind::TaskOpened { task },
            "fixture",
        ))
        .unwrap();
    let input = dir.path().join("links.json");
    fs::write(&input, r#"{"open_criteria":[],"links":[]}"#).unwrap();
    let args = [
        "tensions",
        "--store",
        path.to_str().unwrap(),
        "--task",
        &task.0.to_hex(),
        "--reviewed-links",
        input.to_str().unwrap(),
        "--family",
        "fixture",
        "--repository",
        "fixture",
    ];
    let denied = std::process::Command::new(env!("CARGO_BIN_EXE_tsr"))
        .args(args)
        .output()
        .unwrap();
    assert!(!denied.status.success());
    assert_eq!(store.by_task(&task).unwrap().len(), 1);
    let accepted = std::process::Command::new(env!("CARGO_BIN_EXE_tsr"))
        .args(args)
        .arg("--operator-review")
        .output()
        .unwrap();
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    assert_eq!(store.by_task(&task).unwrap().len(), 3);
    assert!(path.with_extension("artifacts").is_dir());
}

#[test]
fn replay_broker_cli_runs_both_modes_without_live_provider() {
    let dir = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = fs::read_to_string(root.join("fixtures/m3/batch/tasks.jsonl")).unwrap();
    let task: serde_json::Value = serde_json::from_str(fixture.lines().next().unwrap()).unwrap();
    let manifest = dir.path().join("tasks.jsonl");
    fs::write(&manifest, format!("{}\n", task)).unwrap();
    support::repair_manifest(&manifest);
    fs::write(
        dir.path().join("broker-contexts.json"),
        r#"{"t1":{"required_reads":["tests/parser.rs"],"failing_test_id":"cargo-test-parser"}}"#,
    )
    .unwrap();
    fs::write(dir.path().join("broker-replay.json"), r#"[[{"action":"inspect","path":"tests/parser.rs"},null],[{"action":"test","id":"cargo-test-parser"},0.5],[{"action":"stop"},null]]"#).unwrap();
    let out = dir.path().join("out");
    for mode in ["record", "rules"] {
        let result = support::command(Path::new(env!("CARGO_BIN_EXE_tsr")), dir.path())
            .args([
                "task",
                "run",
                "--mode",
                mode,
                "--tasks",
                manifest.to_str().unwrap(),
                "--driver",
                "broker-replay",
                "--out",
                out.to_str().unwrap(),
                "--profile",
                root.join("profiles/default.toml").to_str().unwrap(),
            ])
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let rows = fs::read_to_string(out.join(format!("results-{mode}.jsonl"))).unwrap();
        assert_eq!(rows.lines().count(), 1);
        let row: serde_json::Value = serde_json::from_str(rows.lines().next().unwrap()).unwrap();
        assert_eq!(row["oracle"]["accepted"], true, "{row}");
    }
}
