//! D10: the profile is resolved from `--profile`, else from the store's
//! repository root (as the M0 commands resolve the store), else the command
//! exits 2 naming where it looked. There is no compile-time source path
//! embedded in the binary.
//!
//! On the pre-fix code `repo_root()` walked the working directory's ancestors
//! for `profiles/default.toml` and then fell back to
//! `env!("CARGO_MANIFEST_DIR")/../..` — the checkout the binary was built in.
//! A `tsr task run` invoked from a scratch directory with no profile of its own
//! therefore silently loaded the *build machine's* profile instead of failing.

use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> PathBuf {
    // The integration test binary lives in target/<profile>/deps/; the CLI is
    // two levels up.
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.join("tsr")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// A scratch directory that is NOT under the repository, so no ancestor holds
/// `profiles/default.toml`.
fn scratch() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn run_from(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(bin())
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("spawn tsr")
}

/// D10(a): with no `--profile`, the profile is looked for under the store's
/// repository root — the directory holding `.tsr/` — not the build machine's
/// checkout. A scratch store with no profile must fail naming the scratch path.
#[test]
fn d10_profile_resolves_from_store_root_not_the_build_checkout() {
    let dir = scratch();
    std::fs::create_dir_all(dir.path().join(".tsr")).unwrap();
    let tasks = dir.path().join("tasks.jsonl");
    std::fs::write(&tasks, "").unwrap();

    let out = run_from(
        dir.path(),
        &[
            "task",
            "run",
            "--mode",
            "record",
            "--tasks",
            tasks.to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            dir.path().join("out").to_str().unwrap(),
        ],
    );
    let stderr = String::from_utf8_lossy(&out.stderr);

    assert_eq!(
        out.status.code(),
        Some(2),
        "a scratch store with no profile must exit 2; stderr:\n{stderr}"
    );
    let expected = dir.path().join("profiles/default.toml");
    assert!(
        stderr.contains(expected.to_str().unwrap()),
        "the error must name the store-root profile it looked for ({expected:?}); stderr:\n{stderr}"
    );
    let build_path = repo_root().join("profiles/default.toml");
    assert!(
        !stderr.contains(build_path.to_str().unwrap()),
        "the error must not name the build checkout's profile ({build_path:?}); stderr:\n{stderr}"
    );
}

/// D10(a): `--profile` names the profile directly, and a missing one is
/// reported by the path the operator gave.
#[test]
fn d10_profile_flag_names_the_profile_directly() {
    let dir = scratch();
    let tasks = dir.path().join("tasks.jsonl");
    std::fs::write(&tasks, "").unwrap();
    let custom = dir.path().join("custom-profile.toml");

    let out = run_from(
        dir.path(),
        &[
            "task",
            "run",
            "--mode",
            "record",
            "--tasks",
            tasks.to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            dir.path().join("out").to_str().unwrap(),
            "--profile",
            custom.to_str().unwrap(),
        ],
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(2),
        "a missing --profile must exit 2; stderr:\n{stderr}"
    );
    assert!(
        stderr.contains(custom.to_str().unwrap()),
        "the error must name the --profile path ({custom:?}); stderr:\n{stderr}"
    );
    let build_path = repo_root().join("profiles/default.toml");
    assert!(
        !stderr.contains(build_path.to_str().unwrap()),
        "the error must not name the build checkout's profile; stderr:\n{stderr}"
    );
}

/// D10(a): with no store and no `--profile`, the command exits 2 and names
/// where it looked, rather than silently using a compile-time path.
#[test]
fn d10_no_store_and_no_profile_exits_2_naming_where_it_looked() {
    let dir = scratch();
    let tasks = dir.path().join("tasks.jsonl");
    std::fs::write(&tasks, "").unwrap();

    let out = run_from(
        dir.path(),
        &[
            "task",
            "run",
            "--mode",
            "record",
            "--tasks",
            tasks.to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            dir.path().join("out").to_str().unwrap(),
        ],
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(2),
        "no store and no --profile must exit 2; stderr:\n{stderr}"
    );
    let build_path = repo_root().join("profiles/default.toml");
    assert!(
        !stderr.contains(build_path.to_str().unwrap()),
        "the error must not name the build checkout's profile; stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("profile"),
        "the error must say it was looking for a profile; stderr:\n{stderr}"
    );
}
