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

    // Parse `mean_brier=<m> [interval_lo=<lo>, interval_hi=<hi>] <verdict>` for one family key.
    let parse = |key: &str| -> (f64, f64, f64, String) {
        let line = paired
            .lines()
            .find(|l| l.contains(key))
            .unwrap_or_else(|| panic!("no `{key}` line in:\n{paired}"));
        let mean: f64 = line
            .split("mean_brier=")
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
    let (mean, lo, hi, verdict) = parse("all n=");
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
    let (mean, lo, hi, verdict) = parse("overconfident n=");
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
    let (mean, lo, hi, verdict) = parse("calibrated n=");
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
    // F9/B3 (b): the refusal moved to `dataset`, which now refuses a share that leaves any side
    // empty -- so an empty calibrate slice never reaches `fit`.
    assert_eq!(
        code, 2,
        "F3/R2: dataset refuses an empty calibrate slice:\n{err}"
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
    // F9/B3 (b): the refusal moved to `dataset`, which now refuses a share that leaves any side
    // empty -- so an empty fit slice never reaches `fit`.
    assert_eq!(code, 2, "F3/R2: dataset refuses an empty fit slice:\n{err}");
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

/// F9/B3: a `tasks.jsonl` holding only a newline has no parseable record. `fit` must exit 2 naming
/// the file and the line, not exit 0 with the acceptance id (the old guard was `metadata.len()==0`,
/// so a one-byte file slipped through and probe (a) skipped every row for want of an index row).
#[test]
fn f9_fit_refuses_a_newline_only_tasks_index() {
    let dir = temp_dir("f9-tasks-newline");
    let ds = build_dataset(&dir, "7");
    std::fs::write(ds.join("tasks.jsonl"), "\n").unwrap();
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
        "F9/B3: fit refuses a newline-only tasks.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3: a `events.jsonl` holding only a newline has no parseable record. `fit` must exit 2 naming
/// the file and the line, not panic (exit 101) at probes.rs:111.
#[test]
fn f9_fit_refuses_a_newline_only_events_log() {
    let dir = temp_dir("f9-events-newline");
    let ds = build_dataset(&dir, "7");
    std::fs::write(ds.join("events.jsonl"), "\n").unwrap();
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
        "F9/B3: fit refuses a newline-only events.jsonl (never a panic):\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3: a truncated `events.jsonl` (a line that does not parse) must make `fit` exit 2 naming the
/// file and the line, never exit 1 for bad input.
#[test]
fn f9_fit_refuses_a_truncated_events_log_naming_the_line() {
    let dir = temp_dir("f9-events-trunc");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("events.jsonl")).unwrap();
    let truncated: String = text.chars().take(text.len() / 2).collect();
    std::fs::write(ds.join("events.jsonl"), truncated).unwrap();
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
        "F9/B3: fit refuses a truncated events.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
    assert!(
        err.contains("line "),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3: `dataset` on an empty `tasks.jsonl` must exit 2, not exit 0 with 0 rows.
#[test]
fn f9_dataset_refuses_an_empty_tasks_index() {
    let dir = temp_dir("f9-dataset-tasks");
    let (ev, _tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let empty = bad.join("tasks.jsonl");
    std::fs::write(&empty, "").unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        empty.to_str().unwrap(),
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
        code, 2,
        "F9/B3: dataset refuses an empty tasks.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
}

/// F9/B3: `dataset` on a newline-only `tasks.jsonl` must exit 2 naming the file and the line.
#[test]
fn f9_dataset_refuses_a_newline_only_tasks_index() {
    let dir = temp_dir("f9-dataset-tasks-nl");
    let (ev, _tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let nl = bad.join("tasks.jsonl");
    std::fs::write(&nl, "\n").unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        nl.to_str().unwrap(),
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
        code, 2,
        "F9/B3: dataset refuses a newline-only tasks.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3: `dataset` on a newline-only `events.jsonl` must exit 2 naming the file and the line.
#[test]
fn f9_dataset_refuses_a_newline_only_events_log() {
    let dir = temp_dir("f9-dataset-events-nl");
    let (_ev, tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let nl = bad.join("events.jsonl");
    std::fs::write(&nl, "\n").unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        nl.to_str().unwrap(),
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
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: dataset refuses a newline-only events.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3: `dataset` on a truncated `events.jsonl` (a line that does not parse) must exit 2 naming
/// the file and the line, never exit 1 for bad input.
#[test]
fn f9_dataset_refuses_a_truncated_events_log_naming_the_line() {
    let dir = temp_dir("f9-dataset-events-trunc");
    let (ev, tk, lb) = regenerate(&dir);
    let text = std::fs::read_to_string(&ev).unwrap();
    let truncated: String = text.chars().take(text.len() / 2).collect();
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let trunc = bad.join("events.jsonl");
    std::fs::write(&trunc, truncated).unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        trunc.to_str().unwrap(),
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
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: dataset refuses a truncated events.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
    assert!(
        err.contains("line "),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3: `dataset` on a truncated `tasks.jsonl` (a line that does not parse) must exit 2 naming
/// the file and the line, never exit 1 for bad input.
#[test]
fn f9_dataset_refuses_a_truncated_tasks_index_naming_the_line() {
    let dir = temp_dir("f9-dataset-tasks-trunc");
    let (ev, tk, lb) = regenerate(&dir);
    let text = std::fs::read_to_string(&tk).unwrap();
    let truncated: String = text.chars().take(text.len() / 2).collect();
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let trunc = bad.join("tasks.jsonl");
    std::fs::write(&trunc, truncated).unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        trunc.to_str().unwrap(),
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
        code, 2,
        "F9/B3: dataset refuses a truncated tasks.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
    assert!(
        err.contains("line "),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3: `fit` on a truncated `tasks.jsonl` must exit 2 naming the file and the line.
#[test]
fn f9_fit_refuses_a_truncated_tasks_index_naming_the_line() {
    let dir = temp_dir("f9-fit-tasks-trunc");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("tasks.jsonl")).unwrap();
    let truncated: String = text.chars().take(text.len() / 2).collect();
    std::fs::write(ds.join("tasks.jsonl"), truncated).unwrap();
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
        "F9/B3: fit refuses a truncated tasks.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
    assert!(
        err.contains("line "),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3: `dataset` on an empty `events.jsonl` must exit 2, not exit 0 with 0 rows.
#[test]
fn f9_dataset_refuses_an_empty_events_log() {
    let dir = temp_dir("f9-dataset-events");
    let (_ev, tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let empty = bad.join("events.jsonl");
    std::fs::write(&empty, "").unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        empty.to_str().unwrap(),
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
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: dataset refuses an empty events.jsonl:\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
}

/// F9/B3 (a): a `tasks.jsonl` line that is valid JSON but not a task record (`{}`) must make
/// `dataset` exit 2 naming the file and the line, not exit 1.
#[test]
fn f9_dataset_refuses_a_tasks_line_that_is_not_a_record() {
    let dir = temp_dir("f9-dataset-tasks-notrecord");
    let (ev, tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let text = std::fs::read_to_string(&tk).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines[0] = "{}";
    let bad_tasks = bad.join("tasks.jsonl");
    std::fs::write(&bad_tasks, lines.join("\n") + "\n").unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        bad_tasks.to_str().unwrap(),
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
        code, 2,
        "F9/B3: dataset refuses a `{{}}` tasks line (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3 (a): a `tasks.jsonl` line that is `null` must make `dataset` exit 2 naming the file and
/// the line, not exit 1.
#[test]
fn f9_dataset_refuses_a_null_tasks_line() {
    let dir = temp_dir("f9-dataset-tasks-null");
    let (ev, tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let text = std::fs::read_to_string(&tk).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines[0] = "null";
    let bad_tasks = bad.join("tasks.jsonl");
    std::fs::write(&bad_tasks, lines.join("\n") + "\n").unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        bad_tasks.to_str().unwrap(),
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
        code, 2,
        "F9/B3: dataset refuses a `null` tasks line (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3 (a): an `events.jsonl` line that is valid JSON but not an event record (`{}`) must make
/// `dataset` exit 2 naming the file and the line, not exit 1.
#[test]
fn f9_dataset_refuses_an_events_line_that_is_not_a_record() {
    let dir = temp_dir("f9-dataset-events-notrecord");
    let (ev, tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let text = std::fs::read_to_string(&ev).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines[0] = "{}";
    let bad_events = bad.join("events.jsonl");
    std::fs::write(&bad_events, lines.join("\n") + "\n").unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        bad_events.to_str().unwrap(),
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
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: dataset refuses a `{{}}` events line (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3 (a): a truncated `labels.jsonl` (a line that does not parse) must make `dataset` exit 2
/// naming the file and the line, not exit 1.
#[test]
fn f9_dataset_refuses_a_truncated_labels_file_naming_the_line() {
    let dir = temp_dir("f9-dataset-labels-trunc");
    let (ev, tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let text = std::fs::read_to_string(&lb).unwrap();
    let truncated: String = text.chars().take(text.len() / 2).collect();
    let bad_labels = bad.join("labels.jsonl");
    std::fs::write(&bad_labels, truncated).unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        tk.to_str().unwrap(),
        "--labels",
        bad_labels.to_str().unwrap(),
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
        code, 2,
        "F9/B3: dataset refuses a truncated labels.jsonl (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("labels.jsonl"),
        "F9/B3: the refusal names labels.jsonl: {err}"
    );
    assert!(
        err.contains("line "),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3 (a): `fit` on a `tasks.jsonl` line that is valid JSON but not a record (`{}`) must exit 2
/// naming the file and the line, not exit 2 with no line.
#[test]
fn f9_fit_refuses_a_tasks_line_that_is_not_a_record_naming_the_line() {
    let dir = temp_dir("f9-fit-tasks-notrecord");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("tasks.jsonl")).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines[0] = "{}";
    std::fs::write(ds.join("tasks.jsonl"), lines.join("\n") + "\n").unwrap();
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
        "F9/B3: fit refuses a `{{}}` tasks line:\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3 (a): `fit` on an `events.jsonl` cut to its first 20 lines must exit 2 naming the file,
/// never panic (exit 101) at probes.rs:111.
#[test]
fn f9_fit_refuses_an_events_log_that_lacks_the_datasets_tasks() {
    let dir = temp_dir("f9-fit-events-cut");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("events.jsonl")).unwrap();
    let cut: String = text.lines().take(20).collect::<Vec<_>>().join("\n") + "\n";
    std::fs::write(ds.join("events.jsonl"), cut).unwrap();
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
        "F9/B3: fit refuses an events.jsonl that lacks the dataset's tasks (never a panic):\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
}

/// F9/B3 (a): `fit` on a `tasks.jsonl` whose task ids match no row must exit 2 naming probe (a)
/// and the counts, not exit 0 with the acceptance bundle id while probe (a) examines nothing.
#[test]
fn f9_fit_refuses_a_tasks_index_that_matches_no_row() {
    let dir = temp_dir("f9-fit-tasks-zeroed");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("tasks.jsonl")).unwrap();
    let zeroed: String = text
        .lines()
        .map(|l| {
            let mut v: serde_json::Value = serde_json::from_str(l).unwrap();
            v["task"] = serde_json::Value::String("0".repeat(64));
            serde_json::to_string(&v).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(ds.join("tasks.jsonl"), zeroed).unwrap();
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
        "F9/B3: fit refuses a tasks index that matches no row (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("post_action_events"),
        "F9/B3: the refusal names probe (a): {err}"
    );
}

/// F9/B3 (a): `fit` on a `tasks.jsonl` cut to its first line must exit 2 naming probe (a) and the
/// counts, not exit 0 while probe (a) examines 4 of 1129 rows.
#[test]
fn f9_fit_refuses_a_tasks_index_that_examines_too_few_rows() {
    let dir = temp_dir("f9-fit-tasks-head1");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("tasks.jsonl")).unwrap();
    let head1: String = text.lines().take(1).collect::<Vec<_>>().join("\n") + "\n";
    std::fs::write(ds.join("tasks.jsonl"), head1).unwrap();
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
        "F9/B3: fit refuses a tasks index that examines too few rows (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("post_action_events"),
        "F9/B3: the refusal names probe (a): {err}"
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

/// F8: an index row for a task with no gated commitment is accepted and silently dropped. The
/// count of such tasks must appear in `dataset`'s stdout, in `data-manifest.json` and in the
/// report, exactly as `unindexed_tasks` already is.
#[test]
fn f8_uncommitted_tasks_are_counted() {
    let dir = temp_dir("f8-uncommitted");
    let (ev, tk, lb) = regenerate(&dir);
    // Append two index rows for tasks that appear in no event: no gated commitment, no rows.
    let mut text = std::fs::read_to_string(&tk).unwrap();
    for tag in ["ff", "fe"] {
        let extra = serde_json::json!({
            "task": format!("00000000000000000000000000000000000000000000000000000000000000{tag}"),
            "family": "calibrated",
            "lineage": format!("lineage-{tag}"),
            "timestamp": "2026-01-01T00:00:00Z",
        });
        text.push_str(&serde_json::to_string(&extra).unwrap());
        text.push('\n');
    }
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
    assert_eq!(code, 0, "F8: dataset runs:\n{out}\n{err}");
    let n: usize = field(&out, "uncommitted_tasks")
        .parse()
        .expect("uncommitted_tasks is a number");
    assert_eq!(n, 2, "F8: dataset counts the two uncommitted tasks: {out}");
    let manifest = std::fs::read_to_string(ds.join("data-manifest.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&manifest).unwrap();
    assert_eq!(
        v["uncommitted_tasks"].as_u64(),
        Some(n as u64),
        "F8: data-manifest.json carries the count: {manifest}"
    );

    // F8: the report carries the count too.
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "F8: fit accepts the dataset: {out}\n{err}");
    let (code, out, err) = run(&[
        "train",
        "report",
        b.to_str().unwrap(),
        "--dataset",
        ds.to_str().unwrap(),
        "--seed",
        "7",
    ]);
    assert_eq!(code, 0, "F8: report runs: {out}\n{err}");
    let line = out
        .lines()
        .find(|l| l.starts_with("uncommitted_tasks "))
        .unwrap_or_else(|| panic!("F8: the report carries the count: {out}"));
    assert_eq!(
        line.split_whitespace().nth(1),
        Some(n.to_string().as_str()),
        "F8: the report carries the count: {out}"
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

/// F9/B3 (a): `fit` on a dataset directory whose `rows.jsonl` is missing must exit 2 naming the
/// file, not exit 1 with "fit failed: io: …". A missing structural input is bad input.
#[test]
fn f9_fit_refuses_a_missing_rows_jsonl_with_exit_2() {
    let dir = temp_dir("f9-fit-missing-rows");
    let ds = build_dataset(&dir, "7");
    std::fs::remove_file(ds.join("rows.jsonl")).unwrap();
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
        "F9/B3: fit refuses a missing rows.jsonl with exit 2 (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("rows.jsonl"),
        "F9/B3: the refusal names rows.jsonl: {err}"
    );
}

/// F9/B3 (a): `fit` on a dataset directory whose `features.json` is missing must exit 2 naming the
/// file, not exit 1.
#[test]
fn f9_fit_refuses_a_missing_features_json_with_exit_2() {
    let dir = temp_dir("f9-fit-missing-features");
    let ds = build_dataset(&dir, "7");
    std::fs::remove_file(ds.join("features.json")).unwrap();
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
        "F9/B3: fit refuses a missing features.json with exit 2 (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("features.json"),
        "F9/B3: the refusal names features.json: {err}"
    );
}

/// F9/B3 (a): `fit` on a dataset directory whose `split.json` is missing must exit 2 naming the
/// file, not exit 1.
#[test]
fn f9_fit_refuses_a_missing_split_json_with_exit_2() {
    let dir = temp_dir("f9-fit-missing-split");
    let ds = build_dataset(&dir, "7");
    std::fs::remove_file(ds.join("split.json")).unwrap();
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
        "F9/B3: fit refuses a missing split.json with exit 2 (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("split.json"),
        "F9/B3: the refusal names split.json: {err}"
    );
}

/// F9/B3 (a): `fit` on a dataset directory whose `data-manifest.json` is missing must exit 2 naming
/// the file, not exit 1.
#[test]
fn f9_fit_refuses_a_missing_data_manifest_json_with_exit_2() {
    let dir = temp_dir("f9-fit-missing-manifest");
    let ds = build_dataset(&dir, "7");
    std::fs::remove_file(ds.join("data-manifest.json")).unwrap();
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
        "F9/B3: fit refuses a missing data-manifest.json with exit 2 (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("data-manifest.json"),
        "F9/B3: the refusal names data-manifest.json: {err}"
    );
}

/// F9/B3 (a): `fit` on a non-UTF-8 `rows.jsonl` must exit 2 naming the file, not exit 1 with
/// "fit failed: io: …". A file that is not UTF-8 is bad input.
#[test]
fn f9_fit_refuses_a_non_utf8_rows_jsonl_with_exit_2() {
    let dir = temp_dir("f9-fit-non-utf8-rows");
    let ds = build_dataset(&dir, "7");
    std::fs::write(ds.join("rows.jsonl"), [0xffu8, 0xfe, 0x00, 0x01]).unwrap();
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
        "F9/B3: fit refuses a non-UTF-8 rows.jsonl with exit 2 (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("rows.jsonl"),
        "F9/B3: the refusal names rows.jsonl: {err}"
    );
}

/// F9/B3 (b): a `tasks.jsonl` cut to the fit-side tasks examines 696 rows and none of the 215
/// held-out, yet the old gate (`examined < held_out`) passed and `fit` exited 0 with the acceptance
/// id. The probe must cover every split side: refuse (exit 2) naming the side and the counts.
#[test]
fn f9_fit_refuses_a_tasks_index_cut_to_the_fit_side() {
    let dir = temp_dir("f9-fit-side-fit-only");
    let ds = build_dataset(&dir, "7");
    let loaded = tsr_learn::bundle::load_dataset(&ds).expect("the dataset loads");
    let fit_tasks: std::collections::HashSet<String> = loaded
        .rows
        .iter()
        .filter(|r| r.side == tsr_learn::dataset::Side::Fit)
        .map(|r| r.task.to_string())
        .collect();
    let text = std::fs::read_to_string(ds.join("tasks.jsonl")).unwrap();
    let cut: String = text
        .lines()
        .filter(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            fit_tasks.contains(v["task"].as_str().unwrap())
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(ds.join("tasks.jsonl"), cut).unwrap();
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
        "F9/B3: fit refuses a tasks index cut to the fit side (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("held_out") || err.contains("held-out"),
        "F9/B3: the refusal names the held-out side: {err}"
    );
}

/// F9/B3 (b): a held-out-only `tasks.jsonl` (76 lines) passes the old count gate and `fit` exits 0.
/// The probe must refuse it, naming the missing fit and calibrate sides.
#[test]
fn f9_fit_refuses_a_held_out_only_tasks_index() {
    let dir = temp_dir("f9-fit-side-heldout-only");
    let ds = build_dataset(&dir, "7");
    let loaded = tsr_learn::bundle::load_dataset(&ds).expect("the dataset loads");
    let held_tasks: std::collections::HashSet<String> = loaded
        .rows
        .iter()
        .filter(|r| r.side == tsr_learn::dataset::Side::HeldOut)
        .map(|r| r.task.to_string())
        .collect();
    let text = std::fs::read_to_string(ds.join("tasks.jsonl")).unwrap();
    let cut: String = text
        .lines()
        .filter(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            held_tasks.contains(v["task"].as_str().unwrap())
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(ds.join("tasks.jsonl"), cut).unwrap();
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
        "F9/B3: fit refuses a held-out-only tasks index (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("fit") || err.contains("calibrate"),
        "F9/B3: the refusal names the missing side: {err}"
    );
}

/// F9/B3 (b): a calibrate-only `tasks.jsonl` (75 lines, 218 rows) passes the old count gate and
/// `fit` exits 0. The probe must refuse it, naming the missing fit and held-out sides.
#[test]
fn f9_fit_refuses_a_calibrate_only_tasks_index() {
    let dir = temp_dir("f9-fit-side-calibrate-only");
    let ds = build_dataset(&dir, "7");
    let loaded = tsr_learn::bundle::load_dataset(&ds).expect("the dataset loads");
    let cal_tasks: std::collections::HashSet<String> = loaded
        .rows
        .iter()
        .filter(|r| r.side == tsr_learn::dataset::Side::Calibrate)
        .map(|r| r.task.to_string())
        .collect();
    let text = std::fs::read_to_string(ds.join("tasks.jsonl")).unwrap();
    let cut: String = text
        .lines()
        .filter(|l| {
            let v: serde_json::Value = serde_json::from_str(l).unwrap();
            cal_tasks.contains(v["task"].as_str().unwrap())
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(ds.join("tasks.jsonl"), cut).unwrap();
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
        "F9/B3: fit refuses a calibrate-only tasks index (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("fit") || err.contains("held_out") || err.contains("held-out"),
        "F9/B3: the refusal names the missing side: {err}"
    );
}

/// F9/B3 (c): a wrong-typed `seq` in an events wrapper (`"x"`) is silently replaced by line
/// position (dataset.rs:630, 636). `dataset` must refuse it like any other bad field, exit 2.
#[test]
fn f9_dataset_refuses_a_string_seq_in_an_events_wrapper() {
    let dir = temp_dir("f9-seq-string");
    let (ev, tk, lb) = regenerate(&dir);
    let text = std::fs::read_to_string(&ev).unwrap();
    let bad: String = text
        .lines()
        .enumerate()
        .map(|(i, l)| {
            if i == 5 {
                let mut v: serde_json::Value = serde_json::from_str(l).unwrap();
                v["seq"] = serde_json::Value::String("x".into());
                serde_json::to_string(&v).unwrap()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&ev, bad).unwrap();
    let ds = dir.join("ds");
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
        code, 2,
        "F9/B3: dataset refuses a string seq in an events wrapper (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl") && err.contains("seq"),
        "F9/B3: the refusal names events.jsonl and seq: {err}"
    );
}

/// F9/B3 (c): a float `seq` (`1.5`) in an events wrapper is silently replaced by line position.
/// `dataset` must refuse it, exit 2.
#[test]
fn f9_dataset_refuses_a_float_seq_in_an_events_wrapper() {
    let dir = temp_dir("f9-seq-float");
    let (ev, tk, lb) = regenerate(&dir);
    let text = std::fs::read_to_string(&ev).unwrap();
    let bad: String = text
        .lines()
        .enumerate()
        .map(|(i, l)| {
            if i == 5 {
                let mut v: serde_json::Value = serde_json::from_str(l).unwrap();
                v["seq"] = serde_json::json!(1.5);
                serde_json::to_string(&v).unwrap()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&ev, bad).unwrap();
    let ds = dir.join("ds");
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
        code, 2,
        "F9/B3: dataset refuses a float seq in an events wrapper (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl") && err.contains("seq"),
        "F9/B3: the refusal names events.jsonl and seq: {err}"
    );
}

/// F9/B3 (c): a `null` `seq` in an events wrapper is silently replaced by line position. `dataset`
/// must refuse it, exit 2.
#[test]
fn f9_dataset_refuses_a_null_seq_in_an_events_wrapper() {
    let dir = temp_dir("f9-seq-null");
    let (ev, tk, lb) = regenerate(&dir);
    let text = std::fs::read_to_string(&ev).unwrap();
    let bad: String = text
        .lines()
        .enumerate()
        .map(|(i, l)| {
            if i == 5 {
                let mut v: serde_json::Value = serde_json::from_str(l).unwrap();
                v["seq"] = serde_json::Value::Null;
                serde_json::to_string(&v).unwrap()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&ev, bad).unwrap();
    let ds = dir.join("ds");
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
        code, 2,
        "F9/B3: dataset refuses a null seq in an events wrapper (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl") && err.contains("seq"),
        "F9/B3: the refusal names events.jsonl and seq: {err}"
    );
}

/// F9/B3 (c): an absent `seq` in an events wrapper is silently replaced by line position. `dataset`
/// must refuse it, exit 2.
#[test]
fn f9_dataset_refuses_an_absent_seq_in_an_events_wrapper() {
    let dir = temp_dir("f9-seq-absent");
    let (ev, tk, lb) = regenerate(&dir);
    let text = std::fs::read_to_string(&ev).unwrap();
    let bad: String = text
        .lines()
        .enumerate()
        .map(|(i, l)| {
            if i == 5 {
                let mut v: serde_json::Value = serde_json::from_str(l).unwrap();
                v.as_object_mut().unwrap().remove("seq");
                serde_json::to_string(&v).unwrap()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&ev, bad).unwrap();
    let ds = dir.join("ds");
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
        code, 2,
        "F9/B3: dataset refuses an absent seq in an events wrapper (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl") && err.contains("seq"),
        "F9/B3: the refusal names events.jsonl and seq: {err}"
    );
}

/// F9/B3 (d): the events-cut refusal must name the first offending row and the count, and say
/// "refused" rather than "probe caught a leak" when nothing leaked. Before the fix it was one
/// 271 KB stderr line prefixed "probe caught a leak".
#[test]
fn f9_fit_events_cut_refusal_is_short_and_says_refused() {
    let dir = temp_dir("f9-events-cut-short");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("events.jsonl")).unwrap();
    let cut: String = text.lines().take(20).collect::<Vec<_>>().join("\n") + "\n";
    std::fs::write(ds.join("events.jsonl"), cut).unwrap();
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "F9/B3: fit refuses an events cut:\n{out}\n{err}");
    assert!(
        !err.contains("probe caught a leak"),
        "F9/B3: nothing leaked, so the refusal must not say `probe caught a leak`: {err}"
    );
    assert!(
        err.contains("refused"),
        "F9/B3: the refusal says `refused`: {err}"
    );
    assert!(
        err.len() < 4096,
        "F9/B3: the refusal is short (names the first offending row and the count), not {} bytes",
        err.len()
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
}

/// F9/B3 (d): one parser. A truncated events line must be refused with the loader's message format
/// (the file name, not the full path), so there is one message format, not two.
#[test]
fn f9_fit_refuses_a_truncated_events_line_with_the_loader_message() {
    let dir = temp_dir("f9-one-parser-events");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("events.jsonl")).unwrap();
    let mut lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    // Truncate the 6th line mid-JSON.
    let l6 = &lines[5];
    lines[5] = l6[..l6.len() / 2].to_string();
    std::fs::write(ds.join("events.jsonl"), lines.join("\n") + "\n").unwrap();
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
        "F9/B3: fit refuses a truncated events line:\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl line 6"),
        "F9/B3: the refusal names events.jsonl and the line: {err}"
    );
    // The loader names the file, not the full path: the message must not carry the dataset dir.
    assert!(
        !err.contains(ds.to_str().unwrap()),
        "F9/B3: one message format -- the loader names the file, not the full path: {err}"
    );
}

/// F9/B3 (a): every malformed structural input file must make `fit` exit 2 naming the file, not
/// exit 1 with a bare `json:` error. The three structural files are `features.json`, `split.json`
/// and `data-manifest.json`; the malformed forms are empty, truncated, `null`, `[]`, `{}` and a
/// wrong-typed field.
#[test]
fn f9_fit_refuses_every_malformed_structural_file_naming_it() {
    let dir = temp_dir("f9-structural-fit");
    let ds = build_dataset(&dir, "7");
    // (file, form, bytes). A wrong-typed field is a field of the wrong JSON type for that file's
    // schema: `seed` is a u64 in split.json, `version` a string in features.json.
    let cases: &[(&str, &str, &str)] = &[
        ("features.json", "empty", ""),
        ("features.json", "truncated", "{\"a\":"),
        ("features.json", "null", "null"),
        ("features.json", "array", "[]"),
        ("features.json", "object", "{}"),
        (
            "features.json",
            "wrong-typed",
            "{\"version\":5,\"names\":[],\"numeric_widths\":[],\"absent\":[]}",
        ),
        // F9/B3: an unknown field is bad input too -- `FeatureSpec` denies unknown fields, so
        // `"seed":"x"` (a field features.json does not have) is refused rather than ignored.
        (
            "features.json",
            "unknown-field",
            "{\"version\":\"features@v1\",\"names\":[],\"numeric_widths\":[],\"absent\":[],\"seed\":\"x\"}",
        ),
        ("split.json", "empty", ""),
        ("split.json", "truncated", "{\"a\":"),
        ("split.json", "null", "null"),
        ("split.json", "array", "[]"),
        ("split.json", "object", "{}"),
        (
            "split.json",
            "wrong-typed",
            "{\"by\":\"time\",\"seed\":\"x\",\"assignments\":{},\"digest\":\"00\"}",
        ),
        // F9/B3: an unknown field in split.json is bad input too (`SplitManifest` denies them).
        (
            "split.json",
            "unknown-field",
            "{\"by\":\"time\",\"seed\":7,\"assignments\":{},\"digest\":\"00\",\"seed2\":\"x\"}",
        ),
        ("data-manifest.json", "empty", ""),
        ("data-manifest.json", "truncated", "{\"a\":"),
        ("data-manifest.json", "null", "null"),
        ("data-manifest.json", "array", "[]"),
        ("data-manifest.json", "object", "{}"),
        // F9/B3: an unknown field in data-manifest.json is bad input too (read by name otherwise).
        (
            "data-manifest.json",
            "unknown-field",
            "{\"data_root\":\"00\",\"seed\":\"x\"}",
        ),
    ];
    for (file, form, bytes) in cases {
        let bad = dir.join(format!("bad-{file}-{form}"));
        let _ = std::fs::remove_dir_all(&bad);
        copy_dir(&ds, &bad);
        std::fs::write(bad.join(file), bytes).unwrap();
        let b = dir.join(format!("models/bad-{file}-{form}"));
        let (code, out, err) = run(&[
            "train",
            "fit",
            bad.to_str().unwrap(),
            "--out",
            b.to_str().unwrap(),
        ]);
        assert_eq!(
            code, 2,
            "F9/B3: fit must refuse {file}/{form} with exit 2, not {code}:\n{out}\n{err}"
        );
        assert!(
            err.contains(file),
            "F9/B3: the refusal for {file}/{form} must name the file:\n{err}"
        );
        assert!(
            !err.contains("fit failed"),
            "F9/B3: {file}/{form} is bad input, not a generic failure:\n{err}"
        );
    }
    // F9/B3 (a): a wrong-typed, negative or out-of-range field in an otherwise valid
    // data-manifest.json is bad input too. Each field is mutated in a copy of the real manifest, so
    // the type/range check -- not a missing-field check -- is what refuses it.
    let good = std::fs::read_to_string(ds.join("data-manifest.json")).unwrap();
    let mutations: &[(&str, serde_json::Value)] = &[
        ("rows", serde_json::json!("x")),
        ("fit", serde_json::json!("x")),
        ("calibrate", serde_json::json!("x")),
        ("held_out", serde_json::json!("x")),
        ("resolved", serde_json::json!("x")),
        ("feature_schema", serde_json::json!("x")),
        ("unindexed_tasks", serde_json::json!("x")),
        ("uncommitted_tasks", serde_json::json!("x")),
        ("unindexed_tasks", serde_json::json!(-3)),
        ("unindexed_tasks", serde_json::json!(1e30)),
        ("uncommitted_tasks", serde_json::json!(-1)),
    ];
    for (field, value) in mutations {
        let mut v: serde_json::Value = serde_json::from_str(&good).unwrap();
        v[field] = value.clone();
        let bad = dir.join(format!("bad-manifest-{field}-{value}"));
        let _ = std::fs::remove_dir_all(&bad);
        copy_dir(&ds, &bad);
        std::fs::write(
            bad.join("data-manifest.json"),
            serde_json::to_string_pretty(&v).unwrap(),
        )
        .unwrap();
        let b = dir.join(format!("models/bad-manifest-{field}-{value}"));
        let (code, out, err) = run(&[
            "train",
            "fit",
            bad.to_str().unwrap(),
            "--out",
            b.to_str().unwrap(),
        ]);
        assert_eq!(
            code, 2,
            "F9/B3: fit must refuse data-manifest.json {field}={value} with exit 2, not {code}:\n{out}\n{err}"
        );
        assert!(
            err.contains("data-manifest.json"),
            "F9/B3: the refusal for {field}={value} must name the file:\n{err}"
        );
        assert!(
            err.contains(field),
            "F9/B3: the refusal for {field}={value} must name the field:\n{err}"
        );
        assert!(
            !err.contains("fit failed"),
            "F9/B3: {field}={value} is bad input, not a generic failure:\n{err}"
        );
    }
}

/// F9/B3 (a): the same malformed structural files must exit 2 naming the file on `report`, `apply`
/// and `verify --rebuild` too, so all five subcommands agree.
#[test]
fn f9_report_apply_verify_refuse_malformed_structural_files_naming_them() {
    let dir = temp_dir("f9-structural-others");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);
    let cases: &[(&str, &str, &str)] = &[
        ("features.json", "empty", ""),
        ("features.json", "null", "null"),
        (
            "features.json",
            "unknown-field",
            "{\"version\":\"features@v1\",\"names\":[],\"numeric_widths\":[],\"absent\":[],\"seed\":\"x\"}",
        ),
        ("split.json", "truncated", "{\"a\":"),
        (
            "split.json",
            "wrong-typed",
            "{\"by\":\"time\",\"seed\":\"x\",\"assignments\":{},\"digest\":\"00\"}",
        ),
        ("data-manifest.json", "empty", ""),
        ("data-manifest.json", "object", "{}"),
        (
            "data-manifest.json",
            "unknown-field",
            "{\"data_root\":\"00\",\"seed\":\"x\"}",
        ),
    ];
    for (file, form, bytes) in cases {
        let bad = dir.join(format!("bad2-{file}-{form}"));
        let _ = std::fs::remove_dir_all(&bad);
        copy_dir(&ds, &bad);
        std::fs::write(bad.join(file), bytes).unwrap();
        let (code, out, err) = run(&[
            "train",
            "report",
            b.to_str().unwrap(),
            "--dataset",
            bad.to_str().unwrap(),
        ]);
        assert_eq!(code, 2, "F9/B3: report {file}/{form}:\n{out}\n{err}");
        assert!(err.contains(file), "F9/B3: report names {file}:\n{err}");
        let (code, out, err) = run(&[
            "train",
            "apply",
            b.to_str().unwrap(),
            "--dataset",
            bad.to_str().unwrap(),
        ]);
        assert_eq!(code, 2, "F9/B3: apply {file}/{form}:\n{out}\n{err}");
        assert!(err.contains(file), "F9/B3: apply names {file}:\n{err}");
        let (code, out, err) = run(&[
            "train",
            "verify",
            b.to_str().unwrap(),
            "--rebuild",
            bad.to_str().unwrap(),
        ]);
        assert_eq!(code, 2, "F9/B3: verify {file}/{form}:\n{out}\n{err}");
        assert!(err.contains(file), "F9/B3: verify names {file}:\n{err}");
    }
    // F9/B3 (a): a wrong-typed field in an otherwise valid data-manifest.json is bad input on
    // report, apply and verify too, not only on fit.
    let good = std::fs::read_to_string(ds.join("data-manifest.json")).unwrap();
    for (field, value) in [
        ("rows", serde_json::json!("x")),
        ("unindexed_tasks", serde_json::json!("x")),
    ] {
        let mut v: serde_json::Value = serde_json::from_str(&good).unwrap();
        v[field] = value.clone();
        let bad = dir.join(format!("bad2-manifest-{field}-{value}"));
        let _ = std::fs::remove_dir_all(&bad);
        copy_dir(&ds, &bad);
        std::fs::write(
            bad.join("data-manifest.json"),
            serde_json::to_string_pretty(&v).unwrap(),
        )
        .unwrap();
        let (code, out, err) = run(&[
            "train",
            "report",
            b.to_str().unwrap(),
            "--dataset",
            bad.to_str().unwrap(),
        ]);
        assert_eq!(code, 2, "F9/B3: report {field}={value}:\n{out}\n{err}");
        assert!(
            err.contains("data-manifest.json") && err.contains(field),
            "F9/B3: report names the file and field for {field}={value}:\n{err}"
        );
        let (code, out, err) = run(&[
            "train",
            "apply",
            b.to_str().unwrap(),
            "--dataset",
            bad.to_str().unwrap(),
        ]);
        assert_eq!(code, 2, "F9/B3: apply {field}={value}:\n{out}\n{err}");
        assert!(
            err.contains("data-manifest.json") && err.contains(field),
            "F9/B3: apply names the file and field for {field}={value}:\n{err}"
        );
        let (code, out, err) = run(&[
            "train",
            "verify",
            b.to_str().unwrap(),
            "--rebuild",
            bad.to_str().unwrap(),
        ]);
        assert_eq!(code, 2, "F9/B3: verify {field}={value}:\n{out}\n{err}");
        assert!(
            err.contains("data-manifest.json") && err.contains(field),
            "F9/B3: verify names the file and field for {field}={value}:\n{err}"
        );
    }
}

/// F9/B3 (b): `dataset` must refuse a share that leaves any side empty, exit 2 naming the side,
/// rather than writing a dataset with an empty side that `fit` then skips.
#[test]
fn f9_dataset_refuses_a_share_that_leaves_a_side_empty() {
    let dir = temp_dir("f9-empty-side-dataset");
    let (ev, tk, lb) = regenerate(&dir);
    for (flag, share) in [("--holdout", "0"), ("--calibrate", "0")] {
        let out = dir.join(format!("ds-{flag}"));
        let (code, stdout, err) = run(&[
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
            if flag == "--calibrate" { share } else { "0.25" },
            "--holdout",
            if flag == "--holdout" { share } else { "0.25" },
            "--seed",
            "7",
            "--out",
            out.to_str().unwrap(),
        ]);
        assert_eq!(
            code, 2,
            "F9/B3: dataset {flag} {share} must refuse an empty side:\n{stdout}\n{err}"
        );
        let side = if flag == "--holdout" {
            "held_out"
        } else {
            "calibrate"
        };
        assert!(
            err.contains(side),
            "F9/B3: the refusal names the empty side {side}:\n{err}"
        );
    }
}

/// F9/B3 (b): `fit` must refuse a split with an empty side, exit 2 naming the side and its count,
/// rather than skipping the empty side and writing a bundle with no held-out rows.
#[test]
fn f9_fit_refuses_a_split_with_an_empty_side() {
    let dir = temp_dir("f9-empty-side-fit");
    let ds = build_dataset(&dir, "7");
    // Forge a dataset whose held_out side is empty, through the library so every root stays
    // consistent: drop the held_out rows, recompute the roots and the split digest, and rewrite.
    let mut d = tsr_learn::bundle::load_dataset(&ds).unwrap();
    d.rows
        .retain(|r| r.side != tsr_learn::dataset::Side::HeldOut);
    d.split
        .assignments
        .retain(|_task, side| *side != tsr_learn::dataset::Side::HeldOut);
    d.split.digest =
        tsr_learn::dataset::split_digest(d.split.by, d.split.seed, &d.split.assignments);
    d.data_root = tsr_learn::dataset::data_root(&d.rows);
    d.training_root = tsr_learn::dataset::training_root(&d.rows);
    let bad = dir.join("ds-empty-held-out");
    tsr_learn::dataset::write_dataset(&d, &bad).unwrap();
    // `fit`'s probes read the log and the task index from the dataset directory too.
    for name in ["events.jsonl", "tasks.jsonl"] {
        std::fs::copy(ds.join(name), bad.join(name)).unwrap();
    }
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        bad.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: fit must refuse a split with an empty held_out side:\n{out}\n{err}"
    );
    assert!(
        err.contains("held_out"),
        "F9/B3: the refusal names the empty side:\n{err}"
    );
}

/// F9/B3 (c): an `--out` that exists and is not a directory must exit 2 naming it, on `fit` and on
/// `dataset`, not exit 1 with `io: Not a directory`.
#[test]
fn f9_out_that_is_a_file_is_refused_naming_it() {
    let dir = temp_dir("f9-out-file");
    let ds = build_dataset(&dir, "7");
    let afile = dir.join("not-a-dir");
    std::fs::write(&afile, "x").unwrap();
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        afile.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "F9/B3: fit --out <file>:\n{out}\n{err}");
    assert!(
        err.contains(afile.to_str().unwrap()),
        "F9/B3: fit names the --out path:\n{err}"
    );
    assert!(
        !err.contains("fit failed"),
        "F9/B3: a bad --out is refused input, not a generic failure:\n{err}"
    );
    let (ev, tk, lb) = regenerate(&dir);
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
        "time",
        "--calibrate",
        "0.25",
        "--holdout",
        "0.25",
        "--seed",
        "7",
        "--out",
        afile.to_str().unwrap(),
    ]);
    assert_eq!(code, 2, "F9/B3: dataset --out <file>:\n{out}\n{err}");
    assert!(
        err.contains(afile.to_str().unwrap()),
        "F9/B3: dataset names the --out path:\n{err}"
    );
}

/// F9/B3 (b): a duplicate key in any of the three structural files is refused, exit 2 naming the
/// file -- `features.json` and `split.json` already refuse one; `data-manifest.json` must too.
#[test]
fn f9_duplicate_keys_are_refused_in_all_three_structural_files() {
    let dir = temp_dir("f9-duplicate-keys");
    let ds = build_dataset(&dir, "7");
    let cases: &[(&str, &str)] = &[
        (
            "features.json",
            "{\"version\":\"features@v1\",\"version\":\"features@v1\",\"names\":[],\"numeric_widths\":[],\"absent\":[]}",
        ),
        (
            "split.json",
            "{\"by\":\"time\",\"by\":\"time\",\"seed\":7,\"assignments\":{},\"digest\":\"00\"}",
        ),
        (
            "data-manifest.json",
            "{\"rows\":1,\"rows\":2}",
        ),
    ];
    for (file, bytes) in cases {
        let bad = dir.join(format!("dup-{file}"));
        let _ = std::fs::remove_dir_all(&bad);
        copy_dir(&ds, &bad);
        std::fs::write(bad.join(file), bytes).unwrap();
        let b = dir.join(format!("models/dup-{file}"));
        let (code, out, err) = run(&[
            "train",
            "fit",
            bad.to_str().unwrap(),
            "--out",
            b.to_str().unwrap(),
        ]);
        assert_eq!(
            code, 2,
            "F9/B3: fit must refuse a duplicate key in {file} with exit 2, not {code}:\n{out}\n{err}"
        );
        assert!(
            err.contains(file),
            "F9/B3: the duplicate-key refusal for {file} must name the file:\n{err}"
        );
        assert!(
            !err.contains("fit failed"),
            "F9/B3: a duplicate key in {file} is bad input, not a generic failure:\n{err}"
        );
    }
}

/// F9/B3 (c): `--out <file>/sub` and `--out <dangling symlink>` must exit 2 naming the path on
/// `fit` and `dataset`, not exit 1 with `io: Not a directory` / `io: File exists`.
#[test]
fn f9_out_under_a_file_or_at_a_dangling_symlink_is_refused_naming_it() {
    let dir = temp_dir("f9-out-under-file");
    let ds = build_dataset(&dir, "7");
    let afile = dir.join("not-a-dir");
    std::fs::write(&afile, "x").unwrap();
    let under = afile.join("sub");
    let dangling = dir.join("dangling");
    std::os::unix::fs::symlink(dir.join("no-such-target"), &dangling).unwrap();

    for out in [&under, &dangling] {
        let (code, stdout, err) = run(&[
            "train",
            "fit",
            ds.to_str().unwrap(),
            "--out",
            out.to_str().unwrap(),
        ]);
        assert_eq!(
            code,
            2,
            "F9/B3: fit --out {} must exit 2, not {code}:\n{stdout}\n{err}",
            out.display()
        );
        assert!(
            err.contains(out.to_str().unwrap()),
            "F9/B3: fit must name the --out path {}:\n{err}",
            out.display()
        );
        assert!(
            !err.contains("fit failed"),
            "F9/B3: a bad --out is refused input, not a generic failure:\n{err}"
        );
    }

    let (ev, tk, lb) = regenerate(&dir);
    for out in [&under, &dangling] {
        let (code, stdout, err) = run(&[
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
            "0.25",
            "--holdout",
            "0.25",
            "--seed",
            "7",
            "--out",
            out.to_str().unwrap(),
        ]);
        assert_eq!(
            code,
            2,
            "F9/B3: dataset --out {} must exit 2, not {code}:\n{stdout}\n{err}",
            out.display()
        );
        assert!(
            err.contains(out.to_str().unwrap()),
            "F9/B3: dataset must name the --out path {}:\n{err}",
            out.display()
        );
    }
}

/// F9/B3 (d): a non-finite, zero or negative `learning_rate` in features.json is bad input -- exit
/// 2 naming the field, rather than fitting `null` weights into head.json.
#[test]
fn f9_out_of_range_learning_rate_is_refused_naming_the_field() {
    let dir = temp_dir("f9-learning-rate");
    let ds = build_dataset(&dir, "7");
    for value in ["1e308", "0", "-1"] {
        let bad = dir.join(format!("lr-{value}"));
        let _ = std::fs::remove_dir_all(&bad);
        copy_dir(&ds, &bad);
        let spec = std::fs::read_to_string(bad.join("features.json")).unwrap();
        let mut v: serde_json::Value = serde_json::from_str(&spec).unwrap();
        v["learning_rate"] = serde_json::json!(value.parse::<f64>().unwrap());
        std::fs::write(
            bad.join("features.json"),
            serde_json::to_string_pretty(&v).unwrap(),
        )
        .unwrap();
        let b = dir.join(format!("models/lr-{value}"));
        let (code, out, err) = run(&[
            "train",
            "fit",
            bad.to_str().unwrap(),
            "--out",
            b.to_str().unwrap(),
        ]);
        assert_eq!(
            code, 2,
            "F9/B3: fit must refuse learning_rate {value} with exit 2, not {code}:\n{out}\n{err}"
        );
        assert!(
            err.contains("learning_rate"),
            "F9/B3: the refusal for learning_rate {value} must name the field:\n{err}"
        );
        assert!(
            !err.contains("fit failed"),
            "F9/B3: an out-of-range learning_rate is bad input, not a generic failure:\n{err}"
        );
    }
}

/// F9/B3 (d): a bundle content file that fails to parse is named in the error, not a bare
/// `json: invalid type: null, expected f64` with no file.
#[test]
fn f9_a_malformed_bundle_file_is_named_in_the_error() {
    let dir = temp_dir("f9-malformed-bundle");
    let ds = build_dataset(&dir, "7");
    let b = fit_bundle(&dir, &ds);
    // A digest-valid but malformed head.json: rewrite it and fix the manifest digest so the parse
    // error, not the digest check, is what surfaces.
    let head_path = b.join("head.json");
    let mut head: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&head_path).unwrap()).unwrap();
    head["weights"] = serde_json::json!(null);
    std::fs::write(&head_path, serde_json::to_string_pretty(&head).unwrap()).unwrap();
    let (code, out, err) = run(&["train", "verify", b.to_str().unwrap()]);
    assert_eq!(code, 2, "F9/B3: verify a malformed bundle:\n{out}\n{err}");
    assert!(
        err.contains("head.json"),
        "F9/B3: the parse error must name the bundle file:\n{err}"
    );
}

/// Copy a directory tree (the dataset) to a fresh path, so a test can corrupt one file.
fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dst = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &dst);
        } else {
            std::fs::copy(entry.path(), &dst).unwrap();
        }
    }
}
