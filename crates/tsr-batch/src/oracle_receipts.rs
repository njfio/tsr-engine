//! Controller-owned final-oracle receipts. Never mounted into a candidate or sent to a broker.
use crate::{CorpusTask, Mode};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, path::Path};
use tsr_corpus::{mine, Limits, RunResult, Runner};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub manifest: String,
    pub verified_success: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    version: String,
    task: String,
    mode: String,
    snapshot: String,
    tests: Vec<String>,
    state_root: String,
    raw_accepted: bool,
    runs: Vec<RunRef>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RunRef {
    id: String,
    max_output_bytes: usize,
}
/// Persist actual runner outputs before any oracle normalization or integrity verdict.
pub(crate) struct Recorder<'a> {
    pub runner: &'a dyn Runner,
    pub out: &'a Path,
    pub snapshot: &'a str,
    pub runs: RefCell<Vec<RunRef>>,
    pub error: RefCell<Option<String>>,
}
impl Runner for Recorder<'_> {
    fn run(&self, copy: &Path, argv: &[String], limits: &Limits) -> RunResult {
        let result = self.runner.run(copy, argv, limits);
        let saved = if tsr_corpus::digest(&result.stdout) != result.stdout_digest
            || tsr_corpus::digest(&result.stderr) != result.stderr_digest
        {
            Err("runner output digest mismatch".into())
        } else {
            tsr_corpus::persist_run(
                self.out,
                self.snapshot,
                argv,
                &result,
                self.runner.preparation_id(),
            )
        };
        match saved {
            Ok(id) => self.runs.borrow_mut().push(RunRef {
                id,
                max_output_bytes: limits.max_output_bytes,
            }),
            Err(e) => {
                *self.error.borrow_mut() = Some(e);
            }
        }
        result
    }
}
impl Recorder<'_> {
    pub fn finish(
        &self,
        task: &CorpusTask,
        mode: Mode,
        state_root: String,
        accepted: bool,
    ) -> Result<Evidence, String> {
        if let Some(error) = self.error.borrow().as_ref() {
            return Err(error.clone());
        }
        let manifest = Manifest {
            version: "final-oracle-receipts@v1".into(),
            task: task.id.clone(),
            mode: mode.system().into(),
            snapshot: task.snapshot().into(),
            tests: task.failing_tests().to_vec(),
            state_root,
            raw_accepted: accepted,
            runs: self.runs.borrow().clone(),
        };
        let bytes = serde_json::to_vec(&manifest).map_err(|e| e.to_string())?;
        let id = tsr_corpus::digest(&bytes);
        mine::write_cas(&self.out.join("cas"), &id, &bytes).map_err(|e| e.to_string())?;
        let verified_success = verify(self.out, &id, task, mode, &manifest.state_root)?;
        Ok(Evidence {
            manifest: id,
            verified_success,
        })
    }
}
#[derive(Deserialize)]
struct Receipt {
    snapshot: String,
    argv: Vec<String>,
    result: RunResult,
    preparation_id: Option<String>,
}
/// Re-read content-addressed receipts. Missing/corrupt evidence is an error, never success.
/// A raw verdict is insufficient: require every exact test and a completed full suite.
pub fn verify(
    out: &Path,
    id: &str,
    task: &CorpusTask,
    mode: Mode,
    state_root: &str,
) -> Result<bool, String> {
    let read = |id: &str| mine::read_cas(&out.join("cas"), id).map_err(|e| e.to_string());
    let manifest: Manifest = serde_json::from_slice(&read(id)?).map_err(|e| e.to_string())?;
    if manifest.version != "final-oracle-receipts@v1"
        || manifest.task != task.id
        || manifest.mode != mode.system()
        || manifest.snapshot != task.snapshot()
        || manifest.tests != task.failing_tests()
        || manifest.state_root != state_root
    {
        return Err("oracle manifest context mismatch".into());
    }
    // Load even failed receipts so integrity failures are not hidden by a negative verdict.
    let mut results = Vec::new();
    let mut captures_complete = true;
    for run in &manifest.runs {
        let mut receipt: Receipt =
            serde_json::from_slice(&read(&run.id)?).map_err(|e| e.to_string())?;
        receipt.result.stdout = read(&receipt.result.stdout_digest)?;
        receipt.result.stderr = read(&receipt.result.stderr_digest)?;
        if receipt.snapshot != task.snapshot() || receipt.preparation_id != task.preparation_id {
            return Err("oracle run context mismatch".into());
        }
        // UTF-8 truncation can stop up to three bytes before the configured cap.
        // Conservatively refuse verification when either stream could have hit it.
        captures_complete &= receipt.result.stdout.len() < run.max_output_bytes.saturating_sub(3)
            && receipt.result.stderr.len() < run.max_output_bytes.saturating_sub(3);
        results.push(receipt);
    }
    if !captures_complete
        || !manifest.raw_accepted
        || manifest.tests.is_empty()
        || results.len() != manifest.tests.len() + 1
    {
        return Ok(false);
    }
    for (run, name) in results.iter().zip(&manifest.tests) {
        if run.argv != crate::run::exact_test_argv(Path::new("."), name)
            || run.result.exit != 0
            || !String::from_utf8_lossy(&run.result.stdout)
                .lines()
                .any(|l| l.trim() == format!("test {name} ... ok"))
            || !complete_success(&run.result)
        {
            return Ok(false);
        }
    }
    let suite = results.last().unwrap();
    Ok(suite.argv == crate::run::full_suite_argv(Path::new("."))
        && suite.result.exit == 0
        && complete_success(&suite.result))
}

fn complete_success(result: &RunResult) -> bool {
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    crate::admission::completed_suite(&result.stdout) == Ok(0)
        && stdout
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("test result: "))
            .all(|l| l.starts_with("test result: ok. "))
        && !stdout.lines().any(|l| l.trim().ends_with(" ... FAILED"))
        && ![
            "could not compile",
            "could not execute process",
            "process didn't exit successfully",
        ]
        .iter()
        .any(|s| stderr.contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn task() -> CorpusTask {
        let mut task: CorpusTask = serde_json::from_str(
            include_str!("../../../fixtures/m3/batch/tasks.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        task.flipped_tests = vec!["check".into()];
        task.failing_tests.clear();
        task.mutated_snapshot = None;
        task
    }
    fn output() -> &'static [u8] {
        b"running 1 test\ntest check ... ok\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0s\n"
    }
    #[test]
    fn m3_receipts_01_failures_incomplete_and_corrupt_outputs_never_verify() {
        let task = task();
        for bad in [
            RunResult::from_output(124, output(), b"timeout", 100),
            RunResult::exit_only(0),
            RunResult::from_output(0, b"running 1 test\ntest check ... ok\n", b"", 1),
        ] {
            let tmp = tempfile::tempdir().unwrap();
            let runner = tsr_corpus::ScriptedRunner::new(bad.clone());
            let recorder = Recorder {
                runner: &runner,
                out: tmp.path(),
                snapshot: task.snapshot(),
                runs: Default::default(),
                error: Default::default(),
            };
            recorder.run(
                Path::new("unused"),
                &crate::run::exact_test_argv(Path::new("."), "check"),
                &Limits::default(),
            );
            recorder.run(
                Path::new("unused"),
                &crate::run::full_suite_argv(Path::new(".")),
                &Limits::default(),
            );
            // Even a contradictory accepted flag cannot bless failed/incomplete runs.
            let evidence = recorder
                .finish(&task, Mode::Record, "root".into(), true)
                .unwrap();
            assert!(!evidence.verified_success);
            let original = mine::read_cas(&tmp.path().join("cas"), &bad.stderr_digest).unwrap();
            assert_eq!(original, bad.stderr);
            fs_write_corrupt(tmp.path(), &bad.stdout_digest);
            assert!(verify(tmp.path(), &evidence.manifest, &task, Mode::Record, "root").is_err());
        }
    }
    #[test]
    fn m3_receipts_03_capture_limit_and_contradictory_summary_are_not_verified() {
        let task = task();
        let mut cases = (0..=3)
            .map(|margin| {
                (
                    RunResult::from_output(0, output(), b"", 1),
                    output().len() + margin,
                )
            })
            .collect::<Vec<_>>();
        cases.push((RunResult::from_output(0, output(), &[b'x'; 200], 1), 200));
        cases.push((
            RunResult::from_output(0, output(), b"could not compile fixture", 1),
            1024,
        ));
        cases.push((
            RunResult::from_output(
                0,
                String::from_utf8_lossy(output())
                    .replace("test result: ok.", "test result: FAILED.")
                    .as_bytes(),
                b"",
                1,
            ),
            1024,
        ));
        for (result, limit) in cases {
            let tmp = tempfile::tempdir().unwrap();
            let runner = tsr_corpus::ScriptedRunner::new(result);
            let recorder = Recorder {
                runner: &runner,
                out: tmp.path(),
                snapshot: task.snapshot(),
                runs: Default::default(),
                error: Default::default(),
            };
            recorder.run(
                Path::new("unused"),
                &crate::run::exact_test_argv(Path::new("."), "check"),
                &Limits::new(1000, limit),
            );
            recorder.run(
                Path::new("unused"),
                &crate::run::full_suite_argv(Path::new(".")),
                &Limits::new(1000, limit),
            );
            assert!(
                !recorder
                    .finish(&task, Mode::Record, "root".into(), true)
                    .unwrap()
                    .verified_success
            );
        }
    }
    fn fs_write_corrupt(out: &Path, id: &str) {
        std::fs::write(out.join("cas").join(id), b"inert corruption fixture").unwrap();
    }
    #[test]
    fn m3_receipts_02_missing_capture_and_persistence_failure_refuse() {
        let task = task();
        let tmp = tempfile::tempdir().unwrap();
        let mut absent = RunResult::from_output(0, output(), b"stderr", 1);
        absent.stdout.clear();
        let runner = tsr_corpus::ScriptedRunner::new(absent);
        let recorder = Recorder {
            runner: &runner,
            out: tmp.path(),
            snapshot: task.snapshot(),
            runs: Default::default(),
            error: Default::default(),
        };
        recorder.run(
            Path::new("unused"),
            &crate::run::full_suite_argv(Path::new(".")),
            &Limits::default(),
        );
        assert!(recorder
            .finish(&task, Mode::Rules, "root".into(), true)
            .is_err());
        let blocked = tmp.path().join("not-a-directory");
        std::fs::write(&blocked, b"sentinel").unwrap();
        let runner = tsr_corpus::ScriptedRunner::new(RunResult::from_output(0, output(), b"", 1));
        let recorder = Recorder {
            runner: &runner,
            out: &blocked,
            snapshot: task.snapshot(),
            runs: Default::default(),
            error: Default::default(),
        };
        recorder.run(
            Path::new("unused"),
            &crate::run::full_suite_argv(Path::new(".")),
            &Limits::default(),
        );
        assert!(recorder
            .finish(&task, Mode::Rules, "root".into(), true)
            .is_err());
    }
}
