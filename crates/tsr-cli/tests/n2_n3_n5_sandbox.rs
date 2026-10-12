//! N2, N3, N5: the sandbox boundary (PRD §B10 — the disposable copy is the only writable mount).
//!
//! Three defects, one boundary:
//!
//! * **N2** — provisioning ran with `cwd = <out>/copies` and passed relative paths unresolved, so a
//!   relative `--out` landed the copy in `brel/copies/brel/copies/…` and the task was silently
//!   recorded rejected; a repository mined by relative path could not be materialised.
//! * **N3** — provisioning went through `tsr exec`, whose sealed sandbox mounts only its working
//!   directory (the copies root), so a real `tsr exec` could not read the source repository; and the
//!   session/oracle steps were not given the task's copy as their working directory.
//! * **N5** — `tsr corpus mutants` ran the kill run with the operator's repository as the writable
//!   mount and left `mutants.out/` in it.
//!
//! The decision this test pins: provisioning is a **trusted host step** — the runner materialises
//! the snapshot into the task's copy itself (`git archive` from the repository, which is only read),
//! never through `tsr exec`; every `tsr exec` the batch issues runs with `cwd` = that one task's
//! copy; and the mutants kill run happens in a disposable copy materialised from HEAD, never in the
//! repository, leaving nothing in it.
//!
//! The test drives the built `tsr` binary end to end with a **logging stand-in `tsr`** on `PATH`
//! that records its working directory on every `tsr exec -- <argv>` invocation and then runs the
//! argv there. It asserts every recorded cwd is one task copy — never the repository, never the
//! copies root — and that `git status` in the repository is clean after both `mine` and `mutants`.
//! It mines a repository by **relative** path into a **relative** `--out`, so N2 is exercised too.

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

fn chmod_x(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path).expect("stat").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod");
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

/// A stand-in `tsr` on `PATH` that **logs its working directory** on every `tsr exec -- <argv>`
/// invocation, then runs the argv there; every other subcommand is delegated to the real binary.
///
/// The log is the evidence for N3: each line is the cwd of one `tsr exec` the batch issued. The
/// batch must issue them only with `cwd` = one task's copy — never the repository, never the copies
/// root. The log path is read from `$TSR_EXEC_LOG`.
fn logging_standin_tsr(dir: &Path, real: &Path) -> PathBuf {
    let path = dir.join("tsr");
    let script = format!(
        "#!/bin/sh\n\
         if [ \"$1\" = \"exec\" ] && [ \"$2\" = \"--\" ]; then\n\
         \tif [ -n \"$TSR_EXEC_LOG\" ]; then\n\
         \t\tprintf '%s\\n' \"$PWD\" >> \"$TSR_EXEC_LOG\"\n\
         \tfi\n\
         \tshift 2\n\
         \texec \"$@\"\n\
         fi\n\
         exec \"{}\" \"$@\"\n",
        real.display()
    );
    fs::write(&path, script).expect("write stand-in tsr");
    chmod_x(&path);
    path
}

/// A stand-in suite the mine run invokes through `tsr exec`: it writes the suite report mine reads
/// into its working directory (the snapshot), naming the one test `adds`.
///
/// It deliberately does **not** run `cargo`: the outer `cargo test` holds the shared `CARGO_HOME`
/// lock, so a nested cargo would deadlock. The suite's only contract is the report file.
fn standin_suite(dir: &Path) -> PathBuf {
    let path = dir.join("suite.sh");
    // The suite decides from the snapshot's own source: the buggy `a - b` fails, the fixed `a + b`
    // passes. That is what makes the commit a red->green flip, so mine writes a task for it.
    let script = "#!/bin/sh\n\
         if grep -q 'a - b' src/lib.rs; then passed=false; else passed=true; fi\n\
         printf '{\"tests\":[{\"name\":\"adds\",\"passed\":%s}]}' \"$passed\" > .tsr-suite-report.json\n\
         [ \"$passed\" = true ]\n";
    fs::write(&path, script).expect("write stand-in suite");
    chmod_x(&path);
    path
}

/// A stand-in `cargo` that answers `cargo mutants --list --json` and `cargo mutants` the way
/// cargo-mutants does, writing `mutants.out/` **inside its working directory** (the disposable copy
/// the kill run must use). It records its cwd to `$CARGO_MUTANTS_LOG` so the test can prove the kill
/// run never ran in the repository.
///
/// The list names one mutant in `src/lib.rs`; the kill run reports it caught, with one log per
/// scenario naming the failing test `adds`.
fn standin_cargo(dir: &Path) -> PathBuf {
    let path = dir.join("cargo");
    let script = r#"#!/bin/sh
if [ "$1" = "mutants" ]; then
  if [ -n "$CARGO_MUTANTS_LOG" ]; then
    printf '%s\n' "$PWD" >> "$CARGO_MUTANTS_LOG"
  fi
  mkdir -p mutants.out
  case "$*" in
    *--list*)
      cat > mutants.out/mutants.json <<'JSON'
[{"name":"src/lib.rs:1:5: replace add -> i32 with 0","package":"demo","file":"src/lib.rs","function":{"function_name":"add","return_type":"i32"},"span":{"start":{"line":1,"column":5},"end":{"line":1,"column":5}},"replacement":"0","genre":"replace","diff":"--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-pub fn add(a: i32, b: i32) -> i32 { a + b }\n+pub fn add(a: i32, b: i32) -> i32 { 0 }\n"}]
JSON
      exit 0
      ;;
  esac
  cat > mutants.out/outcomes.json <<'JSON'
{"outcomes":[{"scenario":"Baseline","summary":"Success","log_path":"log/baseline.log"},{"scenario":{"Mutant":{"name":"src/lib.rs:1:5: replace add -> i32 with 0","package":"demo","file":"src/lib.rs","function":{"function_name":"add","return_type":"i32"},"span":{"start":{"line":1,"column":5},"end":{"line":1,"column":5}},"replacement":"0","genre":"replace","diff":"--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-pub fn add(a: i32, b: i32) -> i32 { a + b }\n+pub fn add(a: i32, b: i32) -> i32 { 0 }\n"}},"summary":"CaughtMutant","log_path":"log/mutant.log"}],"total_mutants":1,"caught":1,"missed":0,"timeout":0,"unviable":0,"success":0}
JSON
  mkdir -p mutants.out/log
  printf 'test adds ... FAILED\n' > mutants.out/log/mutant.log
  printf 'test adds ... ok\n' > mutants.out/log/baseline.log
  exit 2
fi
exec /usr/local/cargo/bin/cargo "$@"
"#;
    fs::write(&path, script).expect("write stand-in cargo");
    chmod_x(&path);
    path
}

/// `git status --porcelain` in `repo`, trimmed. Empty means the repository is untouched.
fn git_status(repo: &Path) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["status", "--porcelain"])
        .output()
        .expect("git status");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// N2, N3, N5: every `tsr exec` runs in a task copy, provisioning is a host step, and neither mine
/// nor mutants leaves anything in the repository.
#[test]
fn n2_n3_n5_every_exec_runs_in_a_task_copy_and_the_repository_stays_clean() {
    let ws = workspace();
    let scratch = ws.join("target/n2-n3-n5-sandbox");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();

    let repo = two_commit_repo(&scratch);
    let standin_dir = scratch.join("bin");
    fs::create_dir_all(&standin_dir).unwrap();
    let _standin = logging_standin_tsr(&standin_dir, &tsr_bin());
    let suite = standin_suite(&standin_dir);
    let _cargo = standin_cargo(&standin_dir);

    let exec_log = scratch.join("exec-cwds.log");
    let mutants_log = scratch.join("mutants-cwds.log");
    let path_env = format!(
        "{}:{}",
        standin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    // N2: mine the repository by a RELATIVE path into a RELATIVE --out, from the scratch directory.
    // On the buggy code the relative --out lands the copy in `brel/copies/brel/copies/…` and the
    // task is silently recorded rejected; a relatively-mined repo cannot be materialised at all.
    let mine = Command::new(tsr_bin())
        .args([
            "corpus",
            "mine",
            "repo",
            "--out",
            "brel",
            "--suite",
            suite.to_str().unwrap(),
        ])
        .current_dir(&scratch)
        .env("PATH", &path_env)
        .env("TSR_EXEC_LOG", &exec_log)
        .output()
        .expect("run tsr corpus mine");
    assert!(
        mine.status.success(),
        "N2: mining a relatively-named repo into a relative --out must succeed; stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&mine.stdout),
        String::from_utf8_lossy(&mine.stderr)
    );
    assert_eq!(
        git_status(&repo),
        "",
        "N5: `tsr corpus mine` must leave the repository clean"
    );

    let corpus = scratch.join("brel");
    let manifest = fs::read_to_string(corpus.join("tasks.jsonl")).expect("tasks.jsonl");
    let first = manifest.lines().next().unwrap_or("");
    assert!(
        !first.trim().is_empty(),
        "N2: mining a relatively-named repo into a relative --out must write at least one task; \
         tasks.jsonl is empty (the repo could not be materialised)"
    );
    let task: serde_json::Value = serde_json::from_str(first).expect("task json");
    let task_id = task["id"].as_str().expect("task id").to_string();

    // A scripted session that fixes the bug in the copy.
    let session = serde_json::json!({
        "name": "n2-n3-fix-in-copy",
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

    // N2/N3: run the task with a RELATIVE --out, from the scratch directory. The exec log is
    // cleared first: the mine run's suite invocations are not the batch's `tsr exec` calls, and the
    // N3 assertion below is about the batch's own invocations only.
    let _ = fs::remove_file(&exec_log);
    let run = Command::new(tsr_bin())
        .args([
            "task",
            "run",
            "--mode",
            "record",
            "--tasks",
            "brel/tasks.jsonl",
            "--driver",
            "scripted",
            "--out",
            "bout",
            "--profile",
            ws.join("profiles/default.toml").to_str().unwrap(),
        ])
        .current_dir(&scratch)
        .env("PATH", &path_env)
        .env("TSR_EXEC_LOG", &exec_log)
        .output()
        .expect("run tsr task run");
    let stdout = String::from_utf8_lossy(&run.stdout);
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert_eq!(
        run.status.code(),
        Some(0),
        "N2: the batch must run with a relative --out; stdout: {stdout}\nstderr: {stderr}"
    );

    // The copy must be at the absolute location the relative --out names, not nested twice.
    let copy = scratch.join("bout/copies").join(&task_id).join("record");
    assert!(
        copy.join("src/lib.rs").is_file(),
        "N2: the copy must be at {} (a relative --out must not nest `brel/copies/brel/copies/…`); \
         stdout: {stdout}\nstderr: {stderr}",
        copy.display()
    );
    assert!(
        !scratch.join("bout/copies/bout").exists(),
        "N2: a relative --out must not nest the copies root inside itself"
    );

    // N3: every `tsr exec` the batch issued ran with cwd = one task copy — never the repository,
    // never the copies root.
    let logged = fs::read_to_string(&exec_log).unwrap_or_default();
    let cwds: Vec<&str> = logged.lines().filter(|l| !l.trim().is_empty()).collect();
    assert!(
        !cwds.is_empty(),
        "N3: the batch must issue at least one `tsr exec` (the oracle); log: {logged:?}"
    );
    let copies_root = scratch.join("bout/copies");
    for cwd in &cwds {
        let cwd = Path::new(cwd);
        assert_ne!(
            cwd,
            repo.as_path(),
            "N3: a `tsr exec` ran in the repository itself: {cwd:?}"
        );
        assert_ne!(
            cwd,
            copies_root.as_path(),
            "N3: a `tsr exec` ran in the copies root, not a task copy: {cwd:?}"
        );
        assert!(
            cwd.starts_with(&copies_root),
            "N3: a `tsr exec` ran outside the copies root: {cwd:?}"
        );
        assert!(
            cwd.starts_with(copy.as_path()),
            "N3: a `tsr exec` ran outside the task's own copy {copy:?}: {cwd:?}"
        );
    }

    // N5: `tsr corpus mutants` must run the kill run in a disposable copy, never in the repository,
    // and leave nothing in it.
    let mutants_out = scratch.join("mout");
    let mutants = Command::new(tsr_bin())
        .args([
            "corpus",
            "mutants",
            "repo",
            "--out",
            mutants_out.to_str().unwrap(),
        ])
        .current_dir(&scratch)
        .env("PATH", &path_env)
        .env("CARGO_MUTANTS_LOG", &mutants_log)
        .output()
        .expect("run tsr corpus mutants");
    assert!(
        mutants.status.success(),
        "N5: `tsr corpus mutants` must succeed; stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&mutants.stdout),
        String::from_utf8_lossy(&mutants.stderr)
    );
    assert_eq!(
        git_status(&repo),
        "",
        "N5: `tsr corpus mutants` must leave the repository clean (no mutants.out/)"
    );
    assert!(
        !repo.join("mutants.out").exists(),
        "N5: the kill run must not leave mutants.out/ in the repository"
    );
    let kill_cwds = fs::read_to_string(&mutants_log).unwrap_or_default();
    for cwd in kill_cwds.lines().filter(|l| !l.trim().is_empty()) {
        assert_ne!(
            Path::new(cwd),
            repo.as_path(),
            "N5: the kill run ran in the repository itself: {cwd}"
        );
    }
}
