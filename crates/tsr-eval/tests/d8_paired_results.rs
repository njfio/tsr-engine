//! D8: the documented `tsr eval baseline` invocation pairs by task id.
//!
//! The README and `docs/m3-evidence.md` document the two-run form
//!
//! ```sh
//! tsr eval baseline --record <dir> --rules <dir> --pair-by task --seed 1
//! ```
//!
//! where each directory holds one system's `results.jsonl`. This test drives
//! the built `tsr` binary with exactly that invocation over `fixtures/m3` and
//! checks that it pairs by task id and reports the unpaired task. Before the
//! D8 fix the CLI accepted only a single positional `<results-dir>`, so the
//! documented flags were rejected with exit 2.

use std::path::PathBuf;
use std::process::Command;

/// The repository root (CARGO_MANIFEST_DIR is crates/tsr-eval).
fn repo_root() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/
    p.pop(); // repo root
    p
}

/// The `tsr` binary built alongside this test.
fn tsr_bin() -> PathBuf {
    // The integration test binary lives in target/<profile>/deps; the CLI is
    // two levels up in target/<profile>/tsr.
    let mut p = std::env::current_exe().expect("current_exe");
    p.pop(); // deps/
    p.pop(); // <profile>/
    p.push("tsr");
    p
}

#[test]
fn d8_documented_record_rules_invocation_pairs_by_task() {
    let root = repo_root();
    let record = root.join("fixtures/m3/eval/record");
    let rules = root.join("fixtures/m3/eval/rules");
    let bin = tsr_bin();
    assert!(
        bin.exists(),
        "the tsr binary must be built before this test: {}",
        bin.display()
    );

    let out = Command::new(&bin)
        .args([
            "eval",
            "baseline",
            "--record",
            record.to_str().unwrap(),
            "--rules",
            rules.to_str().unwrap(),
            "--pair-by",
            "task",
            "--seed",
            "1",
        ])
        .output()
        .expect("spawn tsr eval baseline");

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "the documented invocation must succeed (exit {}):\nstdout:\n{stdout}\nstderr:\n{stderr}",
        out.status.code().unwrap_or(-1)
    );

    // Paired by task id: the two systems share rts-t1/rts-t2/kamn-t1/kamn-t2/
    // zeta-t1, and the rules run has one extra task (zeta-t2) the record run
    // lacks.
    assert!(
        stdout.contains("paired tasks: 5"),
        "the report must pair the five shared tasks:\n{stdout}"
    );
    assert!(
        stdout.contains("unpaired tasks: 1"),
        "the report must count the one unpaired task:\n{stdout}"
    );
    assert!(
        stdout.contains("zeta-t2 (only in rules)"),
        "the report must name the unpaired task and its system:\n{stdout}"
    );
    assert!(
        stdout.contains("pair-by: task"),
        "the report must echo the pairing rule:\n{stdout}"
    );
}

#[test]
fn d8_single_results_file_holding_both_systems_still_works() {
    // The one-file form (both systems in one results.jsonl) must keep working.
    let root = repo_root();
    let dir = root.join("fixtures/m3/eval/clear");
    let bin = tsr_bin();
    let out = Command::new(&bin)
        .args([
            "eval",
            "baseline",
            dir.to_str().unwrap(),
            "--pair-by",
            "task",
            "--seed",
            "1",
        ])
        .output()
        .expect("spawn tsr eval baseline");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "the one-file form must succeed:\nstdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        stdout.contains("paired tasks: 6"),
        "the one-file form must pair the six shared tasks:\n{stdout}"
    );
}
