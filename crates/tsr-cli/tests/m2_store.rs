//! R5: the M2 commands resolve the M0 store and never create one they only read.
//!
//! The M2 commands (`reflect`, `review`, `note`, `task reconcile`) used to default `--store` to
//! `./tsr.db`, so with `TSR_DB` set `tsr reflect` created `./tsr.db`, printed an empty Mirror and
//! exited 0 — reading a store nobody wrote. They must resolve the store the M0 commands use
//! (`store_path::db_path()`: `TSR_DB`, else `.tsr/tsr.db`), and a command that only reads must
//! refuse a missing store with exit 2 naming the path rather than create it.
//!
//! Every case runs the *built binary* from a scratch cwd with `TSR_DB` set, so the cwd is where a
//! stray `./tsr.db` or `.tsr/tsr.db` would appear.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tsr_domain::{Event, EventKind, Hash, Source, TaskId};
use tsr_store::Store;

/// The `tsr` binary cargo built for this test.
fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

/// A scratch directory under the target dir (never `/tmp`, which this host prunes).
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target-r5/m2-store")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// A task id from a fixed seed, so tests are reproducible.
fn task(seed: u8) -> TaskId {
    TaskId(Hash::of(&[b"m2-store-task".as_slice(), &[seed]]))
}

/// Seed `db` with one task: a `TaskOpened` event, so `tsr reflect <task>` has a log to render.
fn seed(db: &Path, t: &TaskId) {
    let store = Store::open(db).expect("open seed store");
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::TaskOpened { task: *t },
            "unix:1",
        ))
        .expect("append TaskOpened");
}

/// Run `tsr <args>` with `cwd` as the working directory and `TSR_DB` set to `db`.
/// Returns (exit code, stdout, stderr).
fn run_tsr_in(db: &Path, cwd: &Path, args: &[&str]) -> (i32, String, String) {
    let out = Command::new(tsr_bin())
        .args(args)
        .current_dir(cwd)
        .env("TSR_DB", db)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn tsr");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// R5: `tsr reflect <task>` with `TSR_DB` set reads that store and does not create `./tsr.db`.
#[test]
fn r5_reflect_uses_tsr_db_and_does_not_create_cwd_store() {
    let dir = scratch("reflect-tsr-db");
    let db = dir.join("m0-store.db");
    let cwd = dir.join("cwd");
    std::fs::create_dir_all(&cwd).unwrap();
    let t = task(1);
    seed(&db, &t);

    let (code, stdout, stderr) = run_tsr_in(&db, &cwd, &["reflect", &t.to_string()]);
    assert_eq!(code, 0, "reflect exits 0; stderr={stderr}");
    // The Mirror of the seeded task is not the empty "no such task" output: the TSR_DB store was
    // read. (An empty store would print a Mirror with no events.)
    assert!(
        !stdout.trim().is_empty(),
        "reflect printed a Mirror from the TSR_DB store"
    );

    // The defect: a stray `./tsr.db` in the cwd. It must not exist.
    assert!(
        !cwd.join("tsr.db").exists(),
        "reflect must not create ./tsr.db in the cwd"
    );
    assert!(
        !cwd.join(".tsr").join("tsr.db").exists(),
        "reflect must not create .tsr/tsr.db in the cwd"
    );
}

/// R5: a command that only reads refuses a missing store with exit 2 naming the path.
#[test]
fn r5_missing_store_exits_2_naming_the_path() {
    let dir = scratch("missing-store");
    let db = dir.join("does-not-exist.db");
    let cwd = dir.join("cwd");
    std::fs::create_dir_all(&cwd).unwrap();
    let t = task(2);

    let (code, _stdout, stderr) = run_tsr_in(&db, &cwd, &["reflect", &t.to_string()]);
    assert_eq!(code, 2, "a missing store is refused with exit 2; stderr={stderr}");
    assert!(
        stderr.contains(&db.display().to_string()),
        "the error names the missing path; stderr={stderr}"
    );
    assert!(
        !db.exists(),
        "reflect must not create the store it only reads"
    );
    assert!(
        !cwd.join("tsr.db").exists(),
        "reflect must not fall back to ./tsr.db"
    );
}

/// R5: `tsr task reconcile` with `TSR_DB` set does not create `.tsr/tsr.db` in the cwd.
#[test]
fn r5_task_reconcile_does_not_create_cwd_store() {
    let dir = scratch("reconcile-tsr-db");
    let db = dir.join("m0-store.db");
    let cwd = dir.join("cwd");
    std::fs::create_dir_all(&cwd).unwrap();
    let t = task(3);
    seed(&db, &t);

    let (code, _stdout, stderr) = run_tsr_in(&db, &cwd, &["task", "reconcile", &t.to_string()]);
    // No pending dispatches: exit 0. The point is the store it used.
    assert_eq!(code, 0, "reconcile exits 0; stderr={stderr}");
    assert!(
        !cwd.join(".tsr").join("tsr.db").exists(),
        "reconcile must not create .tsr/tsr.db in the cwd"
    );
    assert!(
        !cwd.join("tsr.db").exists(),
        "reconcile must not create ./tsr.db in the cwd"
    );
}