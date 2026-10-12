//! `tsr train` — the M4 (Calibrate) CLI: dataset, fit, verify, report (PRD §B8, §B9, §B11 M4).
//!
//! This module is the thin CLI layer over `tsr-learn`. It reads the log (a store or an
//! `events.jsonl`), builds the dataset with its split and leakage probes, fits a bundle, verifies a
//! bundle's digests, and scores the controller against the generator on held-out rows.
//!
//! Exit codes: 0 on success; 2 for a refused input (a non-operator label, a caught probe, a fit
//! below the minimum data, a digest mismatch, a forged split manifest); 1 for an I/O or store
//! failure. Every printed number goes through `tsr_domain::Proxy` with its engineering name.

use std::path::{Path, PathBuf};

use clap::Subcommand;
use tsr_domain::{Event, Hash};
use tsr_learn::dataset::{
    build_dataset, read_labels, read_task_index, write_dataset, SplitBy, SplitConfig,
    SplitFractions, TaskIndexRow,
};
use tsr_learn::LearnError;

/// The `tsr train` subcommands.
#[derive(Subcommand)]
pub enum TrainCmd {
    /// Build the dataset, its split and its manifests from the log (B8, B9).
    Dataset {
        /// The store to read the log from (defaults to `TSR_DB` / `.tsr/tsr.db`).
        #[arg(long)]
        store: Option<PathBuf>,
        /// An `events.jsonl` file to read the log from (alternative to `--store`).
        #[arg(long)]
        events: Option<PathBuf>,
        /// The task index: JSONL of `{ task, family, lineage, timestamp, clone_group?, tool_ms_ceiling? }`.
        /// Required: without it every task silently becomes its own lineage with family `unknown`.
        #[arg(long)]
        tasks: PathBuf,
        /// The labels input: JSONL of `{ task, criteria_met, diff_label, reviewer, recorded_at }`.
        /// When omitted, no task is labelled (every row is unresolved).
        #[arg(long)]
        labels: Option<PathBuf>,
        /// The split rule: `time` (forward-time) or `repo` (whole lineages by a seeded shuffle).
        #[arg(long, default_value = "time")]
        by: String,
        /// The calibrate share of the tasks.
        #[arg(long, default_value_t = 0.25)]
        calibrate: f64,
        /// The held-out share of the tasks.
        #[arg(long, default_value_t = 0.25)]
        holdout: f64,
        /// The split seed.
        #[arg(long, default_value_t = 0)]
        seed: u64,
        /// The output directory for the dataset.
        #[arg(long)]
        out: PathBuf,
    },
    /// Fit a bundle: the logistic head and its isotonic calibrator (B8).
    Fit {
        /// The dataset directory written by `tsr train dataset`.
        dataset_dir: PathBuf,
        /// The bundle output directory, e.g. `models/<name>`.
        #[arg(long)]
        out: PathBuf,
        /// The support floor recorded in the bundle (defaults to the Tune default, 30).
        #[arg(long)]
        support_floor: Option<u32>,
    },
    /// Verify a bundle: recompute every digest, and with `--rebuild` retrain and compare ids (B8).
    Verify {
        /// The bundle directory.
        bundle: PathBuf,
        /// A dataset directory to rebuild from and compare ids.
        #[arg(long)]
        rebuild: Option<PathBuf>,
    },
    /// Score the controller against the generator on held-out rows (B7, B9).
    Report {
        /// The bundle directory.
        bundle: PathBuf,
        /// The dataset directory.
        #[arg(long)]
        dataset: PathBuf,
        /// The bootstrap seed.
        #[arg(long, default_value_t = 0)]
        seed: u64,
        /// Emit JSON instead of the human report.
        #[arg(long)]
        json: bool,
    },
    /// Apply a bundle to a dataset's rows: raw p, calibrated p (or uncalibrated) and support (B7, B10).
    Apply {
        /// The bundle directory.
        bundle: PathBuf,
        /// The dataset directory.
        #[arg(long)]
        dataset: PathBuf,
        /// The side to apply to: `held_out` (default), `fit`, `calibrate` or `all`.
        #[arg(long, default_value = "held_out")]
        side: String,
    },
}

/// Run a `tsr train` subcommand, returning the process exit code.
pub fn run(cmd: TrainCmd) -> std::process::ExitCode {
    match cmd {
        TrainCmd::Dataset {
            store,
            events,
            tasks,
            labels,
            by,
            calibrate,
            holdout,
            seed,
            out,
        } => run_dataset(
            store.as_deref(),
            events.as_deref(),
            &tasks,
            labels.as_deref(),
            &by,
            calibrate,
            holdout,
            seed,
            &out,
        ),
        TrainCmd::Fit {
            dataset_dir,
            out,
            support_floor,
        } => run_fit(&dataset_dir, &out, support_floor),
        TrainCmd::Verify { bundle, rebuild } => run_verify(&bundle, rebuild.as_deref()),
        TrainCmd::Report {
            bundle,
            dataset,
            seed,
            json,
        } => run_report(&bundle, &dataset, seed, json),
        TrainCmd::Apply {
            bundle,
            dataset,
            side,
        } => run_apply(&bundle, &dataset, &side),
    }
}

/// Read the log: from an explicit store path, an `events.jsonl` file, or the default store.
fn read_log(store: Option<&Path>, events: Option<&Path>) -> Result<Vec<(i64, Event)>, String> {
    match (store, events) {
        (Some(_), Some(_)) => Err("pass either --store or --events, not both".to_string()),
        (Some(path), None) => {
            let store = tsr_cli::store_path::open_store_at(path)
                .map_err(|e| format!("cannot open store {}: {e}", path.display()))?;
            store.all().map_err(|e| format!("cannot read store: {e}"))
        }
        (None, Some(path)) => read_events_file(path),
        (None, None) => {
            let store =
                tsr_cli::store_path::open_store().map_err(|e| format!("cannot open store: {e}"))?;
            store.all().map_err(|e| format!("cannot read store: {e}"))
        }
    }
}

/// Read an `events.jsonl` file: one JSON object per line, each a bare `Event` or `{"seq": n, "event": …}`.
///
/// F9/B3: this goes through the one loader (`tsr_learn::dataset::read_events_records`), so a line
/// that is valid JSON but not an event record (`{}`, `null`) is refused naming the file, the line
/// and the missing or wrong field -- never exit 1.
fn read_events_file(path: &Path) -> Result<Vec<(i64, Event)>, String> {
    tsr_learn::dataset::read_events_records(path).map_err(|e| e.to_string())
}

/// The source root: the store's `head_root`, or the events file's digest.
fn source_root(store: Option<&Path>, events: Option<&Path>) -> Result<Hash, String> {
    if let Some(path) = store {
        let store = tsr_cli::store_path::open_store_at(path)
            .map_err(|e| format!("cannot open store {}: {e}", path.display()))?;
        return store
            .head_root()
            .map_err(|e| format!("cannot read store head_root: {e}"));
    }
    if let Some(path) = events {
        let bytes =
            std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        return Ok(Hash::of(&[&bytes]));
    }
    let store = tsr_cli::store_path::open_store().map_err(|e| format!("cannot open store: {e}"))?;
    store
        .head_root()
        .map_err(|e| format!("cannot read store head_root: {e}"))
}

/// Write the log and the task index into the dataset directory, so `fit` can re-run the probes.
fn write_log_into(
    dir: &Path,
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let mut ev_text = String::new();
    for (seq, ev) in events {
        let line = serde_json::json!({ "seq": seq, "event": ev });
        ev_text.push_str(&serde_json::to_string(&line).map_err(|e| e.to_string())?);
        ev_text.push('\n');
    }
    std::fs::write(dir.join("events.jsonl"), ev_text)
        .map_err(|e| format!("cannot write events.jsonl: {e}"))?;
    let mut idx_text = String::new();
    for row in index {
        idx_text.push_str(&serde_json::to_string(row).map_err(|e| e.to_string())?);
        idx_text.push('\n');
    }
    std::fs::write(dir.join("tasks.jsonl"), idx_text)
        .map_err(|e| format!("cannot write tasks.jsonl: {e}"))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_dataset(
    store: Option<&Path>,
    events: Option<&Path>,
    tasks: &Path,
    labels: Option<&Path>,
    by: &str,
    calibrate: f64,
    holdout: f64,
    seed: u64,
    out: &Path,
) -> std::process::ExitCode {
    let by = match by {
        "time" => SplitBy::Time,
        "repo" => SplitBy::Repo,
        other => {
            eprintln!("tsr: --by must be `time` or `repo`, not {other:?}");
            return std::process::ExitCode::from(2);
        }
    };
    // F9/B3 (d): one parser. Each input is parsed once, by its record loader below
    // (`read_events_records` / `read_task_index`), which refuses a line that is not a record naming
    // the file, the line and the field. There is no JSON-only pre-pass: a file that is not data is
    // refused by the loader with one message format.
    let log = match read_log(store, events) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("tsr: {e}");
            // F9/B3: an events file that is not data is bad input -- exit 2, never exit 1.
            let code = if events.is_some() { 2 } else { 1 };
            return std::process::ExitCode::from(code);
        }
    };
    // R6: `--tasks` is required. Without it every task silently becomes its own lineage with family
    // `unknown`, so the index is read from the file the caller named.
    let index = match read_task_index(tasks) {
        Ok(i) => i,
        Err(e) => {
            // F9/B3: a tasks.jsonl line that is not a task record is bad input -- exit 2.
            eprintln!("tsr: dataset refused: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    let labels = match labels {
        Some(path) => match read_labels(path) {
            Ok(l) => l,
            Err(e) => {
                // F9/B3: a labels.jsonl line that is not a label record is bad input -- exit 2.
                eprintln!("tsr: dataset refused: {e}");
                return std::process::ExitCode::from(2);
            }
        },
        None => Vec::new(),
    };
    let src = match source_root(store, events) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("tsr: {e}");
            return std::process::ExitCode::from(1);
        }
    };
    let cfg = SplitConfig {
        by,
        fractions: SplitFractions { calibrate, holdout },
        seed,
    };
    let ds = match build_dataset(&log, &index, &labels, tsr_learn::synth::OPERATOR, cfg, src) {
        Ok(ds) => ds,
        Err(LearnError::NonOperatorLabel { reviewer }) => {
            eprintln!("tsr: label record names reviewer {reviewer:?}, who is not the operator (B8: labels only after review)");
            return std::process::ExitCode::from(2);
        }
        Err(e) => {
            eprintln!("tsr: cannot build dataset: {e}");
            return std::process::ExitCode::from(1);
        }
    };
    // A12: run all three probes before writing anything. A leaking split is refused with exit 2
    // rather than written and left for `fit` to catch.
    {
        let families = tsr_learn::features::families_from_spec(&ds.spec);
        let extractor = tsr_learn::probes::default_extractor(&ds.spec, &families);
        let report = tsr_learn::probes::run_all(&ds, &log, &index, &ds.spec, &extractor, None);
        if !report.is_clean() {
            for leak in &report.leaks {
                eprintln!(
                    "tsr: probe {} caught a leak at {}: {}",
                    leak.probe.name(),
                    leak.row,
                    leak.detail
                );
            }
            // F9/B3: a probe that examined no rows, or fewer rows than the held-out slice, is
            // refused naming the probe and the counts -- never skipped.
            for refusal in &report.refusals {
                eprintln!("tsr: {refusal}");
            }
            return std::process::ExitCode::from(2);
        }
    }
    if let Err(e) = write_dataset(&ds, out) {
        eprintln!("tsr: cannot write dataset: {e}");
        return std::process::ExitCode::from(1);
    }
    if let Err(e) = write_log_into(out, &log, &index) {
        eprintln!("tsr: {e}");
        return std::process::ExitCode::from(1);
    }
    println!("dataset {}", out.display());
    println!("rows={}", ds.rows.len());
    println!("data_root={}", ds.data_root.to_hex());
    println!("training_root={}", ds.training_root.to_hex());
    println!("split_digest={}", ds.split.digest.to_hex());
    println!("unindexed_tasks={}", ds.unindexed_tasks);
    println!("uncommitted_tasks={}", ds.uncommitted_tasks);
    std::process::ExitCode::SUCCESS
}

fn run_fit(dataset_dir: &Path, out: &Path, support_floor: Option<u32>) -> std::process::ExitCode {
    // B3: the probes need the log and the task index. Without them they pass silently (and without
    // events.jsonl the probe pass panics), so refuse with exit 2 naming the missing file.
    //
    // F9/B3 (d): one parser. Each file is parsed once, by its record loader inside `write_bundle`
    // (`read_events_records` / `read_task_index`), which refuses a line that is not a record naming
    // the file, the line and the field. There is no JSON-only pre-pass here: a file that is not data
    // is refused by the loader with one message format, and a missing file is refused by name.
    for name in ["events.jsonl", "tasks.jsonl"] {
        let path = dataset_dir.join(name);
        if !path.exists() {
            eprintln!(
                "tsr: fit refused: {} is missing from the dataset directory {}",
                name,
                dataset_dir.display()
            );
            return std::process::ExitCode::from(2);
        }
    }
    let floor = support_floor.unwrap_or_else(tsr_learn::support::default_support_floor);
    match tsr_learn::bundle::write_bundle(dataset_dir, out, floor) {
        Ok(id) => {
            println!("{}", id.to_hex());
            std::process::ExitCode::SUCCESS
        }
        Err(LearnError::Invalid(msg)) => {
            eprintln!("tsr: fit refused: {msg}");
            std::process::ExitCode::from(2)
        }
        Err(e) => {
            eprintln!("tsr: fit failed: {e}");
            std::process::ExitCode::from(1)
        }
    }
}

fn run_verify(bundle: &Path, rebuild: Option<&Path>) -> std::process::ExitCode {
    match tsr_learn::bundle::verify_bundle(bundle, rebuild) {
        Ok(()) => {
            println!("bundle {} verified", bundle.display());
            // P1: `code_commit` is recorded provenance, not a verified property. Print it with the
            // label the operator chose, so a reader never mistakes it for a checked value.
            if let Ok(b) = tsr_learn::bundle::load_bundle(bundle) {
                println!(
                    "code_commit (recorded, not verified)={}",
                    b.manifest.code_commit
                );
            }
            std::process::ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("tsr: verify failed: {e}");
            std::process::ExitCode::from(2)
        }
    }
}

/// The report path beside the bundle: `<bundle>.report.<ext>`.
fn report_path(bundle: &Path, ext: &str) -> std::path::PathBuf {
    let mut s = bundle.as_os_str().to_os_string();
    s.push(format!(".report.{ext}"));
    std::path::PathBuf::from(s)
}

/// Apply a bundle to a dataset's rows (B7, B10): per row its id, family, `raw p`, then `p` with its
/// calibrator version or `uncalibrated`, and `support=<n>`.
///
/// `--side held_out` (the default) reads only held_out rows and logs each read to
/// `holdout-access.jsonl`; `fit`, `calibrate` and `all` read the other sides.
fn run_apply(bundle: &Path, dataset: &Path, side: &str) -> std::process::ExitCode {
    // F14: a tampered bundle or dataset is a refused input, exit 2, not a generic failure.
    let b = match tsr_learn::bundle::load_bundle(bundle) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("tsr: cannot load bundle: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    let ds = match tsr_learn::bundle::load_dataset(dataset) {
        Ok(ds) => ds,
        Err(e) => {
            eprintln!("tsr: cannot load dataset: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    // R4: refuse a dataset that does not belong to the bundle, exactly as report does.
    if let Err(e) = tsr_learn::report::check_dataset_matches_bundle(&b, &ds) {
        eprintln!("tsr train apply: {e}");
        return std::process::ExitCode::from(2);
    }
    let want: Option<tsr_learn::dataset::Side> = match side {
        "held_out" => Some(tsr_learn::dataset::Side::HeldOut),
        "fit" => Some(tsr_learn::dataset::Side::Fit),
        "calibrate" => Some(tsr_learn::dataset::Side::Calibrate),
        "all" => None,
        other => {
            eprintln!("tsr: --side must be `held_out`, `fit`, `calibrate` or `all`, not {other:?}");
            return std::process::ExitCode::from(2);
        }
    };
    // B9: every read of a held_out row is logged, the apply pass included.
    for r in &ds.rows {
        if r.side == tsr_learn::dataset::Side::HeldOut {
            if let Err(e) = tsr_learn::probes::log_holdout_access(dataset, r, "apply") {
                eprintln!("tsr: cannot log holdout access: {e}");
                return std::process::ExitCode::from(1);
            }
        }
    }
    let mut printed = 0u32;
    for r in &ds.rows {
        if let Some(w) = want {
            if r.side != w {
                continue;
            }
        }
        let applied = match b.apply(r) {
            Ok(a) => a,
            Err(e) => {
                eprintln!("tsr: cannot apply bundle to row {}: {e}", r.id.to_hex());
                return std::process::ExitCode::from(2);
            }
        };
        let calibrated = match &applied.calibrated {
            Some(c) => format!(
                "{} ({})",
                tsr_learn::report::proxy("p", c.p.get()),
                c.calibrator
            ),
            None => "uncalibrated".to_string(),
        };
        println!(
            "row {} family {} {} {} support={}",
            r.id.to_hex(),
            r.family,
            tsr_learn::report::proxy("raw p", applied.raw_p),
            calibrated,
            applied.support.episodes
        );
        printed += 1;
    }
    println!("rows={printed}");
    std::process::ExitCode::SUCCESS
}

fn run_report(bundle: &Path, dataset: &Path, seed: u64, json: bool) -> std::process::ExitCode {
    // F14: a tampered bundle or dataset is a refused input, exit 2, not a generic failure.
    let b = match tsr_learn::bundle::load_bundle(bundle) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("tsr: cannot load bundle: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    let ds = match tsr_learn::bundle::load_dataset(dataset) {
        Ok(ds) => ds,
        Err(e) => {
            eprintln!("tsr: cannot load dataset: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    let r = match tsr_learn::report::report_logged(&b, &ds, seed, dataset) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("tsr train report: {e}");
            return std::process::ExitCode::from(2);
        }
    };
    print!("{}", tsr_learn::report::render_report(&r, json));
    // B9: the evaluation report sits beside the bundle as `<bundle>.report.json` (and the text
    // form when not `--json`), naming the bundle id and the dataset's split digest.
    let json_path = report_path(bundle, "json");
    if let Err(e) = std::fs::write(&json_path, tsr_learn::report::render_report(&r, true)) {
        eprintln!("tsr: cannot write {}: {e}", json_path.display());
        return std::process::ExitCode::from(1);
    }
    if !json {
        let text_path = report_path(bundle, "txt");
        if let Err(e) = std::fs::write(&text_path, tsr_learn::report::render_report(&r, false)) {
            eprintln!("tsr: cannot write {}: {e}", text_path.display());
            return std::process::ExitCode::from(1);
        }
    }
    std::process::ExitCode::SUCCESS
}
