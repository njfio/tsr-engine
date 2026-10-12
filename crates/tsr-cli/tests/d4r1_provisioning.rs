//! D4, R1: `tsr task run` provisions a real copy from a batch-owned root, and a snapshot value
//! that is not a hex object id is refused before anything is mounted.
//!
//! Pre-fix, `run_one` ran provisioning with the working directory set to `task.snapshot()` — a
//! tree id, not a directory — so every task failed `provisioning failed … exited 127`, and
//! `provision_argv` passed no `--repo`, so `materialize` defaulted to `.`. The snapshot value from
//! `tasks.jsonl` also became `ExecRunner`'s working directory, so `"parent_snapshot":"/etc"` made
//! `/etc` the sandbox's writable mount (R1; PRD B10: the disposable copy is the only writable
//! mount).
//!
//! These tests drive the built `tsr` binary. The provisioning run itself goes through `tsr exec`
//! in the sealed sibling sandbox, which is absent here, so the run cannot complete end to end; what
//! is asserted is the part this task owns: the snapshot is validated before anything is mounted,
//! and the provisioning working directory is a batch-owned root, never the snapshot value.

use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

/// A task JSON object with the given id and snapshot.
fn task_json(id: &str, snapshot: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "repo": "rts",
        "parent": "a1b2c3",
        "commit": "d4e5f6",
        "parent_snapshot": snapshot,
        "flipped_tests": ["parser::tests::handles_empty_input"],
        "hidden_diff": "cas-diff-rts-1",
        "family": "parser",
        "lineage": "rts",
        "timestamp": "2026-01-01T00:00:00Z",
        "outcome": {
            "id": "0000000000000000000000000000000000000000000000000000000000000001",
            "name": "flip@v1",
            "version": 1,
            "horizon": "task_close",
            "resolves_true_when": "the flipped tests pass and the full suite does not regress",
            "censored_when": "the suite could not run",
            "intervention_rule": "the hidden diff stays hidden"
        }
    })
}

/// R1: a `tasks.jsonl` naming `/etc` as a snapshot exits 2 and mounts nothing.
///
/// The `claude-code` driver calls `run_one` directly per task, so the snapshot check fires before
/// any session is loaded — the refusal is the snapshot check, not a missing session.
#[test]
fn r1_etc_snapshot_exits_2_and_mounts_nothing() {
    let ws = workspace();
    let scratch = ws.join("target/d4r1-etc");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let manifest = scratch.join("tasks.jsonl");
    std::fs::write(&manifest, format!("{}\n", task_json("r1-etc", "/etc"))).unwrap();
    let out = scratch.join("out");

    let output = Command::new(tsr_bin())
        .args([
            "task",
            "run",
            "--mode",
            "rules",
            "--tasks",
            manifest.to_str().unwrap(),
            "--driver",
            "claude-code",
            "--out",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("run tsr");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "R1: an unsafe snapshot must exit 2, got {:?}; stderr: {stderr}",
        output.status.code()
    );
    assert!(
        stderr.contains("unsafe snapshot"),
        "R1: the refusal must name the unsafe snapshot; stderr: {stderr}"
    );
    // Nothing was mounted: no copy directory was created for the task.
    assert!(
        !out.join("copies").join("r1-etc").exists(),
        "R1: no copy may be mounted for an unsafe snapshot"
    );
}

/// D4: `tsr task run --mode record` without `--scripted-runner` no longer fails with the tree-id
/// working-directory error.
///
/// Pre-fix the provisioning run's working directory was the snapshot value (a tree id), which does
/// not exist, so the child could not start and the run failed `provisioning failed … exited 127`.
/// Post-fix the working directory is the batch-owned copies root, which exists, so the child starts
/// and the failure (if any) is the sandbox's own refusal, not a missing directory.
#[test]
fn d4_record_mode_provisions_from_a_batch_owned_root() {
    let ws = workspace();
    let scratch = ws.join("target/d4r1-record");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let manifest = scratch.join("tasks.jsonl");
    // A task whose snapshot is a real hex tree id, so it passes the R1 check and reaches
    // provisioning.
    std::fs::write(
        &manifest,
        format!(
            "{}\n",
            task_json("d4-record", "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        ),
    )
    .unwrap();
    // The scripted driver needs a session next to the manifest.
    let sessions = scratch.join("sessions");
    std::fs::create_dir_all(&sessions).unwrap();
    std::fs::copy(
        ws.join("fixtures/m3/batch/session-inspect-test-propose.json"),
        sessions.join("d4-record-record.json"),
    )
    .unwrap();
    let out = scratch.join("out");

    // `ExecRunner` invokes `tsr` on PATH; put the built binary's directory first so the
    // provisioning run can start. Without this the child cannot be found and the run fails 127 for
    // a reason unrelated to the working directory.
    let bin_dir = tsr_bin().parent().unwrap().to_path_buf();
    let path = std::env::var_os("PATH").unwrap_or_default();
    let mut paths = vec![bin_dir];
    paths.extend(std::env::split_paths(&path));
    let joined = std::env::join_paths(paths).unwrap();

    let output = Command::new(tsr_bin())
        .env("PATH", joined)
        .args([
            "task",
            "run",
            "--mode",
            "record",
            "--tasks",
            manifest.to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            out.to_str().unwrap(),
        ])
        .output()
        .expect("run tsr");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // The pre-fix failure: the provisioning working directory was the snapshot value (a tree id),
    // which does not exist, so the child could not start and the run failed `exited 127`. Post-fix
    // the working directory is the batch-owned copies root, which exists, so the child starts and
    // the failure (if any) is the sandbox's own refusal, never a missing directory.
    assert!(
        !stderr.contains("exited 127"),
        "D4: provisioning must not fail because the working directory does not exist; stderr: {stderr}"
    );
    assert!(
        !stderr.contains("could not run"),
        "D4: the provisioning child must be startable; stderr: {stderr}"
    );
    // The batch-owned copies root was created, so provisioning ran from a real directory.
    assert!(
        out.join("copies").is_dir(),
        "D4: the batch-owned copies root must exist after a run; stderr: {stderr}"
    );
}
