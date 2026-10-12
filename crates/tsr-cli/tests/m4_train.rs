//! M4 train test (lane 06, item 2/4/7/8): the `tsr train` pipeline end to end over `fixtures/m4/`.
//!
//! The corpus is regenerated into a temp dir (the generator is seeded and clock-free), then the
//! binary is driven through `dataset` → `fit` → `verify` → `report`. The test asserts the pipeline
//! runs, that the dataset is reproducible (same data root and split digest twice), that a
//! non-operator label is refused with exit 2, and that a bundle verifies.
//!
//! It does not re-assert the corpus's byte-identity or the reconstruct invariants — `m4_corpus.rs`
//! owns those.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The `tsr` binary cargo built for this test.
fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

/// A fresh temp directory for this test run, unique per process and tag.
fn temp_dir(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("tsr-m4-train-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("a temp dir is creatable");
    base
}

/// Regenerate the corpus into `dir` and return `(events, tasks, labels)`.
fn regenerate(dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let db = dir.join("corpus.db");
    let out = dir.join("fixtures");
    tsr_learn::synth::generate_fixtures(&db, &out).expect("the corpus generates");
    (
        out.join("events.jsonl"),
        out.join("tasks.jsonl"),
        out.join("labels.jsonl"),
    )
}

/// Run `tsr <args>` and return `(exit_code, stdout, stderr)`.
fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(tsr_bin())
        .args(args)
        .output()
        .expect("the tsr binary runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Parse a `key=value` line out of the dataset command's stdout.
fn field(stdout: &str, key: &str) -> String {
    stdout
        .lines()
        .find_map(|l| l.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("no `{key}=` line in:\n{stdout}"))
        .trim()
        .to_string()
}

#[test]
fn the_train_pipeline_runs_end_to_end() {
    let dir = temp_dir("pipeline");
    let (events, tasks, labels) = regenerate(&dir);

    // dataset
    let ds = dir.join("dataset");
    let (code, stdout, stderr) = run(&[
        "train",
        "dataset",
        "--events",
        events.to_str().unwrap(),
        "--tasks",
        tasks.to_str().unwrap(),
        "--labels",
        labels.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.25",
        "--holdout",
        "0.25",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "dataset failed:\n{stdout}\n{stderr}");
    let rows: usize = field(&stdout, "rows").parse().expect("rows is a number");
    assert!(rows > 0, "the dataset has rows:\n{stdout}");
    let data_root = field(&stdout, "data_root");
    let split_digest = field(&stdout, "split_digest");
    assert!(!data_root.is_empty() && !split_digest.is_empty());

    // dataset twice: same data root and split digest
    let ds2 = dir.join("dataset2");
    let (code2, stdout2, stderr2) = run(&[
        "train",
        "dataset",
        "--events",
        events.to_str().unwrap(),
        "--tasks",
        tasks.to_str().unwrap(),
        "--labels",
        labels.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.25",
        "--holdout",
        "0.25",
        "--seed",
        "7",
        "--out",
        ds2.to_str().unwrap(),
    ]);
    assert_eq!(code2, 0, "dataset (2) failed:\n{stdout2}\n{stderr2}");
    assert_eq!(
        field(&stdout2, "data_root"),
        data_root,
        "data root is reproducible"
    );
    assert_eq!(
        field(&stdout2, "split_digest"),
        split_digest,
        "split digest is reproducible"
    );

    // fit
    let bundle = dir.join("models").join("m4");
    let (code, stdout, stderr) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        bundle.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "fit failed:\n{stdout}\n{stderr}");
    let bundle_id = stdout.trim();
    assert!(!bundle_id.is_empty(), "fit prints the bundle id:\n{stdout}");

    // verify
    let (code, stdout, stderr) = run(&["train", "verify", bundle.to_str().unwrap()]);
    assert_eq!(code, 0, "verify failed:\n{stdout}\n{stderr}");

    // report
    let (code, stdout, stderr) = run(&[
        "train",
        "report",
        bundle.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "report failed:\n{stdout}\n{stderr}");
    assert!(
        stdout.contains("brier"),
        "the report names the brier proxy:\n{stdout}"
    );
}

#[test]
fn a_non_operator_label_is_refused_with_exit_2() {
    let dir = temp_dir("nonop");
    let (events, tasks, labels) = regenerate(&dir);

    // Rewrite the labels so the first record names a non-operator reviewer.
    let text = std::fs::read_to_string(&labels).expect("labels.jsonl is readable");
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    assert!(!lines.is_empty(), "the corpus has labels");
    let mut first: serde_json::Value = serde_json::from_str(&lines[0]).expect("a label is JSON");
    first["reviewer"] = serde_json::Value::String("generator".to_string());
    lines[0] = serde_json::to_string(&first).expect("re-serializes");
    let bad = dir.join("labels-bad.jsonl");
    std::fs::write(&bad, lines.join("\n") + "\n").expect("writes");

    let ds = dir.join("dataset");
    let (code, stdout, stderr) = run(&[
        "train",
        "dataset",
        "--events",
        events.to_str().unwrap(),
        "--tasks",
        tasks.to_str().unwrap(),
        "--labels",
        bad.to_str().unwrap(),
        "--by",
        "time",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "a non-operator label is refused:\n{stdout}\n{stderr}"
    );
    assert!(
        stderr.contains("operator"),
        "the refusal names the operator rule:\n{stderr}"
    );
}

#[test]
fn f15_a_different_seed_splits_differently_under_by_repo() {
    let dir = temp_dir("seed");
    let (events, tasks, labels) = regenerate(&dir);

    let mut digests = Vec::new();
    for seed in ["1", "2"] {
        let ds = dir.join(format!("dataset-{seed}"));
        let (code, stdout, stderr) = run(&[
            "train",
            "dataset",
            "--events",
            events.to_str().unwrap(),
            "--tasks",
            tasks.to_str().unwrap(),
            "--labels",
            labels.to_str().unwrap(),
            "--by",
            "repo",
            "--seed",
            seed,
            "--out",
            ds.to_str().unwrap(),
        ]);
        assert_eq!(code, 0, "dataset seed {seed} failed:\n{stdout}\n{stderr}");
        digests.push(field(&stdout, "split_digest"));
    }
    assert_ne!(
        digests[0], digests[1],
        "a different seed splits differently"
    );
}

/// Build a dataset from the regenerated corpus into `dir/ds` and return its path.
fn build_dataset(dir: &Path, seed: &str) -> PathBuf {
    let (ev, tk, lb) = regenerate(dir);
    let ds = dir.join("ds");
    let (code, _out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        tk.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        seed,
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "dataset failed: {err}");
    ds
}

/// Fit a bundle from `ds` into `dir/models/m4` and return its path.
fn fit_bundle(dir: &Path, ds: &Path) -> PathBuf {
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "fit failed: {err}");
    assert!(!out.trim().is_empty(), "fit prints the bundle id");
    b
}

/// §B8's minimum data: `fit` exits 2 with the counts when fit+calibrate hold fewer than 150 tasks.
#[test]
fn fit_refuses_fewer_than_150_mature_tasks() {
    let dir = temp_dir("min-data");
    let (ev, tk, lb) = regenerate(&dir);
    // Keep only the first 40 tasks' events, so fit+calibrate hold far fewer than 150 tasks.
    let keep: std::collections::HashSet<String> = std::fs::read_to_string(&tk)
        .unwrap()
        .lines()
        .take(40)
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            v["task"].as_str().unwrap().to_string()
        })
        .collect();
    let small_ev = dir.join("small-events.jsonl");
    let small_tk = dir.join("small-tasks.jsonl");
    let small_lb = dir.join("small-labels.jsonl");
    let ev_text: String = std::fs::read_to_string(&ev)
        .unwrap()
        .lines()
        .filter(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            // An events.jsonl line is `{"seq": n, "event": {...}}`; the task id is inside `event`.
            keep.contains(v["event"]["task"].as_str().unwrap_or(""))
        })
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&small_ev, ev_text).unwrap();
    std::fs::write(
        &small_tk,
        std::fs::read_to_string(&tk)
            .unwrap()
            .lines()
            .filter(|l| {
                let v: serde_json::Value = serde_json::from_str(l).unwrap();
                keep.contains(v["task"].as_str().unwrap_or(""))
            })
            .map(|l| format!("{l}\n"))
            .collect::<String>(),
    )
    .unwrap();
    std::fs::write(
        &small_lb,
        std::fs::read_to_string(&lb)
            .unwrap()
            .lines()
            .filter(|l| {
                let v: serde_json::Value = serde_json::from_str(l).unwrap();
                keep.contains(v["task"].as_str().unwrap_or(""))
            })
            .map(|l| format!("{l}\n"))
            .collect::<String>(),
    )
    .unwrap();
    let ds = dir.join("small-ds");
    let (code, _, err) = run(&[
        "train",
        "dataset",
        "--events",
        small_ev.to_str().unwrap(),
        "--tasks",
        small_tk.to_str().unwrap(),
        "--labels",
        small_lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "small dataset failed: {err}");
    let (code, _, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        dir.join("models").join("small").to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "fit must refuse fewer than 150 mature tasks: {err}"
    );
    assert!(
        err.contains("mature") && err.contains("150"),
        "the refusal names the counts: {err}"
    );
}

/// §B8's per-family minimum: a family with fewer than 30 tasks stays outside the population, so its
/// rows are reported `uncalibrated` and never get a fabricated number.
#[test]
fn a_family_below_30_tasks_is_uncalibrated() {
    let dir = temp_dir("rare-family");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);
    let (code, out, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 0, "report failed: {err}");
    // The `rare` family has 20 tasks, below the 30 floor: it is not in the population.
    let pop: String = out
        .lines()
        .skip_while(|l| !l.starts_with("population"))
        .take_while(|l| !l.starts_with("floor sweep"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !pop.contains("family rare"),
        "rare must stay outside the population:\n{pop}"
    );
    assert!(
        out.contains("uncalibrated"),
        "the report counts uncalibrated rows:\n{out}"
    );
}

/// A one-byte change to a bundle content file makes `verify` exit 2 naming the file.
#[test]
fn a_one_byte_bundle_change_is_refused_naming_the_file() {
    let dir = temp_dir("one-byte");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);
    let (code, _, err) = run(&["train", "verify", b.to_str().unwrap()]);
    assert_eq!(code, 0, "a fresh bundle verifies: {err}");
    // Flip one byte of head.json.
    let head = b.join("head.json");
    let mut bytes = std::fs::read(&head).unwrap();
    let last = bytes.len() - 2;
    bytes[last] ^= 0x01;
    std::fs::write(&head, &bytes).unwrap();
    let (code, _, err) = run(&["train", "verify", b.to_str().unwrap()]);
    assert_eq!(code, 2, "a changed bundle is refused: {err}");
    assert!(
        err.contains("head.json"),
        "the refusal names the file whose digest differs: {err}"
    );
}

/// §B7: the paired interval is INCONCLUSIVE when it spans zero. The `calibrated` family's generator
/// is honest, so the controller's advantage there is small and the interval straddles zero.
#[test]
fn a_paired_interval_that_spans_zero_is_inconclusive() {
    let dir = temp_dir("inconclusive");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);
    let (code, out, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 0, "report failed: {err}");
    assert!(
        out.contains("INCONCLUSIVE"),
        "a straddling interval is INCONCLUSIVE:\n{out}"
    );
}

/// §B9: every read of a held_out row appends a line to `<dir>/holdout-access.jsonl`.
#[test]
fn held_out_reads_are_logged() {
    let dir = temp_dir("holdout-log");
    let ds = build_dataset(&dir, "7");
    let log = ds.join("holdout-access.jsonl");
    assert!(log.exists(), "the dataset writes the holdout-access log");
    let before = std::fs::read_to_string(&log).unwrap().lines().count();
    assert!(before > 0, "the split itself reads held_out rows");
    let b = fit_bundle(&dir, &ds);
    let (code, _, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 0, "report failed: {err}");
    let after = std::fs::read_to_string(&log).unwrap().lines().count();
    assert!(
        after > before,
        "scoring held_out rows appends to the log ({before} -> {after})"
    );
}

/// §B8/§B12: no output contains "percent certain" or a TSR term followed by a number; a calibrated
/// number prints as `p` with its calibrator version, a raw one as `raw p`.
#[test]
fn the_report_uses_engineering_names_and_a_calibrator_version() {
    let dir = temp_dir("names");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);
    let (code, out, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 0, "report failed: {err}");
    assert!(
        !out.to_lowercase().contains("percent certain"),
        "no 'percent certain' in the report:\n{out}"
    );
    assert!(
        out.contains("features@v1"),
        "the calibrator version is printed:\n{out}"
    );
    assert!(
        out.contains("brier") && out.contains("log_loss"),
        "the engineering names are printed:\n{out}"
    );
}

/// Run the binary with an extra environment variable set.
fn run_env(args: &[&str], key: &str, value: &str) -> (i32, String, String) {
    let out = Command::new(tsr_bin())
        .args(args)
        .env(key, value)
        .output()
        .expect("the tsr binary runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// B16: a different seed changes the bundle id (the split, and so the fitted head, differ).
#[test]
fn b16_a_different_seed_changes_the_bundle_id() {
    let dir = temp_dir("b16-seed");
    let (ev, tk, lb) = regenerate(&dir);
    let mut ids = Vec::new();
    for seed in ["1", "2"] {
        let ds = dir.join(format!("ds-{seed}"));
        let (code, out, err) = run(&[
            "train",
            "dataset",
            "--events",
            ev.to_str().unwrap(),
            "--tasks",
            tk.to_str().unwrap(),
            "--labels",
            lb.to_str().unwrap(),
            "--by",
            "repo",
            "--calibrate",
            "0.2",
            "--holdout",
            "0.2",
            "--seed",
            seed,
            "--out",
            ds.to_str().unwrap(),
        ]);
        assert_eq!(code, 0, "dataset seed {seed} failed:\n{out}\n{err}");
        let b = dir.join(format!("models-{seed}"));
        let (code, out, err) = run(&[
            "train",
            "fit",
            ds.to_str().unwrap(),
            "--out",
            b.to_str().unwrap(),
        ]);
        assert_eq!(code, 0, "fit seed {seed} failed:\n{out}\n{err}");
        ids.push(out.trim().to_string());
    }
    assert_ne!(
        ids[0], ids[1],
        "B16: a different seed changes the bundle id"
    );
}

/// B16: `TSR_CODE_COMMIT` is recorded but does not change the bundle id.
#[test]
fn b16_tsr_code_commit_leaves_the_bundle_id_unchanged() {
    let dir = temp_dir("b16-commit");
    let ds = build_dataset(&dir, "7");
    let b1 = dir.join("models-a");
    let (code, out, err) = run_env(
        &[
            "train",
            "fit",
            ds.to_str().unwrap(),
            "--out",
            b1.to_str().unwrap(),
        ],
        "TSR_CODE_COMMIT",
        "deadbeef",
    );
    assert_eq!(code, 0, "fit failed:\n{out}\n{err}");
    let id1 = out.trim().to_string();
    let b2 = dir.join("models-b");
    let (code, out, err) = run_env(
        &[
            "train",
            "fit",
            ds.to_str().unwrap(),
            "--out",
            b2.to_str().unwrap(),
        ],
        "TSR_CODE_COMMIT",
        "cafebabe",
    );
    assert_eq!(code, 0, "fit failed:\n{out}\n{err}");
    let id2 = out.trim().to_string();
    assert_eq!(
        id1, id2,
        "B16: TSR_CODE_COMMIT leaves the bundle id unchanged"
    );
}

/// B16: no report or apply output holds "percent certain" or a TSR term followed by a number.
#[test]
fn b16_no_forbidden_strings_in_report_or_apply() {
    let dir = temp_dir("b16-strings");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);
    let (code, report_out, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 0, "report failed: {err}");
    let (code, apply_out, err) = run(&[
        "train",
        "apply",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "apply failed: {err}");
    for (label, out) in [("report", &report_out), ("apply", &apply_out)] {
        assert!(
            !out.to_lowercase().contains("percent certain"),
            "B16: no 'percent certain' in {label}:\n{out}"
        );
        // A TSR term followed by a number, e.g. "TSR-3" or "tsr 3".
        let lower = out.to_lowercase();
        for (i, _) in lower.match_indices("tsr") {
            let rest = &lower[i + 3..];
            let rest = rest.trim_start_matches(['-', '_', ' ']);
            if rest.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                panic!("B16: a TSR term followed by a number in {label}:\n{out}");
            }
        }
    }
}

/// §B7/§B8, B4/B5/B6, DoD: the paired difference (controller minus generator) on held_out rows,
/// per task, with the 90% cluster-bootstrap interval over lineage.
///
/// On the honest corpus (items (2) and (7): the generator's p and the label share one state, and
/// every forecaster is scored on the same comparison set with the task as the unit) the controller's
/// advantage is NOT conclusive: the point estimate is negative (the controller is better on average)
/// but the interval spans zero, overall and for the overconfident family. The calibrated family is
/// conclusive and POSITIVE — the controller is worse there. This test asserts those measured numbers
/// rather than the pre-repair claim that the controller beats the generator conclusively; the DoD
/// allows exactly this ("if that no longer holds once the corpus and the scoring are honest, the test
/// is changed to assert what the corrected numbers show"), and docs/m4-evidence.md says so plainly.
#[test]
fn the_controller_beats_the_generator_on_held_out() {
    let dir = temp_dir("beats");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);
    let (code, out, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 0, "report failed: {err}");
    let paired: String = out
        .lines()
        .skip_while(|l| !l.starts_with("paired controller"))
        .take_while(|l| !l.starts_with("population"))
        .collect::<Vec<_>>()
        .join("\n");

    // Parse `mean=brier=<m> [interval_lo=<lo>, interval_hi=<hi>] <verdict>` for one family key.
    let parse = |key: &str| -> (f64, f64, f64, String) {
        let line = paired
            .lines()
            .find(|l| l.contains(key))
            .unwrap_or_else(|| panic!("no `{key}` line in:\n{paired}"));
        let mean: f64 = line
            .split("mean=brier=")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| panic!("no mean in `{line}`"));
        let lo: f64 = line
            .split("interval_lo=")
            .nth(1)
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or_else(|| panic!("no interval_lo in `{line}`"));
        let hi: f64 = line
            .split("interval_hi=")
            .nth(1)
            .and_then(|s| s.split(']').next())
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or_else(|| panic!("no interval_hi in `{line}`"));
        let verdict = if line.contains("INCONCLUSIVE") {
            "INCONCLUSIVE"
        } else if line.contains("conclusive") {
            "conclusive"
        } else {
            "interval n/a"
        };
        (mean, lo, hi, verdict.to_string())
    };

    // Overall: the controller is better on average (mean < 0) but the interval spans zero.
    let (mean, lo, hi, verdict) = parse("all tasks=");
    assert!(
        mean < 0.0,
        "the controller is better on average overall, got mean={mean}"
    );
    assert!(
        lo < 0.0 && hi > 0.0,
        "the overall interval spans zero on the honest corpus, got [{lo}, {hi}]"
    );
    assert_eq!(verdict, "INCONCLUSIVE", "overall verdict");

    // The overconfident family: same shape — better on average, not conclusive.
    let (mean, lo, hi, verdict) = parse("overconfident tasks=");
    assert!(
        mean < 0.0,
        "the controller is better on average for the overconfident family, got mean={mean}"
    );
    assert!(
        lo < 0.0 && hi > 0.0,
        "the overconfident interval spans zero on the honest corpus, got [{lo}, {hi}]"
    );
    assert_eq!(verdict, "INCONCLUSIVE", "overconfident verdict");

    // The calibrated family is conclusive and positive: the controller is worse there.
    let (mean, lo, hi, verdict) = parse("calibrated tasks=");
    assert!(
        mean > 0.0 && lo > 0.0,
        "the calibrated family's interval is wholly above zero, got mean={mean} [{lo}, {hi}]"
    );
    assert_eq!(verdict, "conclusive", "calibrated verdict");
}

/// R4: `tsr train apply` must refuse a dataset that does not belong to the bundle, exactly as
/// `report` does — a foreign split digest or feature spec is not the bundle's evaluation set.
#[test]
fn apply_refuses_a_foreign_dataset() {
    let dir = temp_dir("apply-foreign");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);

    // A dataset built from a different seed has a different split digest.
    let other = dir.join("other");
    let (ev, tk, lb) = regenerate(&dir);
    let (code, _, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        tk.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        "8",
        "--out",
        other.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "foreign dataset builds: {err}");

    let (code, out, err) = run(&[
        "train",
        "apply",
        b.to_str().unwrap(),
        "--dataset",
        other.to_str().unwrap(),
        "--side",
        "held_out",
    ]);
    assert_eq!(
        code, 2,
        "R4: apply refuses a foreign dataset:\n{out}\n{err}"
    );
    assert!(
        err.contains("split digest") || err.contains("feature"),
        "R4: the refusal names the split digest or feature spec: {err}"
    );
}

/// F14: `report` and `apply` exit 2 (not 1) on a tampered bundle, and a bad row is named by an id
/// that appears in rows.jsonl.
#[test]
fn f14_report_and_apply_exit_2_on_tampered_inputs() {
    let dir = temp_dir("f14-exit");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);

    // A forged calibrator in manifest.toml: report and apply must both refuse with exit 2.
    let manifest = b.join("manifest.toml");
    let text = std::fs::read_to_string(&manifest).unwrap();
    let forged: String = text
        .lines()
        .map(|l| {
            if l.trim_start().starts_with("calibrator =") {
                "calibrator = \"000000000000 features@v9\"".to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&manifest, forged).unwrap();

    let (code, _, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 2, "F14: report exits 2 on a forged calibrator: {err}");
    assert!(
        err.contains("manifest.toml"),
        "F14: report names manifest.toml: {err}"
    );

    let (code, _, err) = run(&[
        "train",
        "apply",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--side",
        "held_out",
    ]);
    assert_eq!(code, 2, "F14: apply exits 2 on a forged calibrator: {err}");
    assert!(
        err.contains("manifest.toml"),
        "F14: apply names manifest.toml: {err}"
    );
}

/// F7/F14: editing a row's `family` in rows.jsonl makes `report` exit 2, naming the row by an id
/// that appears in rows.jsonl.
#[test]
fn f14_a_bad_row_is_named_by_an_id_in_rows_jsonl() {
    let dir = temp_dir("f14-row");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);

    let rows = ds.join("rows.jsonl");
    let text = std::fs::read_to_string(&rows).unwrap();
    let mut out = String::new();
    let mut first_id = String::new();
    for line in text.lines() {
        let mut v: serde_json::Value = serde_json::from_str(line).unwrap();
        if first_id.is_empty() {
            first_id = v["id"].as_str().unwrap().to_string();
            v["family"] = serde_json::Value::String("forged-family".to_string());
        }
        out.push_str(&serde_json::to_string(&v).unwrap());
        out.push('\n');
    }
    std::fs::write(&rows, out).unwrap();

    let (code, _, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 2, "F14: report exits 2 on a tampered row: {err}");
    assert!(
        err.contains(&first_id),
        "F14: the bad row is named by its id {first_id}: {err}"
    );
}

/// F3/R2: `fit` must refuse an empty calibrate slice (exit 2, naming "calibrate") rather than
/// writing an empty calibrator that acts as the identity and labelling raw output Calibrated.
#[test]
fn f3_fit_refuses_an_empty_calibrate_slice() {
    let dir = temp_dir("f3-cal");
    let (ev, tk, lb) = regenerate(&dir);
    let ds = dir.join("ds");
    let (code, _out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        tk.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.0",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "dataset failed: {err}");
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F3/R2: fit refuses an empty calibrate slice:\n{out}\n{err}"
    );
    assert!(
        err.contains("calibrate"),
        "F3/R2: the refusal names the empty calibrate slice: {err}"
    );
}

/// F3/R2: `fit` must refuse an empty fit slice (exit 2, naming "fit") rather than fitting a
/// zero-weight head that prints every row "calibrated" at the same p.
#[test]
fn f3_fit_refuses_an_empty_fit_slice() {
    let dir = temp_dir("f3-fit");
    let (ev, tk, lb) = regenerate(&dir);
    let ds = dir.join("ds");
    let (code, _out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        tk.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.8",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "dataset failed: {err}");
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F3/R2: fit refuses an empty fit slice:\n{out}\n{err}"
    );
    assert!(
        err.contains("fit"),
        "F3/R2: the refusal names the empty fit slice: {err}"
    );
}

/// F9/B3: an empty `tasks.jsonl` in the dataset directory must make `fit` exit 2 naming the file,
/// not silently turn probe (a) off and accept a leaking dataset.
#[test]
fn f9_fit_refuses_an_empty_tasks_index() {
    let dir = temp_dir("f9-tasks");
    let ds = build_dataset(&dir, "7");
    std::fs::write(ds.join("tasks.jsonl"), "").unwrap();
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: fit refuses an empty tasks.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
}

/// F9/B3: an empty `events.jsonl` must make `fit` exit 2 naming the file, not panic (exit 101).
#[test]
fn f9_fit_refuses_an_empty_events_log() {
    let dir = temp_dir("f9-events");
    let ds = build_dataset(&dir, "7");
    std::fs::write(ds.join("events.jsonl"), "").unwrap();
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: fit refuses an empty events.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
}

/// R6: `--tasks` is required by the brief; `dataset` without it must exit 2, not silently make
/// every task its own lineage with family `unknown`.
#[test]
fn r6_dataset_requires_tasks() {
    let dir = temp_dir("r6-tasks");
    let (ev, _tk, lb) = regenerate(&dir);
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "R6: dataset requires --tasks:\n{out}\n{err}");
    assert!(
        err.contains("--tasks"),
        "R6: the refusal names --tasks: {err}"
    );
}

/// A8: a `--tasks` index truncated to the first 300 lines drops tasks with no index row; the count
/// must appear in `dataset`'s stdout, in `data-manifest.json` and in the report.
#[test]
fn a8_unindexed_tasks_are_counted() {
    let dir = temp_dir("a8-unindexed");
    let (ev, tk, lb) = regenerate(&dir);
    let short = dir.join("tasks-short.jsonl");
    let text = std::fs::read_to_string(&tk).unwrap();
    let kept: String = text.lines().take(300).collect::<Vec<_>>().join("\n") + "\n";
    std::fs::write(&short, kept).unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        short.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "dataset failed: {err}");
    let n: usize = field(&out, "unindexed_tasks")
        .parse()
        .expect("unindexed_tasks is a number");
    assert!(n > 0, "A8: dataset counts the dropped tasks: {out}");
    let manifest = std::fs::read_to_string(ds.join("data-manifest.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    assert_eq!(
        v["unindexed_tasks"].as_u64(),
        Some(n as u64),
        "A8: data-manifest.json carries the count: {manifest}"
    );

    // A8: the report carries the count too.
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "A8: fit accepts the truncated dataset: {out}\n{err}"
    );
    let (code, out, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 0, "A8: report runs: {out}\n{err}");
    let line = out
        .lines()
        .find(|l| l.starts_with("unindexed_tasks "))
        .unwrap_or_else(|| panic!("A8: the report carries the count: {out}"));
    assert_eq!(
        line.split_whitespace().nth(1),
        Some(n.to_string().as_str()),
        "A8: the report carries the count: {out}"
    );
}

/// F8: `dataset` must not assign a side to an index task that has no gated commitment (no rows);
/// it counts such tasks and continues, so `fit` accepts the tool's own output. Before the fix,
/// `dataset` exits 0 but `fit` refuses with `probe caught a leak: boundary`.
#[test]
fn f8_fit_accepts_a_dataset_whose_index_names_a_task_with_no_commitment() {
    let dir = temp_dir("f8-nogate");
    let (ev, tk, lb) = regenerate(&dir);
    // Append an index row for a task that appears in no event: no gated commitment, no rows.
    let mut text = std::fs::read_to_string(&tk).unwrap();
    let extra = serde_json::json!({
        "task": "00000000000000000000000000000000000000000000000000000000000000ff",
        "family": "calibrated",
        "lineage": "lineage-99",
        "timestamp": "2026-01-01T00:00:00Z",
    });
    text.push_str(&serde_json::to_string(&extra).unwrap());
    text.push('\n');
    let tk2 = dir.join("tasks-extra.jsonl");
    std::fs::write(&tk2, text).unwrap();

    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        tk2.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "F8: dataset accepts an index task with no commitment:\n{out}\n{err}"
    );
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "F8: fit accepts dataset's own output when the index names a task with no gated commitment:\n{out}\n{err}"
    );
}

/// F8/R5: under `--by time` a clone group goes to the side of its earliest member, and probe (c)
/// accepts exactly that. Before the fix, `dataset` refuses its own split when a clone group's
/// members have different timestamps.
#[test]
fn f8_dataset_accepts_a_clone_group_spanning_time() {
    let dir = temp_dir("f8-clone-time");
    let (ev, tk, lb) = regenerate(&dir);
    // Give the first two tasks one clone group; they have different timestamps.
    let text = std::fs::read_to_string(&tk).unwrap();
    let mut lines: Vec<serde_json::Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    assert!(lines.len() >= 2, "the corpus has at least two tasks");
    // The earliest and the latest task in one clone group: the unit's earliest timestamp is the
    // corpus's earliest, so the whole unit goes to the fit side, and the latest member's own
    // timestamp is far past the fit/calibrate boundary.
    let last = lines.len() - 1;
    lines[0]["clone_group"] = serde_json::Value::String("cg-span".to_string());
    lines[last]["clone_group"] = serde_json::Value::String("cg-span".to_string());
    let mut out = String::new();
    for v in &lines {
        out.push_str(&serde_json::to_string(v).unwrap());
        out.push('\n');
    }
    let tk2 = dir.join("tasks-clone.jsonl");
    std::fs::write(&tk2, out).unwrap();

    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        tk2.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "F8/R5: dataset accepts its own split when a clone group spans time:\n{out}\n{err}"
    );
}

/// Run the binary with `TSR_CODE_COMMIT` removed from the environment.
fn run_no_commit(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(tsr_bin())
        .args(args)
        .env_remove("TSR_CODE_COMMIT")
        .output()
        .expect("the tsr binary runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// P1: `code_commit` is recorded provenance, not a verified property. An untouched bundle verifies
/// with `TSR_CODE_COMMIT` unset, equal to the recorded value, and different from it; `verify` prints
/// the value labelled `code_commit (recorded, not verified)=<value>`.
#[test]
fn p1_verify_ignores_the_code_commit_environment() {
    let dir = temp_dir("p1-commit");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);

    // Unset: the bundle still verifies, and the label is printed.
    let (code, out, err) = run_no_commit(&["train", "verify", b.to_str().unwrap()]);
    assert_eq!(code, 0, "P1: verify with TSR_CODE_COMMIT unset: {err}");
    assert!(
        out.contains("code_commit (recorded, not verified)="),
        "P1: verify prints the labelled code_commit: {out}"
    );

    // Equal to the recorded value.
    let recorded = out
        .lines()
        .find_map(|l| l.strip_prefix("code_commit (recorded, not verified)="))
        .expect("the label is printed")
        .trim()
        .to_string();
    let (code, _, err) = run_env(
        &["train", "verify", b.to_str().unwrap()],
        "TSR_CODE_COMMIT",
        &recorded,
    );
    assert_eq!(code, 0, "P1: verify with the recorded commit: {err}");

    // Different from the recorded value: still verifies.
    let (code, _, err) = run_env(
        &["train", "verify", b.to_str().unwrap()],
        "TSR_CODE_COMMIT",
        "abc123",
    );
    assert_eq!(code, 0, "P1: verify from another commit: {err}");
}

/// F6/B8: `verify`, `report` and `apply` run the one `load_bundle`, so every manifest or content
/// tamper `verify` refuses, `report` and `apply` refuse too — all exit 2.
#[test]
fn f6_verify_report_and_apply_refuse_the_same_tampers() {
    let dir = temp_dir("f6-shared");
    let ds = build_dataset(&dir, "7");

    // Each tamper is applied to a fresh bundle; all three commands must exit 2.
    type Tamper = (&'static str, Box<dyn Fn(&Path)>);
    let tampers: Vec<Tamper> = vec![
        (
            "a comment",
            Box::new(|b: &Path| {
                let p = b.join("manifest.toml");
                let text = std::fs::read_to_string(&p).unwrap();
                std::fs::write(&p, format!("# a comment\n{text}")).unwrap();
            }),
        ),
        (
            "a whitespace edit",
            Box::new(|b: &Path| {
                let p = b.join("manifest.toml");
                let text = std::fs::read_to_string(&p).unwrap();
                std::fs::write(&p, text.replace("state = ", "state  = ")).unwrap();
            }),
        ),
        (
            "a forged calibrator",
            Box::new(|b: &Path| {
                let p = b.join("manifest.toml");
                let text = std::fs::read_to_string(&p).unwrap();
                let forged: String = text
                    .lines()
                    .map(|l| {
                        if l.trim_start().starts_with("calibrator =") {
                            "calibrator = \"000000000000 features@v9\"".to_string()
                        } else {
                            l.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(&p, forged).unwrap();
            }),
        ),
        (
            "state=\"active\"",
            Box::new(|b: &Path| {
                let p = b.join("manifest.toml");
                let text = std::fs::read_to_string(&p).unwrap();
                let forged: String = text
                    .lines()
                    .map(|l| {
                        if l.trim_start().starts_with("state =") {
                            "state = \"active\"".to_string()
                        } else {
                            l.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(&p, forged).unwrap();
            }),
        ),
        (
            "a missing head.json",
            Box::new(|b: &Path| {
                std::fs::remove_file(b.join("head.json")).unwrap();
            }),
        ),
        (
            "a one-byte isotonic.json change",
            Box::new(|b: &Path| {
                let p = b.join("isotonic.json");
                let mut bytes = std::fs::read(&p).unwrap();
                let last = bytes.len() - 2;
                bytes[last] ^= 0x01;
                std::fs::write(&p, &bytes).unwrap();
            }),
        ),
        (
            // P1: an edited `code_commit` line is refused only because it changes the canonical
            // rendering, not because it is compared with the environment.
            "an edited code_commit line",
            Box::new(|b: &Path| {
                let p = b.join("manifest.toml");
                let text = std::fs::read_to_string(&p).unwrap();
                let forged: String = text
                    .lines()
                    .map(|l| {
                        if l.trim_start().starts_with("code_commit =") {
                            "code_commit = \"deadbeef\"".to_string()
                        } else {
                            l.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                std::fs::write(&p, forged).unwrap();
            }),
        ),
    ];

    for (name, tamper) in tampers {
        // A fresh bundle per tamper: `fit` writes a bundle directory once.
        let b = dir.join("models").join(name.replace([' ', '=', '"'], "-"));
        let (code, _, err) = run(&[
            "train",
            "fit",
            ds.to_str().unwrap(),
            "--out",
            b.to_str().unwrap(),
        ]);
        assert_eq!(code, 0, "fit for {name} failed: {err}");
        tamper(&b);
        let (code, _, err) = run(&["train", "verify", b.to_str().unwrap()]);
        assert_eq!(code, 2, "F6/B8: verify must refuse {name}: {err}");
        let (code, _, err) = run(&[
            "train",
            "report",
            b.to_str().unwrap(),
            "--dataset",
            ds.to_str().unwrap(),
            "--seed",
            "7",
        ]);
        assert_eq!(code, 2, "F6/B8: report must refuse {name}: {err}");
        let (code, _, err) = run(&[
            "train",
            "apply",
            b.to_str().unwrap(),
            "--dataset",
            ds.to_str().unwrap(),
            "--side",
            "held_out",
        ]);
        assert_eq!(code, 2, "F6/B8: apply must refuse {name}: {err}");
    }
}

/// P2: `report` and `apply` refuse (exit 2, naming both roots) a dataset whose data root differs
/// from the one the bundle recorded, even when its split digest and feature spec still match.
#[test]
fn p2_report_and_apply_refuse_a_dataset_with_another_data_root() {
    let dir = temp_dir("p2-root");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);

    // The P2 repro: edit a held-out `generator_raw`, then recompute the row id and the data root
    // with the public `row_id_of`/`data_root`. The split digest and the feature spec are unchanged,
    // so only the data root differs from the bundle's.
    let mut ds2 = tsr_learn::bundle::load_dataset(&ds).expect("the dataset loads");
    let held = ds2
        .rows
        .iter_mut()
        .find(|r| r.side == tsr_learn::dataset::Side::HeldOut)
        .expect("the dataset has a held-out row");
    held.generator_raw = Some(0.99);
    held.id = tsr_learn::dataset::row_id_of(held);
    ds2.data_root = tsr_learn::dataset::data_root(&ds2.rows);
    let ds2_dir = dir.join("tampered-dataset");
    tsr_learn::dataset::write_dataset(&ds2, &ds2_dir).expect("the tampered dataset writes");

    let (code, _, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds2_dir.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 2, "P2: report refuses a foreign data root: {err}");
    assert!(
        err.contains("data root"),
        "P2: report names the data root: {err}"
    );

    let (code, _, err) = run(&[
        "train",
        "apply",
        b.to_str().unwrap(),
        "--dataset",
        ds2_dir.to_str().unwrap(),
        "--side",
        "held_out",
    ]);
    assert_eq!(code, 2, "P2: apply refuses a foreign data root: {err}");
    assert!(
        err.contains("data root"),
        "P2: apply names the data root: {err}"
    );
}
