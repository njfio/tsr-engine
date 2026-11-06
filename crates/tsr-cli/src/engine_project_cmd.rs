//! One project entry point; delegates to existing controller and budgeted transport.
use anyhow::{ensure, Result};
use clap::Subcommand;
use std::path::{Path, PathBuf};
use tsr_cli::engine::Engine;

#[derive(Subcommand)]
pub enum ProjectCmd {
    CriterionContext,
    CriterionPolicyRefresh {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        previous: String,
    },
    CriterionPolicy {
        #[arg(long)]
        file: PathBuf,
    },
    CriterionApply {
        #[arg(long)]
        file: PathBuf,
    },
    /// Records one reading (spec #10). The controller calls this after the reading turn.
    ReadingRecord {
        #[arg(long)]
        file: PathBuf,
    },
    /// Records what came back from you, the operator. It may name a reading or none.
    Return {
        #[arg(long)]
        text: String,
        #[arg(long)]
        reading: Option<String>,
    },
    /// Records a controller-recorded check as a return.
    CheckReturn {
        #[arg(long)]
        file: PathBuf,
    },
    ReadingView,
    PursuitContext,
    PursuitReflectionApply {
        #[arg(long)]
        file: PathBuf,
    },
    PursuitReflectionRequest,
    PursuitReflectionPropose {
        #[arg(long)]
        file: PathBuf,
    },
    PursuitReflectionAdmit {
        #[arg(long)]
        file: PathBuf,
    },
    /// Reviewed completed mutation prefix, unavailable read-only observation; never replay.
    PartialExecutionStatus,
    PartialExecutionEvidence {
        #[arg(long)]
        file: PathBuf,
    },
    DisposePartialExecution {
        #[arg(long)]
        file: PathBuf,
    },
    /// Explicit bounded public corpus admission; never reads bodies or grants test authority.
    InformationConfigure {
        #[arg(long)]
        file: PathBuf,
    },
    InformationStatus,
    ObjectiveConfigure {
        #[arg(long)]
        file: PathBuf,
    },
    ObjectiveStatus,
    ObjectiveRetireStale {
        #[arg(long)]
        state_root: String,
    },
    ObjectiveReview {
        #[arg(long)]
        file: PathBuf,
    },
    ObjectiveAccept {
        #[arg(long)]
        file: PathBuf,
    },
    /// Admit an independently reviewed public meaning; never changes task acceptance.
    DesireAdmit {
        #[arg(long)]
        file: PathBuf,
    },
    /// Administrative suppression/restoration only; semantic development uses reviewed lineage.
    DesireStatus {
        #[arg(long)]
        file: PathBuf,
    },
    /// State-driven local cycle; never invokes a provider or grants scope.
    Cycle {
        #[command(subcommand)]
        command: CycleCmd,
    },
    /// Admit bounded provider accounting without claiming a known monetary ceiling.
    /// Explicit operator policy removes cumulative resource ceilings only.
    ResourceUnlimitedAdmit {
        #[arg(long)]
        state_root: String,
        #[arg(long)]
        policy_sha256: String,
    },
    ResourceUnpricedAdmit {
        #[arg(long)]
        state_root: String,
    },
    /// Reserve call/token resources while explicitly retaining unknown external cost.
    ResourceReserveUnpriced {
        #[arg(long)]
        file: PathBuf,
    },
    /// Freeze an explicitly evaluator-authored diagnostic forecast before execution.
    EvaluatorForecast {
        #[arg(long)]
        file: PathBuf,
    },
    /// Inspect separately attributed evaluator forecasts and actual return bindings.
    EvaluatorForecastStatus,
    /// Source-bound qualitative guidance; proposal storage alone never admits it.
    Relational {
        #[command(subcommand)]
        command: RelationalCmd,
    },
    /// Copy only owner-admitted paths into a disposable engine workspace.
    Init {
        #[arg(long)]
        spec: PathBuf,
        #[arg(long)]
        source: Option<PathBuf>,
        /// Register requirements during initialization; inherits the previous episode's if omitted with --carry-from.
        #[arg(long)]
        requirements: Option<PathBuf>,
        /// Explicit local prior episode; preserves its requirements and reviewed outcomes.
        #[arg(long)]
        carry_from: Option<PathBuf>,
    },
    Config,
    /// Admit a fresh snapshot-bound verification action; prints a response for normal run/resume.
    VerifyAdmit {
        #[arg(long)]
        state_root: String,
        #[arg(long)]
        snapshot: String,
    },
    /// Experimental: generate a proposal using an already-running local Ollama model; never executes it.
    Generate {
        #[arg(long)]
        possibilities: bool,
        #[arg(long)]
        model: String,
        #[arg(long)]
        out: PathBuf,
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        endpoint: String,
        #[arg(long, default_value_t = 2048)]
        max_output_tokens: u32,
        #[arg(long, default_value_t = 180)]
        timeout_seconds: u32,
    },
    /// Quarantined verifier reports with opt-in local transport; no automatic acceptance.
    Verifier {
        #[command(subcommand)]
        command: VerifierCmd,
    },
    /// Operator-only independent acceptance; never dispatched by a model proposal.
    Acceptance {
        #[command(subcommand)]
        command: AcceptanceCmd,
    },
    /// Verify dependencies offline in the pinned image.
    Prepare,
    /// Download locked public crates into a new local image; never execute their code.
    PrepareImage {
        #[arg(long)]
        out: PathBuf,
    },
    /// Inspect source, handles and saved feedback for the next proposal.
    Request {
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Execute one local response, or forward explicit transport arguments after --.
    Run {
        #[arg(long)]
        response: Option<PathBuf>,
        /// Freeze the proposal without executing; inspect status/plan and use project resume.
        #[arg(long)]
        commit_only: bool,
        #[arg(last = true)]
        transport_args: Vec<String>,
    },
    /// Resume an unattempted local commitment, or the existing budgeted transport after --.
    Resume {
        #[arg(last = true)]
        transport_args: Vec<String>,
    },
    Status,
    /// Inspect next attention among admitted residual desires; never generates or executes.
    Attention,
    Export {
        #[arg(long)]
        accepted_only: bool,
        /// Original generated-edit episode; verified without changing its history.
        #[arg(long, requires = "origin_commitment")]
        origin_run: Option<PathBuf>,
        #[arg(long, requires = "origin_run")]
        origin_commitment: Option<String>,
    },
}
#[derive(Subcommand)]
pub enum CycleCmd {
    /// Open one reviewed decision from an attributed operator instruction; no acceptance or authority.
    Continue {
        #[arg(long)]
        file: PathBuf,
    },
    ContinuationReceipt {
        #[arg(long)]
        file: PathBuf,
    },
    CloseContinuation {
        #[arg(long)]
        state_root: String,
        #[arg(long)]
        continuation: String,
    },
    InformationResume {
        #[arg(long)]
        binding: String,
    },
    ServiceResume {
        #[arg(long)]
        binding: String,
    },
    Request,
    /// Read-only context for an already supplied response; not a new model-call grant.
    ResponseContext,
    /// Attribute a bounded advisory result without admission or execution.
    Acknowledge {
        #[arg(long)]
        file: PathBuf,
    },
    Preview {
        #[arg(long)]
        response: PathBuf,
    },
    /// Commit a separately reviewed response; never runs target commands.
    Advance {
        #[arg(long)]
        file: PathBuf,
    },
    /// Atomically admit one independently semantically reviewed public meaning.
    PreferenceAdvance {
        #[arg(long)]
        file: PathBuf,
    },
    MeaningAdvance {
        #[arg(long)]
        file: PathBuf,
    },
    /// Read-only recovery of an exact completed same-identity meaning transition.
    MeaningAdvanceReceipt {
        #[arg(long)]
        file: PathBuf,
    },
    /// Explicitly execute only the current unattempted bound native plan.
    Resume {
        #[arg(long)]
        binding: String,
    },
}
#[derive(Subcommand)]
pub enum RelationalCmd {
    Inspect,
    Admit {
        #[arg(long)]
        state_root: String,
        #[arg(long)]
        proposal: String,
        #[arg(long)]
        index: usize,
    },
    Status {
        #[arg(long)]
        state_root: String,
        #[arg(long)]
        admission: String,
        #[arg(long)]
        withdrawn: bool,
    },
    /// Review an unchanged generated experiment. Still requires a separate episode and budget.
    ExperimentReadmit {
        #[arg(long)]
        file: PathBuf,
    },
    ExperimentAdmit {
        #[arg(long)]
        file: PathBuf,
    },
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ExperimentReview {
    state_root: tsr_domain::Hash,
    proposal: tsr_domain::EventId,
    index: usize,
    author: String,
    reviewer: String,
    generator: String,
    review: String,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ExperimentReadmission {
    state_root: tsr_domain::Hash,
    previous_admission: tsr_domain::EventId,
    snapshot: tsr_domain::Hash,
    review: tsr_cli::engine::adaptive::coding::project::relational::ExperimentReview,
}
#[derive(Subcommand)]
pub enum VerifierCmd {
    SupplementalAdmit {
        #[arg(long)]
        file: PathBuf,
    },
    SupplementalSpec {
        #[arg(long)]
        admission: String,
    },
    SupplementalLink {
        #[arg(long)]
        admission: String,
        #[arg(long)]
        from: PathBuf,
        #[arg(long)]
        receipt: String,
    },
    SupplementalInspect,
    SupplementalRevoke {
        #[arg(long)]
        admission: String,
        #[arg(long)]
        reason: String,
    },
    LocalConfigure {
        #[arg(long)]
        admission: String,
        #[arg(long)]
        file: PathBuf,
    },
    LocalRun {
        #[arg(long)]
        request: String,
    },
    LocalCancel {
        #[arg(long)]
        request: String,
    },
    LocalReconcile {
        #[arg(long)]
        request: String,
        #[arg(long)]
        reason: String,
    },
    Revoke {
        #[arg(long)]
        admission: String,
        #[arg(long)]
        reason: String,
    },
    Admit {
        #[arg(long)]
        file: PathBuf,
    },
    Request {
        #[arg(long)]
        admission: String,
    },
    Inspect,
    Adopt {
        #[arg(long)]
        report: String,
    },
}
#[derive(Subcommand)]
pub enum AcceptanceCmd {
    /// Read authenticated published contributor history without activation or private criteria.
    PublishedHistory,
    /// Inspect private requirements, evidence and the current repair cycle locally.
    Inspect,
    /// Create a bound draft with blank evidence; never overwrites an existing file.
    Draft {
        #[arg(value_enum)]
        kind: DraftKind,
        #[arg(long)]
        out: PathBuf,
    },
    /// Carry reviewed outcomes from an explicit local episode whose requirements are preserved.
    Carry {
        #[arg(long)]
        from: PathBuf,
    },
    /// Publish sanitized, bounded guidance for the next proposal (does not dispatch it).
    Feedback {
        #[arg(long)]
        file: PathBuf,
    },
    Status,
    Requirements {
        #[arg(long)]
        file: PathBuf,
    },
    Review {
        #[arg(long)]
        file: PathBuf,
    },
}
#[derive(Clone, clap::ValueEnum)]
pub enum DraftKind {
    Review,
    Feedback,
}

fn write_operator_file(directory: &Path, path: &Path, value: &serde_json::Value) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    ensure!(
        !parent
            .canonicalize()?
            .starts_with(directory.join("coding-work").canonicalize()?),
        "operator artifacts must stay outside candidate workspace"
    );
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}
/// Six text fields at the domain bound, JSON-escaped, fit well inside this.
fn reading_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(262145)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 262144, "reading input exceeds bound");
    Ok(serde_json::from_slice(&bytes)?)
}

fn print_project(value: serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}

fn acceptance_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    use std::io::Read;
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(16385)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 16384, "acceptance input exceeds bound");
    Ok(serde_json::from_slice(&bytes)?)
}
fn script(directory: &Path, name: &str, args: &[String]) -> Result<()> {
    ensure!(
        !args
            .iter()
            .any(|a| a == "--directory" || a.starts_with("--directory=")),
        "transport cannot override the selected engine directory"
    );
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../scripts")
        .join(name);
    ensure!(
        path.is_file(),
        "checkout helper missing: {}",
        path.display()
    );
    // These are fixed, trusted controller helpers. No proposal chooses a script or shell.
    let status = std::process::Command::new("python3")
        .arg(path)
        .arg("--directory")
        .arg(directory)
        .args(args)
        .status()?;
    ensure!(
        status.success(),
        "project helper stopped; see its reported reason"
    );
    Ok(())
}
pub fn run(directory: &Path, command: ProjectCmd) -> Result<()> {
    use super::{run_engine, EngineCmd};
    match command {
        ProjectCmd::InformationConfigure { file } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?
                        .project_information_configure(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::ObjectiveConfigure { file } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?
                        .project_objective_configure(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::ObjectiveRetireStale { state_root } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_objective_retire_stale(
                        serde_json::from_value(serde_json::Value::String(state_root))?
                    )?
                )?
            );
            Ok(())
        }
        ProjectCmd::ObjectiveStatus => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_objective_context()?
                )?
            );
            Ok(())
        }
        ProjectCmd::ObjectiveReview { file } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_objective_review(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::ObjectiveAccept { file } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_objective_accept(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::InformationStatus => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_information_context()?
                )?
            );
            Ok(())
        }
        ProjectCmd::Cycle { command } => {
            let engine = Engine::open(directory)?;
            let value = match command {
                CycleCmd::Continue { file } => {
                    engine.project_cycle_continue(acceptance_file(&file)?)?
                }
                CycleCmd::ContinuationReceipt { file } => {
                    engine.project_cycle_continuation_receipt(acceptance_file(&file)?)?
                }
                CycleCmd::CloseContinuation {
                    state_root,
                    continuation,
                } => engine.project_cycle_close_continuation(
                    serde_json::from_value(serde_json::Value::String(state_root))?,
                    serde_json::from_value(serde_json::Value::String(continuation))?,
                )?,
                CycleCmd::ServiceResume { binding } => engine.project_objective_resume(
                    serde_json::from_value(serde_json::Value::String(binding))?,
                )?,
                CycleCmd::InformationResume { binding } => engine.project_information_resume(
                    serde_json::from_value(serde_json::Value::String(binding))?,
                )?,
                CycleCmd::Request => engine.project_cycle_request()?,
                CycleCmd::ResponseContext => engine.project_cycle_response_context()?,
                CycleCmd::Preview { response } => {
                    engine.project_cycle_preview(engine.proposal_response_from_file(&response)?)?
                }
                CycleCmd::Acknowledge { file } => {
                    use std::io::Read;
                    let mut bytes = Vec::new();
                    std::fs::File::open(file)?
                        .take(8193)
                        .read_to_end(&mut bytes)?;
                    ensure!(bytes.len() <= 8192, "advisory acknowledgment exceeds bound");
                    engine.project_cycle_acknowledge(serde_json::from_slice(&bytes)?)?
                }
                cmd @ (CycleCmd::Advance { .. }
                | CycleCmd::PreferenceAdvance { .. }
                | CycleCmd::MeaningAdvance { .. }
                | CycleCmd::MeaningAdvanceReceipt { .. }) => {
                    let recovery = matches!(&cmd, CycleCmd::MeaningAdvanceReceipt { .. });
                    let preference = matches!(&cmd, CycleCmd::PreferenceAdvance { .. });
                    let meaning = matches!(&cmd, CycleCmd::MeaningAdvance { .. });
                    let file = match cmd {
                        CycleCmd::Advance { file }
                        | CycleCmd::PreferenceAdvance { file }
                        | CycleCmd::MeaningAdvance { file }
                        | CycleCmd::MeaningAdvanceReceipt { file } => file,
                        _ => unreachable!(),
                    };
                    use std::io::Read;
                    let limit = if engine.project_configuration().is_ok_and(|p| p.browser) {
                        262144
                    } else {
                        65536
                    };
                    let mut bytes = Vec::new();
                    std::fs::File::open(file)?
                        .take(limit + 1)
                        .read_to_end(&mut bytes)?;
                    ensure!(bytes.len() as u64 <= limit, "cycle response exceeds bound");
                    if recovery {
                        engine.project_meaning_advance_receipt(serde_json::from_slice(&bytes)?)?
                    } else if preference {
                        engine.project_cycle_preference_advance(serde_json::from_slice(&bytes)?)?
                    } else if meaning {
                        engine.project_cycle_meaning_advance(serde_json::from_slice(&bytes)?)?
                    } else {
                        engine.project_cycle_advance(serde_json::from_slice(&bytes)?)?
                    }
                }
                CycleCmd::Resume { binding } => {
                    let plan = engine.project_cycle_plan(serde_json::from_value(
                        serde_json::Value::String(binding),
                    )?)?;
                    serde_json::to_value(engine.coding_run(plan.commitment, plan.source_before)?)?
                }
            };
            println!("{}", serde_json::to_string_pretty(&value)?);
            Ok(())
        }

        ProjectCmd::ResourceUnlimitedAdmit {
            state_root,
            policy_sha256,
        } => {
            let root = serde_json::from_value(serde_json::Value::String(state_root))?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?
                        .project_resource_unlimited_admit(root, policy_sha256)?
                )?
            );
            Ok(())
        }
        ProjectCmd::ResourceUnpricedAdmit { state_root } => {
            let root = serde_json::from_value(serde_json::Value::String(state_root))?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_resource_unpriced_admit(root)?
                )?
            );
            Ok(())
        }
        ProjectCmd::ResourceReserveUnpriced { file } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?
                        .project_resource_reserve_unpriced(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::ReadingRecord { file } => {
            print_project(Engine::open(directory)?.project_reading_record(reading_file(&file)?)?)
        }
        ProjectCmd::Return { text, reading } => {
            let reading = reading
                .map(|id| serde_json::from_value(serde_json::Value::String(id)))
                .transpose()?;
            print_project(Engine::open(directory)?.project_return_record(
                tsr_domain::Source::Operator,
                tsr_domain::ReadingReturn {
                    reading,
                    text,
                    evidence: None,
                },
            )?)
        }
        ProjectCmd::CheckReturn { file } => print_project(
            Engine::open(directory)?
                .project_return_record(tsr_domain::Source::Rules, reading_file(&file)?)?,
        ),
        ProjectCmd::ReadingView => print_project(Engine::open(directory)?.project_reading_view()?),
        ProjectCmd::PursuitReflectionApply { file } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?
                        .project_pursuit_reflection_apply(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::CriterionContext => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_criterion_context()?
                )?
            );
            Ok(())
        }
        ProjectCmd::CriterionPolicy { file } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_criterion_policy(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::CriterionPolicyRefresh { file, previous } => {
            let previous = serde_json::from_value(serde_json::Value::String(previous))?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?
                        .project_criterion_policy_refresh(acceptance_file(&file)?, previous)?
                )?
            );
            Ok(())
        }
        ProjectCmd::CriterionApply { file } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_criterion_apply_from_file(&file)?
                )?
            );
            Ok(())
        }
        ProjectCmd::PursuitContext => {
            println!(
                "{}",
                serde_json::to_string_pretty(&Engine::open(directory)?.project_pursuit_context()?)?
            );
            Ok(())
        }
        ProjectCmd::PursuitReflectionRequest => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_pursuit_reflection_request()?
                )?
            );
            Ok(())
        }
        ProjectCmd::PursuitReflectionPropose { file } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?
                        .project_pursuit_reflection_propose(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::PursuitReflectionAdmit { file } => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?
                        .project_pursuit_reflection_admit(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::PartialExecutionStatus => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_partial_execution_dispositions()?
                )?
            );
            Ok(())
        }
        ProjectCmd::PartialExecutionEvidence { file } => {
            let e = Engine::open(directory)?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &e.project_partial_execution_evidence(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::DisposePartialExecution { file } => {
            let e = Engine::open(directory)?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &e.project_dispose_partial_execution(acceptance_file(&file)?)?
                )?
            );
            Ok(())
        }
        ProjectCmd::DesireAdmit { file } => {
            let e = Engine::open(directory)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&e.project_desire_admit(acceptance_file(&file)?)?)?
            );
            Ok(())
        }
        ProjectCmd::DesireStatus { file } => {
            let e = Engine::open(directory)?;
            e.project_desire_status(acceptance_file(&file)?)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&e.project_relational_context()?)?
            );
            Ok(())
        }
        ProjectCmd::Relational { command } => {
            let engine = Engine::open(directory)?;
            let parse = |s: String| serde_json::Value::String(s);
            let value = match command {
                RelationalCmd::Inspect => engine.project_relational_context()?,
                RelationalCmd::Admit {
                    state_root,
                    proposal,
                    index,
                } => serde_json::to_value(engine.project_relational_admit(
                    serde_json::from_value(parse(state_root))?,
                    serde_json::from_value(parse(proposal))?,
                    index,
                )?)?,
                RelationalCmd::Status {
                    state_root,
                    admission,
                    withdrawn,
                } => {
                    engine.project_relational_status(
                        serde_json::from_value(parse(state_root))?,
                        serde_json::from_value(parse(admission))?,
                        withdrawn,
                    )?;
                    engine.project_relational_context()?
                }
                RelationalCmd::ExperimentReadmit { file } => {
                    let r: ExperimentReadmission = acceptance_file(&file)?;
                    serde_json::to_value(engine.project_experiment_readmit(
                        r.state_root,
                        r.previous_admission,
                        r.snapshot,
                        r.review,
                    )?)?
                }
                RelationalCmd::ExperimentAdmit { file } => {
                    let r: ExperimentReview = acceptance_file(&file)?;
                    serde_json::to_value(engine.project_experiment_admit(
                        r.state_root,
                        r.proposal,
                        r.index,
                        tsr_cli::engine::adaptive::coding::project::relational::ExperimentReview {
                            author: r.author,
                            reviewer: r.reviewer,
                            generator: r.generator,
                            review: r.review,
                        },
                    )?)?
                }
            };
            println!("{}", serde_json::to_string_pretty(&value)?);
            Ok(())
        }
        ProjectCmd::EvaluatorForecast { file } => {
            let admission = serde_json::from_slice(&std::fs::read(file)?)?;
            let result = Engine::open(directory)?.project_evaluator_forecast(admission)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
        ProjectCmd::EvaluatorForecastStatus => {
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &Engine::open(directory)?.project_evaluator_forecasts()?
                )?
            );
            Ok(())
        }
        ProjectCmd::Attention => {
            let result = Engine::open(directory)?.project_attention()?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
        ProjectCmd::VerifyAdmit {
            state_root,
            snapshot,
        } => {
            let parse = |s: String| -> Result<tsr_domain::Hash> {
                Ok(serde_json::from_value(serde_json::Value::String(s))?)
            };
            let result = Engine::open(directory)?
                .project_admit_verification(parse(state_root)?, parse(snapshot)?)?;
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
        ProjectCmd::Init {
            spec,
            source,
            requirements,
            carry_from,
        } => {
            use tsr_cli::engine::adaptive::coding::project::{
                acceptance::validate_requirements, ProjectSpec,
            };
            let prior = carry_from
                .map(|from| -> Result<_> {
                    ensure!(
                        from.join("events.sqlite").is_file(),
                        "existing source episode required"
                    );
                    Engine::open(&from)
                })
                .transpose()?;
            let requirements = match requirements {
                Some(file) => {
                    Some(acceptance_file::<std::collections::BTreeMap<String, String>>(&file)?)
                }
                None => prior
                    .as_ref()
                    .map(|e| e.project_acceptance_requirements_view())
                    .transpose()?,
            };
            if let Some(requirements) = &requirements {
                validate_requirements(requirements)?;
                if let Some(prior) = &prior {
                    ensure!(
                        prior
                            .project_acceptance_requirements_view()?
                            .iter()
                            .all(|(id, text)| requirements.get(id) == Some(text)),
                        "preserve prior requirements when carrying an episode"
                    );
                }
            }
            let engine = Engine::open(directory)?;
            let spec = ProjectSpec::from_file(&spec)?;
            match source {
                Some(source) => {
                    engine.project_import(spec, &source)?;
                }
                None => {
                    engine.project_initialize(spec)?;
                }
            }
            if let Some(requirements) = requirements {
                engine.project_acceptance_requirements(requirements)?;
            }
            if let Some(prior) = prior {
                engine.project_acceptance_carry(&prior)?;
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&engine.adaptive_state()?)?
            );
            Ok(())
        }
        ProjectCmd::Verifier { command } => {
            let engine = Engine::open(directory)?;
            let value = match command {
                VerifierCmd::SupplementalAdmit { file } => serde_json::to_value(
                    engine.project_supplemental_admit(acceptance_file(&file)?)?,
                )?,
                VerifierCmd::SupplementalSpec { admission } => {
                    serde_json::to_value(engine.project_supplemental_spec(
                        serde_json::from_value(serde_json::Value::String(admission))?,
                    )?)?
                }
                VerifierCmd::SupplementalLink {
                    admission,
                    from,
                    receipt,
                } => {
                    ensure!(
                        from.join("events.sqlite").is_file(),
                        "existing explicit controller-owned check episode required"
                    );
                    let other = Engine::open(&from)?;
                    serde_json::to_value(engine.project_supplemental_link(
                        serde_json::from_value(serde_json::Value::String(admission))?,
                        &other,
                        serde_json::from_value(serde_json::Value::String(receipt))?,
                    )?)?
                }
                VerifierCmd::SupplementalRevoke { admission, reason } => {
                    serde_json::to_value(engine.project_supplemental_revoke(
                        serde_json::from_value(serde_json::Value::String(admission))?,
                        reason,
                    )?)?
                }
                VerifierCmd::SupplementalInspect => engine.project_supplemental_view()?,
                VerifierCmd::LocalConfigure { admission, file } => {
                    serde_json::to_value(engine.project_verifier_local_configure(
                        serde_json::from_value(serde_json::Value::String(admission))?,
                        acceptance_file(&file)?,
                    )?)?
                }
                VerifierCmd::LocalRun { request } => {
                    serde_json::to_value(engine.project_verifier_local_run(
                        serde_json::from_value(serde_json::Value::String(request))?,
                    )?)?
                }
                VerifierCmd::LocalCancel { request } => {
                    serde_json::to_value(engine.project_verifier_local_cancel(
                        serde_json::from_value(serde_json::Value::String(request))?,
                    )?)?
                }
                VerifierCmd::LocalReconcile { request, reason } => {
                    serde_json::to_value(engine.project_verifier_local_reconcile(
                        serde_json::from_value(serde_json::Value::String(request))?,
                        reason,
                    )?)?
                }
                VerifierCmd::Admit { file } => {
                    serde_json::to_value(engine.project_verifier_admit(acceptance_file(&file)?)?)?
                }
                VerifierCmd::Request { admission } => {
                    serde_json::to_value(engine.project_verifier_request(
                        serde_json::from_value(serde_json::Value::String(admission))?,
                    )?)?
                }
                VerifierCmd::Revoke { admission, reason } => {
                    serde_json::to_value(engine.project_verifier_revoke(
                        serde_json::from_value(serde_json::Value::String(admission))?,
                        reason,
                    )?)?
                }
                VerifierCmd::Inspect => engine.project_verifier_view()?,
                VerifierCmd::Adopt { report } => {
                    serde_json::to_value(engine.project_verifier_adopt(serde_json::from_value(
                        serde_json::Value::String(report),
                    )?)?)?
                }
            };
            println!("{}", serde_json::to_string_pretty(&value)?);
            Ok(())
        }
        ProjectCmd::Acceptance { command } => {
            let engine = Engine::open(directory)?;
            let status = match command {
                AcceptanceCmd::PublishedHistory => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(
                            &engine.project_published_guidance_history()?
                        )?
                    );
                    return Ok(());
                }
                AcceptanceCmd::Inspect => {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&engine.project_acceptance_inspect()?)?
                    );
                    return Ok(());
                }
                AcceptanceCmd::Draft { kind, out } => {
                    let feedback = matches!(kind, DraftKind::Feedback);
                    write_operator_file(
                        directory,
                        &out,
                        &engine.project_acceptance_draft(feedback)?,
                    )?;
                    println!(
                        "{}",
                        serde_json::json!({"draft":out,"submitted":false,"next_step":if feedback {"Fill shareable guidance, then acceptance feedback --file"} else {"Inspect the result, fill every evidence entry and decision, then acceptance review --file"}})
                    );
                    return Ok(());
                }
                AcceptanceCmd::Status => engine.project_acceptance_status()?,
                AcceptanceCmd::Carry { from } => {
                    ensure!(
                        from.join("events.sqlite").is_file(),
                        "existing source episode required"
                    );
                    engine.project_acceptance_carry(&Engine::open(&from)?)?
                }
                AcceptanceCmd::Feedback { file } => {
                    engine.project_acceptance_feedback(acceptance_file(&file)?)?
                }
                AcceptanceCmd::Requirements { file } => {
                    engine.project_acceptance_requirements(acceptance_file(&file)?)?
                }
                AcceptanceCmd::Review { file } => {
                    engine.project_acceptance_review(acceptance_file(&file)?)?
                }
            };
            println!("{}", serde_json::to_string_pretty(&status)?);
            Ok(())
        }
        ProjectCmd::Generate {
            possibilities,
            model,
            out,
            endpoint,
            max_output_tokens,
            timeout_seconds,
        } => {
            let mut args = vec![
                "--model".into(),
                model,
                "--out".into(),
                out.to_string_lossy().into_owned(),
                "--endpoint".into(),
                endpoint,
                "--max-output-tokens".into(),
                max_output_tokens.to_string(),
                "--timeout-seconds".into(),
                timeout_seconds.to_string(),
            ];
            if possibilities {
                args.push("--possibilities".into());
            }
            script(directory, "engine-local-model.py", &args)
        }
        ProjectCmd::Config => run_engine(directory, EngineCmd::ProjectConfig),
        ProjectCmd::Prepare => run_engine(directory, EngineCmd::ProjectPrepare),
        ProjectCmd::PrepareImage { out } => script(
            directory,
            "engine-prepare-image.py",
            &["--out".into(), out.to_string_lossy().into_owned()],
        ),
        ProjectCmd::Request { out } => {
            if let Some(out) = out {
                write_operator_file(
                    directory,
                    &out,
                    &serde_json::to_value(Engine::open(directory)?.proposal_request()?)?,
                )?;
                println!("{}", serde_json::json!({"request":out,"dispatched":false}));
                Ok(())
            } else {
                run_engine(directory, EngineCmd::Request)
            }
        }
        ProjectCmd::Export {
            accepted_only,
            origin_run,
            origin_commitment,
        } => {
            if accepted_only {
                ensure!(Engine::open(directory)?.project_acceptance_status()?.accepted,"goal is not independently accepted; ordinary export remains available for inspection");
            }
            let engine = Engine::open(directory)?;
            let result = if let (Some(origin), Some(commitment)) = (origin_run, origin_commitment) {
                engine.project_export_origin(
                    &origin,
                    serde_json::from_value(serde_json::Value::String(commitment))?,
                )?
            } else {
                engine.project_export()?
            };
            if accepted_only {
                ensure!(
                    result["goal_accepted"] == true,
                    "acceptance changed during export; inspect current status"
                );
            }
            println!("{}", serde_json::to_string_pretty(&result)?);
            Ok(())
        }
        ProjectCmd::Status => {
            println!(
                "{}",
                serde_json::to_string_pretty(&Engine::open(directory)?.project_status()?)?
            );
            Ok(())
        }
        ProjectCmd::Run {
            response,
            commit_only,
            transport_args,
        } => {
            if !transport_args.is_empty() {
                ensure!(
                    response.is_none() && !commit_only,
                    "choose a local response or transport arguments; commit-only applies to local responses"
                );
                return script(directory, "engine-run.py", &transport_args);
            }
            let response = response.ok_or_else(|| {
                anyhow::anyhow!("provide --response FILE, or explicit transport arguments after --")
            })?;
            let engine = Engine::open(directory)?;
            let proposal = engine.proposal_response_from_file(&response)?;
            let review_only = proposal.proposals.iter().all(|p| {
                matches!(
                    p,
                    tsr_cli::engine::adaptive::provider::Suggestion::ContrastPreference { .. }
                        | tsr_cli::engine::adaptive::provider::Suggestion::PublicDesire { .. }
                        | tsr_cli::engine::adaptive::provider::Suggestion::Relational { .. }
                        | tsr_cli::engine::adaptive::provider::Suggestion::Experiment { .. }
                )
            });
            let result = engine.evaluate_proposal(proposal, true)?;
            if review_only && !result.committed {
                println!(
                    "{}",
                    serde_json::to_string_pretty(
                        &serde_json::json!({"status":"review_required","executed":false,"proposal_record":engine.project_latest_proposal()?,"evaluation":result})
                    )?
                );
                return Ok(());
            }
            ensure!(result.committed, "no eligible proposal committed");
            if commit_only {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&engine.project_status()?)?
                );
                Ok(())
            } else {
                execute_pending(&engine)
            }
        }
        ProjectCmd::Resume { transport_args } => {
            if !transport_args.is_empty() {
                return script(directory, "engine-run.py", &transport_args);
            }
            execute_pending(&Engine::open(directory)?)
        }
    }
}
fn execute_pending(engine: &Engine) -> Result<()> {
    ensure!(
        engine.project_status()?["status"] != "execution_recovery_required",
        "execution was already attempted; reconcile saved receipts before continuing"
    );
    let state = engine.adaptive_state()?;
    if state.pending.is_none() {
        println!(
            "{}",
            serde_json::to_string_pretty(&engine.project_status()?)?
        );
        return Ok(());
    }
    let plan = engine.coding_plan()?;
    // coding_run's durable intent prevents replay after an attempted/uncertain execution.
    let result = engine.coding_run(plan.commitment, plan.source_before)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
