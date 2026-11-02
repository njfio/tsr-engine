//! Explicit owner-authored Rust project scope. Proposals cannot change this configuration.
use super::*;
use std::io::Read;
pub mod acceptance;
pub mod attention;
pub mod browser;
pub mod budget;
pub(super) mod cargo;
pub mod cost;
pub mod criterion_return;
pub mod cycle;
pub mod dataset;
pub mod desire;
pub mod evaluator;
pub mod expansion;
mod export_origin;
pub mod information;
pub mod lifecycle;
pub mod objective;
pub mod observations;
pub mod partial;
pub mod provisional;
pub mod pursuit;
pub mod relational;
pub mod success;
mod supporting;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeCase {
    pub id: String,
    pub args: Vec<String>,
    pub expected_stdout: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectSpec {
    #[serde(default, skip_serializing_if = "is_false")]
    pub full_workspace_oracle: bool,
    pub goal: String,
    pub image: String,
    pub user: String,
    pub editable: BTreeMap<String, String>,
    pub protected: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub edit_windows: BTreeMap<String, EditWindow>,
    /// Explicit read-only implementation observations; initial UTF-8 offsets, never edit authority.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub supporting_spans: BTreeMap<String, Vec<EditWindow>>,
    #[serde(default)]
    pub cargo: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub browser: bool,
    /// Owner-only opt-in; Chromium sandbox disabled, sealed container controls unchanged.
    #[serde(default, skip_serializing_if = "is_false")]
    pub browser_container_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub named_package: Option<String>,
    #[serde(default)]
    pub probe_entry: String,
    pub cases: Vec<ProbeCase>,
    pub timeout_ms: u64,
}
/// Owner-admitted UTF-8 byte offsets in the initial file. Everything outside is immutable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditWindow {
    pub start: usize,
    pub end: usize,
}
fn is_false(v: &bool) -> bool {
    !*v
}
pub(super) fn execution_command_count(spec: &ProjectSpec) -> usize {
    (if spec.named_package.is_some() || spec.cargo {
        3
    } else {
        2 + spec.cases.len()
    }) + usize::from(spec.full_workspace_oracle)
}
const MAX_SNAPSHOT: usize = 1024 * 1024;
fn build_asset(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_some_and(|p| matches!(p, "Cargo.toml" | "Cargo.lock"))
}
fn file_limit(spec: &ProjectSpec, path: &str) -> usize {
    if spec.browser && spec.editable.contains_key(path) {
        MAX_SNAPSHOT
    } else if spec.named_package.is_some() && spec.protected.contains_key(path) {
        512 * 1024
    } else if spec.protected.contains_key(path) && build_asset(path) {
        256 * 1024
    } else if spec.protected.contains_key(path) || spec.edit_windows.contains_key(path) {
        64 * 1024
    } else {
        4096
    }
}
/// Validate immutable prefix/suffix before exposing a bounded implementation window.
fn visible_source<'a>(spec: &ProjectSpec, path: &str, source: &'a str) -> Result<&'a str> {
    if let Some(window) = spec.edit_windows.get(path) {
        let original = spec
            .editable
            .get(path)
            .ok_or_else(|| anyhow::anyhow!("window must name editable source"))?;
        ensure!(
            window.start <= window.end
                && original.is_char_boundary(window.start)
                && original.is_char_boundary(window.end),
            "invalid UTF-8 edit window"
        );
        let prefix = &original[..window.start];
        let suffix = &original[window.end..];
        ensure!(
            source.len() >= prefix.len() + suffix.len()
                && source.starts_with(prefix)
                && source.ends_with(suffix),
            "protected bytes outside edit window changed: {path}"
        );
        let body = &source[prefix.len()..source.len() - suffix.len()];
        ensure!(body.len() <= 4096, "editable window exceeds 4096 bytes");
        Ok(body)
    } else {
        Ok(source)
    }
}
pub(in crate::engine::adaptive::coding) fn validate_snapshot(
    spec: &ProjectSpec,
    snapshot: &BTreeMap<String, String>,
) -> Result<()> {
    ensure!(
        snapshot.values().map(String::len).sum::<usize>()
            <= if spec.named_package.is_some() {
                16 * MAX_SNAPSHOT
            } else {
                MAX_SNAPSHOT
            },
        "project snapshot exceeds configured bound"
    );
    let mut context = 0;
    for (path, source) in snapshot {
        ensure!(
            source.len() <= file_limit(spec, path),
            "file exceeds admitted bound: {path}"
        );
        if let Some(protected) = spec.protected.get(path) {
            ensure!(
                source == protected,
                "protected project file changed: {path}"
            );
        }
        let visible = visible_source(spec, path, source)?;
        if !(spec.protected.contains_key(path)
            && (spec.browser
                || spec.named_package.is_some()
                || build_asset(path)
                || source.len() > 4096))
        {
            context += visible.len();
        }
    }
    context += supporting::context(spec, snapshot)?
        .iter()
        .map(|v| v["source"].as_str().unwrap().len())
        .sum::<usize>();
    ensure!(
        context <= if spec.browser { MAX_SNAPSHOT } else { 8192 },
        "project proposal context exceeds admitted byte bound"
    );
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedEdit {
    pub path: String,
    pub replacement: String,
}
pub fn case_criterion(id: &str) -> String {
    format!("test:{id}")
}
pub fn digest(snapshot: &BTreeMap<String, String>) -> Result<Hash> {
    Ok(Hash::of(&[
        b"rust-project-snapshot@v1",
        &serde_json::to_vec(snapshot)?,
    ]))
}
fn valid_path(path: &str) -> bool {
    path.len() <= 128
        && path.split('/').all(|part| {
            !part.is_empty()
                && !part.starts_with('.')
                && !part.starts_with('-')
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        })
}
fn read_file(root: &Path, path: &str, limit: usize) -> Result<String> {
    ensure!(valid_path(path), "invalid scoped Rust path");
    let mut current = root.to_path_buf();
    ensure!(
        fs::symlink_metadata(root)?.is_dir()
            && !fs::symlink_metadata(root)?.file_type().is_symlink(),
        "workspace type changed"
    );
    for part in path.split('/') {
        current.push(part);
        let meta = fs::symlink_metadata(&current)?;
        ensure!(
            !meta.file_type().is_symlink() && (meta.is_dir() || meta.is_file()),
            "nonregular project path"
        );
    }
    let meta = fs::symlink_metadata(&current)?;
    ensure!(
        meta.is_file() && meta.len() <= limit as u64,
        "project file type/size changed"
    );
    let mut bytes = vec![];
    fs::File::open(current)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= limit, "project file exceeds bound");
    Ok(String::from_utf8(bytes)?)
}
impl ProjectSpec {
    pub fn from_file(path: &Path) -> Result<Self> {
        let mut bytes = vec![];
        fs::File::open(path)?
            .take(20 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() <= 20 * 1024 * 1024,
            "project specification exceeds bound"
        );
        let spec: Self = serde_json::from_slice(&bytes)?;
        Ok(spec)
    }
    pub(super) fn validate(&self) -> Result<()> {
        ensure!(short(&self.goal), "bounded goal required");
        ensure!(
            !self.browser_container_only || self.browser,
            "container-only browser mode requires browser profile"
        );
        if self.browser {
            browser::validate(self)?;
        }
        ensure!(!self.full_workspace_oracle || self.named_package.is_some(), "extra full-workspace oracle requires named-package Cargo scope; ordinary Cargo already runs full suite");
        ensure!(
            self.named_package.is_none() || self.cargo,
            "named package requires Cargo"
        );
        ensure!(
            !self.editable.is_empty()
                && self.editable.len() <= 8
                && !self.protected.is_empty()
                && self.editable.len() + self.protected.len()
                    <= if self.named_package.is_some() {
                        512
                    } else {
                        12
                    },
            "file scope bounds"
        );
        ensure!(
            self.editable
                .keys()
                .all(|p| !self.protected.contains_key(p)),
            "editable and protected scopes overlap"
        );
        ensure!(
            self.cargo || self.protected.contains_key(&self.probe_entry),
            "probe entry must be protected"
        );
        ensure!(
            self.editable
                .iter()
                .chain(&self.protected)
                .all(|(p, s)| valid_path(p) && s.len() <= file_limit(self, p)),
            "invalid path or file size"
        );
        ensure!(
            self.edit_windows
                .keys()
                .all(|p| self.editable.contains_key(p)),
            "window must name editable source"
        );
        let snapshot = self
            .editable
            .iter()
            .chain(&self.protected)
            .map(|(p, s)| (p.clone(), s.clone()))
            .collect();
        validate_snapshot(self, &snapshot)?;
        ensure!(
            !self.cases.is_empty()
                && self.cases.len() <= 4
                && self
                    .cases
                    .iter()
                    .map(|c| &c.id)
                    .collect::<BTreeSet<_>>()
                    .len()
                    == self.cases.len(),
            "unique bounded test cases required"
        );
        ensure!(
            self.cases.iter().all(|c| !c.id.is_empty()
                && c.id.len() <= 256
                && c.id.split("::").all(|part| !part.is_empty()
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b)))
                && c.args.len() <= 8
                && c.args.iter().all(|a| a.len() <= 64 && !a.contains('\0'))
                && c.expected_stdout.len() <= 128),
            "invalid probe case"
        );
        ensure!(
            (1..=if self.named_package.is_some() || self.browser {
                600000
            } else {
                20000
            })
                .contains(&self.timeout_ms),
            "project command deadline exceeds configured bounds"
        );
        ensure!(
            self.user
                .split(':')
                .next()
                .and_then(|s| s.parse::<u32>().ok())
                .is_some_and(|n| n != 0),
            "nonroot user required"
        );
        ensure!(
            self.editable.keys().all(|p| (self.browser
                && [".html", ".css", ".js"].iter().any(|ext| p.ends_with(ext)))
                || (!self.browser && p.ends_with(".rs") && !p.ends_with("build.rs"))),
            "source extension outside admitted project profile"
        );
        if self.cargo {
            cargo::validate(self)?;
        }
        SealedRunner::new(Runtime::Docker, self.image.clone(), self.user.clone())
            .map_err(anyhow::Error::msg)?;
        Ok(())
    }
}
impl Engine {
    /// Read durable controller state; never dispatch work while reporting status.
    pub fn project_status(&self) -> Result<serde_json::Value> {
        let spec = self.project_configuration()?;
        let snapshot = digest(&self.project_snapshot(&spec)?)?;
        let ready = self.dependencies_ready(&spec)?;
        let state = self.adaptive_state()?;
        let records = self.records()?;
        let latest = records.iter().rev().find_map(|(_, r)| match r {
            Record::Coding {
                record:
                    CodingRecord::Receipt {
                        commitment,
                        verdict,
                        runs,
                        source_after,
                        ..
                    },
            } => {
                Some(serde_json::json!({"commitment":commitment,"verdict":verdict,"receipts":runs,"source_after":source_after}))
            }
            _ => None,
        });
        let attempted = state.pending.as_ref().is_some_and(|p| records.iter().any(|(_,r)| matches!(r, Record::Coding { record: CodingRecord::Intent { plan } } if plan.commitment == p.commitment)));
        let status = if attempted {
            "execution_recovery_required"
        } else if state.pending.is_some() {
            "committed_not_executed"
        } else if latest
            .as_ref()
            .is_some_and(|r| r["source_after"] != serde_json::json!(snapshot))
        {
            "source_changed_since_execution"
        } else if !ready {
            "dependencies_required"
        } else {
            match latest.as_ref().and_then(|r| r["verdict"].as_str()) {
                Some("passed") => "passed",
                Some("failed_tests") => "needs_repair",
                Some(_) => "execution_stopped",
                None => "ready_for_proposal",
            }
        };
        let completion = self.project_acceptance_status()?;
        let operator_next = if completion.requirements_revision.is_none() {
            "register_requirements"
        } else if status == "dependencies_required" {
            "prepare"
        } else if status == "committed_not_executed" {
            "resume"
        } else if status == "execution_recovery_required" {
            "reconcile_execution_receipts"
        } else if completion.accepted {
            "export_accepted"
        } else if completion.status == "rejected"
            && state
                .acceptance_cycle
                .as_ref()
                .is_some_and(|c| c["repair_ready"] == true)
        {
            "request_corrective_proposal"
        } else if completion.status == "rejected" {
            "draft_shareable_feedback"
        } else if status == "passed" && self.acceptance_pending_feedback()? {
            "request_proposal"
        } else if status == "passed" {
            "draft_independent_review"
        } else {
            "request_proposal"
        };
        let execution_status = status;
        let status = if execution_status == "passed" {
            if completion.accepted {
                "goal_accepted"
            } else {
                "tests_passed_unaccepted"
            }
        } else {
            execution_status
        };
        Ok(
            serde_json::json!({"status":status,"operator_next":operator_next,"execution_status":execution_status,"acceptance_cycle":state.acceptance_cycle,"goal_completion":completion,"dependencies_ready":ready,"protected_files_intact":true,"pending":state.pending,"completed":state.completed,"latest_execution":latest,"scope_files":spec.editable.len()+spec.protected.len(),"goal":state.goal}),
        )
    }
    pub fn project_initialize(&self, spec: ProjectSpec) -> Result<State> {
        spec.validate()?;
        ensure!(self.events()?.is_empty(), "directory already initialized");
        let root = self.directory.join("coding-work");
        fs::create_dir(&root)?;
        for (path, source) in spec.editable.iter().chain(&spec.protected) {
            let target = root.join(path);
            fs::create_dir_all(target.parent().unwrap())?;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(target)?;
            file.write_all(source.as_bytes())?;
            file.sync_all()?;
        }
        let mut criteria: Vec<_> = spec.cases.iter().map(|c| case_criterion(&c.id)).collect();
        if spec.full_workspace_oracle {
            criteria.push(case_criterion("full_workspace_oracle@v1"));
        }
        self.configure(Spec { goal:spec.goal.clone(), criteria:criteria.clone(), actions:vec![Action{id:"scope".into(),description:if spec.browser {"Owner-admitted source paths and protected browser harness; no proposal command or permission authority".into()} else {"Owner-admitted source paths and protected Rust probe; no proposal command or permission authority".into()},serves:criteria,cost:0.1}],prior:vec![],initial_commitment:None })?;
        self.write(|| {
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::Config {
                        config: Config {
                            image: spec.image.clone(),
                            user: spec.user.clone(),
                            generated: true,
                            project: Some(spec),
                        },
                    },
                },
                Source::Operator,
            )?;
            self.adaptive_state_inner()
        })
    }
    pub fn project_configuration(&self) -> Result<ProjectSpec> {
        self.coding_config()?
            .and_then(|c| c.project)
            .ok_or_else(|| anyhow::anyhow!("run project-init first"))
    }
    pub(super) fn project_snapshot(&self, spec: &ProjectSpec) -> Result<BTreeMap<String, String>> {
        let root = self.directory.join("coding-work");
        let mut snapshot = BTreeMap::new();
        for path in spec.editable.keys().chain(spec.protected.keys()) {
            let source = read_file(&root, path, file_limit(spec, path))?;
            if let Some(protected) = spec.protected.get(path) {
                ensure!(
                    &source == protected,
                    "protected project file changed: {path}"
                );
            }
            snapshot.insert(path.clone(), source);
        }
        validate_snapshot(spec, &snapshot)?;
        // Only admitted files, their directories and the controller-created probe artifact exist.
        let mut pending = vec![root.clone()];
        let mut count = 0;
        while let Some(dir) = pending.pop() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                count += 1;
                ensure!(
                    count
                        <= if spec.named_package.is_some() {
                            2048
                        } else {
                            64
                        },
                    "workspace inventory exceeds bound"
                );
                let kind = entry.file_type()?;
                ensure!(!kind.is_symlink(), "workspace symlink refused");
                let path = entry.path();
                let relative = path
                    .strip_prefix(&root)?
                    .to_str()
                    .ok_or_else(|| anyhow::anyhow!("nonutf8 workspace path"))?;
                if kind.is_dir() {
                    ensure!(
                        snapshot
                            .keys()
                            .any(|p| p.starts_with(&format!("{relative}/"))),
                        "unadmitted workspace directory"
                    );
                    pending.push(path);
                } else {
                    ensure!(
                        kind.is_file()
                            && (snapshot.contains_key(relative)
                                || (!spec.cargo && !spec.browser && relative == "fixture-probe")),
                        "unadmitted workspace file"
                    );
                }
            }
        }
        Ok(snapshot)
    }
    fn project_handle(
        &self,
        spec: &ProjectSpec,
        snapshot: &BTreeMap<String, String>,
        path: &str,
    ) -> Result<Hash> {
        Ok(Hash::of(&[
            b"project-inspection@v1",
            self.directory.canonicalize()?.to_string_lossy().as_bytes(),
            &serde_json::to_vec(spec)?,
            &digest(snapshot)?.0,
            path.as_bytes(),
        ]))
    }
    pub(super) fn project_validate_patch(
        &self,
        spec: &ProjectSpec,
        patch: &ExactPatch,
    ) -> Result<String> {
        ensure!(
            self.dependencies_ready(spec)?,
            "run project-prepare before proposing dependency-bearing repairs"
        );
        ensure!(
            spec.editable.contains_key(&patch.path),
            "path outside admitted editable scope"
        );
        let snapshot = self.project_snapshot(spec)?;
        ensure!(
            patch.inspection_handle == self.project_handle(spec, &snapshot, &patch.path)?,
            "stale project inspection handle"
        );
        let replacement = tsr_batch::broker_v2::prepare_edit_with_limit(
            &self.directory.join("coding-work"),
            &patch.path,
            patch.source_digest,
            &patch.old_text,
            &patch.new_text,
            if spec.browser {
                MAX_SNAPSHOT
            } else {
                64 * 1024
            },
        )
        .map_err(|e| anyhow::anyhow!("{e}"))?;
        let mut edited = snapshot;
        edited.insert(patch.path.clone(), replacement.clone());
        validate_snapshot(spec, &edited)?;
        Ok(replacement)
    }
    pub(super) fn project_context(&self, spec: &ProjectSpec) -> Result<serde_json::Value> {
        let snapshot = self.project_snapshot(spec)?;
        let files: Vec<_> = snapshot.iter().filter(|(path,_)| spec.named_package.is_none() || spec.editable.contains_key(*path)).map(|(path, source)| {
            let mut file = serde_json::json!({"path":path,"source_digest":Hash::of(&[source.as_bytes()]),"editable":spec.editable.contains_key(path),"inspection_handle":self.project_handle(spec,&snapshot,path)?,"bytes":source.len()});
            if spec.protected.contains_key(path) && (spec.browser || spec.cargo || build_asset(path) || source.len() > 4096) {
                file["source_omitted"] = serde_json::json!("protected asset; original bytes retained and checked");
            } else {
                file["source"] = serde_json::json!(visible_source(spec,path,source)?);
            }
            if let Some(window) = spec.edit_windows.get(path) {
                file["edit_window"] = serde_json::json!({"initial_start":window.start,"initial_end":window.end,"outside_window":"immutable; source_digest covers the entire file"});
            }
            Ok(file)
        }).collect::<Result<_>>()?;
        let supporting = supporting::context(spec, &snapshot)?;
        let mut issued = files.clone();
        issued.extend(supporting.iter().cloned());
        if let Err(error) = self.record_project_context(&issued, digest(&snapshot)?) {
            eprintln!("context evidence unavailable: {error}");
        }
        let mut context = serde_json::json!({"kind":if spec.browser {"browser_project@v1"} else if spec.cargo {"cargo_project@v1"} else {"rust_project@v1"},"generated_edits":true,"dependencies_ready":self.dependencies_ready(spec)?,"files":files,"cases":spec.cases,"probe_entry":spec.probe_entry,"snapshot":digest(&snapshot)?,"named_package":spec.named_package,"protected_files":spec.protected.len(),"execution":"Only owner-admitted source paths may be edited. Protected files, test cases, goals and commands cannot be changed by a proposal. Propose one exact_edit or one edit_set containing up to four different files; the controller evaluates the owner-admitted test scope. Each patch/set identity has a separate prior."});
        if spec.browser {
            context["browser_isolation"] = serde_json::json!({"chromium_sandbox": !spec.browser_container_only, "outer_container_required": true});
        }
        if !supporting.is_empty() {
            context["supporting_context"] = serde_json::json!(supporting);
        }
        context["relational"] = self.project_relational_context()?;
        let pursuit = self.project_pursuit_context()?;
        if pursuit["history"].as_array().is_some_and(|h| !h.is_empty()) {
            context["pursuit"] = pursuit;
        }
        let objective = self.project_objective_context()?;
        if !objective.is_null() {
            context["objective"] = objective;
        }
        let information = self.project_information_context()?;
        if !information.is_null() {
            context["information"] = information;
        }
        Ok(context)
    }
    /// Operator admission of a verification action, not a model-controlled scope change.
    pub fn project_admit_verification(
        &self,
        expected_root: Hash,
        expected_snapshot: Hash,
    ) -> Result<serde_json::Value> {
        self.write(|| {
            ensure!(self.store.head_root()? == expected_root, "stale verification state");
            ensure!(self.adaptive_state_inner()?.pending.is_none() && !self.project_objective_pending()?, "pending execution prevents admission");
            let budget = self.project_budget()?;
            ensure!(budget.active && !budget.stopped && !budget.exceeded && budget.unknown_usage == 0 && budget.unknown_compute_requests == 0 && budget.unsettled_requests == 0 && budget.unresolved_tool_requests == 0, "open resolved resource admission required");
            let spec = self.project_configuration()?;
            ensure!(self.dependencies_ready(&spec)?, "prepare dependencies first");
            let upper = spec.timeout_ms.checked_mul(execution_command_count(&spec) as u64).ok_or_else(|| anyhow::anyhow!("verification budget overflow"))?;
            let (_, admission, _, _) = self.project_resource_admission()?.ok_or_else(|| anyhow::anyhow!("resource admission missing"))?;
            let limits = budget.limits.as_ref().ok_or_else(|| anyhow::anyhow!("non-tool limits required"))?;
            ensure!(budget.tool_ms_charged.checked_add(upper).is_some_and(|n| n <= admission.tool_ms_ceiling) && budget.elapsed_wall_ms.checked_add(upper).is_some_and(|n| n <= limits.wall_ms), "verification tool/wall budget exhausted");
            ensure!(self.records()?.iter().filter(|(_,r)| matches!(r, Record::Coding { record:CodingRecord::Verification { .. } })).count() < 128, "verification admission limit reached");
            let snapshot = digest(&self.project_snapshot(&spec)?)?;
            ensure!(snapshot == expected_snapshot, "stale verification snapshot");
            let configuration = Hash::of(&[&serde_json::to_vec(&spec)?]);
            let id = format!("verify-{}", Hash::of(&[b"project-verification@v1", &expected_root.0, &snapshot.0, &configuration.0]));
            let action = Action { id: id.clone(), description: "Verify the admitted current source without editing it".into(), serves: self.coding_criteria()?, cost: 0.1 };
            self.adaptive_record(&Record::Coding { record: CodingRecord::Verification { state_root: expected_root, snapshot, configuration, action } }, Source::Operator)?;
            Ok(serde_json::json!({"protocol":"tsr-proposal@v1", "attention_binding":self.proposal_attention_binding()?, "state_root":self.store.head_root()?, "proposals":[{"kind":"existing","action":id,"rationale":"Operator-admitted verification of the exact current source; no patch or generator forecast"}]}))
        })
    }
    pub(crate) fn project_verification_admitted(
        &self,
        action: &str,
    ) -> Result<Option<(Hash, Hash)>> {
        let events = self.events()?;
        for (id, record) in self.records()? {
            if let Record::Coding {
                record:
                    CodingRecord::Verification {
                        snapshot,
                        configuration,
                        action: a,
                        ..
                    },
            } = record
            {
                if a.id == action {
                    ensure!(
                        events
                            .iter()
                            .any(|(_, e)| e.id == id && e.source == Source::Operator),
                        "verification admission requires operator provenance"
                    );
                    return Ok(Some((snapshot, configuration)));
                }
            }
        }
        Ok(None)
    }
    pub(crate) fn project_verification_for(&self, action: &str) -> Result<bool> {
        if let Some((snapshot, configuration)) = self.project_verification_admitted(action)? {
            let spec = self.project_configuration()?;
            ensure!(
                configuration == Hash::of(&[&serde_json::to_vec(&spec)?])
                    && snapshot == digest(&self.project_snapshot(&spec)?)?,
                "stale verification source or scope"
            );
            return Ok(true);
        }
        Ok(false)
    }
    pub(super) fn project_plan(
        &self,
        config: &Config,
        spec: &ProjectSpec,
        p: Pending,
    ) -> Result<Plan> {
        ensure!(
            self.dependencies_ready(spec)?,
            "run project-prepare: locked dependencies are not ready in the pinned image"
        );
        let mut snapshot = self.project_snapshot(spec)?;
        let before = digest(&snapshot)?;
        let verification = self.project_verification_for(&p.action)?;
        let patches = if verification {
            vec![]
        } else {
            let patches = self
                .patches_for(&p.action)?
                .ok_or_else(|| anyhow::anyhow!("missing patch candidate"))?;
            self.validate_patch_set(&patches)?;
            patches
        };
        let browser_patches = if spec.browser {
            Some(serde_json::to_string(&patches)?)
        } else {
            None
        };
        let mut edits = vec![];
        for patch in patches {
            let replacement = self.validate_patch(&patch)?;
            snapshot.insert(patch.path.clone(), replacement.clone());
            edits.push(PreparedEdit {
                path: patch.path,
                replacement,
            });
        }
        validate_snapshot(spec, &snapshot)?;
        let after = digest(&snapshot)?;
        let mut edit = strings(&[
            "sh",
            "-c",
            "while [ \"$#\" -gt 0 ]; do printf '%s' \"$2\" > \"$1\" || exit 1; shift 2; done",
            "tsr-scoped-edit",
        ]);
        if let Some(payload) = browser_patches {
            // Send only exact spans, never the entire reconstructed file. The
            // immutable source snapshot was verified before this sealed plan.
            edit=strings(&["sh","-c","script=\"$1\"; shift; exec node -e \"$script\" -- \"$@\"","tsr-scoped-edit", "const fs=require('fs');const edits=JSON.parse(process.argv.slice(1).join(''));for(const p of edits){const s=fs.readFileSync(p.path,'utf8');const i=s.indexOf(p.old_text);if(!p.old_text||i<0||s.indexOf(p.old_text,i+1)>=0)throw Error('exact source span drift');fs.writeFileSync(p.path,s.slice(0,i)+p.new_text+s.slice(i+p.old_text.length));}"]);
            let mut start = 0;
            while start < payload.len() {
                let mut end = (start + 16384).min(payload.len());
                while !payload.is_char_boundary(end) {
                    end -= 1;
                }
                edit.push(payload[start..end].to_owned());
                start = end;
            }
        } else {
            for e in &edits {
                edit.extend([e.path.clone(), e.replacement.clone()]);
            }
        }
        let mut commands = vec![
            if verification {
                strings(&["true"])
            } else {
                edit
            },
            strings(&[
                "rustc",
                "--edition=2021",
                &spec.probe_entry,
                "-o",
                "fixture-probe",
            ]),
        ];
        for case in &spec.cases {
            let mut command = vec!["./fixture-probe".into()];
            command.extend(case.args.clone());
            commands.push(command);
        }
        if spec.browser {
            commands.truncate(1);
            commands.push(strings(&["node", "--check", &spec.probe_entry]));
            for case in &spec.cases {
                let mut command = vec![
                    "node".into(),
                    spec.probe_entry.clone(),
                    case.args[0].clone(),
                ];
                if spec.browser_container_only {
                    command.push("--container-only".into());
                }
                commands.push(command);
            }
        } else if let Some(package) = &spec.named_package {
            commands.truncate(1);
            commands.push(strings(&["true"]));
            commands.push(strings(&[
                "cargo",
                "test",
                "--quiet",
                "--locked",
                "--offline",
                "-p",
                package,
                "--lib",
                "-j",
                "1",
                "--target-dir",
                "/tsr-build",
                "--",
                &spec.cases[0].id,
                "--exact",
                "--format=pretty",
                "--test-threads=1",
            ]));
        } else if spec.cargo {
            let mut build = cargo::command();
            build.push("--no-run".into());
            let mut test = cargo::command();
            test.extend(strings(&["--", "--test-threads=1"]));
            commands.truncate(1);
            commands.extend([build, test]);
        }
        if spec.full_workspace_oracle {
            let mut full = tsr_corpus::preparation::full_test_argv();
            full.extend(strings(&[
                "--target-dir",
                "/tsr-build",
                "--",
                "--format=pretty",
                "--test-threads=1",
            ]));
            commands.push(full);
        }
        ensure!(
            commands.len() == execution_command_count(spec),
            "project command budget contract changed"
        );
        Ok(Plan {
            commitment: p.commitment,
            action: p.action,
            source_before: before,
            replacement: String::new(),
            image: config.image.clone(),
            user: config.user.clone(),
            commands,
            timeout_ms: spec.timeout_ms,
            protected_tests: Hash::of(&[&serde_json::to_vec(&spec.protected)?]),
            generated: true,
            edits,
            cases: spec.cases.clone(),
            expected_after: Some(after),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spec() -> ProjectSpec {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/engine-invoice/project.json"
        )))
        .unwrap()
    }
    #[test]
    fn verifier_integrates_with_observed_tests_and_separate_operator_adoption() -> Result<()> {
        use acceptance::reviewer::{
            Admission, Assessment, Finding, Output, Report, Request, Verifier,
        };
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "semantic".into(),
            "Synthetic scoped criterion".into(),
        )]))?;
        e.evaluate_proposal(
            offer(
                &e,
                vec![patch(
                    &e,
                    "src/pricing.rs",
                    "unit_cents + quantity",
                    "unit_cents * quantity",
                )?],
            )?,
            true,
        )?;
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        let before = e.project_acceptance_status()?;
        assert!(!before.accepted);
        let spec = e.project_configuration()?;
        let paths = spec
            .editable
            .keys()
            .chain(spec.protected.keys())
            .cloned()
            .chain(std::iter::once("test_receipt".into()))
            .collect::<BTreeSet<_>>();
        let expires = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
            + 3_600_000;
        let a = Admission {
            prerequisites: BTreeMap::new(),
            version: "verifier-admission@v1".into(),
            scope: Hash::of(&[&serde_json::to_vec(&spec)?]),
            requirements_revision: before.requirements_revision.unwrap(),
            verifier_identity: "test-host-reviewer".into(),
            generator_identity: "test-generator".into(),
            artifact: Hash::of(&[b"synthetic-verifier"]),
            coverage: BTreeMap::from([("semantic".into(), paths)]),
            max_requests: 1,
            tokens_per_request: 100,
            timeout_ms: 1000,
            expires_at: lifecycle::timestamp(&format!("unix-ms:{expires}"))?,
        };
        struct TestVerifier(Admission);
        impl Verifier for TestVerifier {
            fn identity(&self) -> (&str, Hash) {
                (&self.0.verifier_identity, self.0.artifact)
            }
            fn evaluate(&self, r: &Request, _: u64, _: u64) -> Result<Output> {
                let receipt: serde_json::Value = serde_json::from_str(&r.evidence["test_receipt"])?;
                assert_eq!(receipt["tested_source"], serde_json::to_value(r.snapshot)?);
                Ok(Output {
                    tokens: Some(10),
                    json: serde_json::to_string(&Report {
                        findings: BTreeMap::from([(
                            "semantic".into(),
                            Assessment {
                                basis: None,
                                finding: Finding::Met,
                                evidence: self.0.coverage["semantic"].clone(),
                                explanation: "Synthetic integration finding".into(),
                                limitations: "Fake runner, not correctness evidence".into(),
                            },
                        )]),
                    })?,
                })
            }
        }
        let id = e.project_verifier_admit(a.clone())?;
        let (request, _) = e.project_verifier_request(id)?;
        let report = e.project_verifier_run(request, &TestVerifier(a))?;
        assert!(!e.project_acceptance_status()?.accepted);
        assert!(
            Engine::open(dir.path())?
                .project_verifier_adopt(report)?
                .accepted
        );
        assert!(e.project_verifier_adopt(report).is_err());
        assert_eq!(
            e.project_acceptance_status()?.test_receipt,
            before.test_receipt
        );
        assert!(e
            .acceptance_current_aims()?
            .values()
            .all(|id| Ledger::new(&e.store)
                .collective()
                .unwrap()
                .entries
                .iter()
                .any(|p| p.preference.id == *id && p.status == Some(PreferenceStatus::Fulfilled))));
        Ok(())
    }
    #[test]
    fn guidance_carry_is_state_bound_without_review_or_authority_carry() -> Result<()> {
        guidance_carry_under_policy(false)
    }
    #[test]
    fn unlimited_guidance_carry_ignores_legacy_ceilings_not_closed_authority() -> Result<()> {
        guidance_carry_under_policy(true)
    }
    fn guidance_carry_under_policy(unlimited: bool) -> Result<()> {
        use acceptance::{Decision, Evidence, Feedback, Review};
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "semantic".into(),
            "PRIVATE evidence criterion".into(),
        )]))?;
        e.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 86_400_000,
            reason: "synthetic guidance regression".into(),
            non_tool: Some(budget::Limits {
                model_calls: 2,
                model_tokens: 1000,
                wall_ms: 600_000,
                patch_bytes: 100_000,
                external_cost_microusd: 0,
            }),
        })?;
        if unlimited {
            let root = e.store.head_root()?;
            e.project_resource_unlimited_admit(root, "ab".repeat(32))?;
            let previous = e.project_resource_admission()?.unwrap().1;
            e.project_resource_extend(lifecycle::ResourceExtension {
                state_root: root,
                previous,
                limits: budget::Limits {
                    model_calls: 0,
                    model_tokens: 0,
                    wall_ms: 0,
                    patch_bytes: 0,
                    external_cost_microusd: 0,
                },
                tool_ms_ceiling: 0,
                reason: "Authored legacy ceiling exhaustion under explicit unlimited policy".into(),
            })?;
        }
        let initial = e.project_acceptance_status()?;
        e.project_acceptance_feedback(Feedback {
            requirements_revision: initial.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([("semantic".into(), "Published stable intent".into())]),
        })?;
        let before = e.proposal_request()?;
        let cycle = e.acceptance_cycle()?;
        let publication = cycle["guidance_provenance"]["publication"].clone();
        let review = Review {
            snapshot: initial.snapshot,
            requirements_revision: initial.requirements_revision.unwrap(),
            test_receipt: None,
            decision: Decision::Reject,
            evidence: BTreeMap::from([(
                "semantic".into(),
                Evidence {
                    met: false,
                    evidence: "PRIVATE independent finding".into(),
                },
            )]),
        };
        e.project_acceptance_review(review.clone())?;
        let carried = e.acceptance_cycle()?;
        assert_eq!(carried["guidance"], cycle["guidance"]);
        assert_eq!(carried["guidance_provenance"]["publication"], publication);
        assert_eq!(
            carried["guidance_provenance"]["status"],
            "unchanged_guidance_rebound"
        );
        assert_eq!(carried["repair_ready"], true);
        assert_eq!(carried, Engine::open(dir.path())?.acceptance_cycle()?);
        assert_ne!(before.next_attention, e.proposal_request()?.next_attention);
        assert!(
            !serde_json::to_string(&e.proposal_request()?)?.contains("PRIVATE independent finding")
        );
        let response = offer(
            &e,
            vec![patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "unit_cents * quantity",
            )?],
        )?;
        // The synthetic runner returns a changed source under the existing admission.
        e.evaluate_proposal(response.clone(), true)?;
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        assert_eq!(e.project_acceptance_status()?.status, "stale_review");
        assert!(e.project_acceptance_review(review).is_err());
        assert!(e.evaluate_proposal(response, true).is_err());
        assert_eq!(e.acceptance_cycle()?["guidance"], cycle["guidance"]);
        assert_eq!(e.acceptance_cycle()?["repair_ready"], false);
        let fresh = e.project_acceptance_status()?;
        e.project_acceptance_review(Review {
            snapshot: fresh.snapshot,
            requirements_revision: fresh.requirements_revision.unwrap(),
            test_receipt: fresh.test_receipt,
            decision: Decision::Reject,
            evidence: BTreeMap::from([(
                "semantic".into(),
                Evidence {
                    met: false,
                    evidence: "Fresh independent observation".into(),
                },
            )]),
        })?;
        e.project_lifecycle_close()?;
        assert!(e.acceptance_cycle()?["guidance"]
            .as_object()
            .unwrap()
            .is_empty());
        assert!(e
            .validate_proposal_attention(e.proposal_attention_binding()?)
            .is_err());
        let root = e.store.head_root()?;
        let retained = e.project_published_guidance_history()?;
        assert_eq!(
            retained["publications"][0]["contributors"][0]["meaning"],
            "Published stable intent"
        );
        assert_eq!(
            retained["publications"][0]["contributors"][0]["eligible"],
            false
        );
        assert_eq!(retained["publications"][0]["publication"], publication);
        assert!(!retained.to_string().contains("PRIVATE"));
        assert_eq!(e.store.head_root()?, root);
        assert_eq!(
            Engine::open(dir.path())?.project_published_guidance_history()?,
            retained
        );
        Ok(())
    }
    #[test]
    fn published_guidance_history_preserves_revision_identity_and_rejects_forged_links(
    ) -> Result<()> {
        for forged in 0..3 {
            let dir = tempfile::tempdir()?;
            let e = Engine::open(dir.path())?;
            e.project_initialize(spec())?;
            for (private, public) in [
                ("PRIVATE first", "Published first"),
                ("PRIVATE second", "Published second"),
            ] {
                e.project_acceptance_requirements(BTreeMap::from([(
                    "semantic".into(),
                    private.into(),
                )]))?;
                let status = e.project_acceptance_status()?;
                e.project_acceptance_feedback(acceptance::Feedback {
                    requirements_revision: status.requirements_revision.unwrap(),
                    decision_record: status.decision_record,
                    guidance: BTreeMap::from([("semantic".into(), public.into())]),
                })?;
            }
            let root = e.store.head_root()?;
            let history = e.project_published_guidance_history()?;
            let publications = history["publications"].as_array().unwrap();
            assert_eq!(publications.len(), 2);
            assert_eq!(publications[0]["current_revision"], false);
            assert_eq!(publications[0]["contributors"][0]["eligible"], false);
            assert_eq!(
                publications[0]["contributors"][0]["meaning"],
                "Published first"
            );
            assert_eq!(
                publications[1]["contributors"][0]["meaning"],
                "Published second"
            );
            assert_ne!(
                publications[0]["contributors"][0]["id"],
                publications[1]["contributors"][0]["id"]
            );
            assert!(!history.to_string().contains("PRIVATE"));
            assert_eq!(e.store.head_root()?, root);
            if forged == 1 {
                e.adaptive_record(
                    &Record::Coding {
                        record: CodingRecord::AcceptanceGuidanceScope {
                            publication: serde_json::from_value(
                                publications[1]["publication"].clone(),
                            )?,
                            scope: Hash::of(&[&serde_json::to_vec(&e.project_configuration()?)?]),
                            snapshot: e.project_acceptance_status()?.snapshot,
                        },
                    },
                    Source::Generator,
                )?;
            } else if forged == 2 {
                e.adaptive_record(
                    &Record::Coding {
                        record: CodingRecord::AcceptanceAims {
                            revision: e
                                .project_acceptance_status()?
                                .requirements_revision
                                .unwrap(),
                            preferences: e.acceptance_current_aims()?,
                        },
                    },
                    Source::Generator,
                )?;
            }
            if forged != 0 {
                assert!(e.project_published_guidance_history().is_err());
            }
        }
        Ok(())
    }
    #[test]
    fn published_guidance_history_rejects_forged_publication() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "semantic".into(),
            "PRIVATE criterion".into(),
        )]))?;
        let status = e.project_acceptance_status()?;
        e.adaptive_record(
            &Record::Coding {
                record: CodingRecord::AcceptanceFeedback {
                    feedback: acceptance::Feedback {
                        requirements_revision: status.requirements_revision.unwrap(),
                        decision_record: None,
                        guidance: BTreeMap::from([(
                            "semantic".into(),
                            "Forged publication".into(),
                        )]),
                    },
                },
            },
            Source::Generator,
        )?;
        let root = e.store.head_root()?;
        assert!(e.project_published_guidance_history().is_err());
        assert_eq!(e.store.head_root()?, root);
        Ok(())
    }
    #[test]
    fn guidance_changed_intent_and_withdrawal_fail_closed() -> Result<()> {
        use acceptance::{Decision, Evidence, Feedback, Review};
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "semantic".into(),
            "Original criterion".into(),
        )]))?;
        let c = e.project_acceptance_status()?;
        e.project_acceptance_feedback(Feedback {
            requirements_revision: c.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([("semantic".into(), "Original published intent".into())]),
        })?;
        let aim = e.acceptance_current_aims()?["semantic"];
        e.append(
            EventKind::StatusSet {
                preference: aim,
                status: PreferenceStatus::Withdrawn,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        e.project_acceptance_review(Review {
            snapshot: c.snapshot,
            requirements_revision: c.requirements_revision.unwrap(),
            test_receipt: None,
            decision: Decision::Reject,
            evidence: BTreeMap::from([(
                "semantic".into(),
                Evidence {
                    met: false,
                    evidence: "Still unmet, not permission to revive".into(),
                },
            )]),
        })?;
        assert!(e.acceptance_cycle()?["guidance"]
            .as_object()
            .unwrap()
            .is_empty());
        assert!(Ledger::new(&e.store)
            .collective()?
            .entries
            .iter()
            .any(|p| p.preference.id == aim && p.status == Some(PreferenceStatus::Withdrawn)));
        e.acceptance_reopen_aims()?;
        assert!(Ledger::new(&e.store)
            .collective()?
            .entries
            .iter()
            .any(|p| p.preference.id == aim && p.status == Some(PreferenceStatus::Withdrawn)));
        e.project_acceptance_requirements(BTreeMap::from([(
            "semantic".into(),
            "Changed criterion".into(),
        )]))?;
        assert_eq!(
            e.acceptance_cycle()?["guidance_provenance"]["status"],
            "intent_or_scope_changed"
        );
        assert!(e.acceptance_cycle()?["guidance"]
            .as_object()
            .unwrap()
            .is_empty());
        Ok(())
    }
    #[test]
    fn guidance_scope_budget_expiry_and_legacy_are_not_admissions() -> Result<()> {
        use acceptance::{Decision, Evidence, Feedback, Review};
        for mode in ["legacy", "scope", "exhausted", "expired"] {
            let dir = tempfile::tempdir()?;
            let e = Engine::open(dir.path())?;
            e.project_initialize(spec())?;
            e.project_acceptance_requirements(BTreeMap::from([(
                "semantic".into(),
                "Criterion".into(),
            )]))?;
            e.project_lifecycle_open(lifecycle::Admission {
                training: None,
                state_root: e.store.head_root()?,
                tool_ms_ceiling: 1000,
                reason: "synthetic bounds".into(),
                non_tool: Some(budget::Limits {
                    model_calls: 2,
                    model_tokens: 1000,
                    wall_ms: if mode == "expired" { 1 } else { 600_000 },
                    patch_bytes: 100_000,
                    external_cost_microusd: 0,
                }),
            })?;
            let c = e.project_acceptance_status()?;
            let feedback = Feedback {
                requirements_revision: c.requirements_revision.unwrap(),
                decision_record: None,
                guidance: BTreeMap::from([("semantic".into(), "Published intent".into())]),
            };
            if mode == "legacy" {
                e.adaptive_record(
                    &Record::Coding {
                        record: CodingRecord::AcceptanceFeedback { feedback },
                    },
                    Source::Operator,
                )?;
            } else {
                e.project_acceptance_feedback(feedback)?;
            }
            e.project_acceptance_review(Review {
                snapshot: c.snapshot,
                requirements_revision: c.requirements_revision.unwrap(),
                test_receipt: None,
                decision: Decision::Reject,
                evidence: BTreeMap::from([(
                    "semantic".into(),
                    Evidence {
                        met: false,
                        evidence: "Independent rejection".into(),
                    },
                )]),
            })?;
            if mode == "scope" {
                // Configuration is immutable; a mismatched purported scope binding fails closed.
                let publication = serde_json::from_value(
                    e.acceptance_cycle()?["guidance_provenance"]["publication"].clone(),
                )?;
                e.adaptive_record(
                    &Record::Coding {
                        record: CodingRecord::AcceptanceGuidanceScope {
                            publication,
                            scope: Hash::of(&[b"different-scope"]),
                            snapshot: c.snapshot,
                        },
                    },
                    Source::Operator,
                )?;
            } else if mode == "exhausted" {
                e.reserve_project_preparation(Hash::of(&[b"synthetic-reservation"]), 1000)?;
            } else if mode == "expired" {
                std::thread::sleep(std::time::Duration::from_millis(2));
            }
            let cycle = e.acceptance_cycle()?;
            assert!(cycle["guidance"].as_object().unwrap().is_empty(), "{mode}");
            assert_eq!(cycle["repair_ready"], false, "{mode}");
        }
        Ok(())
    }
    #[test]
    fn next_attention_preserves_residuals_withdrawal_restart_and_no_authority() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        let before = e.store.head_root()?;
        let initial = e.project_attention()?;
        assert!(!initial["selected_preference"].is_null());
        assert_eq!(initial["dispatch_authorized"], false);
        assert_eq!(before, e.store.head_root()?);
        assert_eq!(initial, Engine::open(dir.path())?.project_attention()?);
        let selected: PreferenceId =
            serde_json::from_value(initial["selected_preference"].clone())?;
        e.append(
            EventKind::StatusSet {
                preference: selected,
                status: PreferenceStatus::Withdrawn,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        let withdrawn = e.project_attention()?;
        assert_ne!(
            initial["selected_preference"],
            withdrawn["selected_preference"]
        );
        e.append(
            EventKind::StatusSet {
                preference: selected,
                status: PreferenceStatus::Open,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        assert_eq!(
            initial["selected_preference"],
            e.project_attention()?["selected_preference"]
        );
        for entry in e
            .adaptive_state_inner()?
            .preferences
            .into_iter()
            .filter(|p| p.is_open())
        {
            e.append(
                EventKind::StatusSet {
                    preference: entry.preference.id,
                    status: PreferenceStatus::OutOfScope,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
        }
        assert!(e.project_attention()?["selected_preference"].is_null());
        Ok(())
    }
    #[test]
    fn next_attention_uses_current_returns_and_retains_unknown_semantic_desire() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "semantic".into(),
            "Independent semantic obligation".into(),
        )]))?;
        let proposal = offer(
            &e,
            vec![patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "unit_cents * quantity",
            )?],
        )?;
        e.evaluate_proposal(proposal, true)?;
        assert!(e.project_attention()?["selected_preference"].is_null()); // pending commitment
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        let attention = e.project_attention()?;
        let rows = attention["desires"].as_array().unwrap();
        assert!(rows
            .iter()
            .any(|r| r["exclusion"] == "currently_observed_satisfied"));
        let semantic = e.acceptance_current_aims()?["semantic"];
        assert_eq!(
            attention["selected_preference"],
            serde_json::to_value(semantic)?
        );
        assert_eq!(attention["dispatch_authorized"], false);
        fs::write(
            dir.path().join("coding-work/src/pricing.rs"),
            "// new evidence invalidates old returns\n",
        )?;
        let stale = e.project_attention()?;
        assert!(stale["desires"]
            .as_array()
            .unwrap()
            .iter()
            .all(|r| r["observed"].is_null()));
        Ok(())
    }
    #[test]
    fn next_attention_zero_inference_is_not_zero_tool_capacity() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 1,
            reason: "test verification-only budget".into(),
            non_tool: Some(budget::Limits {
                model_calls: 0,
                model_tokens: 0,
                wall_ms: 600_000,
                patch_bytes: 0,
                external_cost_microusd: 0,
            }),
        })?;
        let a = e.project_attention()?;
        assert_eq!(a["resource_capacity"]["model_calls_available"], false);
        assert_eq!(a["resource_capacity"]["tool_time_available"], true);
        assert!(a["continuation_blockers"].as_array().unwrap().is_empty());
        assert!(!a["selected_preference"].is_null());
        assert_eq!(a["dispatch_authorized"], false);
        e.reserve_project_preparation(Hash::of(&[b"fixture reservation only"]), 1)?;
        let capped = e.project_attention()?;
        assert_eq!(capped["resource_capacity"]["tool_time_available"], false);
        assert!(capped["continuation_blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b == "unresolved_resource_reservations"));
        assert!(!capped["selected_preference"].is_null());
        Ok(())
    }
    #[test]
    fn next_attention_closure_preserves_residual_and_requires_admission() -> Result<()> {
        use acceptance::{Decision, Evidence, Review};
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "semantic".into(),
            "Unresolved semantic requirement".into(),
        )]))?;
        e.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 86_400_000,
            reason: "deterministic fixture".into(),
            non_tool: None,
        })?;
        e.evaluate_proposal(
            offer(
                &e,
                vec![patch(
                    &e,
                    "src/pricing.rs",
                    "unit_cents + quantity",
                    "unit_cents * quantity",
                )?],
            )?,
            true,
        )?;
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_review(Review {
            snapshot: status.snapshot,
            requirements_revision: status.requirements_revision.unwrap(),
            test_receipt: status.test_receipt,
            decision: Decision::Reject,
            evidence: BTreeMap::from([(
                "semantic".into(),
                Evidence {
                    met: false,
                    evidence: "independent fixture rejection".into(),
                },
            )]),
        })?;
        e.project_lifecycle_close()?;
        let a = e.project_attention()?;
        assert_eq!(a["episode_closed"], true);
        assert!(!a["selected_preference"].is_null());
        assert!(a["continuation_blockers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| b == "closed_episode_requires_fresh_admission"));
        assert_eq!(a, Engine::open(dir.path())?.project_attention()?);
        Ok(())
    }
    #[test]
    fn next_attention_latest_same_source_evidence_supersedes_dead_end() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.evaluate_proposal(
            offer(
                &e,
                vec![patch(
                    &e,
                    "src/pricing.rs",
                    "unit_cents + quantity",
                    "unit_cents * quantity",
                )?],
            )?,
            true,
        )?;
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: false },
        )?;
        let mut receipt = e
            .records()?
            .into_iter()
            .rev()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record: r @ CodingRecord::Receipt { .. },
                } => Some(r),
                _ => None,
            })
            .unwrap();
        // Synthetic diagnostic evidence only: no production outcome or acceptance claim.
        if let CodingRecord::Receipt {
            verdict,
            source_after,
            ..
        } = &mut receipt
        {
            *verdict = Verdict::FailedTests;
            *source_after = Some(e.project_acceptance_status()?.snapshot);
        }
        e.adaptive_record(
            &Record::Coding {
                record: receipt.clone(),
            },
            Source::Rules,
        )?;
        assert_eq!(
            e.project_attention()?["blocked_exact_actions"]
                .as_object()
                .unwrap()
                .len(),
            1
        );
        if let CodingRecord::Receipt { verdict, .. } = &mut receipt {
            *verdict = Verdict::Passed;
        }
        e.adaptive_record(&Record::Coding { record: receipt }, Source::Rules)?;
        assert!(e.project_attention()?["blocked_exact_actions"]
            .as_object()
            .unwrap()
            .is_empty());
        Ok(())
    }
    #[test]
    fn proposal_attention_binding_rejects_drift_replay_and_withdrawal() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        let response = offer(
            &e,
            vec![patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "unit_cents * quantity",
            )?],
        )?;
        let root = e.store.head_root()?;
        let mut forged = response.clone();
        forged.attention_binding = Some(Hash::of(&[b"foreign"]));
        assert!(e.evaluate_proposal(forged, true).is_err());
        let mut missing = response.clone();
        missing.attention_binding = None;
        assert!(e.evaluate_proposal(missing, true).is_err());
        assert_eq!(root, e.store.head_root()?);
        let path = dir.path().join("coding-work/src/pricing.rs");
        let source = fs::read_to_string(&path)?;
        fs::write(&path, format!("{source}\n// independent source revision\n"))?;
        assert!(e.evaluate_proposal(response.clone(), true).is_err());
        assert_eq!(root, e.store.head_root()?);
        fs::write(path, source)?;
        let selected: PreferenceId =
            serde_json::from_value(e.project_attention()?["selected_preference"].clone())?;
        e.append(
            EventKind::StatusSet {
                preference: selected,
                status: PreferenceStatus::Withdrawn,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        assert!(e.evaluate_proposal(response, true).is_err());
        let fresh = offer(
            &e,
            vec![patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "unit_cents * quantity",
            )?],
        )?;
        e.evaluate_proposal(fresh.clone(), true)?;
        assert!(e.evaluate_proposal(fresh, true).is_err());
        assert_eq!(
            e.records()?
                .iter()
                .filter(|(_, r)| matches!(r, Record::AttentionConsumed { .. }))
                .count(),
            1
        );
        Ok(())
    }
    #[test]
    fn proposal_attention_is_allowlisted_and_changes_after_observation() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        let before = e.proposal_request()?;
        assert_eq!(
            before.next_attention,
            Engine::open(dir.path())?.proposal_request()?.next_attention
        );
        let proposal = offer(
            &e,
            vec![patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "unit_cents * quantity",
            )?],
        )?;
        e.evaluate_proposal(proposal, true)?;
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        let after = e.proposal_request()?;
        assert_ne!(
            before.next_attention.as_ref().unwrap()["focus"],
            after.next_attention.as_ref().unwrap()["focus"]
        );
        assert_eq!(
            after.next_attention.as_ref().unwrap()["focus"]["kind"],
            "no_eligible_attention"
        );
        e.project_acceptance_requirements(BTreeMap::from([(
            "private_key".into(),
            "PRIVATE_REVIEW_CONTENT_123".into(),
        )]))?;
        let after = e.proposal_request()?;
        assert_eq!(
            after.next_attention.as_ref().unwrap()["focus"]["kind"],
            "operator_guidance_required"
        );
        let wire = serde_json::to_string(&after)?;
        assert!(!wire.contains("PRIVATE_REVIEW_CONTENT_123"));
        assert!(!serde_json::to_string(&after.next_attention)?.contains("private_key"));
        assert!(!serde_json::to_string(&after.next_attention)?.contains("probability_proxy"));
        let root = e.store.head_root()?;
        assert_eq!(root, Engine::open(dir.path())?.store.head_root()?);
        Ok(())
    }
    #[test]
    fn proposal_attention_survives_own_measured_reservation_but_blocks_uncertainty() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 86_400_000,
            reason: "synthetic request boundary".into(),
            non_tool: Some(budget::Limits {
                model_calls: 1,
                model_tokens: 1000,
                wall_ms: 600_000,
                patch_bytes: 10000,
                external_cost_microusd: 0,
            }),
        })?;
        let proposal = offer(
            &e,
            vec![patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "unit_cents * quantity",
            )?],
        )?;
        e.project_resource_reserve(budget::Reserve {
            request_id: "fixture".into(),
            state_root: e.store.head_root()?,
            envelope: "fixture".into(),
            model: "no model called".into(),
            local: true,
            tokens_upper: 100,
            external_cost_upper: 0,
        })?;
        assert!(e.evaluate_proposal(proposal.clone(), true).is_err());
        e.project_resource_settle(budget::Settle {
            request_id: "fixture".into(),
            prompt_tokens: Some(10),
            output_tokens: Some(10),
            duration_ns: Some(1),
            external_cost_microusd: Some(0),
            outcome: "measured".into(),
        })?;
        assert_eq!(proposal.attention_binding, e.proposal_attention_binding()?);
        e.evaluate_proposal(proposal, true)?;
        Ok(())
    }
    #[test]
    fn proposal_attention_residual_survives_accepted_and_rejected_closure() -> Result<()> {
        use acceptance::{Decision, Evidence, Review};
        for accept in [true, false] {
            let dir = tempfile::tempdir()?;
            let e = Engine::open(dir.path())?;
            e.project_initialize(spec())?;
            e.project_acceptance_requirements(BTreeMap::from([(
                "semantic".into(),
                "Independent task contract".into(),
            )]))?;
            e.project_lifecycle_open(lifecycle::Admission {
                training: None,
                state_root: e.store.head_root()?,
                tool_ms_ceiling: 86_400_000,
                reason: "synthetic closure".into(),
                non_tool: None,
            })?;
            e.evaluate_proposal(
                offer(
                    &e,
                    vec![patch(
                        &e,
                        "src/pricing.rs",
                        "unit_cents + quantity",
                        "unit_cents * quantity",
                    )?],
                )?,
                true,
            )?;
            let plan = e.coding_plan()?;
            e.coding_execute(
                plan.commitment,
                plan.source_before,
                &Fake { complete: true },
            )?;
            let status = e.project_acceptance_status()?;
            e.project_acceptance_review(Review {
                snapshot: status.snapshot,
                requirements_revision: status.requirements_revision.unwrap(),
                test_receipt: status.test_receipt,
                decision: if accept {
                    Decision::Accept
                } else {
                    Decision::Reject
                },
                evidence: BTreeMap::from([(
                    "semantic".into(),
                    Evidence {
                        met: accept,
                        evidence: "independent fixture review only".into(),
                    },
                )]),
            })?;
            // Separately admitted standing desire, not another acceptance criterion.
            let evidence = e.project_acceptance_status()?.decision_record.unwrap();
            e.adaptive_want(
                "standing_followup",
                "Standing operator followup",
                vec![],
                evidence,
                Source::Operator,
            )?;
            let response = offer(
                &e,
                vec![patch(
                    &e,
                    "src/pricing.rs",
                    "unit_cents * quantity",
                    "unit_cents * quantity + 0",
                )?],
            )?;
            e.project_lifecycle_close()?;
            assert!(e.evaluate_proposal(response, true).is_err());
            let report = Engine::open(dir.path())?.project_attention()?;
            assert_eq!(report["task_accepted"], accept);
            assert!(report["desires"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["exclusion"].is_null()));
            assert!(!report["selected_preference"].is_null());
            assert!(e
                .validate_proposal_attention(e.proposal_attention_binding()?)
                .is_err());
        }
        Ok(())
    }
    fn patch(e: &Engine, path: &str, old: &str, new: &str) -> Result<ExactPatch> {
        let spec = e.project_configuration()?;
        let snapshot = e.project_snapshot(&spec)?;
        Ok(ExactPatch {
            path: path.into(),
            source_digest: Hash::of(&[snapshot[path].as_bytes()]),
            inspection_handle: e.project_handle(&spec, &snapshot, path)?,
            old_text: old.into(),
            new_text: new.into(),
        })
    }
    fn offer(e: &Engine, patches: Vec<ExactPatch>) -> Result<super::super::provider::Response> {
        Ok(super::super::provider::Response {
            attention_binding: e.proposal_attention_binding()?,
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
            protocol: super::super::provider::PROTOCOL.into(),
            state_root: e.store.head_root()?,
            proposals: vec![super::super::provider::Suggestion::EditSet {
                patches,
                rationale: "repair admitted files".into(),
            }],
        })
    }
    struct Fake {
        complete: bool,
    }
    impl Runner for Fake {
        fn run(&self, copy: &Path, argv: &[String], _: &Limits) -> RunResult {
            match argv[0].as_str() {
                "sh" => {
                    for pair in argv[4..].chunks_exact(2) {
                        fs::write(copy.join(&pair[0]), &pair[1]).unwrap();
                    }
                    RunResult::exit_only(0)
                }
                "rustc" => RunResult::exit_only(0),
                "./fixture-probe" => {
                    let output = if argv[1] == "subtotal" {
                        "300\n"
                    } else if argv[4] == "30" {
                        if self.complete {
                            "270\n"
                        } else {
                            "330\n"
                        }
                    } else if self.complete {
                        "0\n"
                    } else {
                        "150\n"
                    };
                    RunResult::from_output(0, output.as_bytes(), b"", 1)
                }
                _ => panic!("unexpected command"),
            }
        }
    }
    #[test]
    fn edit_windows_preserve_inline_test_bytes_and_bound_context() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut s = spec();
        let body = "unit_cents + quantity";
        let original = format!("{}pub fn subtotal(unit_cents:u32, quantity:u32)->u32 {{ {} }}\n#[cfg(test)] mod tests {{ #[test] fn stays() {{ assert_eq!(2+2,4); }} }}\n", "// protected preface\n".repeat(300), body);
        let start = original.find(body).unwrap();
        s.editable.insert("src/pricing.rs".into(), original.clone());
        s.edit_windows.insert(
            "src/pricing.rs".into(),
            EditWindow {
                start,
                end: start + body.len(),
            },
        );
        // Large protected assets must not consume proposal-context bytes.
        s.protected
            .insert("Cargo.lock".into(), "# owner lock bytes\n".repeat(6000));
        s.protected.insert(
            "tests/large.rs".into(),
            "// protected test bytes\n".repeat(1000),
        );
        e.project_initialize(s.clone())?;
        let request = e.proposal_request()?;
        let context = request.coding.unwrap();
        let files = context["files"].as_array().unwrap();
        let source = files
            .iter()
            .find(|f| f["path"] == "src/pricing.rs")
            .unwrap();
        assert_eq!(source["source"], body);
        assert_eq!(
            source["source_digest"],
            serde_json::json!(Hash::of(&[original.as_bytes()]))
        );
        let lock = files.iter().find(|f| f["path"] == "Cargo.lock").unwrap();
        assert!(lock.get("source").is_none());
        assert!(serde_json::to_vec(&context)?.len() < 8192);
        let good = patch(&e, "src/pricing.rs", body, "unit_cents * quantity")?;
        let replacement = e.validate_patch(&good)?;
        assert!(replacement.starts_with(&original[..start]));
        assert!(replacement.ends_with(&original[start + body.len()..]));
        for (old, new) in [
            ("assert_eq!(2+2,4)", "assert_eq!(2+2,5)"),
            ("#[cfg(test)]", "#[cfg(any())]"),
            ("pub fn subtotal", "pub fn renamed"),
        ] {
            let denied = patch(&e, "src/pricing.rs", old, new)?;
            assert!(e.validate_patch(&denied).is_err());
        }
        fs::write(dir.path().join("coding-work/src/pricing.rs"), &replacement)?;
        drop(e);
        let e = Engine::open(dir.path())?;
        assert!(e.project_snapshot(&s).is_ok());
        assert!(e.validate_patch(&good).is_err()); // Entire-file handle is stale after an in-window edit.
        e.project_export()?;
        fs::write(
            dir.path().join("coding-work/src/pricing.rs"),
            replacement.replace("assert_eq!(2+2,4)", "assert_eq!(2+2,5)"),
        )?;
        assert!(e.project_snapshot(&s).is_err());
        fs::write(dir.path().join("coding-work/src/pricing.rs"), replacement)?;
        fs::write(dir.path().join("coding-work/Cargo.lock"), "tampered")?;
        assert!(e.project_snapshot(&s).is_err());
        Ok(())
    }
    #[test]
    fn window_and_snapshot_limits_are_enforced() -> Result<()> {
        let mut s = spec();
        s.edit_windows
            .insert("tests/probe.rs".into(), EditWindow { start: 0, end: 1 });
        assert!(s.validate().is_err());
        s.edit_windows.clear();
        s.editable.insert("src/pricing.rs".into(), "é".into());
        s.edit_windows
            .insert("src/pricing.rs".into(), EditWindow { start: 1, end: 2 });
        assert!(s.validate().is_err());
        s.edit_windows
            .insert("src/pricing.rs".into(), EditWindow { start: 0, end: 999 });
        assert!(s.validate().is_err());
        s.edit_windows.clear();
        s.protected
            .insert("Cargo.lock".into(), "x".repeat(256 * 1024 + 1));
        assert!(s.validate().is_err());
        Ok(())
    }
    #[test]
    fn namespaced_tests_are_admitted_without_command_authority() -> Result<()> {
        let mut s = spec();
        s.cases[0].id = "validation::tests::valid_e164".into();
        s.validate()?;
        for name in [
            "::check",
            "check::",
            "check:::other",
            "check --ignored",
            "../check",
        ] {
            s.cases[0].id = name.into();
            assert!(s.validate().is_err());
        }
        Ok(())
    }
    #[test]
    fn qualified_test_names_keep_bounded_literal_identity() -> Result<()> {
        let mut s = spec();
        s.cases[0].id =
            "surface::tests::a_named_list_is_exact_and_an_unknown_name_is_an_error".into();
        s.validate()?;
        s.cases[0].id = "x".repeat(256);
        s.validate()?;
        s.cases[0].id.push('x');
        assert!(s.validate().is_err());
        s.cases[0].id = "surface::tests::valid --ignored".into();
        assert!(s.validate().is_err());
        Ok(())
    }
    fn supporting_scope() -> ProjectSpec {
        let mut s = spec();
        let path = "src/pricing.rs";
        let original = format!(
            "{}const LIMIT: u32 = 100;\n#[cfg(test)] mod tests {{}}\n",
            s.editable[path]
        );
        let start = original.find("unit_cents + quantity").unwrap();
        s.edit_windows.insert(
            path.into(),
            EditWindow {
                start,
                end: start + "unit_cents + quantity".len(),
            },
        );
        let start = original.find("const LIMIT").unwrap();
        s.supporting_spans.insert(
            path.into(),
            vec![EditWindow {
                start,
                end: start + "const LIMIT: u32 = 100;".len(),
            }],
        );
        s.editable.insert(path.into(), original);
        s
    }
    #[test]
    fn supporting_context_survives_resize_restart_without_edit_authority() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let s = supporting_scope();
        e.project_initialize(s.clone())?;
        let first = e.project_context(&s)?;
        let support = &first["supporting_context"][0];
        assert_eq!(support["source"], "const LIMIT: u32 = 100;");
        assert_eq!(support["editable"], false);
        assert!(support.get("inspection_handle").is_none());
        let p = patch(
            &e,
            "src/pricing.rs",
            "unit_cents + quantity",
            "unit_cents.saturating_mul(quantity)",
        )?;
        let replacement = e.validate_patch(&p)?;
        fs::write(dir.path().join("coding-work/src/pricing.rs"), replacement)?;
        drop(e);
        let e = Engine::open(dir.path())?;
        let current = e.project_context(&e.project_configuration()?)?;
        assert_eq!(
            support["source"],
            current["supporting_context"][0]["source"]
        );
        assert_eq!(
            support["span_digest"],
            current["supporting_context"][0]["span_digest"]
        );
        assert_ne!(
            support["source_digest"],
            current["supporting_context"][0]["source_digest"]
        );
        assert_ne!(
            support["current_start"],
            current["supporting_context"][0]["current_start"]
        );
        assert_eq!(current, e.project_context(&s)?); // unchanged control
        assert!(e.validate_patch(&p).is_err()); // old whole-file binding is stale
        let forbidden = patch(
            &e,
            "src/pricing.rs",
            "const LIMIT: u32 = 100;",
            "const LIMIT: u32 = 0;",
        )?;
        assert!(e.validate_patch(&forbidden).is_err());
        let target = dir.path().join("coding-work/src/pricing.rs");
        fs::write(
            &target,
            fs::read_to_string(&target)?.replace("LIMIT: u32 = 100", "LIMIT: u32 = 101"),
        )?;
        assert!(e.project_context(&s).is_err());
        Ok(())
    }
    #[test]
    fn supporting_admission_rejects_overlap_unknown_tests_and_bad_bounds() -> Result<()> {
        let good = supporting_scope();
        good.validate()?;
        assert!(serde_json::to_value(spec())?
            .get("supporting_spans")
            .is_none());
        let mut s = good.clone();
        s.supporting_spans
            .insert("missing.rs".into(), vec![EditWindow { start: 0, end: 1 }]);
        assert!(s.validate().is_err());
        let mut s = good.clone();
        s.supporting_spans.insert(
            "tests/probe.rs".into(),
            vec![EditWindow { start: 0, end: 1 }],
        );
        assert!(s.validate().is_err());
        let path = "src/pricing.rs";
        for spans in [
            vec![good.edit_windows[path].clone()],
            vec![EditWindow {
                start: 0,
                end: usize::MAX,
            }],
            vec![EditWindow { start: 1, end: 0 }],
            vec![good.supporting_spans[path][0].clone(); 9],
            vec![good.supporting_spans[path][0].clone(); 2],
            vec![EditWindow {
                start: good.editable[path].find("#[cfg(test)]").unwrap(),
                end: good.editable[path].len(),
            }],
        ] {
            let mut s = good.clone();
            s.supporting_spans.insert(path.into(), spans);
            assert!(s.validate().is_err());
        }
        let mut s = good.clone();
        s.protected.insert("src/reference.rs".into(), "é".into());
        s.supporting_spans.insert(
            "src/reference.rs".into(),
            vec![EditWindow { start: 1, end: 2 }],
        );
        assert!(s.validate().is_err());
        s.protected
            .insert("src/reference.rs".into(), "x".repeat(2049));
        s.supporting_spans.insert(
            "src/reference.rs".into(),
            vec![EditWindow {
                start: 0,
                end: 2049,
            }],
        );
        assert!(s.validate().is_err());
        Ok(())
    }
    #[test]
    fn project_status_tracks_pending_and_changed_sources() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        assert_eq!(e.project_status()?["status"], "ready_for_proposal");
        let p = patch(
            &e,
            "src/pricing.rs",
            "unit_cents + quantity",
            "unit_cents * quantity",
        )?;
        e.evaluate_proposal(offer(&e, vec![p])?, true)?;
        assert_eq!(e.project_status()?["status"], "committed_not_executed");
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: false },
        )?;
        assert_eq!(e.project_status()?["status"], "needs_repair");
        fs::write(dir.path().join("coding-work/src/pricing.rs"), "changed")?;
        assert_eq!(
            e.project_status()?["status"],
            "source_changed_since_execution"
        );
        Ok(())
    }
    #[test]
    fn partial_project_return_preserves_named_contrast_and_restart() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        let p = patch(
            &e,
            "src/pricing.rs",
            "unit_cents + quantity",
            "unit_cents * quantity",
        )?;
        let stale_discount = patch(
            &e,
            "src/discount.rs",
            "subtotal_cents + discount_cents",
            "subtotal_cents.saturating_sub(discount_cents)",
        )?;
        e.evaluate_proposal(offer(&e, vec![p])?, true)?;
        let plan = e.coding_plan()?;
        assert_eq!(plan.commands.len(), 5);
        let result = e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: false },
        )?;
        assert_eq!(result.verdict, Verdict::FailedTests);
        assert!(e.validate_patch(&stale_discount).is_err()); // Another admitted file changed.
        drop(e);
        let e = Engine::open(dir.path())?;
        let request = e.proposal_request()?;
        assert_eq!(
            request.prior_outcomes[0]["met"],
            serde_json::json!(["test:subtotal"])
        );
        let observations =
            &request.coding.as_ref().unwrap()["recent_observations"][0]["observations"];
        assert_eq!(observations.as_array().unwrap().len(), 3);
        assert_eq!(observations[1]["case_id"], "discount");
        assert_eq!(observations[1]["stdout"], "330\n");
        let p = patch(
            &e,
            "src/discount.rs",
            "subtotal_cents + discount_cents",
            "subtotal_cents.saturating_sub(discount_cents)",
        )?;
        e.evaluate_proposal(offer(&e, vec![p])?, true)?;
        let plan = e.coding_plan()?;
        assert_eq!(
            e.coding_execute(
                plan.commitment,
                plan.source_before,
                &Fake { complete: true }
            )?
            .verdict,
            Verdict::Passed
        );
        let state = Engine::open(dir.path())?.adaptive_state()?;
        assert_eq!(state.options[0].prediction.value, 1.0 / 3.0);
        assert_eq!(state.options[1].prediction.value, 2.0 / 3.0);
        assert_eq!(
            fs::read_to_string(dir.path().join("coding-work/tests/probe.rs"))?,
            spec().protected["tests/probe.rs"]
        );
        Ok(())
    }
    #[test]
    fn novel_bundle_can_fix_both_files_in_one_attempt() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        let a = patch(
            &e,
            "src/pricing.rs",
            "unit_cents + quantity",
            "unit_cents * quantity",
        )?;
        let b = patch(
            &e,
            "src/discount.rs",
            "subtotal_cents + discount_cents",
            "subtotal_cents.saturating_sub(discount_cents)",
        )?;
        assert_eq!(
            e.validate_patch_set(&[a.clone(), b.clone()])?,
            e.validate_patch_set(&[b.clone(), a.clone()])?
        );
        assert!(e.validate_patch_set(&[a.clone(), a.clone()]).is_err());
        e.evaluate_proposal(offer(&e, vec![a, b])?, true)?;
        let plan = e.coding_plan()?;
        assert_eq!(plan.edits.len(), 2);
        assert_eq!(
            e.coding_execute(
                plan.commitment,
                plan.source_before,
                &Fake { complete: true }
            )?
            .verdict,
            Verdict::Passed
        );
        assert_eq!(e.adaptive_state()?.completed, 1);
        Ok(())
    }
    #[test]
    fn budget_limits_block_before_effect_and_measure_local_resources() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        let root = e.store.head_root()?;
        e.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: root,
            tool_ms_ceiling: 86_400_000,
            reason: "bounded fixture".into(),
            non_tool: Some(budget::Limits {
                model_calls: 2,
                model_tokens: 100,
                wall_ms: 600_000,
                patch_bytes: 1,
                external_cost_microusd: 0,
            }),
        })?;
        let r = budget::Reserve {
            request_id: "measured".into(),
            state_root: root,
            envelope: "sha256:fixture".into(),
            model: "installed".into(),
            local: true,
            tokens_upper: 100,
            external_cost_upper: 0,
        };
        e.project_resource_reserve(r)?;
        let measured = budget::Settle {
            request_id: "measured".into(),
            prompt_tokens: Some(20),
            output_tokens: Some(10),
            duration_ns: Some(500),
            external_cost_microusd: Some(0),
            outcome: "measured".into(),
        };
        e.project_resource_settle(measured.clone())?;
        let v = e.project_budget()?;
        assert_eq!(v.tokens_charged, 30);
        assert_eq!(v.external_cost_charged_microusd, 0);
        assert_eq!(v.measured_local_duration_ns, 500);
        let mut rewritten = measured;
        rewritten.prompt_tokens = Some(0);
        assert!(e.project_resource_settle(rewritten).is_err());
        let a = patch(
            &e,
            "src/pricing.rs",
            "unit_cents + quantity",
            "unit_cents * quantity",
        )?;
        e.evaluate_proposal(offer(&e, vec![a])?, true)?;
        let plan = e.coding_plan()?;
        let root = e.store.head_root()?;
        assert!(e
            .coding_execute(
                plan.commitment,
                plan.source_before,
                &Fake { complete: true }
            )
            .is_err());
        assert_eq!(e.store.head_root()?, root);
        assert_eq!(e.project_budget()?.patch_bytes_charged, 0);
        assert_eq!(
            digest(&e.project_snapshot(&e.project_configuration()?)?)?,
            plan.source_before
        );
        Ok(())
    }
    #[test]
    fn wall_budget_and_feature_cutoffs_are_explicit() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        let root = e.store.head_root()?;
        e.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: root,
            tool_ms_ceiling: 86_400_000,
            reason: "wall boundary fixture".into(),
            non_tool: Some(budget::Limits {
                model_calls: 1,
                model_tokens: 100,
                wall_ms: 1,
                patch_bytes: 100,
                external_cost_microusd: 0,
            }),
        })?;
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(e.project_budget()?.stopped);
        assert!(e
            .project_resource_reserve(budget::Reserve {
                request_id: "late".into(),
                state_root: root,
                envelope: "sha256:fixture".into(),
                model: "installed".into(),
                local: true,
                tokens_upper: 10,
                external_cost_upper: 0
            })
            .is_err());
        assert!(e
            .project_receipt_features(&e.events()?, "1970-01-01T00:00:00Z", observations::SCHEMA)
            .is_err());
        assert_eq!(e.project_budget()?.model_calls, 0);
        Ok(())
    }
    #[test]
    fn compatible_cost_receipts_change_ranking_without_changing_beliefs() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 1_000_000,
            reason: "cost-only causal fixture".into(),
            non_tool: Some(budget::Limits {
                model_calls: 2,
                model_tokens: 1000,
                wall_ms: 600_000,
                patch_bytes: 100_000,
                external_cost_microusd: 0,
            }),
        })?;
        let first = patch(
            &e,
            "src/pricing.rs",
            "unit_cents + quantity",
            "unit_cents * quantity",
        )?;
        e.evaluate_proposal(offer(&e, vec![first])?, true)?;
        let p = e.coding_plan()?;
        e.coding_execute(p.commitment, p.source_before, &Fake { complete: false })?;
        let cheap = patch(
            &e,
            "src/pricing.rs",
            "unit_cents * quantity",
            "quantity * unit_cents",
        )?;
        let unknown = patch(
            &e,
            "src/discount.rs",
            "subtotal_cents + discount_cents",
            "subtotal_cents.saturating_sub(discount_cents)",
        )?;
        let a = e.register_patch_set(&[cheap])?;
        let b = e.register_patch_set(&[unknown])?;
        let rank = |engine: &Engine| -> Result<Vec<OptionView>> {
            let state = engine.adaptive_state()?;
            let records = engine.records()?;
            let config = records
                .iter()
                .find_map(|(_, r)| {
                    if let Record::Config { spec } = r {
                        Some(spec)
                    } else {
                        None
                    }
                })
                .unwrap();
            let mut options = state
                .options
                .into_iter()
                .filter(|o| o.action == a || o.action == b)
                .collect::<Vec<_>>();
            for o in &mut options {
                o.prediction = Proxy::new("controlled_probability", 0.5);
            }
            engine.project_rank_options(config, &records, &state.criteria, &mut options, None)?;
            Ok(options)
        };
        let ranked = rank(&e)?;
        let first = ranked.iter().find(|o| o.action == a).unwrap();
        let second = ranked.iter().find(|o| o.action == b).unwrap();
        assert_eq!(first.resource_cost.as_ref().unwrap().samples.len(), 1);
        assert!(second.resource_cost.as_ref().unwrap().samples.is_empty());
        assert_eq!(
            first.decision.as_ref().unwrap().gain,
            second.decision.as_ref().unwrap().gain
        );
        assert!(first.score.value > second.score.value);
        // Remove only the learned cost difference: both candidates have equal value.
        let control_a = first.score.value + first.decision.as_ref().unwrap().effort;
        let control_b = second.score.value + second.decision.as_ref().unwrap().effort;
        assert!((control_a - control_b).abs() < 1e-12);
        let baseline = ranked.iter().map(|o| o.score.value).collect::<Vec<_>>();
        assert_eq!(
            rank(&Engine::open(dir.path())?)?
                .iter()
                .map(|o| o.score.value)
                .collect::<Vec<_>>(),
            baseline
        );
        e.project_resource_reserve(budget::Reserve {
            request_id: "sunk-fixture".into(),
            state_root: e.store.head_root()?,
            envelope: "fixture".into(),
            model: "fixture".into(),
            local: true,
            tokens_upper: 100,
            external_cost_upper: 0,
        })?;
        e.project_resource_settle(budget::Settle {
            request_id: "sunk-fixture".into(),
            prompt_tokens: Some(30),
            output_tokens: Some(20),
            duration_ns: Some(1000),
            external_cost_microusd: Some(0),
            outcome: "measured".into(),
        })?;
        let after = rank(&e)?;
        assert_eq!(
            after.iter().map(|o| o.score.value).collect::<Vec<_>>(),
            baseline
        );
        assert!(after
            .iter()
            .all(|o| o.resource_cost.as_ref().unwrap().sunk_tokens_charged == 50));
        Ok(())
    }
    #[test]
    fn receipt_features_and_prospective_budgets_survive_unknowns_without_leakage() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        let root = e.store.head_root()?;
        e.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: root,
            tool_ms_ceiling: 86_400_000,
            reason: "bounded fixture telemetry".into(),
            non_tool: Some(budget::Limits {
                model_calls: 2,
                model_tokens: 1000,
                wall_ms: 600_000,
                patch_bytes: 100_000,
                external_cost_microusd: 1000,
            }),
        })?;
        e.proposal_request()?;
        e.proposal_request()?;
        let prefix = e.events()?;
        let at = lifecycle::now()?;
        let features = e.project_receipt_features(&prefix, &at, observations::SCHEMA)?;
        assert_eq!(features.contexts_issued, Some(1));
        assert_eq!(features.case_results, 0);
        assert!(e
            .project_receipt_features(&prefix, &at, "features@v1")
            .is_err());
        let r = budget::Reserve {
            request_id: "uncertain-one".into(),
            state_root: root,
            envelope: "sha256:fixture".into(),
            model: "installed-fixture".into(),
            local: true,
            tokens_upper: 300,
            external_cost_upper: 0,
        };
        e.project_resource_reserve(r.clone())?;
        e.project_resource_reserve(r.clone())?;
        assert_eq!(Engine::open(dir.path())?.project_budget()?.model_calls, 1);
        let mut second = r.clone();
        second.request_id = "uncertain-two".into();
        second.local = false;
        second.external_cost_upper = 100;
        assert!(e.project_resource_reserve(second.clone()).is_err());
        let unknown = budget::Settle {
            request_id: r.request_id.clone(),
            prompt_tokens: None,
            output_tokens: None,
            duration_ns: None,
            external_cost_microusd: Some(0),
            outcome: "unknown".into(),
        };
        e.project_resource_settle(unknown.clone())?;
        e.project_resource_settle(unknown)?;
        e.project_resource_reserve(second.clone())?;
        e.project_resource_settle(budget::Settle {
            request_id: second.request_id,
            prompt_tokens: None,
            output_tokens: None,
            duration_ns: None,
            external_cost_microusd: Some(0),
            outcome: "unknown".into(),
        })?;
        let v = e.project_budget()?;
        assert_eq!(v.tokens_charged, 600);
        assert_eq!(v.external_cost_charged_microusd, 100);
        assert_eq!(v.unknown_usage, 2);
        let mut third = r;
        third.request_id = "third".into();
        assert!(e.project_resource_reserve(third).is_err());
        assert_eq!(e.store.head_root()?, root);
        let a = patch(
            &e,
            "src/pricing.rs",
            "unit_cents + quantity",
            "unit_cents * quantity",
        )?;
        e.evaluate_proposal(offer(&e, vec![a])?, true)?;
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        let after = e.project_features()?;
        assert!(after.passed_cases > 0);
        assert_eq!(after.failed_cases, 0);
        assert_eq!(after.unknown_case_results, 0);
        assert!(!after.semantic_review_used);
        assert_eq!(
            serde_json::to_value(features)?,
            serde_json::to_value(e.project_receipt_features(
                &prefix,
                &at,
                observations::SCHEMA
            )?)?
        );
        let v = Engine::open(dir.path())?.project_budget()?;
        assert!(v.patch_bytes_charged > 0);
        assert!(v.tool_ms_charged > 0);
        assert_eq!(v.tokens_charged, 600);
        assert!(!e.project_lifecycle_view()?.native_feature_compatible);
        Ok(())
    }
    #[test]
    fn receipt_schema_builds_reconstructs_and_refuses_synthetic_training() -> Result<()> {
        use acceptance::{Decision, Evidence, Review};
        use tsr_learn::{
            dataset::{SplitBy, SplitConfig, SplitFractions},
            project_eligibility as eligibility,
            schema::Input,
        };
        let mut input = Input {
            artifacts: vec![],
            eligibility: vec![],
        };
        for i in 0..4 {
            let dir = tempfile::tempdir()?;
            let e = Engine::open(dir.path())?;
            e.project_initialize(spec())?;
            e.project_acceptance_requirements(BTreeMap::from([(
                "semantics".into(),
                "independent fixture invoice behavior".into(),
            )]))?;
            let root = e.store.head_root()?;
            let metadata = eligibility::Admission {
                origin: eligibility::Origin::Synthetic,
                original_task: Hash::of(&[format!("fixture-{i}").as_bytes()]),
                family: "fixture".into(),
                lineage: format!("fixture-{i}"),
                clone_group: format!("fixture-{i}"),
                reviewer: "operator".into(),
                no_inference: true,
            };
            e.project_lifecycle_open(lifecycle::Admission {
                training: Some(metadata.clone()),
                non_tool: Some(budget::Limits {
                    model_calls: 1,
                    model_tokens: 1000,
                    wall_ms: 600_000,
                    patch_bytes: 100_000,
                    external_cost_microusd: 0,
                }),
                state_root: root,
                tool_ms_ceiling: 100_000,
                reason: "synthetic mechanics; never mature production evidence".into(),
            })?;
            // Prospective admission captures evidence without a model/configuration.
            e.proposal_request()?;
            let edit = patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "unit_cents * quantity",
            )?;
            let alternative = patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "quantity * unit_cents",
            )?;
            let mut proposals = offer(&e, vec![edit])?;
            proposals
                .proposals
                .extend(offer(&e, vec![alternative])?.proposals);
            proposals
                .common_event_forecasts
                .insert("0".into(), Probability::new(0.6)?);
            proposals
                .common_event_forecasts
                .insert("1".into(), Probability::new(0.4)?);
            e.evaluate_proposal(proposals, true)?;
            let plan = e.coding_plan()?;
            e.coding_execute(
                plan.commitment,
                plan.source_before,
                &Fake { complete: true },
            )?;
            let status = e.project_acceptance_status()?;
            e.project_acceptance_review(Review {
                snapshot: status.snapshot,
                requirements_revision: status.requirements_revision.unwrap(),
                test_receipt: status.test_receipt,
                decision: Decision::Accept,
                evidence: BTreeMap::from([(
                    "semantics".into(),
                    Evidence {
                        met: true,
                        evidence: "deterministic synthetic fixture review".into(),
                    },
                )]),
            })?;
            let closed = e.project_lifecycle_close()?;
            let factual = e.store.head_root()?;
            assert_eq!(
                closed.version,
                tsr_learn::project_eligibility::FORECAST_PROJECTION
            );
            let decisions = tsr_learn::decision::decision_rows(&closed.events);
            assert_eq!(decisions.len(), 1);
            assert!(matches!(decisions[0].generator_raw(),Some(p) if p==0.6||p==0.4));

            let review = eligibility::Review {
                closure: eligibility::closure_digest(&serde_json::to_value(&closed)?)?,
                reviewer: "operator".into(),
                independence: "fixture reviewer separate from supplied proposal; synthetic only"
                    .into(),
                complete_instrumented_coverage: true,
            };
            let result = e.project_eligibility_review(review.clone())?;
            assert_eq!(result["status"], "synthetic_or_replay_excluded");
            assert!(result["label"].is_null());
            assert_eq!(result, e.project_eligibility_review(review)?);
            let at = lifecycle::now()?;
            let part = e.project_training_evidence(&at)?;
            assert_eq!(
                part.artifacts[0].rows.len(),
                2,
                "keep chosen and unchosen receipts"
            );
            for row in &part.artifacts[0].rows {
                let receipt: serde_json::Value = serde_json::from_str(&row.prediction_blob)?;
                assert_eq!(receipt["status"], "evidence_only");
                assert_eq!(receipt["model_evaluation_requested"], false);
                assert!(receipt["bundle_id"].is_null());
                assert!(receipt["calibrated_probability"].is_null());
            }
            assert!(part.labels("2099-01-01T00:00:00Z")?.is_empty());
            assert_eq!(
                part.digest()?,
                Engine::open(dir.path())?
                    .project_training_evidence(&at)?
                    .digest()?
            );
            assert_eq!(e.store.head_root()?, factual);
            input.artifacts.extend(part.artifacts);
            input.eligibility.extend(part.eligibility);
        }
        let cfg = SplitConfig {
            by: SplitBy::Repo,
            fractions: SplitFractions {
                calibrate: 0.25,
                holdout: 0.25,
            },
            seed: 0,
        };
        let at = lifecycle::now()?;
        let out = tempfile::tempdir()?;
        let path = out.path().join("dataset");
        let ds = input.write_dataset(&at, cfg, "operator", &path)?;
        assert_eq!(ds.spec.version, tsr_learn::project_receipts::SCHEMA);
        assert_eq!(ds.rows.len(), 4);
        assert!(ds.rows.iter().all(|r| r.label.is_none()));
        assert!(ds.rows.iter().all(|r| r.generator_raw.is_some()));
        assert_eq!(ds, tsr_learn::bundle::load_dataset(&path)?);
        let (_, _, events, index) = input.build(&at, cfg, "operator")?;
        let mut wrong_groups = index.clone();
        wrong_groups[0].clone_group = Some("unrelated-clone".into());
        assert!(input.validate(&events, &wrong_groups).is_err());
        let mut conflicting = input.clone();
        conflicting
            .eligibility
            .push(conflicting.eligibility[0].clone());
        assert!(conflicting.labels(&at).is_err());
        let extractor = tsr_learn::probes::schema_extractor(&ds.spec, Some(&input));
        assert!(
            tsr_learn::probes::probe_post_action(&ds, &events, &index, &ds.spec, &extractor)
                .is_clean()
        );
        assert!(tsr_learn::bundle::write_bundle(&path, &out.path().join("model"), 30).is_err());
        let evidence = fs::read(path.join("feature-evidence.json"))?;
        fs::write(path.join("feature-evidence.json"), b"{}")?;
        assert!(tsr_learn::bundle::load_dataset(&path).is_err());
        fs::write(path.join("feature-evidence.json"), evidence)?;
        assert_eq!(ds, tsr_learn::bundle::load_dataset(&path)?);
        let before: &[(i64, Event)] = &[];
        assert!(tsr_learn::schema::extract(
            &ds.spec,
            before,
            &index[0],
            ActionKind::ProposePatch,
            None,
            None
        )
        .is_err());
        Ok(())
    }
    #[test]
    fn native_lifecycle_tracks_actual_adapter_returns_review_budget_and_restart() -> Result<()> {
        use acceptance::{Decision, Evidence, Review};
        for (ceiling, accept) in [(1, true), (10_000, true), (10_000, false)] {
            let dir = tempfile::tempdir()?;
            let e = Engine::open(dir.path())?;
            e.project_initialize(spec())?;
            e.project_acceptance_requirements(BTreeMap::from([(
                "semantics".into(),
                "invoice behavior".into(),
            )]))?;
            let root = e.store.head_root()?;
            let admission = lifecycle::Admission {
                training: None,
                non_tool: None,
                state_root: root,
                tool_ms_ceiling: ceiling,
                reason: "explicit fixture budget".into(),
            };
            let opened = e.project_lifecycle_open(admission.clone())?;
            assert_eq!(e.store.head_root()?, root);
            let mut invalid = e.events()?;
            let bad = Event::new(
                Engine::task(),
                vec![invalid.last().unwrap().1.id],
                Source::Rules,
                EventKind::Stop {
                    reason: "diagnostic".into(),
                },
                "unix-ms:0",
            );
            invalid.push((invalid.len() as i64 + 1, bad));
            assert!(e.project_lifecycle_prefix(&invalid).is_err());
            e.configure_shadow(crate::engine::shadow::Update {
                state_root: root,
                reason: "diagnostic unavailable model".into(),
                config: tsr_learn::shadow::Config {
                    bundle_directory: dir.path().join("missing"),
                    bundle_id: Hash::of(&[b"missing"]),
                    dataset_directory: None,
                    event: tsr_learn::DECISION_EVENT.into(),
                    family: "fixture".into(),
                    lineage: "fixture".into(),
                    training_as_of: None,
                },
            })?;
            assert_eq!(
                serde_json::to_value(&opened)?,
                serde_json::to_value(e.project_lifecycle_open(admission.clone())?)?
            );
            let mut reset = admission.clone();
            reset.tool_ms_ceiling += 1;
            assert!(e.project_lifecycle_open(reset).is_err());
            let a = patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "unit_cents * quantity",
            )?;
            e.evaluate_proposal(offer(&e, vec![a])?, true)?;
            assert!(e.project_lifecycle_close().is_err());
            assert_eq!(e.project_lifecycle_view()?.incomplete_executions, 1);
            let shadow = e.shadow_view()?;
            assert!(shadow["records"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r[1].get("project_lifecycle").is_some()));
            let plan = e.coding_plan()?;
            e.coding_execute(
                plan.commitment,
                plan.source_before,
                &Fake { complete: true },
            )?;
            let before_review = e.project_lifecycle_view()?;
            assert_eq!(before_review.incomplete_executions, 0);
            assert!(before_review.consumed_tool_ms > 1);
            assert_eq!(
                Engine::open(dir.path())?
                    .project_lifecycle_view()?
                    .consumed_tool_ms,
                before_review.consumed_tool_ms
            );
            let mut duplicate = e.events()?;
            let executed = duplicate
                .iter()
                .find(|(_, event)| matches!(event.kind, EventKind::Executed { .. }))
                .unwrap()
                .1
                .clone();
            let bad = Event::new(
                Engine::task(),
                vec![duplicate.last().unwrap().1.id],
                Source::Rules,
                executed.kind,
                duplicate.last().unwrap().1.recorded_at.clone(),
            );
            duplicate.push((duplicate.len() as i64 + 1, bad));
            assert!(e.project_lifecycle_prefix(&duplicate).is_err());

            assert!(e.project_lifecycle_close().is_err());
            let status = e.project_acceptance_status()?;
            e.project_acceptance_review(Review {
                snapshot: status.snapshot,
                requirements_revision: status.requirements_revision.unwrap(),
                test_receipt: status.test_receipt,
                decision: if accept {
                    Decision::Accept
                } else {
                    Decision::Reject
                },
                evidence: BTreeMap::from([(
                    "semantics".into(),
                    Evidence {
                        met: accept,
                        evidence: "independent fixture review, not production evidence".into(),
                    },
                )]),
            })?;
            let factual = e.store.head_root()?;
            let closed = e.project_lifecycle_close()?;
            assert!(closed.closed);
            assert_eq!(e.store.head_root()?, factual);
            assert_eq!(closed.consumed_tool_ms, before_review.consumed_tool_ms);
            assert_eq!(closed.reviewed_accepted, Some(accept));
            assert!(closed.label.is_none());
            let cutoff = lifecycle::now()?;
            let artifact = e.project_dataset(&cutoff)?;
            assert!(!artifact.rows.is_empty());
            assert_eq!(artifact.eligible_rows, 0);
            assert!(artifact.require_fit_eligible().is_err());
            let output = dir.path().join("receipt-dataset.json");
            artifact.write(&output)?;
            let loaded = tsr_learn::project_receipts::Artifact::load(&output)?;
            assert_eq!(loaded.digest, artifact.digest);
            assert_eq!(
                Engine::open(dir.path())?.project_dataset(&cutoff)?.digest,
                artifact.digest
            );
            assert_eq!(e.store.head_root()?, factual);
            let mut changed = loaded.clone();
            changed.rows[0].features.passed_cases += 1;
            assert!(changed.validate().is_err());
            let mut wrong_schema = loaded.clone();
            wrong_schema.schema = "features@v1".into();
            assert!(wrong_schema.validate().is_err());
            let mut missing = loaded.clone();
            missing.rows[0].evidence.blobs.clear();
            assert!(missing.validate().is_err());
            // New side evidence cannot enter a prediction's frozen causal prefix,
            // even with an unchanged timestamp. Rebuilding the outer digest is not sufficient.
            let mut extended = loaded.rows.clone();
            let last = extended[0].evidence.side_events.last().unwrap().clone();
            extended[0].evidence.side_events.push((last.0 + 1, last.1));
            assert!(tsr_learn::project_receipts::Artifact::build(
                cutoff.clone(),
                loaded.source_root,
                extended,
                loaded.export_time_lifecycle_inventory.clone()
            )
            .is_err());
            let historical = "2020-01-01T00:00:00Z";
            assert!(tsr_learn::project_receipts::Artifact::build(
                historical.into(),
                loaded.source_root,
                loaded.rows.clone(),
                loaded.export_time_lifecycle_inventory.clone()
            )
            .is_err());
            assert!(e.project_dataset(historical)?.rows.is_empty());

            let labels = tsr_learn::decision::resolve_labels(&closed.events, &[], "operator")?;
            assert_eq!(labels[0].label, tsr_learn::decision::Label::Censored);
            assert_eq!(
                closed
                    .events
                    .iter()
                    .any(|(_, e)| matches!(&e.kind,EventKind::Stop{reason} if reason=="budget")),
                ceiling == 1
            );
            assert!(!closed.native_feature_compatible);
            assert_eq!(
                serde_json::to_value(&closed)?,
                serde_json::to_value(Engine::open(dir.path())?.project_lifecycle_close()?)?
            );
            assert_eq!(
                closed
                    .events
                    .iter()
                    .filter(|(_, e)| matches!(e.kind, EventKind::TaskClosed { .. }))
                    .count(),
                1
            );
            assert!(closed.mapping.iter().all(|(id, _, _)| e
                .events()
                .unwrap()
                .iter()
                .any(|(_, e)| e.id == *id)));
        }
        Ok(())
    }
    #[test]
    fn independent_acceptance_survives_restart_and_invalidates_on_change() -> Result<()> {
        use acceptance::{Decision, Evidence, Review};
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        assert!(!e.project_acceptance_status()?.accepted);
        let requirements = BTreeMap::from([(
            "semantics".into(),
            "PRIVATE_REQUIREMENT_SENTINEL: preserve complete invoice behavior".into(),
        )]);
        let initial = e.project_acceptance_requirements(requirements.clone())?;
        let mut review = Review {
            snapshot: initial.snapshot,
            requirements_revision: initial.requirements_revision.unwrap(),
            test_receipt: initial.test_receipt,
            decision: Decision::Accept,
            evidence: BTreeMap::from([(
                "semantics".into(),
                Evidence {
                    met: true,
                    evidence: "PRIVATE_EVIDENCE_SENTINEL: independent operator inspection".into(),
                },
            )]),
        };
        assert!(e.project_acceptance_review(review.clone()).is_err());
        // Deliberately incomplete source with a passing test observation: acceptance is independent.
        let a = patch(
            &e,
            "src/pricing.rs",
            "unit_cents + quantity",
            "unit_cents * quantity",
        )?;
        e.evaluate_proposal(offer(&e, vec![a])?, true)?;
        let plan = e.coding_plan()?;
        assert!(e
            .project_acceptance_requirements(requirements.clone())
            .is_err());
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        let passed = e.project_acceptance_status()?;
        assert_eq!(passed.test_verdict, Some(Verdict::Passed));
        assert_eq!(passed.desire_status, "open");
        assert_eq!(e.project_status()?["status"], "tests_passed_unaccepted");
        review.snapshot = passed.snapshot;
        review.test_receipt = passed.test_receipt;
        review.decision = Decision::Reject;
        review.evidence.get_mut("semantics").unwrap().met = false;
        let rejected = e.project_acceptance_review(review.clone())?;
        assert_eq!(rejected.status, "rejected");
        let reopened = Engine::open(dir.path())?;
        assert_eq!(reopened.project_acceptance_status()?.desire_status, "open");
        assert_eq!(reopened.project_export()?["goal_accepted"], false);
        let request = serde_json::to_string(&reopened.proposal_request()?)?;
        assert!(!request.contains("PRIVATE_REQUIREMENT_SENTINEL"));
        assert!(!request.contains("PRIVATE_EVIDENCE_SENTINEL"));
        let mut missing = review.clone();
        missing.evidence.clear();
        assert!(e.project_acceptance_review(missing).is_err());
        // Complete the remaining behavior and independently accept its new evidence.
        let b = patch(
            &e,
            "src/discount.rs",
            "subtotal_cents + discount_cents",
            "subtotal_cents.saturating_sub(discount_cents)",
        )?;
        e.evaluate_proposal(offer(&e, vec![b])?, true)?;
        let plan = e.coding_plan()?;
        assert!(e.project_acceptance_review(review.clone()).is_err());
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        assert!(e.project_acceptance_review(review.clone()).is_err());
        let current = e.project_acceptance_status()?;
        review.snapshot = current.snapshot;
        review.test_receipt = current.test_receipt;
        review.decision = Decision::Accept;
        review.evidence.get_mut("semantics").unwrap().met = true;
        assert!(e.project_acceptance_review(review.clone())?.accepted);
        assert_eq!(
            Engine::open(dir.path())?.project_status()?["status"],
            "goal_accepted"
        );
        let exported = e.project_export()?;
        assert_eq!(exported["goal_accepted"], true);
        assert_eq!(exported["goal_completion"]["desire_status"], "fulfilled");
        e.project_acceptance_requirements(requirements)?;
        assert_eq!(e.project_acceptance_status()?.status, "stale_review");
        assert!(e.project_acceptance_review(review.clone()).is_err());
        review.requirements_revision = e
            .project_acceptance_status()?
            .requirements_revision
            .unwrap();
        e.project_acceptance_review(review.clone())?;
        fs::write(
            dir.path().join("coding-work/src/pricing.rs"),
            "// changed source\n",
        )?;
        assert_eq!(e.project_acceptance_status()?.status, "stale_review");
        assert!(e.project_acceptance_review(review).is_err());
        assert!(!e.project_acceptance_status()?.accepted);
        Ok(())
    }
    // Fixture-only inference reads the CURRENT request; it emits fresh edits, not action menu IDs.
    fn feedback_proposal(e: &Engine, limit: usize) -> Result<super::super::provider::Response> {
        let request = e.proposal_request()?;
        let cycle = request.acceptance_cycle.as_ref().unwrap();
        let mut keys = cycle["guidance"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        keys.sort_by(|a, b| {
            cycle["belief"][a]["prediction"]["value"]
                .as_f64()
                .unwrap()
                .total_cmp(&cycle["belief"][b]["prediction"]["value"].as_f64().unwrap())
                .then(a.cmp(b))
        });
        let files = request.coding.as_ref().unwrap()["files"]
            .as_array()
            .unwrap();
        let mut edits = vec![];
        for key in keys {
            let text = cycle["guidance"][&key].as_str().unwrap();
            let (path, old, new) = if text.contains("product") {
                (
                    "src/pricing.rs",
                    "unit_cents + quantity",
                    "unit_cents * quantity",
                )
            } else {
                (
                    "src/discount.rs",
                    "subtotal_cents + discount_cents",
                    "subtotal_cents.saturating_sub(discount_cents)",
                )
            };
            let file = files.iter().find(|f| f["path"] == path).unwrap();
            if file["source"].as_str().unwrap().contains(old) {
                edits.push(patch(e, path, old, new)?);
            }
            if edits.len() == limit {
                break;
            }
        }
        offer(e, edits)
    }
    #[test]
    fn learning_intervention_preserves_unoffered_previous_commitment() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "correct".into(),
            "Independent behavior review".into(),
        )]))?;
        let response = offer(
            &e,
            vec![patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "unit_cents * quantity",
            )?],
        )?;
        e.evaluate_proposal(response, true)?;
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        let other = patch(
            &e,
            "src/discount.rs",
            "subtotal_cents + discount_cents",
            "subtotal_cents.saturating_sub(discount_cents)",
        )?;
        let id = e.write(|| e.register_patch_set(&[other]))?;
        let state = e.adaptive_state()?;
        assert_ne!(state.last_commitment.as_ref(), Some(&id));
        let factual = state.options.iter().find(|o| o.action == id).unwrap();
        assert_eq!(factual.commitment_supported, Some(true));
        let control = e.success_prior_options(&state, std::slice::from_ref(&id))?;
        assert_eq!(control.len(), 1);
        assert_eq!(control[0].commitment_supported, Some(true));
        assert_eq!(control[0].score.value, factual.score.value);
        assert_eq!(
            serde_json::to_value(&control)?,
            serde_json::to_value(Engine::open(dir.path())?.success_prior_options(&state, &[id])?)?
        );
        Ok(())
    }
    #[test]
    fn selected_support_excludes_inactive_semantic_aims_without_executing() -> Result<()> {
        for inactive in [PreferenceStatus::Withdrawn, PreferenceStatus::OutOfScope] {
            let dir = tempfile::tempdir()?;
            let e = Engine::open(dir.path())?;
            e.project_initialize(spec())?;
            e.project_acceptance_requirements(BTreeMap::from([
                ("retain".into(), "Preserve the standing intention".into()),
                ("inactive".into(), "Separately admitted intention".into()),
            ]))?;
            let aims = e.acceptance_current_aims()?;
            e.append(
                EventKind::StatusSet {
                    preference: aims["inactive"],
                    status: inactive,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
            let source = e.project_acceptance_status()?.snapshot;
            let response = offer(
                &e,
                vec![patch(
                    &e,
                    "src/pricing.rs",
                    "unit_cents + quantity",
                    "unit_cents * quantity",
                )?],
            )?;
            let result = e.evaluate_proposal(response, true)?;
            assert!(result.committed);
            assert_eq!(result.state.completed, 0);
            let mut seen = 0;
            for (_, event) in e.events()? {
                if let EventKind::Proposed { candidate } = event.kind {
                    assert!(!candidate.support.contains(&aims["inactive"]));
                    assert!(candidate.support.contains(&aims["retain"]));
                    seen += 1;
                }
            }
            assert_eq!(seen, 1);
            let restarted = Engine::open(dir.path())?;
            assert_eq!(source, restarted.project_acceptance_status()?.snapshot);
            assert!(!restarted.project_acceptance_status()?.accepted);
            assert_eq!(
                result.state.pending.unwrap().commitment,
                restarted.adaptive_state()?.pending.unwrap().commitment
            );
            assert_eq!(aims, restarted.acceptance_current_aims()?);
        }
        Ok(())
    }
    #[test]
    fn shared_kernel_uses_independent_review_and_preserves_unknown_on_restart() -> Result<()> {
        use acceptance::{Decision, Evidence, Review};
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "correct".into(),
            "Independently check behavior".into(),
        )]))?;
        let response = offer(
            &e,
            vec![patch(
                &e,
                "src/pricing.rs",
                "unit_cents + quantity",
                "unit_cents * quantity",
            )?],
        )?;
        e.evaluate_proposal(response, true)?;
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        let before = e.adaptive_state()?;
        let score_before = before.options[0].score.value;
        assert_eq!(before.options[0].commitment_supported, Some(true));
        // Source revision releases support; restoring the exact evidence restores it.
        let source = dir.path().join("coding-work/src/pricing.rs");
        let original = fs::read(&source)?;
        fs::write(&source, "// revised source\n")?;
        let changed = e.adaptive_state()?;
        assert!(changed
            .options
            .iter()
            .all(|o| o.commitment_supported == Some(false)));
        assert_eq!(
            serde_json::to_value(&changed.options)?,
            serde_json::to_value(Engine::open(dir.path())?.adaptive_state()?.options)?
        );
        fs::write(&source, original)?;
        assert_eq!(
            serde_json::to_value(&before.options)?,
            serde_json::to_value(e.adaptive_state()?.options)?
        );

        assert_eq!(
            before.options[0].decision.as_ref().unwrap().kernel,
            tsr_domain::decision::KERNEL
        );
        assert!(!before.options[0]
            .decision
            .as_ref()
            .unwrap()
            .unresolved
            .is_empty());
        let current = e.project_acceptance_status()?;
        e.project_acceptance_review(Review {
            snapshot: current.snapshot,
            requirements_revision: current.requirements_revision.unwrap(),
            test_receipt: current.test_receipt,
            decision: Decision::Reject,
            evidence: BTreeMap::from([(
                "correct".into(),
                Evidence {
                    met: false,
                    evidence:
                        "Independent bounded fixture review rejects missing discount behavior"
                            .into(),
                },
            )]),
        })?;
        let after = e.adaptive_state()?;
        assert!(after
            .options
            .iter()
            .all(|o| o.commitment_supported == Some(false)));
        // Omission from support is not evidence of destroying a previously passing test.
        let records = e.records()?;
        let configured = records
            .iter()
            .find_map(|(_, r)| {
                if let Record::Config { spec } = r {
                    Some(spec)
                } else {
                    None
                }
            })
            .unwrap();
        let mut narrow = vec![after.options[0].clone()];
        narrow[0].serves.truncate(1);
        e.project_rank_options(configured, &records, &after.criteria, &mut narrow, None)?;
        let ids = records
            .iter()
            .filter_map(|(_, r)| match r {
                Record::Want {
                    criterion,
                    preference,
                    ..
                } if !narrow[0].serves.contains(criterion) => Some(*preference),
                _ => None,
            })
            .collect::<Vec<_>>();
        for id in ids {
            if let Some(Some(1.0)) = narrow[0].decision.as_ref().unwrap().observation.get(&id) {
                assert_eq!(narrow[0].decision.as_ref().unwrap().projected[&id], 1.0);
            }
        }
        assert!(e
            .proposal_request()?
            .estimates
            .iter()
            .all(|o| o.decision.is_none()));
        assert_ne!(score_before, after.options[0].score.value);
        assert!(after.options[0]
            .decision
            .as_ref()
            .unwrap()
            .unresolved
            .is_empty());
        assert!(!e.project_acceptance_status()?.accepted);
        assert_eq!(
            serde_json::to_value(&after.options)?,
            serde_json::to_value(Engine::open(dir.path())?.adaptive_state()?.options)?
        );
        assert_eq!(
            after.options[0].score.value,
            e.adaptive_state()?.options[0].score.value
        );
        // New alternatives, not previously executed identities. The review of pricing
        // transfers to the new pricing direction; discount starts without that evidence.
        let related = patch(
            &e,
            "src/pricing.rs",
            "unit_cents * quantity",
            "quantity * unit_cents",
        )?;
        let other = patch(
            &e,
            "src/discount.rs",
            "subtotal_cents + discount_cents",
            "subtotal_cents.saturating_sub(discount_cents)",
        )?;
        let related_id = e.write(|| e.register_patch_set(std::slice::from_ref(&related)))?;
        let other_id = e.write(|| e.register_patch_set(std::slice::from_ref(&other)))?;
        let choices = e.adaptive_state()?;
        let a = choices
            .options
            .iter()
            .find(|o| o.action == related_id)
            .unwrap();
        let b = choices
            .options
            .iter()
            .find(|o| o.action == other_id)
            .unwrap();
        assert!(a.score.value < b.score.value);
        let semantic_id = *e.acceptance_current_aims()?.get("correct").unwrap();
        assert!(
            a.decision.as_ref().unwrap().projected[&semantic_id]
                < b.decision.as_ref().unwrap().projected[&semantic_id]
        );
        // Same source, receipts, alternatives and cost; unknown is not negative evidence.
        let rejected_review = e
            .records()?
            .into_iter()
            .rev()
            .find_map(|(_, record)| match record {
                Record::Coding {
                    record: CodingRecord::AcceptanceReview { review },
                } => Some(review),
                _ => None,
            })
            .unwrap();
        let mut unresolved_review = rejected_review.clone();
        unresolved_review.decision = Decision::Unresolved;
        e.project_acceptance_review(unresolved_review)?;
        let unresolved = Engine::open(dir.path())?;
        let unknown_choices = unresolved.adaptive_state()?;
        let unknown_a = unknown_choices
            .options
            .iter()
            .find(|o| o.action == related_id)
            .unwrap();
        let unknown_b = unknown_choices
            .options
            .iter()
            .find(|o| o.action == other_id)
            .unwrap();
        assert!((unknown_a.score.value - unknown_b.score.value).abs() < 1e-12);
        for action in [&related_id, &other_id] {
            let inputs = unresolved.acceptance_kernel_inputs(Some(action))?;
            assert_eq!(inputs[0].0.observed, None);
            assert_eq!(inputs[0].1, 0.5);
            assert!(inputs[0].2.is_empty());
        }
        assert!(unknown_choices.pending.is_none());
        assert!(!unresolved.project_acceptance_status()?.accepted);
        assert_eq!(
            serde_json::to_value(&unknown_choices.options)?,
            serde_json::to_value(Engine::open(dir.path())?.adaptive_state()?.options)?
        );
        // Restore the original independent judgment; it supersedes rather than doubles evidence.
        e.project_acceptance_review(rejected_review)?;
        let choices = e.adaptive_state()?;
        let a = choices
            .options
            .iter()
            .find(|o| o.action == related_id)
            .unwrap();
        let b = choices
            .options
            .iter()
            .find(|o| o.action == other_id)
            .unwrap();
        assert!(a.score.value < b.score.value);
        // Inactive semantic desires cannot keep biasing the same fixed alternatives.
        // Withdrawal changes active desire, not source, review evidence, or acceptance.
        let learned_inputs = serde_json::to_value(e.acceptance_kernel_inputs(Some(&related_id))?)?;
        for status in [PreferenceStatus::Withdrawn, PreferenceStatus::OutOfScope] {
            e.append(
                EventKind::StatusSet {
                    preference: semantic_id,
                    status,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
            let inactive = Engine::open(dir.path())?;
            assert!(
                inactive
                    .acceptance_kernel_inputs(Some(&related_id))?
                    .is_empty(),
                "inactive semantic aim must not remain an admitted ranking obligation"
            );
            let inactive_state = inactive.adaptive_state()?;
            let related_option = inactive_state
                .options
                .iter()
                .find(|o| o.action == related_id)
                .unwrap();
            let other_option = inactive_state
                .options
                .iter()
                .find(|o| o.action == other_id)
                .unwrap();
            assert!((related_option.score.value - other_option.score.value).abs() < 1e-12);
            for option in &inactive_state.options {
                if let Some(decision) = &option.decision {
                    assert!(!decision.observation.contains_key(&semantic_id));
                    assert!(!decision.selected.contains(&semantic_id));
                }
            }
            assert!(!inactive.project_acceptance_status()?.accepted);
            e.append(
                EventKind::StatusSet {
                    preference: semantic_id,
                    status: PreferenceStatus::Open,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
            let restored = Engine::open(dir.path())?;
            assert_eq!(
                learned_inputs,
                serde_json::to_value(restored.acceptance_kernel_inputs(Some(&related_id))?)?
            );
            assert_eq!(
                serde_json::to_value(&choices.options)?,
                serde_json::to_value(restored.adaptive_state()?.options)?
            );
        }
        // Same observation/cost/test forecast, remove only the learned semantic estimate.
        use tsr_domain::decision as shared;
        let trace = a.decision.as_ref().unwrap();
        let obligations = trace
            .observation
            .iter()
            .map(|(id, value)| shared::Obligation {
                id: *id,
                refines: None,
                admitted: true,
                observed: *value,
            })
            .collect::<Vec<_>>();
        let projections = trace
            .projected
            .iter()
            .map(|(id, value)| shared::Projection {
                obligation: *id,
                anticipated: if *id == semantic_id { 0.5 } else { *value },
                lower: 0.0,
                upper: 1.0,
            })
            .collect::<Vec<_>>();
        let control = shared::assess(shared::Request {
            obligations: &obligations,
            selected: &trace.selected,
            proposal: &shared::Commitment {
                family: "scoped-patch".into(),
                supports: trace.selected.clone(),
            },
            previous: None,
            projections: &projections,
            probability: trace.probability,
            effort: trace.effort,
            authorized: true,
        })
        .map_err(anyhow::Error::msg)?;
        assert!((control.score - b.score.value).abs() < 1e-12);
        struct Alternatives(Vec<String>);
        impl ProposalProvider for Alternatives {
            fn propose(&self, _: &State) -> Vec<String> {
                self.0.clone()
            }
        }
        let next = e.next(&Alternatives(vec![related_id, other_id.clone()]))?;
        let pending = next.pending.unwrap();
        assert_eq!(pending.action, other_id);
        assert!(!e.project_acceptance_status()?.accepted); // choice is never acceptance
                                                           // Recovery boundary: a receipt exists, but its matching return has not arrived.
        let snapshot = e.project_acceptance_status()?.snapshot;
        e.write(|| {
            e.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::Receipt {
                        commitment: pending.commitment,
                        verdict: Verdict::Passed,
                        runs: vec![],
                        source_after: Some(snapshot),
                        observations: vec![],
                    },
                },
                Source::Rules,
            )
        })?;
        let interrupted = e.adaptive_state()?;
        for option in interrupted.options {
            assert!(option
                .decision
                .unwrap()
                .observation
                .values()
                .all(Option::is_none));
        }
        Ok(())
    }
    #[test]
    fn feedback_drives_repair_and_carries_multiple_aims_across_episodes() -> Result<()> {
        use acceptance::{Decision, Evidence, Feedback, Review};
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        let requirements = BTreeMap::from([
            ("a_price".into(), "PRIVATE_PRICE_REQUIREMENT".into()),
            ("b_discount".into(), "PRIVATE_DISCOUNT_REQUIREMENT".into()),
        ]);
        let guidance = BTreeMap::from([
            ("a_price".into(), "Compute product in pricing".into()),
            (
                "b_discount".into(),
                "Subtract discount with floor at zero".into(),
            ),
        ]);
        let initial = e.project_acceptance_requirements(requirements.clone())?;
        e.project_acceptance_feedback(Feedback {
            requirements_revision: initial.requirements_revision.unwrap(),
            decision_record: None,
            guidance: guidance.clone(),
        })?;
        let original = e.proposal_request()?;
        assert_eq!(original.admitted_criteria.len(), spec().cases.len());
        let proposal = feedback_proposal(&e, 1)?;
        e.evaluate_proposal(proposal, true)?;
        let first = e.coding_plan()?;
        assert_eq!(first.edits[0].path, "src/pricing.rs");
        let frozen = e
            .records()?
            .iter()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record: CodingRecord::AcceptanceExpectation { forecast, .. },
                } => Some(forecast.clone()),
                _ => None,
            })
            .unwrap();
        assert_eq!(frozen.raw.get(), 0.5);
        e.coding_execute(
            first.commitment,
            first.source_before,
            &Fake { complete: true },
        )?;
        let current = e.project_acceptance_status()?;
        let evidence = BTreeMap::from([
            (
                "a_price".into(),
                Evidence {
                    met: true,
                    evidence: "PRIVATE_SOURCE_INSPECTION: product works".into(),
                },
            ),
            (
                "b_discount".into(),
                Evidence {
                    met: false,
                    evidence: "PRIVATE_SOURCE_INSPECTION: discount still adds".into(),
                },
            ),
        ]);
        let reject = e.project_acceptance_review(Review {
            snapshot: current.snapshot,
            requirements_revision: current.requirements_revision.unwrap(),
            test_receipt: current.test_receipt,
            decision: Decision::Reject,
            evidence,
        })?;
        assert_eq!(e.acceptance_cycle()?["repair_ready"], false);
        assert!(e
            .project_acceptance_feedback(Feedback {
                requirements_revision: initial.requirements_revision.unwrap(),
                decision_record: None,
                guidance: guidance.clone()
            })
            .is_err());
        e.project_acceptance_feedback(Feedback {
            requirements_revision: reject.requirements_revision.unwrap(),
            decision_record: reject.decision_record,
            guidance: guidance.clone(),
        })?;
        let request = Engine::open(dir.path())?.proposal_request()?;
        let cycle = request.acceptance_cycle.as_ref().unwrap();
        assert_eq!(cycle["repair_ready"], true);
        assert_eq!(cycle["prediction"]["value"], 1.0 / 3.0);
        assert_eq!(cycle["belief"]["a_price"]["met"], 1);
        assert_eq!(cycle["belief"]["b_discount"]["met"], 0);
        assert!(request.unmet_pressure.value > 0.0);
        assert!(!serde_json::to_string(&request)?.contains("PRIVATE_"));
        let aims = e.acceptance_current_aims()?;
        assert_eq!(aims.len(), 2);
        assert!(e
            .adaptive_state()?
            .preferences
            .iter()
            .filter(|p| aims.values().any(|id| *id == p.preference.id))
            .all(|p| p.is_open()));
        let returns = e
            .events()?
            .into_iter()
            .filter_map(|(_, e)| match e.kind {
                EventKind::Observed {
                    observation: Observation::Return { surprise, .. },
                } => Some(surprise),
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>();
        assert!(returns
            .iter()
            .any(|s| s.forecast == frozen.id && s.residual == 0.5));
        // Same new source, different history: prior rejection changes the first generated edit.
        let later_dir = tempfile::tempdir()?;
        let later = Engine::open(later_dir.path())?;
        later.project_initialize(spec())?;
        let next = later.project_acceptance_requirements(requirements.clone())?;
        later.project_acceptance_feedback(Feedback {
            requirements_revision: next.requirements_revision.unwrap(),
            decision_record: None,
            guidance: guidance.clone(),
        })?;
        let fresh = feedback_proposal(&later, 2)?;
        later.project_acceptance_carry(&e)?;
        let learned = feedback_proposal(&later, 2)?;
        let paths =
            |r: super::super::provider::Response| match r.proposals.into_iter().next().unwrap() {
                super::super::provider::Suggestion::EditSet { patches, .. } => {
                    patches.into_iter().map(|p| p.path).collect::<Vec<_>>()
                }
                _ => panic!(),
            };
        assert_eq!(paths(fresh), vec!["src/pricing.rs", "src/discount.rs"]);
        assert_eq!(
            paths(learned.clone()),
            vec!["src/discount.rs", "src/pricing.rs"]
        );
        let root = later.store.head_root()?;
        later.project_acceptance_carry(&e)?;
        assert_eq!(root, later.store.head_root()?);
        // Both desires culminate together in a generated edit set, with a learned frozen expectation.
        later.evaluate_proposal(learned, true)?;
        let plan = later.coding_plan()?;
        assert_eq!(plan.edits.len(), 2);
        let forecast = later
            .records()?
            .iter()
            .rev()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record: CodingRecord::AcceptanceExpectation { forecast, .. },
                } => Some(forecast.clone()),
                _ => None,
            })
            .unwrap();
        assert_eq!(forecast.raw.get(), 1.0 / 3.0);
        later.coding_execute(
            plan.commitment,
            plan.source_before,
            &Fake { complete: true },
        )?;
        assert!(!later.project_acceptance_status()?.accepted);
        let current = later.project_acceptance_status()?;
        let evidence = requirements
            .keys()
            .map(|id| {
                (
                    id.clone(),
                    Evidence {
                        met: true,
                        evidence: "Independent fixture inspection verifies both operations".into(),
                    },
                )
            })
            .collect();
        later.project_acceptance_review(Review {
            snapshot: current.snapshot,
            requirements_revision: current.requirements_revision.unwrap(),
            test_receipt: current.test_receipt,
            decision: Decision::Accept,
            evidence,
        })?;
        assert_eq!(
            Engine::open(later_dir.path())?.project_status()?["status"],
            "goal_accepted"
        );
        assert_eq!(later.acceptance_cycle()?["reviewed_candidates"], 2);
        assert_eq!(later.acceptance_cycle()?["prediction"]["value"], 0.5);
        let aims = later.acceptance_current_aims()?;
        assert!(later
            .adaptive_state()?
            .preferences
            .iter()
            .filter(|p| aims.values().any(|id| *id == p.preference.id))
            .all(|p| p.status == Some(PreferenceStatus::Fulfilled)));
        assert!(later.next(&DeterministicProvider).is_err());
        // Corrected reviews replace imported observations instead of accumulating stale votes.
        let status = e.project_acceptance_status()?;
        let mut revised = Review {
            snapshot: status.snapshot,
            requirements_revision: status.requirements_revision.unwrap(),
            test_receipt: status.test_receipt,
            decision: Decision::Unresolved,
            evidence: BTreeMap::from([
                (
                    "a_price".into(),
                    Evidence {
                        met: true,
                        evidence: "inspection retained".into(),
                    },
                ),
                (
                    "b_discount".into(),
                    Evidence {
                        met: false,
                        evidence: "review under reconsideration".into(),
                    },
                ),
            ]),
        };
        e.project_acceptance_review(revised.clone())?;
        later.project_acceptance_carry(&e)?;
        assert_eq!(later.acceptance_cycle()?["reviewed_candidates"], 1);
        revised.decision = Decision::Reject;
        let status = e.project_acceptance_review(revised)?;
        e.project_acceptance_feedback(Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: status.decision_record,
            guidance,
        })?;
        later.project_acceptance_carry(&e)?;
        assert_eq!(later.acceptance_cycle()?["reviewed_candidates"], 2);
        // A later objective may add a desire, retaining old evidence only on unchanged dimensions.
        let expanded_dir = tempfile::tempdir()?;
        let expanded = Engine::open(expanded_dir.path())?;
        expanded.project_initialize(spec())?;
        let mut expanded_requirements = requirements.clone();
        expanded_requirements.insert(
            "c_new".into(),
            "New independent rounding requirement".into(),
        );
        let status = expanded.project_acceptance_requirements(expanded_requirements)?;
        expanded.project_acceptance_feedback(Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([
                ("a_price".into(), "Compute product".into()),
                ("c_new".into(), "Preserve rounding".into()),
            ]),
        })?;
        expanded.project_acceptance_carry(&e)?;
        let growing = expanded.acceptance_cycle()?;
        assert_eq!(growing["reviewed_candidates"], 1);
        assert_eq!(growing["fully_reviewed_candidates"], 0);
        assert_eq!(growing["prediction"]["value"], 0.5);
        assert_eq!(growing["belief"]["a_price"]["observed"], 1);
        assert_eq!(growing["belief"]["c_new"]["observed"], 0);
        assert!(!expanded.project_acceptance_status()?.accepted);
        expanded.project_acceptance_requirements(BTreeMap::from([(
            "a_price".into(),
            "Changed meaning".into(),
        )]))?;
        assert!(expanded.project_acceptance_carry(&e).is_err());
        // The original incomplete episode can also resume a fresh generated correction.
        let response = feedback_proposal(&e, 2)?;
        assert_eq!(paths(response.clone()), vec!["src/discount.rs"]);
        e.evaluate_proposal(response, true)?;
        let repair = e.coding_plan()?;
        e.coding_execute(
            repair.commitment,
            repair.source_before,
            &Fake { complete: true },
        )?;
        assert_eq!(e.acceptance_cycle()?["repair_ready"], false); // review binds the old snapshot
        assert!(!e.project_acceptance_status()?.accepted);
        Ok(())
    }
    #[test]
    fn oversized_bundle_is_rejected_before_commit() -> Result<()> {
        let mut config = spec();
        for source in config.editable.values_mut() {
            source.push_str(&format!("//{}\n", "x".repeat(2897)));
        }
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(config)?;
        let a = patch(
            &e,
            "src/pricing.rs",
            "unit_cents + quantity",
            &format!("unit_cents * quantity /*{}*/", "y".repeat(1000)),
        )?;
        let b = patch(
            &e,
            "src/discount.rs",
            "subtotal_cents + discount_cents",
            &format!(
                "subtotal_cents.saturating_sub(discount_cents) /*{}*/",
                "y".repeat(1000)
            ),
        )?;
        assert!(e.validate_patch(&a).is_ok());
        assert!(e.validate_patch(&b).is_ok());
        let before = e.store.head_root()?;
        assert!(e.evaluate_proposal(offer(&e, vec![a, b])?, true).is_err());
        assert_eq!(e.store.head_root()?, before);
        assert!(e.adaptive_state()?.pending.is_none());
        Ok(())
    }
    #[test]
    fn project_scope_protects_tests_and_refuses_path_and_configuration_authority() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(spec())?;
        let suggestion = super::super::provider::Response {
            attention_binding: e.proposal_attention_binding()?,
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
            protocol: super::super::provider::PROTOCOL.into(),
            state_root: e.store.head_root()?,
            proposals: vec![super::super::provider::Suggestion::NewCandidate {
                description: "Change tests and admit an extra source file".into(),
                serves: vec!["weaker_goal".into()],
                rationale: "Proposed goal refinement only".into(),
            }],
        };
        assert!(!e.evaluate_proposal(suggestion, true)?.committed);
        assert_eq!(e.project_configuration()?.protected, spec().protected);
        assert_eq!(e.project_configuration()?.editable, spec().editable);
        let mut p = patch(
            &e,
            "src/pricing.rs",
            "unit_cents + quantity",
            "unit_cents * quantity",
        )?;
        for path in ["tests/probe.rs", "../escape.rs", "src/extra.rs"] {
            p.path = path.into();
            assert!(e.validate_patch(&p).is_err());
        }
        let mut value = serde_json::to_value(spec())?;
        value["argv"] = serde_json::json!(["sh"]);
        assert!(serde_json::from_value::<ProjectSpec>(value).is_err());
        let mut invalid = spec();
        invalid.editable.insert("../escape.rs".into(), "x".into());
        assert!(invalid.validate().is_err());
        fs::write(dir.path().join("coding-work/tests/probe.rs"), "changed")?;
        assert!(e.proposal_request().is_err());
        Ok(())
    }
    /// Authored transport/runner evidence only; not a browser or model performance claim.
    #[test]
    fn browser_native_multifile_feedback_restart_and_stale_controls() -> Result<()> {
        use super::super::provider::{Response, Suggestion};
        struct BrowserFixture;
        impl Runner for BrowserFixture {
            fn run(&self, copy: &Path, argv: &[String], _: &Limits) -> RunResult {
                match argv[0].as_str() {
                    "sh" => {
                        // Authored runner mirrors the current exact-span transport,
                        // including concatenation of chunked JSON; no host execution.
                        assert_eq!(
                            &argv[..4],
                            &strings(&[
                                "sh",
                                "-c",
                                "script=\"$1\"; shift; exec node -e \"$script\" -- \"$@\"",
                                "tsr-scoped-edit"
                            ])
                        );
                        let edits: Vec<ExactPatch> =
                            serde_json::from_str(&argv[5..].concat()).unwrap();
                        for edit in edits {
                            let path = copy.join(&edit.path);
                            let source = fs::read_to_string(&path).unwrap();
                            assert!(!edit.old_text.is_empty());
                            assert_eq!(
                                source
                                    .as_bytes()
                                    .windows(edit.old_text.len())
                                    .filter(|s| *s == edit.old_text.as_bytes())
                                    .count(),
                                1
                            );
                            fs::write(path, source.replacen(&edit.old_text, &edit.new_text, 1))
                                .unwrap();
                        }
                        RunResult::exit_only(0)
                    }
                    "node" if argv[1] == "--check" => RunResult::exit_only(0),
                    "node" => {
                        assert_eq!(
                            argv,
                            &strings(&["node", "browser-harness.cjs", "probe.json"])
                        );
                        let files = BTreeMap::from([
                            ("index.html", "a".repeat(64)),
                            ("app.js", "b".repeat(64)),
                        ]);
                        let value = serde_json::json!({"version":"browser-observation@v1","passed":true,"files":files,"observations":[{"kind":"render","sha256":"c".repeat(64)},{"kind":"text","selector":"#start","text":"Play","met":true,"abbreviated":false,"original_length":4},{"kind":"render","sha256":"d".repeat(64)}],"errors":[],"error_count":0});
                        {
                            let mut r = RunResult::from_output(
                                0,
                                &serde_json::to_vec(&value).unwrap(),
                                b"",
                                1,
                            );
                            r.execution = Some(tsr_corpus::ExecutionDiagnostics {
                                version: 1,
                                termination: "runtime_exit".into(),
                                runtime_exit: Some(0),
                                runtime_signal: None,
                                stdout_complete: true,
                                stderr_complete: true,
                                worker: "authored-local-browser-double".into(),
                            });
                            r
                        }
                    }
                    _ => panic!("unadmitted authored fixture command"),
                }
            }
        }
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut spec = browser::tests::spec();
        let sealed = std::env::var_os("TSR_BROWSER_SEALED_AUTHORED").is_some();
        if sealed {
            spec.image =
                "sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883".into();
            spec.browser_container_only = true;
            let mut probe: serde_json::Value = serde_json::from_str(&spec.protected["probe.json"])?;
            probe["checks"][0]["text"] = serde_json::json!("Play");
            spec.protected
                .insert("probe.json".into(), serde_json::to_string(&probe)?);
        }
        e.project_initialize(spec.clone())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "playability".into(),
            "User can start and understand the controls".into(),
        )]))?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            training: None,
            tool_ms_ceiling: 100000,
            reason: "Authored browser integration only".into(),
            non_tool: Some(budget::Limits {
                model_calls: 10,
                model_tokens: 10000,
                wall_ms: 600000,
                patch_bytes: 10000,
                external_cost_microusd: 0,
            }),
        })?;
        let before = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: before.requirements_revision.unwrap(),
            decision_record: before.decision_record,
            guidance: BTreeMap::from([(
                "playability".into(),
                "Authored fixture feedback: make controls legible".into(),
            )]),
        })?;
        let context = e.project_context(&spec)?;
        for f in context["files"].as_array().unwrap() {
            if f["editable"] != true {
                assert!(f.get("source").is_none());
            }
        }
        let patches = vec![
            patch(&e, "index.html", "Start", "Play")?,
            patch(&e, "app.js", "// authored fixture", "// authored revision")?,
        ];
        let stale = patches[0].clone();
        let mut response = Response {
            protocol: super::super::provider::PROTOCOL.into(),
            state_root: e.store.head_root()?,
            attention_binding: e.proposal_attention_binding()?,
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
            proposals: vec![Suggestion::EditSet {
                patches,
                rationale: "Authored multi-file option".into(),
            }],
        };
        let parent =
            serde_json::from_value(context["relational"]["inputs"]["parents"][0]["id"].clone())?;
        response.pursuit_expectations.insert("0".into(),pursuit::Expectation{parents:vec![parent],anticipated:"Authored prediction: label remains Start".into(),grounds:"Deliberately wrong authored anticipation for a contradiction regression, not live evidence".into(),uncertainty:"No player observation".into()});
        let preview = e.project_cycle_preview(response.clone())?;
        let c = &preview["cycle"];
        let input = cycle::Advance {
            binding: serde_json::from_value(c["binding"].clone())?,
            response: response.clone(),
            review: cycle::Review {
                binding: serde_json::from_value(c["binding"].clone())?,
                response_digest: serde_json::from_value(preview["response_digest"].clone())?,
                state_root: response.state_root,
                source_snapshot: serde_json::from_value(c["source_snapshot"].clone())?,
                reviewer: "independent authored review fixture".into(),
                generator: "authored fixture".into(),
                decision: "approve".into(),
            },
        };
        e.project_cycle_advance(input)?;
        let plan = e.coding_plan()?;
        assert_eq!(plan.edits.len(), 2);
        assert_eq!(plan.commands.len(), 3);
        assert_eq!(
            plan.commands[1],
            strings(&["node", "--check", "browser-harness.cjs"])
        );
        assert_eq!(
            plan.commands[2].last().unwrap(),
            if sealed {
                "--container-only"
            } else {
                "probe.json"
            }
        );
        let result = if sealed {
            e.coding_run(plan.commitment, plan.source_before)?
        } else {
            e.coding_execute(plan.commitment, plan.source_before, &BrowserFixture)?
        };
        if let Some(out) = std::env::var_os("TSR_BROWSER_AUTHORED_RESULT") {
            fs::write(out, serde_json::to_vec_pretty(&result)?)?;
        }
        assert_eq!(result.verdict, Verdict::Passed);
        assert!(e
            .coding_execute(plan.commitment, plan.source_before, &BrowserFixture)
            .is_err());
        assert!(e.validate_patch(&stale).is_err());
        assert!(!e.project_acceptance_status()?.accepted); // Test passage is not user satisfaction.
        let reflection = e.project_pursuit_reflection_request()?;
        assert!(!reflection["pending"].is_null());
        let f = pursuit::Reflection {
            binding: serde_json::from_value(reflection["binding"].clone())?,
            expectation: serde_json::from_value(reflection["pending"]["id"].clone())?,
            receipt: serde_json::from_value(reflection["pending"]["return"]["receipt"].clone())?,
            source_after: serde_json::from_value(reflection["snapshot"].clone())?,
            conclusion: pursuit::Conclusion::Contradicted,
            observed: "Authored receipt reports Play, not anticipated Start".into(),
            revised_expectation: Some(
                "Changing this label changes the visible control text; enjoyment remains unknown"
                    .into(),
            ),
            uncertainty: "Authored runner, no player evidence".into(),
            contrast:
                "Legibility still needs player evaluation; this is distinct from label prediction"
                    .into(),
            possibilities: vec![
                "Retain without further changes".into(),
                "Inspect next actual feedback".into(),
            ],
        };
        let input = pursuit::Apply {
            state_root: e.store.head_root()?,
            reflection: f,
            reviewer: "authored independent expectation review".into(),
            generator: "authored expectation generator".into(),
            note: "Source-bound authored assessment; no live learning claim".into(),
            decision: "accept".into(),
        };
        let mut bad = input.clone();
        bad.reflection.receipt = EventId(Hash::of(&[b"wrong receipt"]));
        assert!(e.project_pursuit_reflection_apply(bad).is_err());
        let mut bad = input.clone();
        bad.reviewer = bad.generator.clone();
        assert!(e.project_pursuit_reflection_apply(bad).is_err());
        e.project_pursuit_reflection_apply(input.clone())?;
        assert!(e.project_pursuit_reflection_apply(input).is_err());
        assert!(e.project_pursuit_reflection_request()?["pending"].is_null());
        let meaning = e.project_pursuit_context()?;
        assert_eq!(
            meaning["history"][0]["reviewed_reflections"][0]["reflection"]["conclusion"],
            "contradicted"
        );
        assert_eq!(
            meaning["history"][0]["frozen"]["expectation"]["parents"],
            serde_json::json!([parent])
        );
        assert_eq!(
            meaning["history"][0]["reviewed_reflections"][0]["review_source"],
            "council"
        );
        assert_eq!(
            meaning["history"][0]["reviewed_reflections"][0]["review_version"],
            "independent-pursuit-review@v2"
        );
        // Qualitative uncertainty is not a criterion-level unmet return.
        assert!(e.project_current_contrast()?.is_none());
        let (proposal, reviewed) = e
            .records()?
            .into_iter()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record: CodingRecord::PursuitReflectionReviewedV2 { review },
                } => Some((review.proposal, review)),
                _ => None,
            })
            .unwrap();
        e.require_project_bridge_source(proposal, Source::Generator)?;
        let before_wrong_source = e.store.head_root()?;
        for (record, source) in [
            (
                CodingRecord::PursuitReflectionReviewedV2 {
                    review: reviewed.clone(),
                },
                Source::Operator,
            ),
            (
                CodingRecord::PursuitReflectionAdmitted {
                    review: reviewed.clone(),
                },
                Source::Council,
            ),
        ] {
            let rejected: Result<()> = e.write(|| {
                e.adaptive_record(&Record::Coding { record }, source)?;
                e.project_pursuit_context()?;
                Ok(())
            });
            assert!(rejected.is_err());
            assert_eq!(e.store.head_root()?, before_wrong_source);
        }
        // Explicitly authored legacy history remains readable without a forged Council label.
        let legacy = e.adaptive_record(
            &Record::Coding {
                record: CodingRecord::PursuitReflectionAdmitted {
                    review: reviewed.clone(),
                },
            },
            Source::Operator,
        )?;
        let mixed = e.project_pursuit_context()?;
        let legacy_row = mixed["history"][0]["reviewed_reflections"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["admission"] == serde_json::json!(legacy))
            .unwrap();
        let mut expected_legacy = meaning["history"][0]["reviewed_reflections"][0].clone();
        expected_legacy["admission"] = serde_json::json!(legacy);
        expected_legacy
            .as_object_mut()
            .unwrap()
            .remove("review_source");
        expected_legacy
            .as_object_mut()
            .unwrap()
            .remove("review_version");
        assert_eq!(legacy_row, &expected_legacy);
        assert!(legacy_row.get("review_source").is_none());
        assert!(legacy_row.get("review_version").is_none());
        assert_eq!(
            mixed["history"][0]["reviewed_reflections"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let roots = e.store.head_root()?;
        let requests = serde_json::to_value(e.proposal_request()?)?;
        assert!(requests.to_string().contains("browser-observation@v1"));
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(roots, e.store.head_root()?);
        assert_eq!(requests, serde_json::to_value(e.proposal_request()?)?);
        let completion = e.project_acceptance_status()?;
        let aims = e.acceptance_current_aims()?;
        let budget = e.project_budget()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: completion.requirements_revision.unwrap(),
            decision_record: completion.decision_record,
            guidance: BTreeMap::from([(
                "playability".into(),
                "Authored second feedback: retain controls, improve contrast".into(),
            )]),
        })?;
        assert_eq!(aims, e.acceptance_current_aims()?);
        assert_eq!(budget.model_calls, e.project_budget()?.model_calls);
        assert_eq!(
            before.requirements_revision,
            e.project_acceptance_status()?.requirements_revision
        );
        assert_eq!(
            fs::read_to_string(dir.path().join("coding-work/browser-harness.cjs"))?,
            browser::HARNESS
        );
        assert!(e.project_cycle_preview(response).is_err());
        if let Some(out) = std::env::var_os("TSR_BROWSER_AUTHORED_REQUEST") {
            fs::write(out, serde_json::to_vec_pretty(&e.proposal_request()?)?)?;
        }
        Ok(())
    }
}
