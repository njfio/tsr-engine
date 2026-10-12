//! D8: the operator's sequence runs — record then rules into one `--out`, then
//! `tsr eval baseline <out>` pairs every task.
//!
//! Before the fix `tsr task run` wrote a single `<out>/results.jsonl`, so a second
//! mode into the same `--out` overwrote the first run's rows and
//! `tsr eval baseline <out>` reported `paired tasks: 0, unpaired: 5`. The fix
//! writes one file per mode (`results-record.jsonl`, `results-rules.jsonl`) and
//! the one-directory form of `tsr eval baseline` reads both.

use std::path::PathBuf;
use std::process::Command;

/// The repository root (CARGO_MANIFEST_DIR is crates/tsr-cli).
fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/
    p.pop(); // repo root
    p
}

/// The `tsr` binary built alongside this test.
fn tsr_bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("current_exe");
    p.pop(); // deps/
    p.pop(); // <profile>/
    p.push("tsr");
    p
}

/// Five non-mutant tasks, so the scripted runner (which does not materialise a
/// copy) can run them; the mutant tasks' diff application is D4's concern.
fn five_tasks() -> String {
    let fixture = std::fs::read_to_string(repo_root().join("fixtures/m3/batch/tasks.jsonl"))
        .expect("read the fixture tasks");
    let mut rows: Vec<serde_json::Value> = fixture
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("parse a fixture task"))
        .collect();
    // Keep the three non-mutant tasks and add two more of the same shape.
    rows.retain(|r| r.get("parent_snapshot").is_some());
    let base = rows[0].clone();
    for (id, snap) in [
        ("t4", "d4e5f6a7b8c9d4e5f6a7b8c9d4e5f6a7b8c9d4"),
        ("t5", "e5f6a7b8c9d0e5f6a7b8c9d0e5f6a7b8c9d0e5"),
    ] {
        let mut r = base.clone();
        r["id"] = serde_json::Value::String(id.to_string());
        r["parent_snapshot"] = serde_json::Value::String(snap.to_string());
        rows.push(r);
    }
    assert_eq!(rows.len(), 5, "five tasks");
    rows.iter()
        .map(|r| serde_json::to_string(r).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

#[test]
fn d8_record_then_rules_into_one_out_pairs_all_five_tasks() {
    let bin = tsr_bin();
    assert!(
        bin.exists(),
        "the tsr binary must be built before this test: {}",
        bin.display()
    );
    let dir = tempfile::tempdir().unwrap();
    let tasks = dir.path().join("tasks.jsonl");
    std::fs::write(&tasks, five_tasks()).unwrap();
    // The scripted driver replays a session found next to the manifest; copy the
    // fixture's well-formed default session there.
    std::fs::copy(
        repo_root().join("fixtures/m3/batch/session-inspect-test-propose.json"),
        dir.path().join("session-inspect-test-propose.json"),
    )
    .unwrap();
    let out = dir.path().join("out");
    std::fs::create_dir_all(&out).unwrap();

    // The operator's sequence: record, then rules, into the same --out.
    for mode in ["record", "rules"] {
        let run = Command::new(&bin)
            .args([
                "task",
                "run",
                "--mode",
                mode,
                "--tasks",
                tasks.to_str().unwrap(),
                "--driver",
                "scripted",
                "--out",
                out.to_str().unwrap(),
                "--scripted-runner",
            ])
            .output()
            .expect("spawn tsr task run");
        assert!(
            run.status.success(),
            "tsr task run --mode {mode} must succeed (exit {}):\nstdout:\n{}\nstderr:\n{}",
            run.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr),
        );
    }

    // Both modes' rows must survive in the one --out.
    assert!(
        out.join("results-record.jsonl").is_file(),
        "the record run must write results-record.jsonl"
    );
    assert!(
        out.join("results-rules.jsonl").is_file(),
        "the rules run must write results-rules.jsonl"
    );

    // The one-directory form of eval must read what task run wrote.
    let eval = Command::new(&bin)
        .args([
            "eval",
            "baseline",
            out.to_str().unwrap(),
            "--pair-by",
            "task",
            "--seed",
            "1",
        ])
        .output()
        .expect("spawn tsr eval baseline");
    let stdout = String::from_utf8_lossy(&eval.stdout);
    let stderr = String::from_utf8_lossy(&eval.stderr);
    assert!(
        eval.status.success(),
        "tsr eval baseline <out> must succeed (exit {}):\nstdout:\n{stdout}\nstderr:\n{stderr}",
        eval.status.code().unwrap_or(-1)
    );
    assert!(
        stdout.contains("paired tasks: 5"),
        "the one-directory form must pair all five tasks:\n{stdout}"
    );
    assert!(
        !stdout.contains("unpaired tasks:"),
        "no task may be left unpaired:\n{stdout}"
    );
}
