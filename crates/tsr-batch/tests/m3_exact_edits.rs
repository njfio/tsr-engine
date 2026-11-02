//! End-to-end synthetic evidence only: candidate code is never executed.
use serde_json::{json, Value};
use std::{
    cell::{Cell, RefCell},
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use tsr_batch::{
    broker_v2::{Broker, BrokerDriver, Completion, Request, TaskContext},
    driver::Mode,
    results::Outcome,
    run::{run_one, BatchConfig},
    task::{read_tasks, CorpusTask},
};
use tsr_corpus::{Limits, RunResult, Runner};
use tsr_domain::Hash;
const FILE: &str = "crates/rts-daemon/src/error.rs";
const TEST: &str = "error::tests::protocol_error_round_trips_to_wire";
const SOURCE: &str = "pub fn value() -> u8 { 0 }\n// aaa; duplicate duplicate\n#[cfg(test)] mod tests { #[test] fn protected() { assert_eq!(1, 1); } }\n";
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn task_fixture(dir: &Path) -> CorpusTask {
    fs::create_dir_all(dir.join("crates/rts-daemon/src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(dir.join(FILE), SOURCE).unwrap();
    fs::write(dir.join("tests/oracle.rs"), "mod error { mod tests { #[test] fn protocol_error_round_trips_to_wire() { assert_eq!(1, 1); } } }").unwrap();
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "user.name=fixture",
                "-c",
                "user.email=fixture@example.invalid",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-qm", "inert fixture"]);
    let mut task = read_tasks(&root().join("fixtures/m3/batch/tasks.jsonl"))
        .unwrap()
        .remove(0);
    task.id = "0c5a91a6aaa862c279aed1e1602a74eda7c42c4fc0f72247de513fa948f50b70".into();
    task.repo = dir.to_string_lossy().into();
    task.parent_snapshot = Some(git(&["rev-parse", "HEAD^{tree}"]));
    task.flipped_tests = vec![TEST.into()];
    task.oracle_test_files = vec!["tests/oracle.rs".into()];
    task
}
struct FixtureRunner {
    tests: Cell<usize>,
}
impl Runner for FixtureRunner {
    fn run(&self, dest: &Path, argv: &[String], _: &Limits) -> RunResult {
        if let Some((repo, snapshot, dest)) = tsr_corpus::provision_argv_parts(argv) {
            tsr_corpus::mine::materialize(&repo, &snapshot, &dest).unwrap();
            return RunResult::exit_only(0);
        }
        self.tests.set(self.tests.get() + 1);
        let passed = fs::read_to_string(dest.join(FILE))
            .unwrap()
            .contains("-> u8 { 1 }");
        let status = if passed { "ok" } else { "FAILED" };
        RunResult::from_output(if passed {0} else {101}, format!("running 1 test\ntest {TEST} ... {status}\ntest result: {status}. {} passed; {} failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n", usize::from(passed), usize::from(!passed)).as_bytes(), b"inert runner", 1)
    }
}
struct WireFixture {
    case: &'static str,
    call: usize,
}
impl Broker for WireFixture {
    fn complete(&mut self, request: &Request) -> Result<Completion, String> {
        let n = self.call;
        self.call += 1;
        let mut args = json!({"action":"stop", "path":"", "id":"", "expected_digest":"", "old_text":"", "new_text":"", "forecast":null});
        match n {
            0 => {
                args["action"] = json!("inspect");
                args["path"] = json!(FILE);
            }
            1 | 3 => {
                args["action"] = json!("test");
                args["id"] = json!("named-failing-test");
                args["forecast"] = json!(0.5);
            }
            2 => {
                assert!(request
                    .observation
                    .contains(&Hash::of(&[SOURCE.as_bytes()]).to_hex()));
                args["action"] = json!("edit");
                args["path"] = json!(FILE);
                args["expected_digest"] = json!(Hash::of(&[SOURCE.as_bytes()]).to_hex());
                args["old_text"] = json!("-> u8 { 0 }");
                args["new_text"] = json!("-> u8 { 1 }");
                args["forecast"] = json!(0.5);
                match self.case {
                    "stale" => args["expected_digest"] = json!("0".repeat(64)),
                    "overlap" => {
                        args["old_text"] = json!("aa");
                        args["new_text"] = json!("b");
                    }
                    "duplicate" => {
                        args["old_text"] = json!("duplicate");
                        args["new_text"] = json!("single");
                    }
                    "protected" => {
                        args["old_text"] = json!("assert_eq!(1, 1)");
                        args["new_text"] = json!("assert_eq!(2, 2)");
                    }
                    _ => (),
                }
            }
            _ => (),
        }
        let mut response = json!({"choices":[{"finish_reason":"tool_calls", "message":{"tool_calls":[{"type":"function", "function":{"name":"propose", "arguments":serde_json::to_string(&args).unwrap()}}]}}], "usage":{"total_tokens":1}});
        if n == 2 && self.case == "multiple" {
            let call = response["choices"][0]["message"]["tool_calls"][0].clone();
            response["choices"][0]["message"]["tool_calls"]
                .as_array_mut()
                .unwrap()
                .push(call);
        }
        if n == 2 && self.case == "malformed" {
            response["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"] =
                json!("{");
        }
        // Fixed trusted test program, never a candidate-selected command. It invokes only
        // the actual adapter's encode/decode with credential and HTTP functions disabled.
        let mut child = Command::new("python3")
            .arg("-I")
            .arg(root().join("scripts/test_m3_edit_broker.py"))
            .arg("--decode-fixture")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(
                &serde_json::to_vec(&json!({"request":request,"response":response})).unwrap(),
            )
            .unwrap();
        let out = child.wait_with_output().unwrap();
        if !out.status.success() {
            return Err("inert adapter rejected response".into());
        }
        serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())
    }
}
#[test]
fn exact_edit_wire_through_submission_and_final_receipts_both_modes() {
    for mode in [Mode::Record, Mode::Rules] {
        for case in [
            "valid",
            "stale",
            "overlap",
            "duplicate",
            "protected",
            "multiple",
            "malformed",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let repo = dir.path().join("repo");
            fs::create_dir(&repo).unwrap();
            let task = task_fixture(&repo);
            let mut profile =
                tsr_policy::profile::Profile::load(&root().join("profiles/default.toml")).unwrap();
            profile.tests.truncate(1);
            profile.tests[0].id = "named-failing-test".into();
            profile.budgets.generator_calls = 20;
            let config = BatchConfig::new(dir.path().join("out"), profile);
            let runner = FixtureRunner {
                tests: Cell::new(0),
            };
            let driver = BrokerDriver {
                broker: RefCell::new(WireFixture { case, call: 0 }),
                runner: &runner,
                mode,
                context: TaskContext {
                    required_reads: vec![FILE.into()],
                    failing_test_id: "named-failing-test".into(),
                },
            };
            let result = run_one(&task, mode, &driver, &runner, &config);
            let copy = tsr_batch::run::copies_root(&config.out)
                .join(&task.id)
                .join(mode.system());
            let final_source = fs::read_to_string(copy.join(FILE)).unwrap();
            if matches!(case, "stale" | "overlap" | "duplicate") {
                assert!(result.is_err(), "{case} {mode:?}");
                assert_eq!(final_source, SOURCE);
                assert_eq!(runner.tests.get(), 1);
                continue;
            }
            let result = result.unwrap();
            match case {
                "valid" => {
                    assert_eq!(
                        final_source,
                        SOURCE.replacen("-> u8 { 0 }", "-> u8 { 1 }", 1)
                    );
                    assert_eq!(result.outcome, Outcome::Accepted);
                    let evidence = result.oracle.unwrap().evidence.unwrap();
                    assert!(evidence.verified_success);
                    let manifest: Value = serde_json::from_slice(
                        &tsr_corpus::mine::read_cas(&config.out.join("cas"), &evidence.manifest)
                            .unwrap(),
                    )
                    .unwrap();
                    assert!(tsr_batch::oracle_receipts::verify(
                        &config.out,
                        &evidence.manifest,
                        &task,
                        mode,
                        manifest["state_root"].as_str().unwrap()
                    )
                    .unwrap());
                    assert!(runner.tests.get() > 2);
                }
                "protected" => {
                    assert_eq!(result.outcome, Outcome::Rejected);
                    assert!(result.oracle.is_none());
                    assert_eq!(runner.tests.get(), 2);
                    let store = tsr_store::Store::open(
                        &config
                            .out
                            .join("state")
                            .join(&task.id)
                            .join(format!("{}.sqlite", mode.system())),
                    )
                    .unwrap();
                    let events = store.by_task(&tsr_batch::run::task_id(&task)).unwrap();
                    assert!(events.iter().any(|(_, e)| matches!(&e.kind, tsr_domain::EventKind::TaskClosed {reason, ..} if reason.contains("changed full-suite test evidence"))));
                }
                _ => {
                    assert_eq!(result.outcome, Outcome::Unavailable);
                    assert!(result.oracle.is_none());
                    assert_eq!(final_source, SOURCE);
                    assert_eq!(runner.tests.get(), 1);
                }
            }
        }
    }
}
