//! Characterize replacement semantics using inert text, never candidate execution.
use tsr_batch::driver::ScriptedEdit;

#[test]
fn replacement_is_the_entire_file_not_a_merge() {
    let copy = tempfile::tempdir().unwrap();
    let tests = "\n#[cfg(test)] mod tests { #[test] fn example() {} }\n";
    std::fs::write(
        copy.path().join("source.rs"),
        format!("fn old() {{}}{tests}"),
    )
    .unwrap();
    let replacement = "fn repaired() {}\n";
    ScriptedEdit::new("source.rs", replacement)
        .apply(copy.path())
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(copy.path().join("source.rs")).unwrap(),
        replacement
    );
}

#[test]
fn complete_replacement_retains_supplied_test_bytes() {
    let copy = tempfile::tempdir().unwrap();
    let tests = "\n#[cfg(test)] mod tests { #[test] fn example() {} }\n";
    std::fs::write(
        copy.path().join("source.rs"),
        format!("fn old() {{}}{tests}"),
    )
    .unwrap();
    let replacement = format!("fn repaired() {{}}{tests}");
    ScriptedEdit::new("source.rs", &replacement)
        .apply(copy.path())
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(copy.path().join("source.rs")).unwrap(),
        replacement
    );
}
