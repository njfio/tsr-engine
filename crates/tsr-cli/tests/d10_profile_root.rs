//! D10: `tsr task run` resolves `profiles/default.toml` from the repository
//! root, not the working directory.
//!
//! Pre-fix, `batch_cmd::run_cmd` loaded `Path::new("profiles/default.toml")`,
//! which is relative to the process's working directory. Invoked from a
//! subdirectory, the command failed with `loading profiles/default.toml: ...`
//! even though the repository root holds the profile. This test runs the built
//! `tsr` binary with its working directory set to a scratch subdirectory and
//! asserts the profile is found.

use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

#[test]
fn d10_task_run_resolves_the_profile_from_the_repository_root() {
    let ws = workspace();
    let bin = env!("CARGO_BIN_EXE_tsr");
    let tasks = ws.join("fixtures/m3/batch/tasks.jsonl");
    assert!(
        tasks.is_file(),
        "fixture manifest exists: {}",
        tasks.display()
    );

    // A scratch working directory that is NOT the repository root and holds no
    // `profiles/` of its own.
    let scratch = ws.join("target/d10-profile-root-cwd");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let out = ws.join("target/d10-profile-root-out");
    let _ = std::fs::remove_dir_all(&out);

    let output = Command::new(bin)
        .current_dir(&scratch)
        .args([
            "task",
            "run",
            "--mode",
            "rules",
            "--tasks",
            tasks.to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            out.to_str().unwrap(),
            "--scripted-runner",
        ])
        .output()
        .expect("run tsr");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("loading profiles/default.toml"),
        "the profile must resolve from the repository root, not the cwd; stderr:\n{stderr}"
    );
    assert!(
        output.status.success(),
        "tsr task run from a subdirectory must succeed; exit {:?}, stderr:\n{stderr}",
        output.status.code()
    );
}
