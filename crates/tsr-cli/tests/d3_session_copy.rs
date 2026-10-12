//! D3: a session works in the disposable copy of its task, and the oracle judges that copy.
//!
//! The defect (D3): `Driver::drive` received no copy and `run_one` never passed one, so a session's
//! edits never reached `dest` — the directory the oracle runs in. A scripted session that "fixed"
//! the bug left the fix nowhere, and the oracle judged the untouched parent snapshot.
//!
//! This test drives the built `tsr` binary end to end, with **no** `--scripted-runner`: the oracle
//! runs through the production `ExecRunner` (`tsr exec -- <argv>`), which this test satisfies with a
//! stand-in `tsr` on `PATH` that runs the argv in its own working directory. The repository is a
//! real git repository built in the test and mined with `tsr corpus mine`; the task is a real mined
//! task. The scripted session's step edits the source file inside the copy; the oracle then runs the
//! flipped test in that copy and must accept.
//!
//! On the current (buggy) code the edit is never applied to the copy, so the flipped test still
//! fails and the task is rejected — the test fails. After the fix the fix is in the copy and the
//! oracle accepts.

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
///
/// The crate is dependency-free so `cargo test` in a fresh snapshot is quick. The bug is in
/// `src/lib.rs`: `add` returns `a - b` at the root and `a + b` at the flip commit.
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
/// working directory (the snapshot the `ExecRunner` set as cwd), and delegates every other
/// subcommand to the real binary.
///
/// This is the production oracle path with the container replaced by the host: the argv the oracle
/// builds (`cargo test -- <tests>`) really runs, in the copy, so a session that did not fix the copy
/// cannot be accepted.
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

/// D3: a scripted session whose step fixes the bug leaves the fix in the copy, and the oracle then
/// accepts — through `tsr task run` with no `--scripted-runner`.
#[test]
fn d3_session_edits_land_in_the_copy_and_the_oracle_accepts() {
    let ws = workspace();
    let scratch = ws.join("target/d3-session-copy");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();

    let repo = two_commit_repo(&scratch);
    let corpus = scratch.join("corpus");
    let standin_dir = scratch.join("bin");
    fs::create_dir_all(&standin_dir).unwrap();
    let _standin = standin_tsr(&standin_dir, &tsr_bin());
    let suite = standin_suite(&standin_dir);

    // The `PATH` every `tsr` invocation sees: the stand-in first, then the real toolchain.
    let path_env = format!(
        "{}:{}",
        standin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    // 1. Mine the repository. The suite runs through the stand-in `tsr exec`, i.e. `cargo test` in
    //    each snapshot, so the flip is real.
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
        "D3: mining must succeed; stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&mine.stdout),
        String::from_utf8_lossy(&mine.stderr)
    );

    eprintln!(
        "DEBUG mine stdout: {}\nDEBUG mine stderr: {}",
        String::from_utf8_lossy(&mine.stdout),
        String::from_utf8_lossy(&mine.stderr)
    );
    let manifest = fs::read_to_string(corpus.join("tasks.jsonl")).expect("tasks.jsonl");
    let task: serde_json::Value =
        serde_json::from_str(manifest.lines().next().expect("one task")).expect("task json");
    let task_id = task["id"].as_str().expect("task id").to_string();
    let flipped = task["flipped_tests"][0]
        .as_str()
        .expect("flipped test")
        .to_string();

    // 2. A scripted session whose step edits `src/lib.rs` inside the copy to fix the bug.
    let session = serde_json::json!({
        "name": "d3-fix-in-copy",
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
                "edit": {
                    "path": "src/lib.rs",
                    "contents": "pub fn add(a: i32, b: i32) -> i32 { a + b }\n"
                }
            },
            {
                "kind": "stop",
                "args": { "arg": "stop" },
                "forecast": null, "forecast_event": null, "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            }
        ]
    });
    let sessions = corpus.join("sessions");
    fs::create_dir_all(&sessions).unwrap();
    fs::write(
        sessions.join(format!("{task_id}-record.json")),
        serde_json::to_string_pretty(&session).unwrap(),
    )
    .unwrap();

    // 3. Run the task through the production oracle (no `--scripted-runner`).
    let out = scratch.join("out");
    let run = Command::new(tsr_bin())
        .args([
            "task",
            "run",
            "--mode",
            "record",
            "--tasks",
            corpus.join("tasks.jsonl").to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            out.to_str().unwrap(),
            "--profile",
            ws.join("profiles/default.toml").to_str().unwrap(),
        ])
        .env("PATH", &path_env)
        .output()
        .expect("run tsr task run");
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        run.status.code(),
        Some(0),
        "D3: the batch must run; stdout: {stdout}\nstderr: {stderr}"
    );

    // The fix must be in the copy the oracle judged.
    let copy = out.join("copies").join(&task_id).join("record");
    let fixed = fs::read_to_string(copy.join("src/lib.rs")).expect("copy src/lib.rs");
    assert!(
        fixed.contains("a + b"),
        "D3: the session's edit must land in the copy at {}; got: {fixed}",
        copy.display()
    );

    // And the oracle must accept: the flipped test passes in the copy.
    let results = fs::read_to_string(out.join("results-record.jsonl")).expect("results");
    assert!(
        results.contains("\"accepted\""),
        "D3: the oracle must accept the fixed copy (flipped test `{flipped}`); results: {results}"
    );
}
