//! Checksum-locked Cargo workspaces using dependencies prepared in the pinned image.
use super::*;
mod workspace;

pub(super) fn command() -> Vec<String> {
    let mut argv = tsr_corpus::preparation::full_test_argv();
    argv.extend(strings(&["--target-dir", "/tmp/tsr-cargo-target"]));
    argv
}
const REGISTRY: &str = "registry+https://github.com/rust-lang/crates.io-index";

pub(super) fn needs_preparation(spec: &ProjectSpec) -> bool {
    spec.protected
        .get("Cargo.lock")
        .and_then(|s| s.parse::<toml::Value>().ok())
        .and_then(|v| v.get("package").and_then(toml::Value::as_array).cloned())
        .is_some_and(|ps| ps.iter().any(|p| p.get("source").is_some()))
}

fn keys(table: &toml::map::Map<String, toml::Value>, allowed: &[&str]) -> Result<()> {
    ensure!(
        table.keys().all(|k| allowed.contains(&k.as_str())),
        "unsupported Cargo manifest field"
    );
    Ok(())
}
pub(super) fn validate(spec: &ProjectSpec) -> Result<()> {
    ensure!(
        spec.probe_entry.is_empty(),
        "Cargo scope cannot specify a probe entry"
    );
    ensure!(
        spec.protected.contains_key("Cargo.lock"),
        "protected Cargo.lock required"
    );
    ensure!(
        spec.editable
            .keys()
            .all(|p| !p.split('/').any(|c| c == "tests")),
        "Cargo tests must be protected"
    );
    ensure!(
        spec.cases
            .iter()
            .all(|c| c.args.is_empty() && c.expected_stdout.is_empty()),
        "Cargo cases name tests only"
    );
    if spec.named_package.is_some() {
        return workspace::validate(spec);
    }
    let root: toml::Value = spec
        .protected
        .get("Cargo.toml")
        .ok_or_else(|| anyhow::anyhow!("protected root manifest required"))?
        .parse()?;
    let table = root
        .as_table()
        .ok_or_else(|| anyhow::anyhow!("manifest table required"))?;
    let mut packages = vec![];
    if let Some(workspace) = table.get("workspace") {
        let w = workspace
            .as_table()
            .ok_or_else(|| anyhow::anyhow!("workspace table required"))?;
        keys(w, &["members", "resolver"])?;
        let members = w
            .get("members")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow::anyhow!("explicit workspace members required"))?;
        ensure!(
            !members.is_empty() && members.len() <= 4,
            "workspace member bound"
        );
        for member in members {
            let m = member
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("member path required"))?;
            ensure!(
                valid_path(&format!("{m}/Cargo.toml")),
                "literal relative member required"
            );
            packages.push(format!("{m}/"));
        }
    }
    if table.contains_key("package") {
        packages.push(String::new());
    }
    ensure!(!packages.is_empty(), "Cargo package required");
    keys(
        table,
        &[
            "workspace",
            "package",
            "lib",
            "dependencies",
            "dev-dependencies",
        ],
    )?;
    for prefix in &packages {
        let path = format!("{prefix}Cargo.toml");
        let manifest: toml::Value = spec
            .protected
            .get(&path)
            .ok_or_else(|| anyhow::anyhow!("member manifest missing"))?
            .parse()?;
        let t = manifest
            .as_table()
            .ok_or_else(|| anyhow::anyhow!("manifest table required"))?;
        keys(
            t,
            &[
                "workspace",
                "package",
                "lib",
                "dependencies",
                "dev-dependencies",
            ],
        )?;
        for kind in ["dependencies", "dev-dependencies"] {
            if let Some(deps) = t.get(kind) {
                let deps = deps
                    .as_table()
                    .ok_or_else(|| anyhow::anyhow!("dependency table required"))?;
                ensure!(
                    deps.len() <= 8,
                    "at most eight direct dependencies per section"
                );
                for (name, value) in deps {
                    ensure!(
                        !name.is_empty()
                            && name.len() <= 64
                            && name
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-'),
                        "invalid dependency name"
                    );
                    let version = value.as_str().ok_or_else(|| anyhow::anyhow!("dependency must be an exact version string; path/git/registry overrides refused"))?;
                    ensure!(
                        version.starts_with('=')
                            && version.len() <= 64
                            && version[1..].split('.').count() == 3
                            && version[1..]
                                .split('.')
                                .all(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())),
                        "exact stable dependency version required"
                    );
                    let lock: toml::Value = spec.protected["Cargo.lock"].parse()?;
                    ensure!(
                        lock.get("package")
                            .and_then(toml::Value::as_array)
                            .is_some_and(|ps| ps.iter().any(|p| p
                                .get("name")
                                .and_then(toml::Value::as_str)
                                == Some(name.as_str())
                                && p.get("version").and_then(toml::Value::as_str)
                                    == Some(&version[1..])
                                && p.get("source").and_then(toml::Value::as_str)
                                    == Some(REGISTRY))),
                        "direct dependency absent from locked crates.io inventory"
                    );
                }
            }
        }
        let pkg = t
            .get("package")
            .and_then(toml::Value::as_table)
            .ok_or_else(|| anyhow::anyhow!("package table required"))?;
        keys(
            pkg,
            &[
                "name",
                "version",
                "edition",
                "build",
                "autobins",
                "autotests",
                "autobenches",
                "autoexamples",
            ],
        )?;
        ensure!(
            pkg.get("build").is_none_or(|v| v.as_bool() == Some(false)),
            "build scripts refused"
        );
        if let Some(lib) = t.get("lib") {
            let l = lib
                .as_table()
                .ok_or_else(|| anyhow::anyhow!("lib table required"))?;
            keys(l, &["path", "doctest"])?;
            ensure!(
                l.get("path")
                    .is_none_or(|v| v.as_str() == Some("src/lib.rs")),
                "standard lib path required"
            );
        }
        let lib = format!("{prefix}src/lib.rs");
        ensure!(
            spec.editable.contains_key(&lib) || spec.protected.contains_key(&lib),
            "member library source required"
        );
    }
    for path in spec.editable.keys().chain(spec.protected.keys()) {
        ensure!(
            path != "build.rs" && !path.ends_with("/build.rs"),
            "build scripts refused"
        );
        ensure!(
            path == "Cargo.lock"
                || path == "Cargo.toml"
                || packages
                    .iter()
                    .any(|prefix| path == &format!("{prefix}Cargo.toml")
                        || path
                            .strip_prefix(prefix)
                            .is_some_and(|p| p.starts_with("src/") || p.starts_with("tests/"))),
            "file outside admitted Cargo package"
        );
    }
    let lock: toml::Value = spec.protected["Cargo.lock"].parse()?;
    let packages = lock
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("lock package inventory required"))?;
    ensure!(packages.len() <= 512, "at most 512 locked packages");
    for package in packages {
        if let Some(source) = package.get("source") {
            ensure!(
                source.as_str() == Some(REGISTRY),
                "only crates.io registry locks supported"
            );
            ensure!(
                package
                    .get("checksum")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|s| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())),
                "registry checksum required"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
fn classify(run: &RunResult, cases: &[ProbeCase]) -> (Verdict, Vec<String>) {
    classify_bounded(run, cases, 4096)
}

pub(in crate::engine::adaptive::coding) fn classify_bounded(
    run: &RunResult,
    cases: &[ProbeCase],
    output_limit: usize,
) -> (Verdict, Vec<String>) {
    let unavailable = (Verdict::Unavailable, vec![]);
    if run.exit == 124 {
        return (Verdict::Timeout, vec![]);
    }
    if !matches!(run.exit, 0 | 101)
        || run.stdout.len() >= output_limit.saturating_sub(3)
        || run.stderr.len() >= output_limit.saturating_sub(3)
    {
        return unavailable;
    }
    let Ok(failed) = tsr_batch::admission::completed_suite(&run.stdout) else {
        return unavailable;
    };
    let Ok(report) = tsr_corpus::mine::cargo_report(&run.stdout) else {
        return unavailable;
    };
    let names: BTreeSet<_> = report.tests.iter().map(|t| t.name.as_str()).collect();
    let expected: BTreeSet<_> = cases.iter().map(|c| c.id.as_str()).collect();
    if names != expected || report.tests.len() != cases.len() || failed != report.failed().len() {
        return unavailable;
    }
    let met = report
        .passed()
        .iter()
        .map(|id| case_criterion(id))
        .collect();
    match (run.exit, failed) {
        (0, 0) => (Verdict::Passed, met),
        (101, n) if n > 0 => (Verdict::FailedTests, met),
        _ => unavailable,
    }
}

impl Engine {
    fn dependency_binding(&self, spec: &ProjectSpec) -> Result<Hash> {
        Ok(Hash::of(&[
            b"cargo-dependency-preparation@v1",
            &serde_json::to_vec(spec)?,
        ]))
    }
    pub(in crate::engine::adaptive::coding) fn dependencies_ready(
        &self,
        spec: &ProjectSpec,
    ) -> Result<bool> {
        if !spec.cargo || !needs_preparation(spec) {
            return Ok(true);
        }
        let binding = self.dependency_binding(spec)?;
        Ok(self.records()?.iter().any(|(_, r)| matches!(r, Record::Coding { record: CodingRecord::Dependencies { binding: b, ready: true, .. } } if *b == binding)))
    }
    /// Check only pre-fetched dependencies in the immutable image. No compile, test or network.
    pub fn project_prepare(&self) -> Result<serde_json::Value> {
        let spec = self.project_configuration()?;
        ensure!(spec.cargo, "Cargo project required");
        let runner = SealedRunner::new(Runtime::Docker, spec.image.clone(), spec.user.clone())
            .map_err(anyhow::Error::msg)?
            .with_read_only_workspace();
        self.prepare_dependencies(&spec, &runner)
    }
    fn prepare_dependencies(
        &self,
        spec: &ProjectSpec,
        runner: &dyn Runner,
    ) -> Result<serde_json::Value> {
        ensure!(
            self.adaptive_state()?.pending.is_none(),
            "prepare dependencies before committing a repair"
        );
        let before = self.project_snapshot(spec)?;
        let binding = self.dependency_binding(spec)?;
        if self.dependencies_ready(spec)? {
            return Ok(serde_json::json!({"ready":true,"binding":binding,"reused":true}));
        }
        let resource_request = self.reserve_project_preparation(binding, spec.timeout_ms)?;
        let argv = strings(&["cargo", "fetch", "--locked", "--offline"]);
        let run = runner.run(
            &self.directory.join("coding-work").canonicalize()?,
            &argv,
            &Limits::new(spec.timeout_ms, 4096),
        );
        let evidence = self
            .directory
            .join("dependency-receipts")
            .join(binding.to_string());
        fs::create_dir_all(&evidence)?;
        let receipt = tsr_corpus::persist_run(&evidence, &binding.to_string(), &argv, &run, None)
            .map_err(anyhow::Error::msg)?;
        let ready = run.exit == 0
            && run.stdout.len() < 4093
            && run.stderr.len() < 4093
            && self.project_snapshot(spec).is_ok_and(|s| s == before);
        self.write(|| {
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::Dependencies {
                        resource_request,
                        measured_tool_ms: Some(run.duration_ms),
                        binding,
                        ready,
                        receipt: receipt.clone(),
                    },
                },
                Source::Rules,
            )
        })?;
        Ok(
            serde_json::json!({"ready":ready,"binding":binding,"receipt":receipt,"exit":run.exit,"reused":false,"stderr":String::from_utf8_lossy(&run.stderr)}),
        )
    }
    /// Read only the exact admitted paths. Never invoke Cargo in the owner's project.
    pub fn project_import(&self, mut spec: ProjectSpec, source: &Path) -> Result<State> {
        let limits: BTreeMap<_, _> = spec
            .editable
            .keys()
            .chain(spec.protected.keys())
            .map(|path| (path.clone(), file_limit(&spec, path)))
            .collect();
        for (path, text) in spec.editable.iter_mut().chain(spec.protected.iter_mut()) {
            *text = read_file(source, path, limits[path])?;
        }
        spec.validate()?;
        self.project_initialize(spec)
    }
    /// Whole-file unified diff, applicable with git apply; artifacts remain outside candidate scope.
    pub fn project_export(&self) -> Result<serde_json::Value> {
        let spec = self.project_configuration()?;
        let current = self.project_snapshot(&spec)?;
        let mut patch = String::new();
        for (path, old) in &spec.editable {
            let new = &current[path];
            if old == new {
                continue;
            }
            patch.push_str(&format!(
                "--- a/{path}\n+++ b/{path}\n@@ -{},{} +{},{} @@\n",
                usize::from(!old.is_empty()),
                old.lines().count(),
                usize::from(!new.is_empty()),
                new.lines().count()
            ));
            for (prefix, text) in [("-", old), ("+", new)] {
                for line in text.split_inclusive('\n') {
                    patch.push_str(prefix);
                    patch.push_str(line);
                    if !line.ends_with('\n') {
                        patch.push_str("\n\\ No newline at end of file\n");
                    }
                }
            }
        }
        let root = self.directory.join("exports");
        fs::create_dir_all(&root)?;
        let snapshot = digest(&current)?;
        let path = root.join(format!("{snapshot}.patch"));
        fs::write(&path, patch)?;
        let completion = self.project_acceptance_status()?;
        let metadata = root.join(format!("{snapshot}-{}.json", self.store.head_root()?));
        let result = serde_json::json!({"snapshot":snapshot,"patch":path,"metadata":metadata,
            "state":self.adaptive_state()?,"goal_accepted":completion.accepted,"goal_completion":completion});
        fs::write(&metadata, serde_json::to_vec_pretty(&result)?)?;
        Ok(result)
    }
}

/// Full suite is not the named-test inventory. All emitted groups must complete,
/// at least one test must execute, and truncation/exit disagreement stays unknown.
pub(in crate::engine::adaptive::coding) fn classify_full(run: &RunResult) -> Verdict {
    if run.exit == 124 {
        return Verdict::Timeout;
    }
    if !matches!(run.exit, 0 | 101)
        || run.stdout.len() >= (1 << 20) - 3
        || run.stderr.len() >= (1 << 20) - 3
    {
        return Verdict::Unavailable;
    }
    let Ok(failed) = tsr_batch::admission::completed_suite(&run.stdout) else {
        return Verdict::Unavailable;
    };
    let text = match std::str::from_utf8(&run.stdout) {
        Ok(s) => s,
        Err(_) => return Verdict::Unavailable,
    };
    let mut counts = [0usize; 3];
    let mut names = BTreeSet::new();
    let mut executed = 0usize;
    for line in text.lines().map(str::trim) {
        if line.starts_with("running ") {
            counts = [0; 3];
            names.clear();
        }
        if let Some(rest) = line.strip_prefix("test ") {
            if let Some((name, status)) = rest.rsplit_once(" ... ") {
                let slot = match status {
                    "ok" => 0,
                    "FAILED" => 1,
                    s if s.starts_with("ignored") => 2,
                    _ => return Verdict::Unavailable,
                };
                if !names.insert(name) {
                    return Verdict::Unavailable;
                }
                counts[slot] += 1;
            }
        }
        if let Some(summary) = line.strip_prefix("test result: ") {
            let Some((_, numbers)) = summary.split_once(". ") else {
                return Verdict::Unavailable;
            };
            let fields = numbers.split(';').collect::<Vec<_>>();
            for (i, count) in counts.iter().enumerate() {
                if fields
                    .get(i)
                    .and_then(|f| f.split_whitespace().next())
                    .and_then(|v| v.parse::<usize>().ok())
                    != Some(*count)
                {
                    return Verdict::Unavailable;
                }
            }
            if fields.get(3).and_then(|f| f.split_whitespace().next()) != Some("0") {
                return Verdict::Unavailable;
            }
            executed += counts[0] + counts[1];
        }
    }
    if executed == 0 {
        return Verdict::Unavailable;
    }
    match (run.exit, failed) {
        (0, 0) => Verdict::Passed,
        (101, n) if n > 0 => Verdict::FailedTests,
        _ => Verdict::Unavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_output_limit_tracks_capture_and_rejects_truncation() {
        let cases = vec![ProbeCase {
            id: "unit::works".into(),
            args: vec![],
            expected_stdout: String::new(),
        }];
        let output = b"running 1 test\ntest unit::works ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n";
        let run = RunResult::from_output(0, output, &vec![b' '; 6000], 1);
        assert_eq!(classify(&run, &cases).0, Verdict::Unavailable);
        assert_eq!(classify_bounded(&run, &cases, 16384).0, Verdict::Passed);
        let run = RunResult::from_output(0, output, &vec![b' '; 16384], 1);
        assert_eq!(
            classify_bounded(&run, &cases, 16384).0,
            Verdict::Unavailable
        );
        let run = RunResult::from_output(125, output, b"output capped", 1);
        assert_eq!(
            classify_bounded(&run, &cases, 16384).0,
            Verdict::Unavailable
        );
    }
    fn scope() -> ProjectSpec {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/engine-cargo");
        let mut spec = ProjectSpec::from_file(&root.join("scope.json")).unwrap();
        for (p, s) in spec.editable.iter_mut().chain(spec.protected.iter_mut()) {
            *s = read_file(&root, p, 256 * 1024).unwrap();
        }
        spec
    }
    fn locked_scope() -> ProjectSpec {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/engine-cargo-locked");
        let mut spec = ProjectSpec::from_file(&root.join("scope.json")).unwrap();
        for (p, s) in spec.editable.iter_mut().chain(spec.protected.iter_mut()) {
            *s = read_file(&root, p, 256 * 1024).unwrap();
        }
        spec
    }
    fn oracle_scope() -> ProjectSpec {
        let mut s = scope();
        s.named_package = Some("core".into());
        s.full_workspace_oracle = true;
        let text="pub fn value() -> u8 { 0 }\n#[cfg(test)] mod tests { #[test] fn value() { assert_eq!(super::value(),1); } }\n";
        s.editable = BTreeMap::from([("crates/core/src/lib.rs".into(), text.into())]);
        s.edit_windows = BTreeMap::from([(
            "crates/core/src/lib.rs".into(),
            EditWindow { start: 23, end: 24 },
        )]);
        s.protected = BTreeMap::from([
            (
                "Cargo.toml".into(),
                "[workspace]\nmembers = [\"crates/core\"]\nresolver = \"2\"\n".into(),
            ),
            (
                "crates/core/Cargo.toml".into(),
                "[package]\nname = \"core\"\nversion = \"1.0.0\"\nedition = \"2021\"\n".into(),
            ),
            (
                "Cargo.lock".into(),
                "version = 4\n[[package]]\nname = \"core\"\nversion = \"1.0.0\"\n".into(),
            ),
        ]);
        s.cases = vec![ProbeCase {
            id: "tests::value".into(),
            args: vec![],
            expected_stdout: String::new(),
        }];
        s
    }
    fn oracle_engine(dir: &Path) -> Result<Engine> {
        let e = Engine::open(dir)?;
        e.project_initialize(oracle_scope())?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 1_000_000,
            reason: "synthetic oracle accounting".into(),
            training: None,
            non_tool: Some(budget::Limits {
                model_calls: 1,
                model_tokens: 1000,
                wall_ms: 600_000,
                patch_bytes: 100_000,
                external_cost_microusd: 0,
            }),
        })?;
        let request = e.proposal_request()?;
        let f = &request.coding.as_ref().unwrap()["files"][0];
        let patch = ExactPatch {
            path: "crates/core/src/lib.rs".into(),
            old_text: "0".into(),
            new_text: "1".into(),
            source_digest: serde_json::from_value(f["source_digest"].clone())?,
            inspection_handle: serde_json::from_value(f["inspection_handle"].clone())?,
        };
        let response = super::super::super::provider::Response {
            attention_binding: e.proposal_attention_binding()?,
            protocol: super::super::super::provider::PROTOCOL.into(),
            state_root: e.store.head_root()?,
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
            proposals: vec![super::super::super::provider::Suggestion::ExactEdit {
                patch,
                rationale: "synthetic fixture only".into(),
            }],
        };
        e.evaluate_proposal(response, true)?;
        Ok(e)
    }
    struct OracleFixture {
        named_fail: bool,
        full_exit: i32,
    }
    impl Runner for OracleFixture {
        fn run(&self, path: &Path, argv: &[String], _: &Limits) -> RunResult {
            if argv[0] == "sh" {
                for p in argv[4..].chunks_exact(2) {
                    fs::write(path.join(&p[0]), &p[1]).unwrap();
                }
            }
            if argv[0] != "cargo" {
                return RunResult::from_output(0, b"", b"", 7);
            }
            let full = argv.iter().any(|v| v == "--workspace");
            let exit = if full {
                self.full_exit
            } else if self.named_fail {
                101
            } else {
                0
            };
            if exit == 124 {
                return RunResult::from_output(124, b"", b"", 20);
            }
            let text = if exit == 0 {
                "running 1 test\ntest tests::value ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
            } else {
                "running 1 test\ntest tests::value ... FAILED\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
            };
            RunResult::from_output(exit, text.as_bytes(), b"", 7)
        }
    }
    fn verified_fixture(dir: &Path) -> Result<Engine> {
        let e = oracle_engine(dir)?;
        let p = e.coding_plan()?;
        e.coding_execute(
            p.commitment,
            p.source_before,
            &OracleFixture {
                named_fail: false,
                full_exit: 124,
            },
        )?;
        Ok(e)
    }
    fn admit_verify(e: &Engine) -> Result<super::super::super::provider::Response> {
        let snapshot = digest(&e.project_snapshot(&e.project_configuration()?)?)?;
        Ok(serde_json::from_value(e.project_admit_verification(
            e.store.head_root()?,
            snapshot,
        )?)?)
    }
    #[test]
    fn requirement_aliases_preserve_one_desire_across_restart_and_contradiction() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = verified_fixture(dir.path())?;
        let duplicate = BTreeMap::from([
            ("a".into(), "same standing want".into()),
            ("b".into(), "same standing want".into()),
        ]);
        let root = e.store.head_root()?;
        assert!(e
            .project_acceptance_requirements(duplicate.clone())
            .is_err());
        assert_eq!(e.store.head_root()?, root);
        // Reproduce a historical valid admission without rewriting it.
        e.write(|| {
            let revision = e.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::AcceptanceRequirements {
                        requirements: duplicate.clone(),
                    },
                },
                Source::Operator,
            )?;
            e.acceptance_admit_aims(revision, &duplicate)
        })?;
        let before = e.acceptance_kernel_inputs(None)?;
        assert_eq!(before.len(), 1);
        assert_eq!(before[0].0.observed, None);
        let stable = e.store.head_root()?;
        e.project_status()?;
        assert_eq!(e.store.head_root()?, stable);
        drop(e);
        let e = Engine::open(dir.path())?;
        let reloaded = e.acceptance_kernel_inputs(None)?;
        assert_eq!(reloaded[0].0.id, before[0].0.id);
        assert_eq!(reloaded[0].1, before[0].1);
        let status = e.project_acceptance_status()?;
        e.project_acceptance_review(acceptance::Review {
            snapshot: status.snapshot,
            requirements_revision: status.requirements_revision.unwrap(),
            test_receipt: status.test_receipt,
            decision: acceptance::Decision::Reject,
            evidence: BTreeMap::from([
                (
                    "a".into(),
                    acceptance::Evidence {
                        met: true,
                        evidence: "synthetic positive alias".into(),
                    },
                ),
                (
                    "b".into(),
                    acceptance::Evidence {
                        met: false,
                        evidence: "synthetic conflicting alias".into(),
                    },
                ),
            ]),
        })?;
        let after = e.acceptance_kernel_inputs(None)?;
        assert_eq!(after.len(), 1);
        assert_eq!(after[0].0.observed, Some(0.0));
        e.project_status()?;
        Ok(())
    }
    #[test]
    fn origin_export_links_empty_verification_diff_without_recharging() -> Result<()> {
        let old_dir = tempfile::tempdir()?;
        let old = verified_fixture(old_dir.path())?;
        let commitment = old
            .records()?
            .iter()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record: CodingRecord::Intent { plan },
                } => Some(plan.commitment),
                _ => None,
            })
            .unwrap();
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut scope = oracle_scope();
        for (path, value) in scope.editable.iter_mut() {
            *value = old.project_snapshot(&old.project_configuration()?)?[path].clone();
        }
        e.project_initialize(scope)?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "contract".into(),
            "synthetic source contract".into(),
        )]))?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 1_000_000,
            reason: "synthetic verification origin".into(),
            training: None,
            non_tool: Some(budget::Limits {
                model_calls: 0,
                model_tokens: 0,
                wall_ms: 600_000,
                patch_bytes: 0,
                external_cost_microusd: 0,
            }),
        })?;
        e.evaluate_proposal(admit_verify(&e)?, true)?;
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &OracleFixture {
                named_fail: false,
                full_exit: 0,
            },
        )?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_review(acceptance::Review {
            snapshot: status.snapshot,
            requirements_revision: status.requirements_revision.unwrap(),
            test_receipt: status.test_receipt,
            decision: acceptance::Decision::Accept,
            evidence: BTreeMap::from([(
                "contract".into(),
                acceptance::Evidence {
                    met: true,
                    evidence: "synthetic independent fixture".into(),
                },
            )]),
        })?;
        e.project_lifecycle_close()?;
        let old_root = old.store.head_root()?;
        let root = e.store.head_root()?;
        let budget = serde_json::to_value(e.project_budget()?)?;
        let regular = e.project_export()?;
        assert!(fs::read_to_string(regular["patch"].as_str().unwrap())?.is_empty());
        let linked = e.project_export_origin(old_dir.path(), commitment)?;
        assert!(!fs::read_to_string(linked["patch"].as_str().unwrap())?.is_empty());
        assert_eq!(e.project_export_origin(old_dir.path(), commitment)?, linked);
        assert_eq!(old.store.head_root()?, old_root);
        assert_eq!(e.store.head_root()?, root);
        assert_eq!(
            serde_json::to_value(e.project_budget()?)?["patch_bytes_charged"],
            budget["patch_bytes_charged"]
        );
        assert_eq!(
            serde_json::to_value(e.project_budget()?)?["tool_ms_charged"],
            budget["tool_ms_charged"]
        );
        assert!(e
            .project_export_origin(old_dir.path(), plan.commitment)
            .is_err());
        assert!(e
            .project_export_origin(dir.path(), plan.commitment)
            .is_err());
        let path = old_dir.path().join("coding-work/crates/core/src/lib.rs");
        let text = fs::read_to_string(&path)?;
        fs::write(&path, text.replace("{ 1 }", "{ 2 }"))?;
        assert!(e.project_export_origin(old_dir.path(), commitment).is_err());
        fs::write(&path, &text)?;
        fs::write(
            dir.path().join("coding-work/crates/core/src/lib.rs"),
            text.replace("{ 1 }", "{ 2 }"),
        )?;
        assert!(e.project_export_origin(old_dir.path(), commitment).is_err());
        Ok(())
    }
    #[test]
    fn verification_only_preserves_source_and_cost_across_restart() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = verified_fixture(dir.path())?;
        let before = e.project_snapshot(&e.project_configuration()?)?;
        let bytes = e.project_budget()?.patch_bytes_charged;
        let response = admit_verify(&e)?;
        e.evaluate_proposal(response, true)?;
        let p = e.coding_plan()?;
        assert!(p.edits.is_empty());
        assert_eq!(p.commands[0], strings(&["true"]));
        assert_eq!(p.expected_after, Some(p.source_before));
        assert!(e.project_cost_estimate(&p.action)?.is_some());
        e.project_tier2_action_budget(&p.action)?;
        assert!(e.events()?.iter().any(|(_,event)| matches!(&event.kind, EventKind::Proposed { candidate } if candidate.kind == ActionKind::RunApprovedTest && candidate.source == Source::Operator)));
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(
            e.coding_execute(
                p.commitment,
                p.source_before,
                &OracleFixture {
                    named_fail: false,
                    full_exit: 0
                }
            )?
            .verdict,
            Verdict::Passed
        );
        assert_eq!(e.project_budget()?.patch_bytes_charged, bytes);
        assert_eq!(e.project_snapshot(&e.project_configuration()?)?, before);
        assert!(e
            .coding_execute(
                p.commitment,
                p.source_before,
                &OracleFixture {
                    named_fail: false,
                    full_exit: 0
                }
            )
            .is_err());
        assert!(e.project_lifecycle_view()?.incomplete_executions == 0);
        Ok(())
    }
    #[test]
    fn verification_rejects_drift_stale_root_and_forged_provenance() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = verified_fixture(dir.path())?;
        let spec = e.project_configuration()?;
        let snap = digest(&e.project_snapshot(&spec)?)?;
        assert!(e
            .project_admit_verification(Hash::of(&[b"stale"]), snap)
            .is_err());
        let stale_response = admit_verify(&e)?;
        e.write(|| {
            let digest = e.blob(b"independent state observation")?;
            e.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        source: "verification-test-state-change".into(),
                        digest,
                    },
                },
                Source::Operator,
            )?;
            Ok(())
        })?;
        assert!(e.evaluate_proposal(stale_response, true).is_err());
        let response = admit_verify(&e)?;
        let path = dir.path().join("coding-work/crates/core/src/lib.rs");
        let original = fs::read_to_string(&path)?;
        fs::write(&path, original.replacen("{ 1 }", "{ 2 }", 1))?;
        assert!(e.evaluate_proposal(response.clone(), true).is_err());
        fs::write(&path, &original)?;
        e.evaluate_proposal(response, true)?;
        let p = e.coding_plan()?;
        fs::write(&path, original.replacen("{ 1 }", "{ 2 }", 1))?;
        assert!(e
            .coding_execute(
                p.commitment,
                p.source_before,
                &OracleFixture {
                    named_fail: false,
                    full_exit: 0
                }
            )
            .is_err());
        fs::write(&path, &original)?;
        // Configuration is immutable (first admission wins). A mismatched
        // verification binding must still fail even with operator provenance.
        e.write(|| {
            e.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::Verification {
                        state_root: e.store.head_root()?,
                        snapshot: snap,
                        configuration: Hash::of(&[b"different scope"]),
                        action: Action {
                            id: "scope-mismatch".into(),
                            description: "verify".into(),
                            serves: vec![],
                            cost: 0.1,
                        },
                    },
                },
                Source::Operator,
            )?;
            Ok(())
        })?;
        assert!(e.project_verification_for("scope-mismatch").is_err());
        let record = CodingRecord::Verification {
            state_root: e.store.head_root()?,
            snapshot: snap,
            configuration: Hash::of(&[&serde_json::to_vec(&spec)?]),
            action: Action {
                id: "forged-verification".into(),
                description: "forged".into(),
                serves: vec![],
                cost: 0.1,
            },
        };
        e.write(|| {
            e.adaptive_record(&Record::Coding { record }, Source::Intuition)?;
            Ok(())
        })?;
        assert!(e.project_verification_for("forged-verification").is_err());
        assert!(e.adaptive_state().is_err());
        Ok(())
    }
    #[test]
    fn verification_uncertain_execution_retains_reservation_and_refuses_repeat() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = verified_fixture(dir.path())?;
        let response = admit_verify(&e)?;
        e.evaluate_proposal(response, true)?;
        let p = e.coding_plan()?;
        e.write(|| {
            e.reserve_project_execution(&p)?;
            e.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::Intent { plan: p.clone() },
                },
                Source::Rules,
            )?;
            Ok(())
        })?;
        let charged = e.project_budget()?.tool_ms_charged;
        assert_eq!(e.project_lifecycle_view()?.incomplete_executions, 1);
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(e.project_budget()?.tool_ms_charged, charged);
        assert!(e
            .coding_execute(
                p.commitment,
                p.source_before,
                &OracleFixture {
                    named_fail: false,
                    full_exit: 0
                }
            )
            .is_err());
        assert!(admit_verify(&e).is_err());
        Ok(())
    }
    #[test]
    fn verification_zero_patch_model_budget_and_reviewed_closure() -> Result<()> {
        for ceiling in [1, 1_000_000] {
            let dir = tempfile::tempdir()?;
            let e = Engine::open(dir.path())?;
            e.project_initialize(oracle_scope())?;
            e.project_acceptance_requirements(BTreeMap::from([(
                "contract".into(),
                "independently reviewed fixture".into(),
            )]))?;
            e.project_lifecycle_open(lifecycle::Admission {
                state_root: e.store.head_root()?,
                tool_ms_ceiling: ceiling,
                reason: "synthetic verification-only budget".into(),
                training: Some(tsr_learn::project_eligibility::Admission {
                    origin: tsr_learn::project_eligibility::Origin::Synthetic,
                    original_task: Hash::of(&[b"verification-fixture"]),
                    family: "fixture".into(),
                    lineage: "synthetic".into(),
                    clone_group: "fixture".into(),
                    reviewer: "fixture-reviewer".into(),
                    no_inference: true,
                }),
                non_tool: Some(budget::Limits {
                    model_calls: 0,
                    model_tokens: 0,
                    wall_ms: 600_000,
                    patch_bytes: 0,
                    external_cost_microusd: 0,
                }),
            })?;
            if ceiling == 1 {
                assert!(admit_verify(&e).is_err());
                continue;
            }
            let response = admit_verify(&e)?;
            e.evaluate_proposal(response, true)?;
            let p = e.coding_plan()?;
            e.coding_execute(
                p.commitment,
                p.source_before,
                &OracleFixture {
                    named_fail: false,
                    full_exit: 0,
                },
            )?;
            assert_eq!(e.project_budget()?.patch_bytes_charged, 0);
            let status = e.project_acceptance_status()?;
            e.project_acceptance_review(acceptance::Review {
                snapshot: status.snapshot,
                requirements_revision: status.requirements_revision.unwrap(),
                test_receipt: status.test_receipt,
                decision: acceptance::Decision::Accept,
                evidence: BTreeMap::from([(
                    "contract".into(),
                    acceptance::Evidence {
                        met: true,
                        evidence: "synthetic independent review fixture; not operational evidence"
                            .into(),
                    },
                )]),
            })?;
            assert!(e.project_lifecycle_close()?.closed);
            let exported = e.project_dataset(&lifecycle::now()?)?;
            let encoded = serde_json::to_vec(&exported)?;
            let reloaded: tsr_learn::project_receipts::Artifact = serde_json::from_slice(&encoded)?;
            assert!(!reloaded.rows.is_empty());
            let projected: lifecycle::Projection =
                serde_json::from_value(reloaded.export_time_lifecycle_inventory)?;
            assert!(projected.events.iter().any(|(_,event)| matches!(&event.kind, EventKind::Proposed {candidate} if candidate.kind == ActionKind::RunApprovedTest)));
            assert_eq!(
                Engine::open(dir.path())?
                    .project_budget()?
                    .patch_bytes_charged,
                0
            );
            assert!(admit_verify(&e).is_err());
        }
        Ok(())
    }
    #[test]
    fn workspace_oracle_counts_once_preserves_named_failures_and_timeout() -> Result<()> {
        for (named_fail, full_exit, expected) in [
            (false, 0, Verdict::Passed),
            (false, 101, Verdict::FailedTests),
            (true, 0, Verdict::FailedTests),
            (false, 124, Verdict::Timeout),
        ] {
            let dir = tempfile::tempdir()?;
            let e = oracle_engine(dir.path())?;
            let p = e.coding_plan()?;
            assert_eq!(p.commands.len(), 4);
            assert!(p.commands[3].contains(&"--workspace".into()));
            let estimate = e.project_cost_estimate(&p.action)?.unwrap();
            assert_eq!(estimate.status, "reservation_bound_fallback");
            let result = e.coding_execute(
                p.commitment,
                p.source_before,
                &OracleFixture {
                    named_fail,
                    full_exit,
                },
            )?;
            assert_eq!(result.verdict, expected);
            if full_exit != 124 {
                let records = e.records()?;
                let (success, input) = records
                    .iter()
                    .rev()
                    .find_map(|(_, r)| match r {
                        Record::Returned { success, input, .. } => Some((*success, input)),
                        _ => None,
                    })
                    .unwrap();
                assert_eq!(success, !named_fail && full_exit == 0);
                assert_eq!(
                    input
                        .met
                        .contains(&case_criterion("full_workspace_oracle@v1")),
                    full_exit == 0
                );
                assert_eq!(
                    input.met.contains(&case_criterion("tests::value")),
                    !named_fail
                );
            }
            let budget = e.project_budget()?;
            assert_eq!(
                budget.tool_ms_charged,
                if full_exit == 124 { 41 } else { 28 }
            );
            assert!(e
                .coding_execute(
                    p.commitment,
                    p.source_before,
                    &OracleFixture {
                        named_fail,
                        full_exit
                    }
                )
                .is_err());
            let reopened = Engine::open(dir.path())?;
            assert_eq!(
                reopened.project_budget()?.tool_ms_charged,
                budget.tool_ms_charged
            );
            let cost = e.project_cost_estimate(&p.action)?.unwrap();
            assert_eq!(
                cost.status,
                if full_exit == 124 {
                    "reservation_bound_fallback"
                } else {
                    "empirical_complete_path_set"
                }
            );
            assert_eq!(
                serde_json::to_value(&cost)?,
                serde_json::to_value(reopened.project_cost_estimate(&p.action)?.unwrap())?
            );
            if full_exit != 124 {
                assert_eq!(cost.expected_tool_ms, 28.0);
                assert_eq!(cost.samples.len(), 1);
            }
        }
        Ok(())
    }
    #[test]
    #[ignore = "explicit installed Docker image only; no pull/network/inference"]
    fn installed_workspace_oracle_accounting() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = oracle_engine(dir.path())?;
        let p = e.coding_plan()?;
        let result = e.coding_run(p.commitment, p.source_before)?;
        ensure!(
            result.verdict == Verdict::Passed,
            "isolated oracle failed: {:?}",
            result.verdict
        );
        let budget = e.project_budget()?;
        ensure!(
            budget.tool_ms_charged > 0 && budget.unknown_usage == 0,
            "oracle usage missing"
        );
        ensure!(result.receipts.len() == 4, "missing oracle stage");
        println!(
            "isolated full-workspace accounting: {} measured ms, {} receipts",
            budget.tool_ms_charged,
            result.receipts.len()
        );
        Ok(())
    }
    #[test]
    #[ignore = "explicit installed dependency image only; no pull/network/inference"]
    fn installed_preparation_accounting() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(locked_scope())?;
        e.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 100_000,
            reason: "installed offline dependency check fixture".into(),
            non_tool: Some(budget::Limits {
                model_calls: 1,
                model_tokens: 1000,
                wall_ms: 600_000,
                patch_bytes: 100_000,
                external_cost_microusd: 0,
            }),
        })?;
        let prepared = e.project_prepare()?;
        ensure!(
            prepared["ready"] == true,
            "offline preparation failed: {prepared}"
        );
        let charged = e.project_budget()?.tool_ms_charged;
        ensure!(charged > 0, "missing measured preparation duration");
        ensure!(
            e.project_prepare()?["reused"] == true,
            "preparation not reused"
        );
        ensure!(
            e.project_budget()?.tool_ms_charged == charged,
            "preparation double charge"
        );
        println!("isolated preparation accounting: {charged} measured ms, reuse unchanged");
        Ok(())
    }
    #[test]
    fn full_classifier_handles_group_collisions_and_refuses_incomplete_counts() {
        let group="running 1 test\ntest tests::same ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n";
        assert_eq!(
            classify_full(&RunResult::from_output(
                0,
                format!("{group}{group}").as_bytes(),
                b"",
                1
            )),
            Verdict::Passed
        );
        assert_eq!(
            classify_full(&RunResult::from_output(
                0,
                group.replace("1 passed", "0 passed").as_bytes(),
                b"",
                1
            )),
            Verdict::Unavailable
        );
        assert_eq!(classify_full(&RunResult::from_output(0,b"running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n",b"",1)),Verdict::Unavailable);
        assert_eq!(
            classify_full(&RunResult::from_output(0, &vec![b'x'; 1 << 20], b"", 1)),
            Verdict::Unavailable
        );
        assert_eq!(
            classify_full(&RunResult::from_output(124, group.as_bytes(), b"", 1)),
            Verdict::Timeout
        );
    }
    #[test]
    fn preparation_accounting_preserves_failed_and_uncertain_work() -> Result<()> {
        struct Timed(i32);
        impl Runner for Timed {
            fn run(&self, _: &Path, _: &[String], _: &Limits) -> RunResult {
                RunResult::from_output(self.0, b"", b"", 17)
            }
        }
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let s = locked_scope();
        e.project_initialize(s.clone())?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 100_000,
            reason: "prerequisite accounting fixture".into(),
            training: None,
            non_tool: Some(budget::Limits {
                model_calls: 1,
                model_tokens: 1000,
                wall_ms: 600_000,
                patch_bytes: 100_000,
                external_cost_microusd: 0,
            }),
        })?;
        assert_eq!(e.prepare_dependencies(&s, &Timed(101))?["ready"], false);
        assert_eq!(e.project_budget()?.tool_ms_charged, 17);
        assert_eq!(e.prepare_dependencies(&s, &Timed(0))?["ready"], true);
        assert_eq!(e.project_budget()?.tool_ms_charged, 34);
        assert_eq!(e.prepare_dependencies(&s, &Timed(0))?["reused"], true);
        let reopened = Engine::open(dir.path())?;
        assert_eq!(reopened.project_budget()?.tool_ms_charged, 34);
        e.reserve_project_preparation(Hash::of(&[b"uncertain"]), 1000)?;
        assert_eq!(e.project_budget()?.tool_ms_charged, 1034);
        assert_eq!(e.project_budget()?.unresolved_tool_requests, 1);
        assert!(e
            .reserve_project_preparation(Hash::of(&[b"retry"]), 1000)
            .is_err());
        assert_eq!(
            Engine::open(dir.path())?.project_budget()?.tool_ms_charged,
            1034
        );
        Ok(())
    }
    #[test]
    fn named_workspace_preserves_inheritance_and_rejects_external_dependencies() {
        let mut spec = scope();
        spec.named_package = Some("core".into());
        spec.editable = BTreeMap::from([(
            "crates/core/src/lib.rs".into(),
            "pub fn value() -> u8 { 0 }".into(),
        )]);
        spec.edit_windows = BTreeMap::from([(
            "crates/core/src/lib.rs".into(),
            EditWindow { start: 23, end: 24 },
        )]);
        spec.protected = BTreeMap::from([
            ("Cargo.toml".into(), "[workspace]\nmembers = [\"crates/core\"]\n[workspace.package]\nversion = \"1.0.0\"\nedition = \"2021\"\n".into()),
            ("crates/core/Cargo.toml".into(), "[package]\nname = \"core\"\nversion.workspace = true\nedition.workspace = true\n".into()),
            ("Cargo.lock".into(), "version = 4\n[[package]]\nname = \"core\"\nversion = \"1.0.0\"\n".into()),
        ]);
        spec.cases.truncate(1);
        spec.validate().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::open(dir.path()).unwrap();
        engine.project_initialize(spec.clone()).unwrap();
        let context = engine.project_context(&spec).unwrap();
        assert_eq!(context["files"].as_array().unwrap().len(), 1);
        assert_eq!(context["files"][0]["source"], "0");
        assert!(serde_json::to_vec(&context).unwrap().len() < 32768);
        let mut too_long = spec.clone();
        too_long.timeout_ms = 600001;
        assert!(too_long.validate().is_err());
        let manifest = spec.protected["crates/core/Cargo.toml"].clone();
        for dep in [
            "{ git = \"https://example.invalid/repo\" }",
            "{ path = \"../../../outside\" }",
            "{ path = \"/outside\" }",
        ] {
            spec.protected.insert(
                "crates/core/Cargo.toml".into(),
                format!("{manifest}\n[dependencies]\nescape = {dep}\n"),
            );
            assert!(workspace::validate(&spec).is_err());
        }
        spec.protected
            .insert("crates/core/Cargo.toml".into(), manifest);
        spec.edit_windows.clear();
        assert!(workspace::validate(&spec).is_err());
    }
    #[test]
    fn preprepared_large_lock_inventory_is_bounded_separately_from_context() {
        let mut spec = locked_scope();
        // Three original packages plus 509 checksum-locked metadata entries.
        for i in 0..509 {
            spec.protected.get_mut("Cargo.lock").unwrap().push_str(&format!("\n[[package]]\nname = \"cached{i}\"\nversion = \"1.0.0\"\nsource = \"{REGISTRY}\"\nchecksum = \"{}\"\n", "a".repeat(64)));
        }
        assert!(spec.protected["Cargo.lock"].len() > 84080);
        spec.validate().unwrap();
        spec.protected
            .get_mut("Cargo.lock")
            .unwrap()
            .push_str("\n[[package]]\nname = \"too-many\"\nversion = \"1.0.0\"\n");
        assert!(spec.validate().is_err());
    }
    #[test]
    fn locked_dependency_authority_is_bounded() {
        let original = locked_scope();
        original.validate().unwrap();
        for replacement in [
            "\"1.0.18\"",
            "{ git = \"https://example.invalid/repo\" }",
            "{ path = \"../outside\" }",
            "\"=1.0.19\"",
        ] {
            let mut spec = original.clone();
            let manifest = spec.protected.get_mut("Cargo.toml").unwrap();
            *manifest = manifest.replace("\"=1.0.18\"", replacement);
            assert!(spec.validate().is_err());
        }
        for (old, new) in [
            (REGISTRY, "git+https://example.invalid/repo"),
            (
                "8f42a60cbdf9a97f5d2305f08a87dc4e09308d1276d28c869c684d7777685682",
                "missing",
            ),
        ] {
            let mut spec = original.clone();
            let lock = spec.protected.get_mut("Cargo.lock").unwrap();
            *lock = lock.replace(old, new);
            assert!(spec.validate().is_err());
        }
    }
    struct Fetch {
        exit: i32,
    }
    impl Runner for Fetch {
        fn run(&self, _: &Path, argv: &[String], _: &Limits) -> RunResult {
            assert_eq!(argv, strings(&["cargo", "fetch", "--locked", "--offline"]));
            RunResult::exit_only(self.exit)
        }
    }
    #[test]
    fn preparation_failure_cannot_unlock_repairs_and_success_survives_restart() {
        let dir = tempfile::tempdir().unwrap();
        let spec = locked_scope();
        let e = Engine::open(dir.path()).unwrap();
        e.project_initialize(spec.clone()).unwrap();
        assert!(!e.dependencies_ready(&spec).unwrap());
        let snapshot = e.project_snapshot(&spec).unwrap();
        let patch = ExactPatch {
            path: "src/pricing.rs".into(),
            source_digest: Hash::of(&[snapshot["src/pricing.rs"].as_bytes()]),
            inspection_handle: e
                .project_handle(&spec, &snapshot, "src/pricing.rs")
                .unwrap(),
            old_text: "unit_cents + quantity".into(),
            new_text: "unit_cents * quantity".into(),
        };
        assert!(e.project_validate_patch(&spec, &patch).is_err());
        assert_eq!(
            e.prepare_dependencies(&spec, &Fetch { exit: 101 }).unwrap()["ready"],
            false
        );
        assert!(!e.dependencies_ready(&spec).unwrap());
        assert_eq!(
            e.prepare_dependencies(&spec, &Fetch { exit: 0 }).unwrap()["ready"],
            true
        );
        drop(e);
        let e = Engine::open(dir.path()).unwrap();
        assert!(e.dependencies_ready(&spec).unwrap());
        assert!(e.project_validate_patch(&spec, &patch).is_ok());
        assert_eq!(
            e.prepare_dependencies(&spec, &Fetch { exit: 101 }).unwrap()["reused"],
            true
        );
        let mut other = spec.clone();
        other.image = format!("sha256:{}", "f".repeat(64));
        assert!(!e.dependencies_ready(&other).unwrap());
        fs::write(dir.path().join("coding-work/Cargo.lock"), "changed").unwrap();
        assert!(e.prepare_dependencies(&spec, &Fetch { exit: 0 }).is_err());
    }
    #[test]
    fn cargo_scope_refuses_dependency_build_and_test_authority() {
        let original = scope();
        original.validate().unwrap();
        for extra in [
            "\n[dependencies]\nanything = \"1\"\n",
            "\n[lib]\nproc-macro = true\n",
            "\n[[test]]\nname = \"outside\"\npath = \"/host/test.rs\"\n",
        ] {
            let mut s = original.clone();
            s.protected.get_mut("Cargo.toml").unwrap().push_str(extra);
            assert!(s.validate().is_err());
        }
        let mut s = original.clone();
        s.protected.insert("build.rs".into(), "fn main() {}".into());
        assert!(s.validate().is_err());
        let mut s = original;
        let test = s.protected.remove("tests/invoice.rs").unwrap();
        s.editable.insert("tests/invoice.rs".into(), test);
        assert!(s.validate().is_err());
    }
    #[test]
    fn cargo_reports_require_complete_inventory_and_consistent_exit() {
        let cases = vec![ProbeCase {
            id: "check".into(),
            args: vec![],
            expected_stdout: String::new(),
        }];
        let green = b"running 1 test\ntest check ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n";
        assert_eq!(
            classify(&RunResult::from_output(0, green, b"", 0), &cases).0,
            Verdict::Passed
        );
        let red = b"running 1 test\ntest check ... FAILED\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n";
        assert_eq!(
            classify(&RunResult::from_output(101, red, b"", 0), &cases).0,
            Verdict::FailedTests
        );
        for (exit, output) in [(101, green.as_slice()), (0, red.as_slice()), (0, b"running 1 test\ntest check ... ok"), (0, b"running 0 tests\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n")] {
            assert_eq!(classify(&RunResult::from_output(exit, output, b"", 0), &cases).0, Verdict::Unavailable);
        }
        assert_eq!(
            classify(
                &RunResult::from_output(0, green, &vec![b'x'; 4093], 0),
                &cases
            )
            .0,
            Verdict::Unavailable
        );
    }
    #[test]
    fn cargo_import_restart_export_preserves_protected_scope() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("engine");
        let e = Engine::open(&root).unwrap();
        let spec = scope();
        e.project_initialize(spec.clone()).unwrap();
        let edited = root.join("coding-work/src/pricing.rs");
        fs::write(edited, "pub fn subtotal(a: u32, b: u32) -> u32 { a * b }\n").unwrap();
        drop(e);
        let e = Engine::open(&root).unwrap();
        let export = e.project_export().unwrap();
        let patch = fs::read_to_string(export["patch"].as_str().unwrap()).unwrap();
        assert!(patch.contains("--- a/src/pricing.rs\n+++ b/src/pricing.rs"));
        assert!(patch.contains("+pub fn subtotal(a: u32, b: u32) -> u32 { a * b }"));
        fs::write(root.join("coding-work/Cargo.toml"), "changed").unwrap();
        assert!(e.project_export().is_err());
    }
}
