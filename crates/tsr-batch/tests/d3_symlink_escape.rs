//! D3: a session edit must not escape the disposable copy through a symlink.
//!
//! The copy is the only writable mount (PRD §B10). `git archive` preserves committed symlinks, so
//! a mined repository can plant `escape -> <dir outside the repo>` in the snapshot the copy is
//! materialised from; a scripted step whose edit path goes through that symlink must be refused,
//! not written outside the copy.
//!
//! This test names the defect the security review of task #11 reproduced: `ScriptedEdit::apply`
//! refused only absolute paths and `..` components, then `copy.join(rel)` + `fs::write`.

use std::path::Path;

use tsr_batch::driver::ScriptedEdit;

/// A copy dir with a symlink component pointing outside it must not let an edit escape.
#[test]
fn d3_edit_cannot_escape_the_copy_through_a_symlink() {
    let root = std::env::temp_dir().join(format!(
        "tsr-d3-symlink-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let copy = root.join("copy");
    let outside = root.join("outside");
    std::fs::create_dir_all(&copy).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    // The committed symlink a mined repo can carry into the copy.
    std::os::unix::fs::symlink(&outside, copy.join("escape")).unwrap();

    let edit = ScriptedEdit::new("escape/pwned.txt", "PWNED\n");
    let result = edit.apply(&copy);

    let escaped = outside.join("pwned.txt");
    assert!(
        !escaped.exists(),
        "SECURITY: session wrote OUTSIDE the copy via symlink: {}",
        escaped.display()
    );
    assert!(
        result.is_err(),
        "an edit through a symlink component must be refused, got {result:?}"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// A symlink that is the final component must also be refused (it would write through it).
#[test]
fn d3_edit_cannot_write_through_a_symlinked_file() {
    let root = std::env::temp_dir().join(format!(
        "tsr-d3-symlink-file-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let copy = root.join("copy");
    let outside = root.join("outside");
    std::fs::create_dir_all(&copy).unwrap();
    std::fs::create_dir_all(&outside).unwrap();
    std::fs::write(outside.join("target.txt"), "ORIGINAL\n").unwrap();
    std::os::unix::fs::symlink(outside.join("target.txt"), copy.join("link.txt")).unwrap();

    let edit = ScriptedEdit::new("link.txt", "PWNED\n");
    let result = edit.apply(&copy);

    let content = std::fs::read_to_string(outside.join("target.txt")).unwrap();
    assert_eq!(
        content, "ORIGINAL\n",
        "SECURITY: session wrote through a symlinked file outside the copy"
    );
    assert!(
        result.is_err(),
        "a symlinked final component must be refused"
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// A Windows drive prefix and a `..\` component are literal escapes the guard must refuse.
#[test]
fn d3_edit_refuses_windows_style_escapes() {
    let root = std::env::temp_dir().join(format!(
        "tsr-d3-win-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let copy = root.join("copy");
    std::fs::create_dir_all(&copy).unwrap();

    for path in ["C:\\Windows\\evil.txt", "..\\..\\evil.txt"] {
        let edit = ScriptedEdit::new(path, "PWNED\n");
        assert!(
            edit.apply(&copy).is_err(),
            "`{path}` is a literal escape and must be refused"
        );
    }

    let _ = std::fs::remove_dir_all(&root);
}

/// A plain relative path inside the copy still works (the guard must not break the happy path).
#[test]
fn d3_edit_writes_a_plain_relative_path_inside_the_copy() {
    let root = std::env::temp_dir().join(format!(
        "tsr-d3-ok-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let copy = root.join("copy");
    std::fs::create_dir_all(&copy).unwrap();

    ScriptedEdit::new("src/lib.rs", "FIXED\n")
        .apply(&copy)
        .unwrap();
    assert_eq!(
        std::fs::read_to_string(copy.join("src/lib.rs")).unwrap(),
        "FIXED\n"
    );
    assert!(Path::new(&copy).join("src/lib.rs").exists());

    let _ = std::fs::remove_dir_all(&root);
}
