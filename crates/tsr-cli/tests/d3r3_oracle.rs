//! D3, R3: the oracle argv.
use tsr_cli::corpus_cmd::oracle_argv;

#[test]
fn r3_two_flipped_tests_are_passed_after_a_separator() {
    let argv = oracle_argv(Some("tests::other,tests::flipped_add"), false, None).unwrap();
    assert_eq!(
        argv,
        vec!["cargo", "test", "--", "tests::other", "tests::flipped_add"]
    );
}

#[test]
fn r3_one_flipped_test_is_passed_after_a_separator() {
    let argv = oracle_argv(Some("parser::tests::handles_empty_input"), false, None).unwrap();
    assert_eq!(
        argv,
        vec!["cargo", "test", "--", "parser::tests::handles_empty_input"]
    );
}

#[test]
fn r3_full_suite_has_no_test_names() {
    let argv = oracle_argv(None, true, None).unwrap();
    assert_eq!(argv, vec!["cargo", "test"]);
}

#[test]
fn r3_neither_tests_nor_full_suite_is_none() {
    assert!(oracle_argv(None, false, None).is_none());
}

#[test]
fn r3_a_custom_suite_command_is_kept() {
    let argv = oracle_argv(Some("a,b"), false, Some("cargo nextest run")).unwrap();
    assert_eq!(argv, vec!["cargo", "nextest", "run", "--", "a", "b"]);
}
