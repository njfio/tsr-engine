//! The operator's store for the evidence doc's "What the operator runs" section (lane 06 item 9).
//!
//! The operator's log lives in a store, `.tsr/tsr.db`, not in an exported `events.jsonl`. The
//! synthetic generator appends the corpus to a fresh store as it goes, so that store is a real one:
//! `tsr train dataset --store` reads it exactly as it reads an operator's log.
//!
//! `scripts/m4-evidence.sh` runs this test with `TSR_M4_OPERATOR_STORE=<path>` to write the store at
//! `<path>` (which must not exist yet) and then runs the operator's commands against it. Without the
//! variable the store goes to a temporary directory and is removed. Either way the test checks that
//! the store exports the events of `fixtures/m4/events.jsonl` byte for byte, so the operator's
//! commands run on the same corpus as the rest of the evidence.

use std::path::PathBuf;

#[test]
fn write_the_operator_store() {
    let requested = std::env::var_os("TSR_M4_OPERATOR_STORE").map(PathBuf::from);
    let scratch =
        std::env::temp_dir().join(format!("tsr-m4-operator-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("a temp dir is creatable");

    let db = requested.unwrap_or_else(|| scratch.join("tsr.db"));
    assert!(
        !db.exists(),
        "the operator store {} must not exist yet",
        db.display()
    );
    if let Some(parent) = db.parent() {
        std::fs::create_dir_all(parent).expect("the store's directory is creatable");
    }
    let exported = scratch.join("export");
    tsr_learn::synth::generate_fixtures(&db, &exported).expect("the corpus generates");

    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/m4");
    for name in ["events.jsonl", "tasks.jsonl", "labels.jsonl"] {
        let got = std::fs::read(exported.join(name)).expect("the export is readable");
        let want = std::fs::read(fixtures.join(name)).expect("the fixture is readable");
        assert!(
            got == want,
            "the operator store's {name} differs from fixtures/m4/{name}"
        );
    }
    let _ = std::fs::remove_dir_all(&scratch);
}
