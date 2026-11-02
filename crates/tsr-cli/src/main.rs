//! `tsr` — the operator's CLI and the hook binary of the TSR Decision Engine.
//!
//! M0 (PRD §B11) fills `hook`, `exec`, `task` and `forecast`; the later-milestone commands still
//! exit 2 with one sentence naming the PRD section, so scaffolding is never mistaken for a feature.

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;
use tsr_cli::{exec, hook, store_path, task};

use tsr_cli::worker_config;
mod batch_cmd;
pub mod corpus_cmd;
mod engine_project_cmd;
mod eval_cmd;
mod policy_cmd;
use tsr_cli::note_cmd;
use tsr_cli::reconcile_cmd;
use tsr_cli::reflect_cmd;
use tsr_cli::review_cmd;
mod train_cmd;

#[derive(Parser)]
#[command(
    name = "tsr",
    version,
    about = "TSR Decision Engine — PRD v0.3.1 (docs/PRD.md)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the persistent controller, scoped projects and explicit proposal transports.
    Engine {
        #[arg(long)]
        directory: PathBuf,
        #[command(subcommand)]
        cmd: EngineCmd,
    },
    /// Record explicitly operator-reviewed criterion/candidate impact links.
    Tensions {
        #[arg(long)]
        store: PathBuf,
        #[arg(long)]
        task: String,
        #[arg(long)]
        reviewed_links: PathBuf,
        #[arg(long)]
        operator_review: bool,
        #[arg(long)]
        family: String,
        #[arg(long)]
        repository: String,
    },
    /// Select admitted open preferences and record the reproducible trace (M3).
    Select {
        #[arg(long)]
        store: PathBuf,
        #[arg(long)]
        task: String,
        #[arg(long)]
        tune: Option<PathBuf>,
        #[arg(long)]
        family: String,
        #[arg(long)]
        repository: String,
        #[arg(long)]
        seed: u64,
        #[arg(long)]
        minimum_depth: u32,
    },

    /// Task intake, status, trace export (M0: B3, B5, B11).
    Task {
        #[command(subcommand)]
        cmd: TaskCmd,
    },
    /// Record a forecast for the current state version before a gated action (B5, B7).
    Forecast {
        /// The gated action being forecast: run_approved_test | propose_patch | request_approval
        #[arg(long)]
        kind: Option<String>,
        /// The action's arguments: the Bash command, or the Edit/Write file path
        #[arg(long)]
        args: Option<String>,
        /// An already-proposed candidate id (alternative to --kind/--args)
        #[arg(long)]
        event: Option<String>,
        /// Raw probability in [0, 1]; always recorded, acted on only when calibrated
        #[arg(long)]
        p: f64,
        /// Forecaster: generator | controller | operator
        #[arg(long, default_value = "generator")]
        source: String,
        /// The task to record against (defaults to the session task)
        #[arg(long)]
        task: Option<String>,
        /// The session to record against (defaults to the session the hook last saw)
        #[arg(long)]
        session: Option<String>,
    },
    /// The hook entry point: reads a Claude Code hook payload on stdin and records it (M0, B4).
    Hook {
        /// PreToolUse | PostToolUse | SessionStart | Stop
        #[arg(long, default_value = "")]
        stage: String,
        /// Drive realistic payloads through the hook and write fixtures/m0/session.events.jsonl
        #[arg(long)]
        selftest: bool,
    },
    /// Run a command in the sealed sibling sandbox and reconcile the result (M0, B4).
    Exec {
        #[arg(trailing_var_arg = true)]
        argv: Vec<String>,
    },
    /// Evaluate a RATCHET protection profile against an event (M1, B5).
    Policy {
        #[command(subcommand)]
        cmd: PolicyCmd,
    },
    /// Record the operator's experience note on a task (M2, B7).
    Note {
        task: String,
        /// A short note on friction or trust; never reward truth.
        #[arg(long)]
        text: String,
        /// The store holding the task's log (defaults to the M0 store: `TSR_DB`, else `.tsr/tsr.db`).
        #[arg(long)]
        store: Option<String>,
    },
    /// Label outcomes after close (M2, B5, B7).
    Review {
        /// The task whose returns are labelled; omit with --stats to compute over the family.
        task: Option<String>,
        /// accepted | rejected | disputed; omit to list the returns awaiting a label.
        label: Option<String>,
        /// The reason for a rejection; stored apart from tool results.
        #[arg(long)]
        reason: Option<String>,
        /// Name the return to label; a correction supersedes its standing label.
        #[arg(long = "return")]
        return_id: Option<String>,
        /// Sample greens for review at this rate (0, 1].
        #[arg(long)]
        sample: Option<f64>,
        /// Fix the sampling draw.
        #[arg(long)]
        seed: Option<u64>,
        /// Restrict --stats to one task family.
        #[arg(long)]
        family: Option<String>,
        /// Print the false-accept rate (green but rejected), weighted and unweighted.
        #[arg(long)]
        stats: bool,
        /// The store holding the task's log (defaults to the M0 store: `TSR_DB`, else `.tsr/tsr.db`).
        #[arg(long)]
        store: Option<String>,
    },
    /// The Mirror view: one closed episode, in the book's order (M2, B7).
    Reflect {
        /// The task to render; omit with --family to print calibration curves.
        task: Option<String>,
        /// Print each forecaster's calibration curve for this task family.
        #[arg(long)]
        family: Option<String>,
        /// Emit the same content as a JSON document.
        #[arg(long)]
        json: bool,
        /// The store holding the task's log (defaults to the M0 store: `TSR_DB`, else `.tsr/tsr.db`).
        #[arg(long)]
        store: Option<String>,
    },
    /// Corpus tooling: mine, mutants (M3, B9).
    Corpus {
        #[command(subcommand)]
        cmd: CorpusCmd,
    },
    /// Dataset validation and `forget` (M4, B8, B10).
    Data {
        #[command(subcommand)]
        cmd: DataCmd,
    },
    /// Train calibration heads (M4, B8).
    Train {
        #[command(subcommand)]
        cmd: train_cmd::TrainCmd,
    },
    /// Evaluate: calibration, paired comparison, state isolation (M4–M5, B9).
    Eval {
        #[command(subcommand)]
        cmd: EvalCmd,
    },
    /// Bundle registry: promote, rollback (M5, B8).
    Model {
        #[command(subcommand)]
        cmd: ModelCmd,
    },
}

#[derive(Subcommand)]
enum TaskCmd {
    /// Open a task: workspace, intent, criteria, limits, capabilities (FR 001).
    Open {
        /// The task name (defaults to the session id)
        #[arg(long)]
        name: Option<String>,
        /// The workspace the task runs in
        #[arg(long)]
        workspace: Option<String>,
        /// The task's intent
        #[arg(long)]
        intent: Option<String>,
    },
    /// Run a batch of corpus tasks through a driver (M3, B9, B11).
    Run {
        /// record | rules
        #[arg(long)]
        mode: String,
        /// The corpus task manifest (tasks.jsonl).
        #[arg(long)]
        tasks: PathBuf,
        /// scripted | claude-code
        #[arg(long)]
        driver: String,
        /// Output directory: results.jsonl and the per-task stores.
        #[arg(long)]
        out: PathBuf,
        /// Test-only: run the oracle through a scripted runner (a canned success) instead of the
        /// production `ExecRunner`. Never use this for evidence: a scripted oracle cannot tell a
        /// session that fixed the flipped test from one that did not (D3).
        #[arg(long)]
        scripted_runner: bool,
        /// The RATCHET protection profile to run under. Defaults to
        /// `<store-root>/profiles/default.toml`, where `<store-root>` is the directory holding the
        /// store's `.tsr/` (D10). There is no compile-time fallback.
        #[arg(long)]
        profile: Option<PathBuf>,
    },
    Status {
        task: String,
    },
    /// Export a task's trace as JSON (M0 exit evidence).
    Export {
        task: String,
    },
    /// Reconcile dispatches with no recorded outcome (M2, B4/B6).
    Reconcile {
        task: String,
        /// The store holding the task's log (defaults to the M0 store: `TSR_DB`, else `.tsr/tsr.db`).
        #[arg(long)]
        store: Option<String>,
    },
}

#[derive(Subcommand)]
enum PolicyCmd {
    Check { profile: String, event: String },
}

#[derive(Subcommand)]
enum CorpusCmd {
    /// Independently qualify mutant candidates with baseline, named red, and reversal green.
    Qualify {
        #[arg(long)]
        tasks: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Record generated-lock provenance without executing source or resolving dependencies.
    Prepare {
        #[arg(long)]
        request: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Mine a repository's first-parent history for red→green flip tasks (B9).
    Mine {
        repo: String,
        /// Output directory: `tasks.jsonl` and the CAS are written here.
        #[arg(long)]
        out: PathBuf,
        /// Only walk commits reachable from this revision (default: HEAD).
        #[arg(long)]
        since: Option<String>,
        /// The suite command (default: `cargo test`).
        #[arg(long)]
        suite: Option<String>,
    },
    Mutants {
        repo: String,
        /// Bounded operator-selected scope; repeated flags form one explicit candidate set.
        #[arg(long)]
        package: Option<String>,
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        mutant: Option<String>,
        /// Restrict mutation screening to unit-test targets; admission still requires full-suite evidence.
        #[arg(long, value_parser=["lib","bins"])]
        screen_target: Option<String>,
        /// Output directory: `tasks.jsonl` and the CAS are written here.
        #[arg(long)]
        out: PathBuf,
    },
    /// Split a corpus into train and holdout with clone groups and leakage probes (B9).
    Split {
        /// The corpus directory: `tasks.jsonl` and the CAS are read here.
        dir: PathBuf,
        /// The split rule: `time` (forward-time by task) or `repo` (lineage-separated).
        #[arg(long)]
        by: String,
        /// The holdout fraction, in (0, 1).
        #[arg(long)]
        holdout: f64,
        /// The seed: the same seed and corpus give the same split.
        #[arg(long)]
        seed: u64,
    },
    /// Materialise a snapshot into a disposable copy (D4).
    ///
    /// The batch runner provisions the copy it drives a session in by asking the sealed sandbox to
    /// run this command: `tsr corpus materialize --snapshot <sha> --out <dest>`. It is a real
    /// subcommand so the argv the runner builds names a command that exists.
    Materialize {
        /// The snapshot to materialise: a git revision (sha, tag or ref).
        #[arg(long)]
        snapshot: String,
        /// The directory to materialise into.
        #[arg(long)]
        out: PathBuf,
        /// The repository to materialise from (default: the working directory).
        #[arg(long)]
        repo: Option<PathBuf>,
    },
    /// Run the oracle in a materialised copy and report pass/fail (D4).
    ///
    /// The batch runner runs the oracle by asking the sealed sandbox to run this command:
    /// `tsr corpus oracle --snapshot <dir> --tests <csv>` (the flipped tests) or
    /// `tsr corpus oracle --snapshot <dir> --full-suite` (the regression check). It is a real
    /// subcommand so the argv the runner builds names a command that exists.
    Oracle {
        /// The directory the oracle runs in: the session's copy, not the parent snapshot (D3).
        #[arg(long)]
        snapshot: PathBuf,
        /// The comma-separated test ids to run (the flipped tests).
        #[arg(long, conflicts_with = "full_suite")]
        tests: Option<String>,
        /// Run the full suite instead of a named set of tests.
        #[arg(long)]
        full_suite: bool,
        /// The suite command (default: `cargo test`).
        #[arg(long)]
        suite: Option<String>,
        /// Run each named test exactly (`cargo test -- <name> --exact`), so a filter that matches
        /// nothing is a failure rather than a silent pass (D3, P3).
        #[arg(long)]
        exact: bool,
    },
}

#[derive(Subcommand)]
enum DataCmd {
    Validate,
    Forget { episode: String },
}

#[derive(Subcommand)]
enum EvalCmd {
    /// The first numbers with intervals: paired by task, cluster bootstrap by
    /// lineage, 90% intervals, INCONCLUSIVE when an interval spans zero (B9).
    Baseline {
        /// The results directory: `results.jsonl` is read here. Use this when
        /// one file holds both systems; use `--record`/`--rules` when each run
        /// wrote its own directory.
        #[arg(conflicts_with_all = ["record", "rules"])]
        results_dir: Option<PathBuf>,
        /// The record system's results directory (`results.jsonl` is read here).
        #[arg(long, requires = "rules")]
        record: Option<PathBuf>,
        /// The rules system's results directory (`results.jsonl` is read here).
        #[arg(long, requires = "record")]
        rules: Option<PathBuf>,
        /// The pairing rule (only `task` is defined).
        #[arg(long, default_value = "task")]
        pair_by: String,
        /// The bootstrap seed: the same seed gives the same report.
        #[arg(long, default_value_t = tsr_eval::DEFAULT_SEED)]
        seed: u64,
    },
}

#[derive(Subcommand)]
enum ModelCmd {
    List,
    Promote { bundle: String },
    Rollback,
}

#[derive(Subcommand)]
enum EngineCmd {
    ProjectLifecycleOpen {
        #[arg(long)]
        admission: PathBuf,
    },
    ProjectLifecycleView,
    ProjectFeatures,
    ProjectDataset {
        #[arg(long)]
        as_of: String,
        #[arg(long)]
        out: PathBuf,
    },
    ProjectTrainingEvidence {
        #[arg(long)]
        as_of: String,
        #[arg(long)]
        out: PathBuf,
    },
    ProjectEligibilityView,
    ProjectEligibilityReview {
        #[arg(long)]
        review: PathBuf,
    },
    ProjectDatasetValidate {
        #[arg(long)]
        artifact: PathBuf,
    },
    ProjectBudget,
    ProjectResourceExtend {
        #[arg(long)]
        request: PathBuf,
    },
    ProjectResourceReserve {
        #[arg(long)]
        request: PathBuf,
    },
    ProjectResourceNotDispatched {
        #[arg(long)]
        proof: PathBuf,
    },
    ProjectResourceReconcileFailedZero {
        #[arg(long)]
        proof: PathBuf,
    },
    ProjectResourceSettle {
        #[arg(long)]
        usage: PathBuf,
    },
    ProjectLifecycleClose,
    ProjectLifecycleCloseInconclusive {
        #[arg(long)]
        reason: String,
    },
    ProjectLifecycleCloseAdvisory {
        #[arg(long)]
        reason: String,
    },
    ShadowConfigure {
        #[arg(long)]
        update: PathBuf,
    },
    ShadowView,
    Tier2Configure {
        #[arg(long)]
        update: PathBuf,
    },
    Tier2View,
    /// Set a shared operator near-tie policy; exploration requires explicit epsilon > 0.
    DecisionPolicy {
        #[arg(long)]
        update: PathBuf,
    },
    DecisionPolicyView,
    DirectionInit {
        #[arg(long)]
        spec: PathBuf,
    },
    DirectionRequest,
    DirectionRelate {
        #[arg(long)]
        response: PathBuf,
    },
    DirectionRelationStatus {
        #[arg(long)]
        update: PathBuf,
    },
    /// Set durable operator quiet-route attention probability and replay seed.
    DirectionTune {
        #[arg(long)]
        update: PathBuf,
    },
    /// Withdraw, restore or mark a notebook desire out of scope (operator only).
    DirectionStatus {
        #[arg(long)]
        update: PathBuf,
    },
    /// Record operator-observed changes to already admitted evidence; grants no authority.
    DirectionObserve {
        #[arg(long)]
        observation: PathBuf,
    },
    DirectionRefine {
        #[arg(long)]
        response: PathBuf,
    },
    Deliberate {
        #[arg(long)]
        response: PathBuf,
    },
    /// Initialize, prepare, run/resume, inspect and export a scoped Cargo project.
    Project {
        #[command(subcommand)]
        command: engine_project_cmd::ProjectCmd,
    },
    ProjectInit {
        #[arg(long)]
        spec: PathBuf,
        /// Import only explicitly listed paths from this source directory.
        #[arg(long)]
        source: Option<PathBuf>,
    },
    ProjectConfig,
    ProjectExport,
    ProjectPrepare,
    CodingInit {
        /// Admit generated exact edits within the disposable implementation.rs scope.
        #[arg(long)]
        generated: bool,
        #[arg(long)]
        image: String,
        #[arg(long, default_value = "501:20")]
        user: String,
    },
    CodingPlan,
    CodingRun {
        #[arg(long)]
        commitment: String,
        #[arg(long)]
        expected_source: String,
    },
    Request,
    Propose {
        #[arg(long)]
        response: PathBuf,
        #[arg(long)]
        apply: bool,
    },
    Configure {
        #[arg(long)]
        spec: PathBuf,
    },
    Next,
    Observe {
        #[arg(long)]
        commitment: String,
        #[arg(long)]
        result: PathBuf,
    },
    State,
    Accept {
        preference: String,
    },
    Init {
        #[arg(long, default_value_t = 3)]
        batch_size: u32,
    },
    Step {
        #[arg(long, default_value_t = 1)]
        cycles: u32,
        #[arg(long)]
        json: bool,
    },
    Show,
    Admit {
        preference: String,
    },
}
fn read_direction_json<T: serde::de::DeserializeOwned>(
    path: &std::path::Path,
) -> anyhow::Result<T> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(65537)
        .read_to_end(&mut bytes)?;
    anyhow::ensure!(bytes.len() <= 65536, "direction input exceeds 64 KiB");
    Ok(serde_json::from_slice(&bytes)?)
}
fn run_engine(directory: &std::path::Path, cmd: EngineCmd) -> anyhow::Result<()> {
    use tsr_cli::engine::{Engine, LocalProposer};
    if let EngineCmd::Project { command } = cmd {
        return engine_project_cmd::run(directory, command);
    }
    let engine = Engine::open(directory)?;
    match cmd {
        EngineCmd::Project { .. } => unreachable!(),
        EngineCmd::DirectionInit { spec } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.deliberation_initialize(read_direction_json(&spec)?)?
            )?
        ),
        EngineCmd::DirectionRefine { response } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.refine_directions(read_direction_json(&response)?)?
            )?
        ),
        EngineCmd::DirectionRelate { response } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.submit_relational(read_direction_json(&response)?)?
            )?
        ),
        EngineCmd::DirectionRelationStatus { update } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.relational_status(read_direction_json(&update)?)?
            )?
        ),
        EngineCmd::DirectionStatus { update } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.update_desire_status(read_direction_json(&update)?)?
            )?
        ),
        EngineCmd::ProjectLifecycleOpen { admission } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.project_lifecycle_open(read_direction_json(&admission)?)?
            )?
        ),
        EngineCmd::ProjectBudget => println!(
            "{}",
            serde_json::to_string_pretty(&engine.project_budget()?)?
        ),
        EngineCmd::ProjectResourceReserve { request } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.project_resource_reserve(read_direction_json(&request)?)?
            )?
        ),
        EngineCmd::ProjectResourceNotDispatched { proof } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.project_resource_not_dispatched(read_direction_json(&proof)?)?
            )?
        ),
        EngineCmd::ProjectResourceReconcileFailedZero { proof } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.project_resource_reconcile_failed_zero(read_direction_json(&proof)?)?
            )?
        ),
        EngineCmd::ProjectResourceSettle { usage } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.project_resource_settle(read_direction_json(&usage)?)?
            )?
        ),
        EngineCmd::ProjectFeatures => println!(
            "{}",
            serde_json::to_string_pretty(&engine.project_features()?)?
        ),
        EngineCmd::ProjectDataset { as_of, out } => {
            let artifact = engine.project_dataset(&as_of)?;
            artifact.write(&out)?;
            println!(
                "{}",
                serde_json::json!({"artifact":out,"digest":artifact.digest,"rows":artifact.rows.len(),"eligible_rows":artifact.eligible_rows,"exclusion":artifact.exclusion})
            );
        }
        EngineCmd::ProjectTrainingEvidence { as_of, out } => {
            use std::io::Write;
            let input = engine.project_training_evidence(&as_of)?;
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&out)?;
            f.write_all(&serde_json::to_vec_pretty(&input)?)?;
            f.sync_all()?;
            println!(
                "{}",
                serde_json::json!({"path":out,"digest":input.digest()?})
            );
        }
        EngineCmd::ProjectEligibilityView => println!(
            "{}",
            serde_json::to_string_pretty(&engine.project_eligibility_view()?)?
        ),
        EngineCmd::ProjectEligibilityReview { review } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.project_eligibility_review(read_direction_json(&review)?)?
            )?
        ),
        EngineCmd::ProjectDatasetValidate { artifact } => {
            let data = tsr_learn::project_receipts::Artifact::load(&artifact)?;
            println!(
                "{}",
                serde_json::json!({"valid":true,"digest":data.digest,"rows":data.rows.len(),"eligible_rows":data.eligible_rows,"fit_eligible":false})
            );
        }
        EngineCmd::ProjectResourceExtend { request } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.project_resource_extend(read_direction_json(&request)?)?
            )?
        ),
        EngineCmd::ProjectLifecycleView => println!(
            "{}",
            serde_json::to_string_pretty(&engine.project_lifecycle_view()?)?
        ),
        EngineCmd::ProjectLifecycleCloseInconclusive { reason } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.project_lifecycle_close_inconclusive(reason)?)?
        ),
        EngineCmd::ProjectLifecycleCloseAdvisory { reason } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.project_lifecycle_close_advisory(reason)?)?
        ),
        EngineCmd::ProjectLifecycleClose => println!(
            "{}",
            serde_json::to_string_pretty(&engine.project_lifecycle_close()?)?
        ),
        EngineCmd::ShadowConfigure { update } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.configure_shadow(read_direction_json(&update)?)?)?
        ),
        EngineCmd::Tier2Configure { update } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.update_tier2(read_direction_json(&update)?)?)?
        ),
        EngineCmd::Tier2View => {
            println!("{}", serde_json::to_string_pretty(&engine.tier2_view()?)?)
        }
        EngineCmd::ShadowView => {
            println!("{}", serde_json::to_string_pretty(&engine.shadow_view()?)?)
        }
        EngineCmd::DecisionPolicy { update } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.update_decision_policy(read_direction_json(&update)?)?
            )?
        ),
        EngineCmd::DecisionPolicyView => println!(
            "{}",
            serde_json::to_string_pretty(&engine.decision_policy_view()?)?
        ),
        EngineCmd::DirectionTune { update } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.update_selection_tune(read_direction_json(&update)?)?
            )?
        ),
        EngineCmd::DirectionRequest => println!(
            "{}",
            serde_json::to_string_pretty(&engine.deliberation_request()?)?
        ),
        EngineCmd::DirectionObserve { observation } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.update_deliberation_observation(read_direction_json(&observation)?)?
            )?
        ),
        EngineCmd::Deliberate { response } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.deliberate(read_direction_json(&response)?)?)?
        ),
        EngineCmd::ProjectInit { spec, source } => {
            let spec = tsr_cli::engine::adaptive::coding::project::ProjectSpec::from_file(&spec)?;
            let state = match source {
                Some(source) => engine.project_import(spec, &source)?,
                None => engine.project_initialize(spec)?,
            };
            println!("{}", serde_json::to_string_pretty(&state)?);
        }
        EngineCmd::ProjectExport => println!(
            "{}",
            serde_json::to_string_pretty(&engine.project_export()?)?
        ),
        EngineCmd::ProjectPrepare => {
            let result = engine.project_prepare()?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            anyhow::ensure!(result["ready"] == true, "dependencies unavailable; prepare a separate pinned image with the locked crates before execution");
        }
        EngineCmd::ProjectConfig => println!(
            "{}",
            serde_json::to_string_pretty(&engine.project_configuration()?)?
        ),
        EngineCmd::CodingInit {
            image,
            user,
            generated,
        } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.coding_initialize_mode(image, user, generated)?)?
        ),
        EngineCmd::CodingPlan => {
            println!("{}", serde_json::to_string_pretty(&engine.coding_plan()?)?)
        }
        EngineCmd::CodingRun {
            commitment,
            expected_source,
        } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.coding_run(
                tsr_domain::EventId(tsr_domain::Hash::from_hex(&commitment)?),
                tsr_domain::Hash::from_hex(&expected_source)?
            )?)?
        ),
        EngineCmd::Request => println!(
            "{}",
            serde_json::to_string_pretty(&engine.proposal_request()?)?
        ),
        EngineCmd::Propose { response, apply } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.evaluate_proposal(engine.proposal_response_from_file(&response)?, apply)?
            )?
        ),
        EngineCmd::Configure { spec } => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.configure(serde_json::from_slice(&std::fs::read(spec)?)?)?
            )?
        ),
        EngineCmd::Next => println!(
            "{}",
            serde_json::to_string_pretty(
                &engine.next(&tsr_cli::engine::adaptive::DeterministicProvider)?
            )?
        ),
        EngineCmd::State => println!(
            "{}",
            serde_json::to_string_pretty(&engine.adaptive_state()?)?
        ),
        EngineCmd::Observe { commitment, result } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.observe(
                tsr_domain::EventId(tsr_domain::Hash::from_hex(&commitment)?),
                serde_json::from_slice(&std::fs::read(result)?)?
            )?)?
        ),
        EngineCmd::Accept { preference } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.adaptive_admit(tsr_domain::PreferenceId(
                tsr_domain::Hash::from_hex(&preference)?
            ))?)?
        ),
        EngineCmd::Init { batch_size } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.initialize(batch_size)?)?
        ),
        EngineCmd::Show => println!("{}", serde_json::to_string_pretty(&engine.view()?)?),
        EngineCmd::Admit { preference } => println!(
            "{}",
            serde_json::to_string_pretty(&engine.admit(tsr_domain::PreferenceId(
                tsr_domain::Hash::from_hex(&preference)?
            ))?)?
        ),
        EngineCmd::Step { cycles, json } => {
            anyhow::ensure!((1..=100).contains(&cycles), "cycles must be 1..100");
            for _ in 0..cycles {
                let trace = engine.step(&LocalProposer)?;
                if json {
                    println!("{}", serde_json::to_string(&trace)?);
                } else {
                    println!(
                        "Cycle {}: Desire — deliver all {} requested items",
                        trace.cycle, trace.requested
                    );
                    println!(
                        "  Before: Belief {:?}; Expectation {:?}",
                        trace.before.last_route,
                        trace
                            .before
                            .estimates
                            .iter()
                            .map(|e| (e.route, e.prediction.value))
                            .collect::<Vec<_>>()
                    );
                    for choice in &trace.possibilities {
                        println!(
                            "  Intuition {:?}: {} ({})",
                            choice.route,
                            if choice.admitted {
                                "supported"
                            } else {
                                "not admitted"
                            },
                            choice.reason
                        );
                    }
                    println!(
                        "  Belief committed {:?}; Return {}/{} delivered",
                        trace.chosen, trace.delivered, trace.requested
                    );
                    println!("  Surprise residual={:.3}; Contrast unmet={}; unmet_pressure {:.3} -> {:.3}", trace.residual.value, trace.unmet, trace.before.unmet_pressure.value, trace.after.unmet_pressure.value);
                    println!("  After: Expectation {:?}; accumulated preferences={}, pending review={}\n", trace.after.estimates.iter().map(|e| (e.route,e.prediction.value)).collect::<Vec<_>>(), trace.after.preferences.len(), trace.after.preferences.iter().filter(|p| !p.is_active()).count());
                }
            }
        }
    }
    Ok(())
}

fn not_yet(what: &str, section: &str) -> ExitCode {
    eprintln!("tsr: `{what}` is not implemented yet — see docs/PRD.md {section}. This is the M0 skeleton.");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Engine { directory, cmd } => match run_engine(&directory, cmd) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("tsr engine: {e}");
                ExitCode::from(2)
            }
        },
        Cmd::Task { cmd } => run_task(cmd),
        Cmd::Forecast {
            kind,
            args,
            event,
            p,
            source,
            task,
            session,
        } => run_forecast(
            kind.as_deref(),
            args.as_deref(),
            event.as_deref(),
            p,
            &source,
            task.as_deref(),
            session.as_deref(),
        ),
        Cmd::Hook { stage, selftest } => run_hook(&stage, selftest),
        Cmd::Exec { argv } => run_exec(&argv),
        Cmd::Policy { cmd } => match cmd {
            PolicyCmd::Check { profile, event } => policy_cmd::run(&profile, &event),
        },
        Cmd::Tensions {
            store,
            task,
            reviewed_links,
            operator_review,
            family,
            repository,
        } => {
            match tsr_cli::tensions_cmd::run(
                &store,
                &task,
                &reviewed_links,
                operator_review,
                &family,
                &repository,
            ) {
                Ok(trace) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&trace).expect("serializable")
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("tsr: tensions: {e}");
                    ExitCode::from(2)
                }
            }
        }
        Cmd::Select {
            store,
            task,
            tune,
            family,
            repository,
            seed,
            minimum_depth,
        } => {
            match tsr_cli::select_cmd::run(
                &store,
                &task,
                tune.as_deref(),
                &family,
                &repository,
                seed,
                minimum_depth,
            ) {
                Ok(trace) => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&trace).expect("serializable")
                    );
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("tsr: select: {error}");
                    ExitCode::from(2)
                }
            }
        }
        Cmd::Note { task, text, store } => note_cmd::run(&task, &text, store.as_deref()),
        Cmd::Review {
            task,
            label,
            reason,
            return_id,
            sample,
            seed,
            family,
            stats,
            store,
        } => review_cmd::run(&review_cmd::ReviewArgs {
            task: task.unwrap_or_default(),
            label,
            reason,
            return_id,
            sample,
            seed,
            family,
            stats,
            store,
        }),
        Cmd::Reflect {
            task,
            family,
            json,
            store,
        } => reflect_cmd::run(&reflect_cmd::ReflectArgs {
            task,
            family,
            json,
            store,
        }),
        Cmd::Corpus { cmd } => match cmd {
            CorpusCmd::Mine {
                repo,
                out,
                since,
                suite,
            } => corpus_cmd::mine_cmd(repo, out, since, suite),
            CorpusCmd::Qualify { tasks, out } => corpus_cmd::qualify_cmd(tasks, out),
            CorpusCmd::Prepare { request, out } => corpus_cmd::prepare_cmd(request, out),
            CorpusCmd::Mutants {
                repo,
                out,
                package,
                file,
                mutant,
                screen_target,
            } => corpus_cmd::mutants_cmd(repo, out, package, file, mutant, screen_target),
            CorpusCmd::Split {
                dir,
                by,
                holdout,
                seed,
            } => corpus_cmd::split_cmd(dir, by, holdout, seed),
            CorpusCmd::Materialize {
                snapshot,
                out,
                repo,
            } => corpus_cmd::materialize_cmd(repo.as_deref(), &snapshot, &out),
            CorpusCmd::Oracle {
                snapshot,
                tests,
                full_suite,
                suite,
                exact,
            } => corpus_cmd::oracle_cmd(
                &snapshot,
                tests.as_deref(),
                full_suite,
                suite.as_deref(),
                exact,
            ),
        },
        Cmd::Data { .. } => not_yet("data", "B8/B10 (M4)"),
        Cmd::Train { cmd } => train_cmd::run(cmd),
        Cmd::Eval { cmd } => match cmd {
            EvalCmd::Baseline {
                results_dir,
                record,
                rules,
                pair_by,
                seed,
            } => eval_cmd::baseline_cmd(
                results_dir.as_deref(),
                record.as_deref(),
                rules.as_deref(),
                &pair_by,
                seed,
            ),
        },
        Cmd::Model { .. } => not_yet("model", "B8 bundles (M5)"),
    }
}

fn run_task(cmd: TaskCmd) -> ExitCode {
    // N2: `status` and `export` are M0 arms (lane 01's). At d2b5f96 `run_task` opened the shared
    // store up front, so both create the store when it is missing and exit 0. Restore that.
    //
    // `reconcile` is M2 and reads the log itself: it must refuse a missing store with exit 2
    // naming the path rather than create one (R5), so it is dispatched before the shared store is
    // opened.
    if let TaskCmd::Reconcile { task, store } = &cmd {
        return reconcile_cmd::run(task, store.as_deref());
    }
    let store = match store_path::open_store() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store: {e}");
            return ExitCode::from(1);
        }
    };
    match cmd {
        TaskCmd::Open {
            name,
            workspace,
            intent,
        } => {
            let name = name.unwrap_or_else(|| "default".to_string());
            let workspace = workspace.unwrap_or_else(|| {
                std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            });
            let intent = intent.unwrap_or_default();
            match task::open(&store, &name, &workspace, &intent) {
                Ok(id) => {
                    println!("{id}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("tsr: task open failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
        TaskCmd::Status { task: name } => {
            let id = store_path::resolve_task(&store, &name);
            match task::status(&store, &id) {
                Ok((count, last)) => {
                    println!("task {id}: {count} events");
                    if let Some(ev) = last {
                        println!("last: {}", serde_json::to_string(&ev).unwrap_or_default());
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("tsr: task status failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
        TaskCmd::Reconcile { .. } => unreachable!("reconcile is dispatched before the store opens"),
        TaskCmd::Run {
            mode,
            tasks,
            driver,
            out,
            scripted_runner,
            profile,
        } => batch_cmd::run_cmd(
            &mode,
            &tasks,
            &driver,
            &out,
            scripted_runner,
            profile.as_deref(),
        ),
        TaskCmd::Export { task: name } => {
            let id = store_path::resolve_task(&store, &name);
            match task::export(&store, &id) {
                Ok(json) => {
                    print!("{json}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("tsr: task export failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
    }
}

fn run_forecast(
    kind: Option<&str>,
    args: Option<&str>,
    event: Option<&str>,
    p: f64,
    source: &str,
    task: Option<&str>,
    session: Option<&str>,
) -> ExitCode {
    let store = match store_path::open_store() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store: {e}");
            return ExitCode::from(1);
        }
    };
    // One session-resolution rule, shared with the hook and `tsr exec`: an explicit `--task` wins,
    // then `--session`/`TSR_SESSION`/the `.tsr/session` file the hook wrote. No "latest task" and no
    // `hash("default")` fallback — with no session we refuse rather than guess.
    let task_id = match task {
        Some(name) => store_path::resolve_task(&store, name),
        None => match store_path::resolve_session_task(session) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("tsr: {e}");
                return ExitCode::from(2);
            }
        },
    };
    if !matches!(source, "generator" | "controller" | "operator") {
        eprintln!("tsr: unknown --source `{source}` (generator|controller|operator)");
        return ExitCode::from(1);
    }
    let result = match (kind, args, event) {
        (Some(kind), Some(args), None) => {
            let kind = match hook::parse_action_kind(kind) {
                Some(k) => k,
                None => {
                    eprintln!(
                        "tsr: unknown --kind `{kind}` (run_approved_test|propose_patch|request_approval)"
                    );
                    return ExitCode::from(1);
                }
            };
            hook::record_forecast_for_action(&store, &task_id, kind, args, p, source)
        }
        (None, None, Some(event)) => {
            let cid = match hook::parse_candidate_id(event) {
                Some(c) => c,
                None => {
                    eprintln!("tsr: --event must be a 64-hex candidate id");
                    return ExitCode::from(1);
                }
            };
            hook::record_forecast_for_candidate(&store, &task_id, cid, p, source)
        }
        _ => {
            eprintln!("tsr: forecast needs either --kind <kind> --args <string> or --event <id>");
            return ExitCode::from(1);
        }
    };
    match result {
        Ok(id) => {
            println!("{id}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("tsr: forecast failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn run_hook(stage: &str, selftest: bool) -> ExitCode {
    if selftest {
        return match tsr_cli::hook::selftest() {
            Ok(path) => {
                println!("wrote {}", path.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("tsr: hook selftest: {e}");
                ExitCode::from(3)
            }
        };
    }
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        // M0 records and never blocks: an unreadable payload is not a reason to fail the turn.
        return ExitCode::SUCCESS;
    }
    let store = match store_path::open_store() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store: {e}");
            return ExitCode::SUCCESS;
        }
    };
    match hook::run(&store, stage, &input) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            // Never block: a recording failure is logged, not enforced.
            eprintln!("tsr: hook recorded nothing: {e}");
            ExitCode::SUCCESS
        }
    }
}

fn run_exec(argv: &[String]) -> ExitCode {
    let store = match store_path::open_store() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store: {e}");
            return ExitCode::from(1);
        }
    };
    let workspace = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    // The same session-resolution rule the hook used to commit the candidate: `TSR_SESSION` (set by
    // the caller) or the `.tsr/session` file the hook wrote, so the committed-candidate reuse fires.
    let task = match store_path::resolve_session_task(None) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("tsr: {e}");
            return ExitCode::from(2);
        }
    };
    match exec::run(&store, &task, &workspace, argv) {
        Ok(outcome) if outcome.exit_code < 0 => {
            eprintln!("tsr: execution unresolved; consult the recorded outcome");
            ExitCode::from(1)
        }
        Ok(outcome) => ExitCode::from(outcome.exit_code.min(255) as u8),
        Err(exec::ExecRefusal::NoRuntime) => {
            eprintln!(
                "tsr: no container runtime reachable (podman/docker); refusing to run on the host"
            );
            ExitCode::from(exec::EXIT_NO_RUNTIME as u8)
        }
        // `run` records a container that fails to start as `Unresolved` and returns `Ok`, so this
        // arm is unreachable from the CLI; it exists so the match stays exhaustive.
        Err(exec::ExecRefusal::StartFailed(reason)) => {
            eprintln!("tsr: container runtime failed to start: {reason}");
            ExitCode::from(exec::EXIT_NO_RUNTIME as u8)
        }
    }
}
