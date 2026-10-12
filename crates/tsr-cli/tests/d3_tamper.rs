//! D3/P1-P5: the oracle judges the copy with tests the session cannot change, on every layout.
//!
//! The defect (operator's design error): the oracle overwrote each oracle test file in the copy with
//! the snapshot's version. When the tests are inline (`#[cfg(test)] mod tests` in `src/lib.rs`, the
//! default Rust layout) the file restored is the code under repair, so a correct fix is reverted and
//! a do-nothing session on a mutant task is accepted (P1). It also trusted build configuration the
//! session can write: a `.cargo/config.toml` with `runner = "true"` was accepted with the bug present
//! (P2); the "no test matched" guard read run output from a CAS nothing writes, so it never fired
//! (P3); the restore followed a symlinked directory and wrote outside the copy (P4); a listed file
//! the snapshot lacked was skipped silently (P5).
//!
//! The fix: no restore. Before running anything, compare the copy with the task's snapshot and REJECT
//! the session if it added, removed or changed a build-configuration file (`Cargo.toml`, `Cargo.lock`,
//! `build.rs`, `rust-toolchain`/`rust-toolchain.toml`, anything under `.cargo/`) at any depth, or if
//! the copy contains a symlink the snapshot did not. For each oracle test, parse the snapshot's file
//! and the copy's file with `syn`, find the `#[test]` function at the same module path, and REJECT if
//! it is missing or its tokens differ. Then run each oracle test in the copy with
//! `cargo test -- <name> --exact` and read that run's real stdout: pass only when the exit code is 0
//! AND the output contains `test <name> ... ok`.
//!
//! This test drives the built `tsr` binary end to end, with **no** `--scripted-runner`, over real
//! repositories mined in the test — one with inline tests in `src/lib.rs` (repoA) and one with tests
//! in `tests/tests.rs` (repoB) — for a mined task and a mutant task on each.
//!
//! | Variant | Expected |
//! |---|---|
//! | bug present | rejected |
//! | bug fixed | accepted |
//! | oracle test deleted | rejected |
//! | assertion edited | rejected |
//! | test renamed | rejected |
//! | `.cargo/config.toml` runner | rejected |
//! | `build.rs` added | rejected |
//! | symlink to outside | rejected |
//! | do-nothing session on a mutant | rejected |

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

fn chmod_x(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path).expect("stat").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod");
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

/// Which layout the repository uses for its oracle test.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Layout {
    /// `#[cfg(test)] mod tests` inside `src/lib.rs` — the default Rust layout.
    Inline,
    /// A file under `tests/`.
    Integration,
}

impl Layout {
    /// The repository path of the file that holds the oracle test.
    fn test_file(self) -> &'static str {
        match self {
            Layout::Inline => "src/lib.rs",
            Layout::Integration => "tests/tests.rs",
        }
    }

    /// The `cargo test` name of the oracle test.
    fn test_name(self) -> &'static str {
        match self {
            Layout::Inline => "tests::adds",
            Layout::Integration => "adds",
        }
    }
}

/// A two-commit repository whose root has a red test and whose second commit makes it green.
fn two_commit_repo(dir: &Path, layout: Layout) -> PathBuf {
    let repo = dir.join("repo");
    fs::create_dir_all(&repo).expect("mkdir repo");
    git(&repo, &["init", "-q", "-b", "main"]);

    write(
        &repo,
        "Cargo.toml",
        "[package]\nname = \"demo\"\nversion = \"0.0.1\"\nedition = \"2021\"\n\n[workspace]\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    match layout {
        Layout::Inline => write(
            &repo,
            "src/lib.rs",
            "pub fn add(a: i32, b: i32) -> i32 { a - b }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() { assert_eq!(super::add(1, 1), 2); }\n}\n",
        ),
        Layout::Integration => {
            write(
                &repo,
                "src/lib.rs",
                "pub fn add(a: i32, b: i32) -> i32 { a - b }\n",
            );
            write(
                &repo,
                "tests/tests.rs",
                "#[test]\nfn adds() { assert_eq!(demo::add(1, 1), 2); }\n",
            );
        }
    }
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "root: red test"]);

    match layout {
        Layout::Inline => write(
            &repo,
            "src/lib.rs",
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() { assert_eq!(super::add(1, 1), 2); }\n}\n",
        ),
        Layout::Integration => write(
            &repo,
            "src/lib.rs",
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
        ),
    }
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
    chmod_x(&path);
    path
}

/// A stand-in suite the mine run invokes through `tsr exec`: it runs the real `cargo test` in the
/// snapshot (its cwd) and writes the suite report mine reads, naming the layout's oracle test.
fn standin_suite(dir: &Path, layout: Layout) -> PathBuf {
    let path = dir.join("suite.sh");
    let name = layout.test_name();
    let script = format!(
        "#!/bin/sh\n\
         CARGO_TARGET_DIR=\"$PWD/target\" cargo test --quiet -- {name} >/dev/null 2>&1\n\
         if [ $? -eq 0 ]; then\n\
         \tpassed=true\n\
         else\n\
         \tpassed=false\n\
         fi\n\
         printf '{{\"tests\":[{{\"name\":\"{name}\",\"passed\":%s}}]}}' \"$passed\" > .tsr-suite-report.json\n\
         [ \"$passed\" = true ]\n"
    );
    fs::write(&path, script).expect("write stand-in suite");
    chmod_x(&path);
    path
}

/// A stand-in `cargo` that answers `cargo mutants --list --json` and `cargo mutants` the way
/// cargo-mutants does, writing `mutants.out/` inside its working directory. The list names one
/// mutant in `src/lib.rs`; the kill run reports it caught, with one log naming the oracle test.
fn standin_cargo(dir: &Path, layout: Layout) -> PathBuf {
    let path = dir.join("cargo");
    let name = layout.test_name();
    let script = format!(
        r#"#!/bin/sh
if [ "$1" = "mutants" ]; then
  mkdir -p mutants.out
  case "$*" in
    *--list*)
      cat > mutants.out/mutants.json <<'JSON'
[{{"name":"src/lib.rs:1:5: replace add -> i32 with 0","package":"demo","file":"src/lib.rs","function":{{"function_name":"add","return_type":"i32"}},"span":{{"start":{{"line":1,"column":5}},"end":{{"line":1,"column":5}}}},"replacement":"0","genre":"replace","diff":"--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-pub fn add(a: i32, b: i32) -> i32 {{ a + b }}\n+pub fn add(a: i32, b: i32) -> i32 {{ 0 }}\n"}}]
JSON
      exit 0
      ;;
  esac
  cat > mutants.out/outcomes.json <<'JSON'
{{"outcomes":[{{"scenario":"Baseline","summary":"Success","log_path":"log/baseline.log"}},{{"scenario":{{"Mutant":{{"name":"src/lib.rs:1:5: replace add -> i32 with 0","package":"demo","file":"src/lib.rs","function":{{"function_name":"add","return_type":"i32"}},"span":{{"start":{{"line":1,"column":5}},"end":{{"line":1,"column":5}}}},"replacement":"0","genre":"replace","diff":"--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -1 +1 @@\n-pub fn add(a: i32, b: i32) -> i32 {{ a + b }}\n+pub fn add(a: i32, b: i32) -> i32 {{ 0 }}\n"}}}},"summary":"CaughtMutant","log_path":"log/mutant.log"}}],"total_mutants":1,"caught":1,"missed":0,"timeout":0,"unviable":0,"success":0}}
JSON
  mkdir -p mutants.out/log
  printf 'test {name} ... FAILED\n' > mutants.out/log/mutant.log
  printf 'test {name} ... ok\n' > mutants.out/log/baseline.log
  exit 2
fi
exec /usr/local/cargo/bin/cargo "$@"
"#
    );
    fs::write(&path, script).expect("write stand-in cargo");
    chmod_x(&path);
    path
}

/// The scratch corpus, mined once, shared by every row of the table.
struct Fixture {
    scratch: PathBuf,
    corpus: PathBuf,
    task_id: String,
    path_env: String,
}

fn fixture(name: &str, layout: Layout) -> Fixture {
    let ws = workspace();
    let scratch = ws.join("target").join(name);
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).unwrap();

    let repo = two_commit_repo(&scratch, layout);
    let corpus = scratch.join("corpus");
    let standin_dir = scratch.join("bin");
    fs::create_dir_all(&standin_dir).unwrap();
    let _standin = standin_tsr(&standin_dir, &tsr_bin());
    let suite = standin_suite(&standin_dir, layout);
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
        files.iter().any(|f| f.as_str() == Some(layout.test_file())),
        "mine must record {} as an oracle-test file; got {files:?}",
        layout.test_file()
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
        "name": "d3-tamper",
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

/// The code under repair, with the bug present (`a - b`) or fixed (`a + b`), for a layout.
fn lib_source(layout: Layout, fixed: bool) -> String {
    let op = if fixed { "+" } else { "-" };
    match layout {
        Layout::Inline => format!(
            "pub fn add(a: i32, b: i32) -> i32 {{ a {op} b }}\n\n#[cfg(test)]\nmod tests {{\n    #[test]\n    fn adds() {{ assert_eq!(super::add(1, 1), 2); }}\n}}\n"
        ),
        Layout::Integration => format!("pub fn add(a: i32, b: i32) -> i32 {{ a {op} b }}\n"),
    }
}

/// The review's table, row by row, on both layouts, through `tsr task run`.
fn table(layout: Layout, tag: &str) {
    // bug present -> rejected
    let fx = fixture(&format!("d3-tamper-{tag}-present"), layout);
    assert_eq!(
        run_row(
            &fx,
            serde_json::json!({ "path": "src/lib.rs", "contents": lib_source(layout, false) })
        ),
        "rejected",
        "{tag}: bug present must be rejected"
    );

    // bug fixed -> accepted
    let fx = fixture(&format!("d3-tamper-{tag}-fixed"), layout);
    assert_eq!(
        run_row(
            &fx,
            serde_json::json!({ "path": "src/lib.rs", "contents": lib_source(layout, true) })
        ),
        "accepted",
        "{tag}: bug fixed must be accepted"
    );

    // oracle test deleted -> rejected
    let fx = fixture(&format!("d3-tamper-{tag}-deleted"), layout);
    assert_eq!(
        run_row(
            &fx,
            serde_json::json!({ "path": layout.test_file(), "contents": "" })
        ),
        "rejected",
        "{tag}: a deleted oracle test must be rejected"
    );

    // assertion edited -> rejected
    let fx = fixture(&format!("d3-tamper-{tag}-edited"), layout);
    let edited = match layout {
        Layout::Inline => "pub fn add(a: i32, b: i32) -> i32 { a - b }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() { assert_eq!(super::add(1, 1), 0); }\n}\n".to_string(),
        Layout::Integration => "#[test]\nfn adds() { assert_eq!(demo::add(1, 1), 0); }\n".to_string(),
    };
    assert_eq!(
        run_row(
            &fx,
            serde_json::json!({ "path": layout.test_file(), "contents": edited })
        ),
        "rejected",
        "{tag}: an edited assertion must be rejected"
    );

    // test renamed -> rejected
    let fx = fixture(&format!("d3-tamper-{tag}-renamed"), layout);
    let renamed = match layout {
        Layout::Inline => "pub fn add(a: i32, b: i32) -> i32 { a - b }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn zz_moved() { assert_eq!(super::add(1, 1), 2); }\n}\n".to_string(),
        Layout::Integration => "#[test]\nfn zz_moved() { assert_eq!(demo::add(1, 1), 2); }\n".to_string(),
    };
    assert_eq!(
        run_row(
            &fx,
            serde_json::json!({ "path": layout.test_file(), "contents": renamed })
        ),
        "rejected",
        "{tag}: a renamed test must be rejected"
    );

    // `.cargo/config.toml` runner -> rejected
    let fx = fixture(&format!("d3-tamper-{tag}-cargo-config"), layout);
    assert_eq!(
        run_row(
            &fx,
            serde_json::json!({
                "path": ".cargo/config.toml",
                "contents": "[target.'cfg(all())']\nrunner = \"true\"\n"
            })
        ),
        "rejected",
        "{tag}: a `.cargo/config.toml` runner must be rejected"
    );

    // `build.rs` added -> rejected
    let fx = fixture(&format!("d3-tamper-{tag}-build-rs"), layout);
    assert_eq!(
        run_row(
            &fx,
            serde_json::json!({ "path": "build.rs", "contents": "fn main() {}\n" })
        ),
        "rejected",
        "{tag}: an added `build.rs` must be rejected"
    );

    // symlink to outside -> rejected
    let fx = fixture(&format!("d3-tamper-{tag}-symlink"), layout);
    let outside = fx.scratch.join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("pwned.txt"), "pwned\n").unwrap();
    assert_eq!(
        run_row(
            &fx,
            serde_json::json!({ "path": "link", "contents": "", "symlink": outside.to_str().unwrap() })
        ),
        "rejected",
        "{tag}: a symlink to outside must be rejected"
    );
}

#[test]
fn d3_tamper_table_inline_layout() {
    table(Layout::Inline, "inline");
}

#[test]
fn d3_tamper_table_integration_layout() {
    table(Layout::Integration, "integration");
}
