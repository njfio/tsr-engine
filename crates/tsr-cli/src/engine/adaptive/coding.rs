//! Narrow coding adapter: only reviewed exact edits; every effect uses SealedRunner.
#[cfg(test)]
mod b91_fixture;
use super::*;

/// Preserve bounded compiler feedback as untrusted text, without UTF-8 expansion exceeding its cap.
fn diagnostic_excerpt(bytes: &[u8], limit: usize) -> String {
    let mut text = String::from_utf8_lossy(&bytes[..bytes.len().min(limit)]).into_owned();
    let mut end = text.len().min(limit);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text.truncate(end);
    text
}
#[cfg(test)]
mod diagnostic_excerpt_tests {
    use super::diagnostic_excerpt;
    #[test]
    fn compiler_feedback_retains_error_and_has_strict_byte_limit() {
        let errors = format!(
            "error[E0308]: expected &&str, found &String\n{}",
            "details\n".repeat(1000)
        );
        let excerpt = diagnostic_excerpt(errors.as_bytes(), 2048);
        assert!(excerpt.starts_with("error[E0308]"));
        assert_eq!(excerpt.len(), 2048);
        assert!(diagnostic_excerpt(&[0xff; 5000], 2048).len() <= 2048);
        assert_eq!(diagnostic_excerpt(b"complete", 2048), "complete");
    }
}
use tsr_corpus::{
    sealed::{Runtime, SealedRunner},
    Limits, RunResult, Runner,
};
const PROBE: &str = r#"include!("implementation.rs");
fn main() {
    let a: Vec<String> = std::env::args().collect();
    println!("{}", combine(a[1].parse().unwrap(), a[2].parse().unwrap()));
}
"#;
const INITIAL: &str = "pub fn combine(a: i32, b: i32) -> i32 { a - b }\n";
const TESTS:&str="include!(\"implementation.rs\");\n#[test] fn adds_positive() { assert_eq!(combine(2,3),5); }\n#[test] fn adds_zero() { assert_eq!(combine(7,0),7); }\n";
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub image: String,
    pub user: String,
    #[serde(default)]
    pub generated: bool,
    #[serde(default)]
    pub project: Option<project::ProjectSpec>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
pub enum CodingRecord {
    CriterionPolicyRefreshed {
        previous: EventId,
        policy: project::criterion_return::Policy,
    },
    CriterionPolicy {
        policy: project::criterion_return::Policy,
    },
    CriterionReturn {
        assessment: Box<project::criterion_return::Apply>,
    },
    PursuitFrozen {
        frozen: project::pursuit::Frozen,
    },
    PursuitReflectionReceived {
        reflection: project::pursuit::Reflection,
    },
    /// Legacy Operator-tagged history; never emitted by new model-review paths.
    PursuitReflectionAdmitted {
        review: project::pursuit::Review,
    },
    PursuitReflectionReviewedV2 {
        review: project::pursuit::Review,
    },
    PartialDisposition {
        input: Box<project::partial::Disposition>,
        receipts: Vec<String>,
    },
    ProvisionalPreferenceAdmission {
        admission: Box<project::provisional::Admission>,
    },
    SuccessPreferenceAdmission {
        admission: Box<project::success::Admission>,
    },
    GeneratedPreferenceAdmission {
        admission: Box<project::expansion::Admission>,
    },
    PublicDesireAdmission {
        proposal: EventId,
        index: usize,
        reviewer: String,
        note: String,
        original_parent: serde_json::Value,
        parent_origins: Vec<serde_json::Value>,
    },
    PublicDesireStatus {
        update: project::desire::Status,
        original_parent: serde_json::Value,
    },
    ExperimentReadmission {
        previous: EventId,
        admission: EventId,
        state_root: Hash,
        snapshot: Hash,
    },
    EvaluatorForecast {
        admission: project::evaluator::Admission,
        forecast: Forecast,
    },
    EvaluatorResolution {
        admission: EventId,
        commitment: EventId,
        returned: EventId,
        receipt: EventId,
        forecast: Forecast,
        success: bool,
    },
    ExperimentOrigin {
        proposal: EventId,
        index: usize,
        admission: EventId,
        binding: Hash,
    },
    RelationalAdmission {
        proposal: EventId,
        index: usize,
        binding: Hash,
    },
    RelationalStatus {
        admission: EventId,
        withdrawn: bool,
    },
    Verifier {
        event: project::acceptance::reviewer::Record,
    },
    Verification {
        state_root: Hash,
        snapshot: Hash,
        configuration: Hash,
        action: Action,
    },
    AcceptanceHistory {
        source_episode: EventId,
        revision: EventId,
        source_root: Hash,
        observations: Vec<project::acceptance::HistoricalReview>,
    },
    AcceptanceAims {
        revision: EventId,
        preferences: BTreeMap<String, PreferenceId>,
    },
    AcceptanceFeedback {
        feedback: project::acceptance::Feedback,
    },
    AcceptanceGuidanceScope {
        publication: EventId,
        scope: Hash,
        snapshot: Hash,
    },
    AcceptanceExpectation {
        revision: EventId,
        commitment: EventId,
        forecast: Forecast,
    },
    AcceptanceRequirements {
        requirements: BTreeMap<String, String>,
    },
    AcceptanceUncertainty {
        review: EventId,
        unknown: BTreeSet<String>,
    },
    AcceptanceReview {
        review: project::acceptance::Review,
    },
    Dependencies {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        resource_request: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        measured_tool_ms: Option<u64>,
        binding: Hash,
        ready: bool,
        receipt: String,
    },
    PatchSet {
        patches: Vec<ExactPatch>,
        action: Action,
    },
    Patch {
        patch: ExactPatch,
        action: Action,
    },
    Config {
        config: Config,
    },
    Intent {
        plan: Plan,
    },
    Receipt {
        commitment: EventId,
        verdict: Verdict,
        runs: Vec<String>,
        source_after: Option<Hash>,
        #[serde(default)]
        observations: Vec<ProbeObservation>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeObservation {
    #[serde(default)]
    pub passed: Option<bool>,
    pub argv: Vec<String>,
    pub expected_stdout: String,
    pub exit: i32,
    pub stdout: String,
    pub stderr: String,
    pub abbreviated: bool,
    #[serde(default)]
    pub case_id: Option<String>,
}
/// A patch carries data only. All paths, handles and spans are checked by Rust.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactPatch {
    pub path: String,
    pub source_digest: Hash,
    pub inspection_handle: Hash,
    pub old_text: String,
    pub new_text: String,
}
impl ExactPatch {
    pub fn id(&self) -> Result<String> {
        Ok(format!(
            "patch-{}",
            Hash::of(&[b"scoped-patch@v1", &serde_json::to_vec(self)?])
        ))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub commitment: EventId,
    pub action: String,
    pub source_before: Hash,
    pub replacement: String,
    pub image: String,
    pub user: String,
    pub commands: Vec<Vec<String>>,
    pub timeout_ms: u64,
    pub protected_tests: Hash,
    #[serde(default)]
    pub edits: Vec<project::PreparedEdit>,
    #[serde(default)]
    pub cases: Vec<project::ProbeCase>,
    #[serde(default)]
    pub expected_after: Option<Hash>,
    #[serde(default)]
    pub generated: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Passed,
    FailedTests,
    Timeout,
    Unavailable,
    IntegrityFailure,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Execution {
    pub verdict: Verdict,
    pub receipts: Vec<String>,
    pub state: State,
}
fn strings(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}
fn classify(result: &RunResult, test_phase: bool) -> Verdict {
    if result.exit == 124 {
        return Verdict::Timeout;
    }
    if !test_phase {
        return if result.exit == 0 {
            Verdict::Passed
        } else {
            Verdict::Unavailable
        };
    }
    if result.stdout.len() >= 4093 || result.stderr.len() >= 4093 {
        return Verdict::Unavailable;
    }
    let Ok(report) = tsr_corpus::mine::cargo_report(&result.stdout) else {
        return Verdict::Unavailable;
    };
    let names: BTreeSet<_> = report.tests.iter().map(|t| t.name.as_str()).collect();
    if names != BTreeSet::from(["adds_positive", "adds_zero"]) {
        return Verdict::Unavailable;
    }
    let text = String::from_utf8_lossy(&result.stdout);
    if result.exit == 0
        && report.failed().is_empty()
        && text
            .lines()
            .any(|l| l.starts_with("test result: ok. 2 passed; 0 failed;"))
    {
        Verdict::Passed
    } else if result.exit == 101
        && !report.failed().is_empty()
        && text.lines().any(|l| l.starts_with("test result: FAILED."))
    {
        Verdict::FailedTests
    } else {
        Verdict::Unavailable
    }
}
fn classify_probe(run: &RunResult, expected: &[u8]) -> Verdict {
    if run.exit == 124 {
        return Verdict::Timeout;
    }
    if matches!(run.exit, 125..=127 | 137 | 143)
        || run.stdout.len() >= 4093
        || run.stderr.len() >= 4093
    {
        return Verdict::Unavailable;
    }
    if run.exit == 0 && run.stdout == expected && run.stderr.is_empty() {
        Verdict::Passed
    } else {
        Verdict::FailedTests
    }
}
impl Engine {
    pub(super) fn coding_config(&self) -> Result<Option<Config>> {
        Ok(self.records()?.into_iter().find_map(|(_, r)| match r {
            Record::Coding {
                record: CodingRecord::Config { config },
            } => Some(config),
            _ => None,
        }))
    }
    pub fn coding_initialize(&self, image: String, user: String) -> Result<State> {
        self.coding_initialize_mode(image, user, false)
    }
    pub fn coding_initialize_mode(
        &self,
        image: String,
        user: String,
        generated: bool,
    ) -> Result<State> {
        ensure!(
            user.split(':')
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                .is_some_and(|id| id != 0),
            "fixture runs as a nonroot user"
        );
        SealedRunner::new(Runtime::Docker, image.clone(), user.clone())
            .map_err(anyhow::Error::msg)?;
        ensure!(self.events()?.is_empty(), "directory already initialized");
        let work = self.directory.join("coding-work");
        fs::create_dir(&work)?;
        for (name, bytes) in [
            ("implementation.rs", INITIAL),
            ("tests.rs", if generated { PROBE } else { TESTS }),
        ] {
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(work.join(name))?;
            f.write_all(bytes.as_bytes())?;
            f.sync_all()?;
        }
        let actions = if generated {
            vec![Action { id: "scope".into(), description: "Admit exact edits only to coding-work/implementation.rs; protected probe tests (2,3)->5 and (7,0)->7; no proposal commands or scope changes".into(), serves: vec!["protected_tests_pass".into()], cost: 0.1 }]
        } else {
            vec![
                Action {
                    id: "increment".into(),
                    description: "Try a - b + 1".into(),
                    serves: vec!["protected_tests_pass".into()],
                    cost: 0.05,
                },
                Action {
                    id: "correct".into(),
                    description: "Try a + b".into(),
                    serves: vec!["protected_tests_pass".into()],
                    cost: 0.15,
                },
            ]
        };
        self.configure(Spec {
            goal: "Repair combine so both protected addition tests pass".into(),
            criteria: vec!["protected_tests_pass".into()],
            actions,
            prior: vec![],
            initial_commitment: None,
        })?;
        self.write(|| {
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::Config {
                        config: Config {
                            image,
                            user,
                            generated,
                            project: None,
                        },
                    },
                },
                Source::Operator,
            )?;
            self.adaptive_state_inner()
        })
    }
    fn coding_source(&self) -> Result<(PathBuf, String)> {
        let generated = self.coding_config()?.is_some_and(|c| c.generated);
        let work = self.directory.join("coding-work");
        ensure!(
            !fs::symlink_metadata(&work)?.file_type().is_symlink(),
            "workspace symlink refused"
        );
        for name in ["implementation.rs", "tests.rs"] {
            let m = fs::symlink_metadata(work.join(name))?;
            ensure!(
                m.is_file() && !m.file_type().is_symlink() && m.len() <= 4096,
                "fixture file type/size changed"
            );
        }
        ensure!(
            fs::read(work.join("tests.rs"))?
                == if generated {
                    PROBE.as_bytes()
                } else {
                    TESTS.as_bytes()
                },
            "protected tests changed"
        );
        let source = fs::read_to_string(work.join("implementation.rs"))?;
        ensure!(
            generated
                || [
                    INITIAL,
                    "pub fn combine(a: i32, b: i32) -> i32 { a - b + 1 }\n",
                    "pub fn combine(a: i32, b: i32) -> i32 { a + b }\n"
                ]
                .contains(&source.as_str()),
            "source outside reviewed fixture states"
        );
        Ok((work.canonicalize()?, source))
    }
    fn inspection_handle(&self, source: &str) -> Result<Hash> {
        Ok(Hash::of(&[
            b"coding-inspection@v1",
            self.directory.canonicalize()?.to_string_lossy().as_bytes(),
            &serde_json::to_vec(&self.coding_config()?)?,
            b"implementation.rs",
            source.as_bytes(),
            PROBE.as_bytes(),
        ]))
    }
    pub(super) fn validate_patch(&self, patch: &ExactPatch) -> Result<String> {
        let config = self
            .coding_config()?
            .ok_or_else(|| anyhow::anyhow!("coding scope missing"))?;
        ensure!(config.generated, "generated edit scope not admitted");
        let span_limit = if config.project.as_ref().is_some_and(|p| p.browser) {
            1024 * 1024
        } else {
            2048
        };
        ensure!(
            patch.old_text.len() <= span_limit && patch.new_text.len() <= span_limit,
            "patch span exceeds {span_limit} bytes"
        );
        if let Some(spec) = &config.project {
            return self.project_validate_patch(spec, patch);
        }
        ensure!(
            patch.path == "implementation.rs",
            "path outside admitted edit scope"
        );
        ensure!(
            patch.old_text.len() <= 2048 && patch.new_text.len() <= 2048,
            "patch span exceeds 2048 bytes"
        );
        let (work, source) = self.coding_source()?;
        ensure!(
            patch.inspection_handle == self.inspection_handle(&source)?,
            "stale or foreign inspection handle"
        );
        let replacement = tsr_batch::broker_v2::prepare_edit(
            &work,
            &patch.path,
            patch.source_digest,
            &patch.old_text,
            &patch.new_text,
        )
        .map_err(|e| anyhow::anyhow!("{e}"))?;
        ensure!(replacement.len() <= 4096, "source exceeds 4096 bytes");
        Ok(replacement)
    }
    pub(super) fn validate_patch_set(&self, patches: &[ExactPatch]) -> Result<String> {
        ensure!(
            !patches.is_empty() && patches.len() <= 4,
            "1..4 edits required"
        );
        ensure!(
            patches
                .iter()
                .map(|p| &p.path)
                .collect::<BTreeSet<_>>()
                .len()
                == patches.len(),
            "one edit per file per set"
        );
        if patches.len() > 1 {
            ensure!(
                self.coding_config()?.is_some_and(|c| c.project.is_some()),
                "multiple files need project scope"
            );
        }
        for patch in patches {
            self.validate_patch(patch)?;
        }
        if let Some(spec) = self.coding_config()?.and_then(|c| c.project) {
            let mut snapshot = self.project_snapshot(&spec)?;
            for patch in patches {
                snapshot.insert(patch.path.clone(), self.validate_patch(patch)?);
            }
            project::validate_snapshot(&spec, &snapshot)?;
        }
        let mut sorted = patches.to_vec();
        sorted.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(format!(
            "patch-{}",
            Hash::of(&[b"scoped-edit-set@v1", &serde_json::to_vec(&sorted)?])
        ))
    }
    pub(super) fn patches_for(&self, action: &str) -> Result<Option<Vec<ExactPatch>>> {
        Ok(self.records()?.into_iter().find_map(|(_, r)| match r {
            Record::Coding {
                record: CodingRecord::Patch { patch, action: a },
            } if a.id == action => Some(vec![patch]),
            Record::Coding {
                record: CodingRecord::PatchSet { patches, action: a },
            } if a.id == action => Some(patches),
            _ => None,
        }))
    }
    fn coding_criteria(&self) -> Result<Vec<String>> {
        Ok(self
            .coding_config()?
            .and_then(|c| c.project)
            .map(|s| {
                let mut criteria: Vec<_> = s
                    .cases
                    .iter()
                    .map(|c| project::case_criterion(&c.id))
                    .collect();
                if s.full_workspace_oracle {
                    criteria.push(project::case_criterion("full_workspace_oracle@v1"));
                }
                criteria
            })
            .unwrap_or_else(|| vec!["protected_tests_pass".into()]))
    }
    pub(super) fn register_patch_set(&self, patches: &[ExactPatch]) -> Result<String> {
        let id = self.validate_patch_set(patches)?;
        if self.patches_for(&id)?.is_none() {
            ensure!(
                self.records()?
                    .iter()
                    .filter(|(_, r)| matches!(
                        r,
                        Record::Coding {
                            record: CodingRecord::Patch { .. } | CodingRecord::PatchSet { .. }
                        }
                    ))
                    .count()
                    < 128,
                "scope candidate limit reached"
            );
            let mut sorted = patches.to_vec();
            sorted.sort_by(|a, b| a.path.cmp(&b.path));
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::PatchSet {
                        patches: sorted,
                        action: Action {
                            id: id.clone(),
                            description: format!(
                                "Exact edit set: {}",
                                patches
                                    .iter()
                                    .map(|p| p.path.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            serves: self.coding_criteria()?,
                            cost: 0.1,
                        },
                    },
                },
                Source::Rules,
            )?;
        }
        Ok(id)
    }
    pub(super) fn patch_for(&self, action: &str) -> Result<Option<ExactPatch>> {
        Ok(self.records()?.into_iter().find_map(|(_, r)| match r {
            Record::Coding {
                record: CodingRecord::Patch { patch, action: a },
            } if a.id == action => Some(patch),
            _ => None,
        }))
    }
    pub(super) fn register_patch(&self, patch: &ExactPatch) -> Result<String> {
        self.validate_patch(patch)?;
        let id = patch.id()?;
        if self.patch_for(&id)?.is_none() {
            ensure!(
                self.records()?
                    .iter()
                    .filter(|(_, r)| matches!(
                        r,
                        Record::Coding {
                            record: CodingRecord::Patch { .. } | CodingRecord::PatchSet { .. }
                        }
                    ))
                    .count()
                    < 128,
                "scope candidate limit reached"
            );
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::Patch {
                        patch: patch.clone(),
                        action: Action {
                            id: id.clone(),
                            description: format!(
                                "Exact patch to {} from {}",
                                patch.path, patch.source_digest
                            ),
                            serves: self.coding_criteria()?,
                            cost: 0.1,
                        },
                    },
                },
                Source::Rules,
            )?;
        }
        Ok(id)
    }
    pub fn coding_context(&self) -> Result<Option<serde_json::Value>> {
        let Some(config) = self.coding_config()? else {
            return Ok(None);
        };
        let recent_patches: Vec<_> = self
            .records()?
            .into_iter()
            .rev()
            .filter_map(|(_, r)| match r {
                Record::Coding {
                    record: CodingRecord::Patch { patch, action },
                } => Some(serde_json::json!({"action":action.id,"patch":patch})),
                Record::Coding {
                    record: CodingRecord::PatchSet { patches, action },
                } => Some(serde_json::json!({"action":action.id,"patches":patches})),
                _ => None,
            })
            .take(2)
            .collect();
        let recent_observations: Vec<_> = self.records()?.into_iter().rev().filter_map(|(_, r)| match r {
            Record::Coding { record: CodingRecord::Receipt { commitment, verdict, observations, runs, .. } } => Some(serde_json::json!({"commitment":commitment,"verdict":verdict,"observations":observations,"receipts":runs})), _ => None
        }).take(2).collect();
        if let Some(spec) = &config.project {
            let mut context = self.project_context(spec)?;
            context["recent_patches"] = serde_json::to_value(recent_patches)?;
            context["recent_observations"] = serde_json::to_value(recent_observations)?;
            return Ok(Some(context));
        }
        let (_, source) = self.coding_source()?;
        Ok(Some(
            serde_json::json!({"path":"implementation.rs","source":source,"source_digest":Hash::of(&[source.as_bytes()]),"inspection_handle": self.inspection_handle(&source)?,"generated_edits":config.generated,"recent_patches":recent_patches,"recent_observations":recent_observations,"protected_test_source":if config.generated {PROBE} else {TESTS},"cases":[{"args":["2","3"],"stdout":"5\n"},{"args":["7","0"],"stdout":"7\n"}],"execution":"Only exact edits within this path; no proposal argv, test edits, new goals or scope. Each patch identity has its own prior; no cross-patch learning."}),
        ))
    }
    pub fn coding_plan(&self) -> Result<Plan> {
        let config = self
            .coding_config()?
            .ok_or_else(|| anyhow::anyhow!("run coding-init first"))?;
        let state = self.adaptive_state()?;
        let p = state
            .pending
            .ok_or_else(|| anyhow::anyhow!("apply a replay proposal first"))?;
        if let Some(spec) = &config.project {
            return self.project_plan(&config, spec, p);
        }
        let (work, source) = self.coding_source()?;
        let digest = Hash::of(&[source.as_bytes()]);
        let replacement = if config.generated {
            let patches = self
                .patches_for(&p.action)?
                .ok_or_else(|| anyhow::anyhow!("missing generated patch"))?;
            ensure!(patches.len() == 1, "multiple edits need project scope");
            self.validate_patch(&patches[0])?
        } else {
            let expression = match p.action.as_str() {
                "increment" => "a - b + 1",
                "correct" => "a + b",
                _ => bail!("action has no execution grant"),
            };
            let old = source
                .split("{ ")
                .nth(1)
                .and_then(|s| s.strip_suffix(" }\n"))
                .ok_or_else(|| anyhow::anyhow!("fixture expression missing"))?;
            tsr_batch::broker_v2::prepare_edit(&work, "implementation.rs", digest, old, expression)
                .map_err(|e| anyhow::anyhow!("{e}"))?
        };
        let mut commands = vec![vec![
            "sh".into(),
            "-c".into(),
            "printf '%s' \"$1\" > implementation.rs".into(),
            "tsr-reviewed-edit".into(),
            replacement.clone(),
        ]];
        if config.generated {
            commands.extend([
                strings(&["rustc", "tests.rs", "-o", "fixture-probe"]),
                strings(&["./fixture-probe", "2", "3"]),
                strings(&["./fixture-probe", "7", "0"]),
            ]);
        } else {
            commands.extend([
                strings(&["rustc", "--test", "tests.rs", "-o", "fixture-tests"]),
                strings(&["./fixture-tests", "--test-threads=1"]),
            ]);
        }
        Ok(Plan {
            commitment: p.commitment,
            action: p.action,
            source_before: digest,
            replacement,
            image: config.image,
            user: config.user,
            commands,
            timeout_ms: 30000,
            protected_tests: Hash::of(&[if config.generated {
                PROBE.as_bytes()
            } else {
                TESTS.as_bytes()
            }]),
            generated: config.generated,
            edits: vec![],
            cases: vec![],
            expected_after: None,
        })
    }
    pub fn coding_run(&self, commitment: EventId, expected: Hash) -> Result<Execution> {
        let plan = self.coding_plan()?;
        let runner = SealedRunner::new(Runtime::Docker, plan.image.clone(), plan.user.clone())
            .map_err(anyhow::Error::msg)?;
        let named = self
            .coding_config()?
            .and_then(|c| c.project)
            .is_some_and(|s| s.named_package.is_some());
        let mut read_only = SealedRunner::new(Runtime::Docker, plan.image, plan.user)
            .map_err(anyhow::Error::msg)?
            .with_read_only_workspace();
        if named {
            let scratch = self
                .directory
                .join("coding-build")
                .join(commitment.to_string());
            fs::create_dir_all(scratch.parent().unwrap())?;
            fs::create_dir(&scratch)?;
            read_only = read_only
                .with_build_scratch(scratch.canonicalize()?)
                .map_err(anyhow::Error::msg)?;
        }
        struct Staged {
            writable: SealedRunner,
            read_only: SealedRunner,
        }
        impl Runner for Staged {
            fn run(&self, copy: &Path, argv: &[String], limits: &Limits) -> RunResult {
                if argv
                    .first()
                    .is_some_and(|s| s == "./fixture-probe" || s == "cargo" || s == "node")
                {
                    self.read_only.run(copy, argv, limits)
                } else {
                    self.writable.run(copy, argv, limits)
                }
            }
        }
        self.coding_execute(
            commitment,
            expected,
            &Staged {
                writable: runner,
                read_only,
            },
        )
    }
    fn coding_execute(
        &self,
        commitment: EventId,
        expected: Hash,
        runner: &dyn Runner,
    ) -> Result<Execution> {
        let plan=self.write(||{
            self.project_evaluator_forecasts()?;
            let plan=self.coding_plan()?;ensure!(plan.commitment==commitment&&plan.source_before==expected,"stale commitment or source");
            ensure!(!self.records()?.iter().any(|(_,r)|matches!(r,Record::Coding{record:CodingRecord::Intent{plan:p}} if p.commitment==commitment)),"execution already attempted; reconcile saved receipts, never automatically retry");
            self.reserve_project_execution(&plan)?;
            self.adaptive_record(&Record::Coding{record:CodingRecord::Intent{plan:plan.clone()}},Source::Rules)?;
            let candidate=self.events()?.iter().find_map(|(_,e)|match &e.kind{EventKind::Committed{candidate,..} if e.id==commitment=>Some(*candidate),_=>None}).ok_or_else(||anyhow::anyhow!("commitment missing"))?;
            self.append(EventKind::Dispatched{candidate,in_sandbox:true},Source::Rules)?;Ok(plan)
        })?;
        let work = self.directory.join("coding-work").canonicalize()?;
        let evidence = self
            .directory
            .join("coding-receipts")
            .join(commitment.to_string());
        fs::create_dir_all(&evidence)?;
        let mut receipts = vec![];
        let mut observations = vec![];
        let mut verdict = Verdict::Unavailable;
        let mut failed_cases = false;
        let mut met_cases = vec![];
        let project_spec = self.coding_config()?.and_then(|c| c.project);
        let mut last_exit = None;
        let mut duration = 0u64;
        for (index, argv) in plan.commands.iter().enumerate() {
            let allowed_ms = self
                .project_remaining_wall_ms()?
                .map(|ms| ms.min(plan.timeout_ms))
                .unwrap_or(plan.timeout_ms);
            let full_oracle = project_spec
                .as_ref()
                .is_some_and(|s| s.full_workspace_oracle)
                && index == 3;
            let output_bound = if full_oracle {
                1 << 20
            } else if project_spec.as_ref().is_some_and(|s| s.cargo) && index >= 2 {
                16 * 1024
            } else {
                4096
            };
            let run = if allowed_ms == 0 {
                RunResult::exit_only(124)
            } else {
                runner.run(&work, argv, &Limits::new(allowed_ms, output_bound))
            };
            receipts.push(
                tsr_corpus::persist_run(
                    &evidence,
                    &plan.source_before.to_string(),
                    argv,
                    &run,
                    None,
                )
                .map_err(anyhow::Error::msg)?,
            );
            if full_oracle {
                let v = project::cargo::classify_full(&run);
                observations.push(ProbeObservation {
                    passed: match v {
                        Verdict::Passed => Some(true),
                        Verdict::FailedTests => Some(false),
                        _ => None,
                    },
                    argv: argv.clone(),
                    expected_stdout: "complete admitted workspace suite".into(),
                    exit: run.exit,
                    stdout: format!("full workspace: {v:?}"),
                    stderr: String::from_utf8_lossy(&run.stderr[..run.stderr.len().min(128)])
                        .into_owned(),
                    abbreviated: true,
                    case_id: Some("full_workspace_oracle@v1".into()),
                });
            } else if project_spec.as_ref().is_some_and(|s| s.cargo) && index >= 2 {
                let report = tsr_corpus::mine::cargo_report(&run.stdout).ok();
                for case in &plan.cases {
                    let status = report
                        .as_ref()
                        .and_then(|r| r.tests.iter().find(|t| t.name == case.id))
                        .map(|t| if t.passed { "passed" } else { "failed" })
                        .unwrap_or("unavailable");
                    observations.push(ProbeObservation {
                        passed: match status {
                            "passed" => Some(true),
                            "failed" => Some(false),
                            _ => None,
                        },
                        argv: argv.clone(),
                        expected_stdout: "named Cargo test passes".into(),
                        exit: run.exit,
                        stdout: format!("{}: {}", case.id, status),
                        stderr: diagnostic_excerpt(&run.stderr, 2048),
                        abbreviated: true,
                        case_id: Some(case.id.clone()),
                    });
                }
            } else if plan.generated && index >= 2 {
                observations.push(ProbeObservation {
                    passed: match if project_spec.as_ref().is_some_and(|s| s.browser) {
                        self.browser_verdict(&run, project_spec.as_ref().unwrap())?
                    } else {
                        classify_probe(
                            &run,
                            plan.cases
                                .get(index - 2)
                                .map(|c| c.expected_stdout.as_bytes())
                                .unwrap_or(if index == 2 { b"5\n" } else { b"7\n" }),
                        )
                    } {
                        Verdict::Passed => Some(true),
                        Verdict::FailedTests => Some(false),
                        _ => None,
                    },
                    argv: argv.clone(),
                    expected_stdout: if let Some(case) = plan.cases.get(index - 2) {
                        case.expected_stdout.clone()
                    } else if index == 2 {
                        "5\n".into()
                    } else {
                        "7\n".into()
                    },
                    exit: run.exit,
                    stdout: String::from_utf8_lossy(
                        &run.stdout[..run.stdout.len().min(
                            if project_spec.as_ref().is_some_and(|s| s.browser) {
                                4000
                            } else {
                                256
                            },
                        )],
                    )
                    .into_owned(),
                    stderr: String::from_utf8_lossy(&run.stderr[..run.stderr.len().min(128)])
                        .into_owned(),
                    abbreviated: run.stdout.len()
                        > if project_spec.as_ref().is_some_and(|s| s.browser) {
                            4000
                        } else {
                            256
                        }
                        || run.stderr.len() > 128,
                    case_id: plan.cases.get(index - 2).map(|c| c.id.clone()),
                });
            }
            last_exit = Some(run.exit);
            duration = duration
                .checked_add(run.duration_ms)
                .ok_or_else(|| anyhow::anyhow!("execution duration overflow"))?;
            verdict = if full_oracle {
                let v = project::cargo::classify_full(&run);
                if v == Verdict::Passed {
                    met_cases.push(project::case_criterion("full_workspace_oracle@v1"));
                }
                v
            } else if project_spec.as_ref().is_some_and(|s| s.cargo) && index >= 2 {
                let (v, met) = project::cargo::classify_bounded(&run, &plan.cases, output_bound);
                met_cases = met;
                v
            } else if plan.generated
                && index >= 2
                && project_spec.as_ref().is_some_and(|s| s.browser)
            {
                self.browser_verdict(&run, project_spec.as_ref().unwrap())?
            } else if plan.generated && index >= 2 {
                classify_probe(
                    &run,
                    if let Some(case) = plan.cases.get(index - 2) {
                        case.expected_stdout.as_bytes()
                    } else if index == 2 {
                        b"5\n"
                    } else {
                        b"7\n"
                    },
                )
            } else {
                classify(&run, !plan.generated && index == 2)
            };
            let integrity = if let Some(spec) = &project_spec {
                self.project_snapshot(spec)
                    .and_then(|s| project::digest(&s))
                    .is_ok_and(|hash| {
                        Some(hash) == plan.expected_after
                            || (index == 0 && run.exit != 0 && hash == plan.source_before)
                    })
            } else {
                self.coding_source().is_ok_and(|(_, s)| {
                    s == plan.replacement
                        || (index == 0
                            && run.exit != 0
                            && Hash::of(&[s.as_bytes()]) == plan.source_before)
                })
            };
            if !integrity {
                verdict = Verdict::IntegrityFailure;
                break;
            }
            if project_spec.is_some() && index >= 2 {
                if verdict == Verdict::Passed
                    && project_spec.as_ref().is_some_and(|spec| !spec.cargo)
                {
                    met_cases.push(project::case_criterion(&plan.cases[index - 2].id));
                }
                if verdict == Verdict::FailedTests {
                    failed_cases = true;
                    continue;
                }
            }
            if verdict != Verdict::Passed {
                break;
            }
        }
        if project_spec.is_some() && failed_cases && verdict == Verdict::Passed {
            verdict = Verdict::FailedTests;
        }
        self.write(|| {
            let state = self.adaptive_state_inner()?;
            ensure!(
                state
                    .pending
                    .as_ref()
                    .is_some_and(|p| p.commitment == commitment),
                "pending commitment changed during execution"
            );
            let candidate = self
                .events()?
                .iter()
                .find_map(|(_, e)| match &e.kind {
                    EventKind::Committed { candidate, .. } if e.id == commitment => {
                        Some(*candidate)
                    }
                    _ => None,
                })
                .unwrap();
            let source_after = if let Some(spec) = &project_spec {
                self.project_snapshot(spec)
                    .and_then(|s| project::digest(&s))
                    .ok()
            } else {
                self.coding_source()
                    .ok()
                    .map(|(_, s)| Hash::of(&[s.as_bytes()]))
            };
            if !matches!(verdict, Verdict::Passed | Verdict::FailedTests) {
                for observation in &mut observations {
                    observation.passed = None;
                }
            }
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::Receipt {
                        commitment,
                        verdict: verdict.clone(),
                        runs: receipts.clone(),
                        source_after,
                        observations,
                    },
                },
                Source::Rules,
            )?;
            self.append(
                EventKind::Executed {
                    candidate,
                    exit_code: last_exit,
                    log_digest: Some(self.blob(&serde_json::to_vec(&receipts)?)?),
                    duration_ms: duration,
                },
                Source::Rules,
            )?;
            let state = match verdict {
                Verdict::Passed | Verdict::FailedTests => self.observe_unlocked(
                    commitment,
                    ResultInput {
                        met: if project_spec.is_some() {
                            met_cases
                        } else if verdict == Verdict::Passed {
                            vec!["protected_tests_pass".into()]
                        } else {
                            vec![]
                        },
                        evidence: format!(
                            "Independent sealed test receipt: {:?}; {}",
                            verdict,
                            receipts.last().unwrap()
                        ),
                        proposed_refinement: None,
                    },
                    Source::Rules,
                )?,
                _ => {
                    let reason = format!(
                        "Coding observation censored: {verdict:?}; no success/failure learning"
                    );
                    self.append(
                        EventKind::Unresolved {
                            candidate,
                            reason: reason.clone(),
                        },
                        Source::Rules,
                    )?;
                    self.adaptive_record(&Record::Censored { commitment, reason }, Source::Rules)?;
                    self.adaptive_state_inner()?
                }
            };
            Ok(Execution {
                verdict,
                receipts,
                state,
            })
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn generated_patch(e: &Engine, old: &str, new: &str) -> Result<ExactPatch> {
        let (_, source) = e.coding_source()?;
        Ok(ExactPatch {
            path: "implementation.rs".into(),
            source_digest: Hash::of(&[source.as_bytes()]),
            inspection_handle: e.inspection_handle(&source)?,
            old_text: old.into(),
            new_text: new.into(),
        })
    }
    fn offer(e: &Engine, patch: ExactPatch) -> Result<provider::Response> {
        Ok(provider::Response {
            attention_binding: e.proposal_attention_binding()?,
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
            protocol: provider::PROTOCOL.into(),
            state_root: e.store.head_root()?,
            proposals: vec![provider::Suggestion::ExactEdit {
                patch,
                rationale: "Try this implementation".into(),
            }],
        })
    }
    struct FixtureRunner {
        correct: bool,
    }
    impl Runner for FixtureRunner {
        fn run(&self, copy: &Path, argv: &[String], _: &Limits) -> RunResult {
            match argv[0].as_str() {
                "sh" => {
                    fs::write(copy.join("implementation.rs"), &argv[4]).unwrap();
                    RunResult::exit_only(0)
                }
                "rustc" => RunResult::exit_only(0),
                "./fixture-probe" => RunResult::from_output(
                    0,
                    if !self.correct {
                        b"1\n"
                    } else if argv[1] == "2" {
                        b"5\n"
                    } else {
                        b"7\n"
                    },
                    b"",
                    1,
                ),
                _ => panic!("unexpected command"),
            }
        }
    }
    #[test]
    fn generated_patches_survive_restart_with_independent_estimates() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        assert!(e
            .coding_initialize_mode(format!("sha256:{}", "a".repeat(64)), "501:20".into(), true)?
            .options
            .is_empty());
        let patch = generated_patch(&e, "a - b", "(a * 2) - b")?;
        let first_id = patch.id()?;
        let response = offer(&e, patch)?;
        assert!(!e.evaluate_proposal(response.clone(), false)?.committed);
        let first = e.evaluate_proposal(response.clone(), true)?;
        assert_eq!(first.state.pending.unwrap().forecast.raw.get(), 0.5);
        assert!(e.evaluate_proposal(response, true).is_err());
        let plan = e.coding_plan()?;
        let outcome = e.coding_execute(
            plan.commitment,
            plan.source_before,
            &FixtureRunner { correct: false },
        )?;
        assert_eq!(outcome.verdict, Verdict::FailedTests);
        drop(e);
        let e = Engine::open(dir.path())?;
        let request = e.proposal_request()?;
        assert_eq!(request.prior_outcomes[0]["success"], false);
        assert_eq!(
            request.coding.as_ref().unwrap()["recent_observations"][0]["observations"][0]["stdout"],
            "1\n"
        );
        assert_eq!(
            request.coding.unwrap()["recent_patches"][0]["patch"]["new_text"],
            "(a * 2) - b"
        );
        let patch = generated_patch(&e, "(a * 2) - b", "a.checked_add(b).unwrap_or_default()")?;
        let second_id = patch.id()?;
        assert_ne!(first_id, second_id);
        let second = e.evaluate_proposal(offer(&e, patch)?, true)?;
        assert_eq!(second.state.pending.unwrap().forecast.raw.get(), 0.5);
        let plan = e.coding_plan()?;
        assert_eq!(
            plan.commands[1],
            strings(&["rustc", "tests.rs", "-o", "fixture-probe"])
        );
        assert_eq!(
            e.coding_execute(
                plan.commitment,
                plan.source_before,
                &FixtureRunner { correct: true }
            )?
            .verdict,
            Verdict::Passed
        );
        let state = Engine::open(dir.path())?.adaptive_state()?;
        assert_eq!(
            state
                .options
                .iter()
                .find(|o| o.action == first_id)
                .unwrap()
                .prediction
                .value,
            1.0 / 3.0
        );
        assert_eq!(
            state
                .options
                .iter()
                .find(|o| o.action == second_id)
                .unwrap()
                .prediction
                .value,
            2.0 / 3.0
        );
        assert_eq!(
            fs::read_to_string(dir.path().join("coding-work/tests.rs"))?,
            PROBE
        );
        Ok(())
    }
    #[test]
    fn generated_scope_handle_source_and_unique_span_are_enforced() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.coding_initialize_mode(format!("sha256:{}", "a".repeat(64)), "501:20".into(), true)?;
        let patch = generated_patch(&e, "a - b", "a + b")?;
        for path in ["tests.rs", "../implementation.rs", "/tmp/implementation.rs"] {
            let mut invalid = patch.clone();
            invalid.path = path.into();
            assert!(e.evaluate_proposal(offer(&e, invalid)?, true).is_err());
        }
        let mut invalid = patch.clone();
        invalid.inspection_handle = Hash::of(&[b"stale"]);
        assert!(e.validate_patch(&invalid).is_err());
        let mut invalid = patch.clone();
        invalid.source_digest = Hash::of(&[b"stale"]);
        assert!(e.validate_patch(&invalid).is_err());
        let mut invalid = patch.clone();
        invalid.old_text = "a".into();
        assert!(e.validate_patch(&invalid).is_err());
        let mut value = serde_json::to_value(offer(&e, patch.clone())?)?;
        value["proposals"][0]["patch"]["argv"] = serde_json::json!(["sh"]);
        assert!(serde_json::from_value::<provider::Response>(value).is_err());
        fs::write(dir.path().join("coding-work/tests.rs"), "modified")?;
        assert!(e.validate_patch(&patch).is_err());
        Ok(())
    }
    #[test]
    fn probe_checks_behavior_not_candidate_test_claims() {
        assert_eq!(
            classify_probe(
                &RunResult::from_output(0, b"test result: ok. 2 passed; 0 failed;\n", b"", 1),
                b"5\n"
            ),
            Verdict::FailedTests
        );
        assert_eq!(
            classify_probe(&RunResult::exit_only(124), b"5\n"),
            Verdict::Timeout
        );
        assert_eq!(
            classify_probe(&RunResult::exit_only(125), b"5\n"),
            Verdict::Unavailable
        );
        assert_eq!(
            classify_probe(&RunResult::from_output(0, b"5\n", b"", 1), b"5\n"),
            Verdict::Passed
        );
    }
    #[test]
    fn timeout_and_infrastructure_never_become_test_failures() {
        assert_eq!(classify(&RunResult::exit_only(124), true), Verdict::Timeout);
        assert_eq!(
            classify(&RunResult::exit_only(125), true),
            Verdict::Unavailable
        );
        assert_eq!(
            classify(&RunResult::exit_only(0), true),
            Verdict::Unavailable
        );
    }
    #[test]
    fn protected_named_tests_required() {
        let good=b"test adds_positive ... ok\ntest adds_zero ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored;\n";
        assert_eq!(
            classify(&RunResult::from_output(0, good, b"", 1), true),
            Verdict::Passed
        );
        let bad=b"test adds_positive ... FAILED\ntest adds_zero ... ok\ntest result: FAILED. 1 passed; 1 failed; 0 ignored;\n";
        assert_eq!(
            classify(&RunResult::from_output(101, bad, b"", 1), true),
            Verdict::FailedTests
        );
    }
    #[test]
    fn unavailable_edit_is_censored_without_training() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.coding_initialize(format!("sha256:{}", "a".repeat(64)), "501:20".into())?;
        let pending = e.next(&DeterministicProvider)?.pending.unwrap();
        let plan = e.coding_plan()?;
        let result = e.coding_execute(
            pending.commitment,
            plan.source_before,
            &tsr_corpus::ScriptedRunner::new(RunResult::exit_only(125)),
        )?;
        assert_eq!(result.verdict, Verdict::Unavailable);
        assert!(result.state.pending.is_none());
        assert_eq!(result.state.completed, 1);
        assert!(result
            .state
            .options
            .iter()
            .all(|o| o.prediction.value == 0.5));
        let request = e.proposal_request()?;
        assert_eq!(request.prior_outcomes[0]["status"], "censored");
        assert!(!request.prior_outcomes[0]
            .as_object()
            .unwrap()
            .contains_key("success"));
        assert!(dir.path().join("coding-receipts").exists());
        assert!(!dir.path().join("coding-work/coding-receipts").exists());
        assert!(e
            .coding_execute(
                pending.commitment,
                plan.source_before,
                &tsr_corpus::ScriptedRunner::ok()
            )
            .is_err());
        Ok(())
    }
    #[test]
    fn coding_rejects_operator_success_and_stale_source() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.coding_initialize(format!("sha256:{}", "a".repeat(64)), "501:20".into())?;
        let p = e.next(&DeterministicProvider)?.pending.unwrap();
        assert!(e
            .observe(
                p.commitment,
                ResultInput {
                    met: vec!["protected_tests_pass".into()],
                    evidence: "trust me".into(),
                    proposed_refinement: None
                }
            )
            .is_err());
        let plan = e.coding_plan()?;
        assert_eq!(plan.action, "increment");
        assert!(e
            .coding_execute(
                p.commitment,
                Hash::of(&[b"wrong"]),
                &tsr_corpus::ScriptedRunner::ok()
            )
            .is_err());
        fs::write(dir.path().join("coding-work/tests.rs"), b"changed")?;
        assert!(e.coding_plan().is_err());
        Ok(())
    }
}

pub mod project;
