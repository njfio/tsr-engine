//! D3 (item 2): the oracle judges the copy with the snapshot's tests.
//!
//! The defect: no code restored the oracle files, so a session that deleted the flipped test, edited
//! its assertion or renamed it passed with the bug still present (review table at 94775da: deleted ->
//! pass, edited -> pass, renamed -> pass with the filter matching nothing).
//!
//! The fix: each task records the repository paths of the files that define its oracle tests; before
//! the oracle runs, those files are written into the copy from the snapshot, overwriting the
//! session's versions; a filter that matches no test is a failure, not a pass.
//!
//! This test drives the built `tsr` binary end to end, with **no** `--scripted-runner`, over a real
//! repository mined in the test. It makes the review's table true:
//!
//! | Variant | Expected |
//! |---|---|
//! | bug present | fail |
//! | bug fixed | pass |
//! | test deleted, bug present | fail |
//! | assertion edited, bug present | fail |
//! | test renamed, bug present | fail |

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

fn git(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_AUTHOR_NAME", "fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?} failed");
}

fn write(repo: &Path, rel: &str, body: &str) {
    let path = repo.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, body).expect("write");
}

/// A two-commit repository whose root has a red test and whose second commit makes it green.
fn two_commit_repo(dir: &Path) -> PathBuf {
    let repo = dir.join("repo");
    fs::create_dir_all(&repo).expect("mkdir repo");
    git(&repo, &["init", "-q", "-b", "main"]);

    write(
        &repo,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.0.1\"\nedition = \"2021\"\n\n[workspace]\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(
        &repo,
        "src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 { a - b }\n",
    );
    write(
        &repo,
        "tests/demo.rs",
        "#[test]\nfn adds() { assert_eq!(demo::add(1, 1), 2); }\n",
    );
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "root: red test"]);

    write(
        &repo,
        "src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    );
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "flip: fix add"]);

    repo
}

/// A stand-in `tsr` on `PATH` that runs `tsr exec -- <argv>` by executing `<argv>` in its own
/// working directory (the copy the `ExecRunner` set as cwd), and delegates every other subcommand
/// to the real binary.
fn standin_tsr(dir: &Path, real: &Path) -> PathBuf {
    let path = dir.join("tsr");
    let script = format!(
        "#!/bin/sh\n\
         if [ \"$1\" = \"exec\" ] && [ \"$2\" = \"--\" ]; then\n\
         \tshift 2\n\
         \texec \"$@\"\n\
         fi\n\
         exec \"{}\" \"$@\"\n",
        real.display()
    );
    fs::write(&path, script).expect("write stand-in tsr");
    let mut perms = fs::metadata(&path).expect("stat").permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod");
    path
}

/// A stand-in suite the mine run invokes through `tsr exec`: it runs the real `cargo test` in the
/// snapshot (its cwd) and writes the suite report mine reads, naming the one test `adds`.
fn standin_suite(dir: &Path) -> PathBuf {
    let path = dir.join("suite.sh");
    let script = "#!/bin/sh\n\
         CARGO_TARGET_DIR=\"$PWD/target\" cargo test --quiet -- adds >/dev/null 2>&1\n\
         if [ $? -eq 0 ]; then\n\
         \tpassed=true\n\
         else\n\
         \tpassed=false\n\
         fi\n\
         printf '{\"tests\":[{\"name\":\"adds\",\"passed\":%s}]}' \"$passed\" > .tsr-suite-report.json\n\
         [ \"$passed\" = true ]\n";
    fs::write(&path, script).expect("write stand-in suite");
    let mut perms = fs::metadata(&path).expect("stat").permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("chmod");
    path
}

/// The scratch corpus, mined once, shared by every row of the table.
struct Fixture {
    scratch: PathBuf,
    corpus: PathBuf,
    task_id: String,
    path_env: String,
}

fn fixture(name: &str) -> Fixture {
    let ws = workspace();
    let scratch = ws.join("target").join(name);
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();

    let repo = two_commit_repo(&scratch);
    let corpus = scratch.join("corpus");
    let standin_dir = scratch.join("bin");
    fs::create_dir_all(&standin_dir).unwrap();
    let _standin = standin_tsr(&standin_dir, &tsr_bin());
    let suite = standin_suite(&standin_dir);
    let path_env = format!(
        "{}:{}",
        standin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    let mine = Command::new(tsr_bin())
        .args([
            "corpus",
            "mine",
            repo.to_str().unwrap(),
            "--out",
            corpus.to_str().unwrap(),
            "--suite",
            suite.to_str().unwrap(),
        ])
        .env("PATH", &path_env)
        .output()
        .expect("run tsr corpus mine");
    assert!(
        mine.status.success(),
        "mining must succeed; stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&mine.stdout),
        String::from_utf8_lossy(&mine.stderr)
    );

    let manifest = fs::read_to_string(corpus.join("tasks.jsonl")).expect("tasks.jsonl");
    let task: serde_json::Value =
        serde_json::from_str(manifest.lines().next().expect("one task")).expect("task json");
    let task_id = task["id"].as_str().expect("task id").to_string();

    // The task must record the oracle-test file the commit's diff touched (D3, item 2).
    let files = task["oracle_test_files"]
        .as_array()
        .expect("oracle_test_files must be recorded by mine");
    assert!(
        files.iter().any(|f| f.as_str() == Some("tests/demo.rs")),
        "mine must record tests/demo.rs as an oracle-test file; got {files:?}"
    );

    Fixture {
        scratch,
        corpus,
        task_id,
        path_env,
    }
}

/// A scripted session whose step applies `edit`, then stops.
fn session_with(edit: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "name": "d3-oracle-table",
        "steps": [
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": "src/lib.rs" },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            },
            {
                "kind": "propose_patch",
                "args": { "arg": "patch_bytes", "bytes": 64 },
                "forecast": 0.7, "forecast_event": "diff_accepted", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0,
                "edit": edit
            },
            {
                "kind": "stop",
                "args": { "arg": "stop" },
                "forecast": null, "forecast_event": null, "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            }
        ]
    })
}

/// Run one row of the table and return the recorded outcome for the task.
fn run_row(fx: &Fixture, edit: serde_json::Value) -> String {
    let sessions = fx.corpus.join("sessions");
    fs::create_dir_all(&sessions).unwrap();
    fs::write(
        sessions.join(format!("{}-record.json", fx.task_id)),
        serde_json::to_string_pretty(&session_with(edit)).unwrap(),
    )
    .unwrap();

    let out = fx.scratch.join("out");
    let _ = fs::remove_dir_all(&out);
    let run = Command::new(tsr_bin())
        .args([
            "task",
            "run",
            "--mode",
            "record",
            "--tasks",
            fx.corpus.join("tasks.jsonl").to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            out.to_str().unwrap(),
            "--profile",
            workspace().join("profiles/default.toml").to_str().unwrap(),
        ])
        .env("PATH", &fx.path_env)
        .output()
        .expect("run tsr task run");
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        run.status.code(),
        Some(0),
        "the batch must run; stdout: {stdout}\nstderr: {stderr}"
    );

    let results = fs::read_to_string(out.join("results-record.jsonl")).expect("results");
    let line = results.lines().next().expect("one result");
    let value: serde_json::Value = serde_json::from_str(line).expect("result json");
    value["outcome"].as_str().expect("outcome").to_string()
}

/// The review's table, row by row, through `tsr task run`.
#[test]
fn d3_oracle_table_bug_present_fails() {
    let fx = fixture("d3-oracle-table-present");
    let outcome = run_row(
        &fx,
        serde_json::json!({ "path": "src/lib.rs", "contents": "pub fn add(a: i32, b: i32) -> i32 { a - b }\n" }),
    );
    assert_eq!(outcome, "rejected", "bug present must be rejected");
}

#[test]
fn d3_oracle_table_bug_fixed_passes() {
    let fx = fixture("d3-oracle-table-fixed");
    let outcome = run_row(
        &fx,
        serde_json::json!({ "path": "src/lib.rs", "contents": "pub fn add(a: i32, b: i32) -> i32 { a + b }\n" }),
    );
    assert_eq!(outcome, "accepted", "bug fixed must be accepted");
}

#[test]
fn d3_oracle_table_test_deleted_fails() {
    let fx = fixture("d3-oracle-table-deleted");
    let outcome = run_row(
        &fx,
        serde_json::json!({ "path": "tests/demo.rs", "contents": "" }),
    );
    assert_eq!(
        outcome, "rejected",
        "a deleted test must be restored from the snapshot and the bug rejected"
    );
}

#[test]
fn d3_oracle_table_assertion_edited_fails() {
    let fx = fixture("d3-oracle-table-edited");
    let outcome = run_row(
        &fx,
        serde_json::json!({
            "path": "tests/demo.rs",
            "contents": "#[test]\nfn adds() { assert_eq!(demo::add(1, 1), 0); }\n"
        }),
    );
    assert_eq!(
        outcome, "rejected",
        "an edited assertion must be restored from the snapshot and the bug rejected"
    );
}

#[test]
fn d3_oracle_table_test_renamed_fails() {
    let fx = fixture("d3-oracle-table-renamed");
    let outcome = run_row(
        &fx,
        serde_json::json!({
            "path": "tests/demo.rs",
            "contents": "#[test]\nfn zz_moved() { assert_eq!(demo::add(1, 1), 2); }\n"
        }),
    );
    assert_eq!(
        outcome, "rejected",
        "a renamed test must be restored from the snapshot and the bug rejected"
    );
}
