//! Test-binary-only worker. No target code, container, Cargo command or model is run.
//! Every outcome is an evaluator-authored protocol fixture, not B9.1 evidence.
use super::*;
use std::cell::Cell;

struct FixtureRunner {
    plan: Plan,
    stage: String,
    commands: Cell<usize>,
    interrupt: bool,
    overrun: bool,
}
impl Runner for FixtureRunner {
    fn run(&self, copy: &Path, argv: &[String], _: &Limits) -> RunResult {
        // A deliberately impossible accounting fixture, not elapsed runtime or relaxed limits.
        let duration = if self.overrun { 60_001 } else { 1 };
        let index = self.commands.get();
        assert_eq!(argv, self.plan.commands[index]);
        self.commands.set(index + 1);
        if self.interrupt {
            panic!("evaluator-authored interruption after native dispatch intent");
        }
        if index == 0 {
            assert_eq!(
                self.plan.edits.len(),
                usize::from(matches!(self.stage.as_str(), "seed" | "patch" | "failed"))
            );
            for edit in &self.plan.edits {
                assert_eq!(edit.path, "src/lib.rs");
                assert!(edit.replacement.len() <= 2048);
                // Apply only the already committed exact native plan, never evaluate a shell.
                fs::write(copy.join(&edit.path), &edit.replacement).unwrap();
            }
            return RunResult::from_output(0, b"", b"", duration);
        }
        assert_eq!(argv[0], "cargo");
        if index == 1 {
            assert!(argv.iter().any(|a| a == "--no-run"));
            return RunResult::from_output(0, b"", b"", duration);
        }
        assert_eq!(index, 2);
        assert!(!argv.iter().any(|a| a == "--no-run"));
        let seed = matches!(self.stage.as_str(), "seed" | "check_before");
        let mut failed = 0;
        let mut text = format!("running {} tests\n", self.plan.cases.len());
        for case in &self.plan.cases {
            let passes = (self.stage != "failed" || case.id != self.plan.cases.last().unwrap().id)
                && (!seed
                    || matches!(
                        case.id.as_str(),
                        "empty_input_handled" | "requested_behavior_handled"
                    ));
            failed += usize::from(!passes);
            text.push_str(&format!(
                "test {} ... {}\n",
                case.id,
                if passes { "ok" } else { "FAILED" }
            ));
        }
        text.push_str(&format!("test result: {}. {} passed; {} failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n",
            if failed == 0 { "ok" } else { "FAILED" }, self.plan.cases.len() - failed, failed));
        RunResult::from_output(
            if failed == 0 { 0 } else { 101 },
            text.as_bytes(),
            b"evaluator-authored fake Cargo result; not executed",
            duration,
        )
    }
}

/// Invoked only as a child of the deterministic Python integration harness. The
/// ordinary library sweep does not read external environment or perform any work.
#[test]
fn fixture_worker() -> Result<()> {
    let Some(directory) = std::env::var_os("TSR_B91_FIXTURE_DIRECTORY") else {
        return Ok(());
    };
    let directory = PathBuf::from(directory).canonicalize()?;
    let root = directory
        .parent()
        .ok_or_else(|| anyhow::anyhow!("fixture root missing"))?;
    ensure!(
        fs::read_to_string(root.join("fixture-only.marker"))? == "b91-native-fixture-only@v1\n",
        "explicit disposable fixture marker required"
    );
    let output = PathBuf::from(
        std::env::var_os("TSR_B91_FIXTURE_OUTPUT")
            .ok_or_else(|| anyhow::anyhow!("fixture output required"))?,
    );
    ensure!(
        output
            .parent()
            .is_some_and(|p| p.canonicalize().is_ok_and(|p| p == root)),
        "fixture output outside root"
    );
    let stage = std::env::var("TSR_B91_FIXTURE_STAGE")?;
    ensure!(
        matches!(
            stage.as_str(),
            "seed" | "check_before" | "patch" | "check_after" | "failed"
        ),
        "unknown fixture stage"
    );
    eprintln!("fixture stage: open native state");
    let engine = Engine::open(&directory)?;
    eprintln!("fixture stage: read committed plan");
    let plan = engine.coding_plan()?;
    ensure!(
        plan.commands.len() == 3 && plan.timeout_ms == 20000,
        "unexpected fixture command limits"
    );
    let expected_cases = if stage.starts_with("check_") { 3 } else { 2 };
    ensure!(
        plan.cases.len() == expected_cases,
        "fixture case inventory mismatch"
    );
    let spec = engine.project_configuration()?;
    ensure!(
        spec.cargo && !spec.full_workspace_oracle && spec.named_package.is_none(),
        "fixture scope mismatch"
    );
    let runner = FixtureRunner {
        plan: plan.clone(),
        stage,
        commands: Cell::new(0),
        interrupt: std::env::var_os("TSR_B91_FIXTURE_INTERRUPT").is_some(),
        overrun: std::env::var_os("TSR_B91_FIXTURE_OVERRUN").is_some(),
    };
    eprintln!("fixture stage: execute authored runner");
    let result = engine.coding_execute(plan.commitment, plan.source_before, &runner)?;
    eprintln!("fixture stage: authored return complete");
    ensure!(
        runner.commands.get() == 3,
        "fixture did not traverse all native command slots"
    );
    if runner.stage == "seed" {
        let aims = engine.acceptance_current_aims()?;
        ensure!(aims.len() == 2, "both original desires must remain present");
        for id in aims.values() {
            ensure!(
                result
                    .state
                    .preferences
                    .iter()
                    .any(|p| p.preference.id == *id && p.is_open()),
                "expected failure must preserve both open desires"
            );
        }
        let return_event = engine
            .events()?
            .into_iter()
            .rev()
            .find_map(|(_, e)| match e.kind {
                EventKind::Observed {
                    observation:
                        Observation::Return {
                            surprise,
                            contrast: Some(contrast),
                        },
                } => Some((surprise, contrast)),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("native return missing Surprise/Contrast"))?;
        ensure!(
            return_event.1.unmet
                == vec![(
                    CriterionId(Hash::of(&[b"test:existing_behaviour_unchanged"])),
                    UnmetKind::WrongBehaviour
                )],
            "Contrast must identify the unmet second dimension, not a scalar"
        );
        ensure!(
            return_event.0.len() == 2 && return_event.0[0].forecast != return_event.0[1].forecast,
            "evaluator and controller expectations must remain separate"
        );
        let evaluator = engine.project_evaluator_forecasts()?;
        let resolution = evaluator["records"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r.get("residual").is_some())
            .ok_or_else(|| anyhow::anyhow!("evaluator resolution missing"))?;
        ensure!(
            return_event
                .0
                .iter()
                .any(|s| serde_json::to_value(s.forecast).ok()
                    == Some(resolution["forecast"]["id"].clone())
                    && s.residual == 0.1),
            "exact evaluator-authored low residual missing"
        );
    }

    let value = serde_json::json!({"fixture_only":true,"target_commands_executed":0,"simulated_command_slots":runner.commands.get(),"execution":result,"status":engine.project_status()?});
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    serde_json::to_writer(&mut file, &value)?;
    file.sync_all()?;
    Ok(())
}

/// Explicit cross-language integration: build the normal tsr binary before running.
/// No provider or target execution; the child worker remains in this test binary.
#[test]
#[ignore = "requires prebuilt normal tsr binary and Python; run explicitly for B9.1 integration"]
fn native_driver_sequence() -> Result<()> {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    ensure!(
        repository.join("target/debug/tsr").is_file(),
        "build normal tsr CLI first"
    );
    let result = std::process::Command::new("python3")
        .current_dir(repository)
        .env("PYTHONPATH", "scripts")
        .env("TSR_B91_TEST_WORKER_BIN", std::env::current_exe()?)
        .args([
            "-m",
            "unittest",
            "test_engine_b91_driver.NativeSequence.test_full_native_sequence",
        ])
        .output()?;
    ensure!(
        result.status.success(),
        "native fixture driver failed:\n{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
