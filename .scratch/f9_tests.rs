/// F9/B3 (a): a `tasks.jsonl` line that is valid JSON but not a task record (`{}`) must make
/// `dataset` exit 2 naming the file and the line, not exit 1.
#[test]
fn f9_dataset_refuses_a_tasks_line_that_is_not_a_record() {
    let dir = temp_dir("f9-dataset-tasks-notrecord");
    let (ev, tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let text = std::fs::read_to_string(&tk).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines[0] = "{}";
    let bad_tasks = bad.join("tasks.jsonl");
    std::fs::write(&bad_tasks, lines.join("\n") + "\n").unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        bad_tasks.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: dataset refuses a `{{}}` tasks line (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3 (a): a `tasks.jsonl` line that is `null` must make `dataset` exit 2 naming the file and
/// the line, not exit 1.
#[test]
fn f9_dataset_refuses_a_null_tasks_line() {
    let dir = temp_dir("f9-dataset-tasks-null");
    let (ev, tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let text = std::fs::read_to_string(&tk).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines[0] = "null";
    let bad_tasks = bad.join("tasks.jsonl");
    std::fs::write(&bad_tasks, lines.join("\n") + "\n").unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        bad_tasks.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: dataset refuses a `null` tasks line (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3 (a): an `events.jsonl` line that is valid JSON but not an event record (`{}`) must make
/// `dataset` exit 2 naming the file and the line, not exit 1.
#[test]
fn f9_dataset_refuses_an_events_line_that_is_not_a_record() {
    let dir = temp_dir("f9-dataset-events-notrecord");
    let (ev, tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let text = std::fs::read_to_string(&ev).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines[0] = "{}";
    let bad_events = bad.join("events.jsonl");
    std::fs::write(&bad_events, lines.join("\n") + "\n").unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        bad_events.to_str().unwrap(),
        "--tasks",
        tk.to_str().unwrap(),
        "--labels",
        lb.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: dataset refuses a `{{}}` events line (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3 (a): a truncated `labels.jsonl` (a line that does not parse) must make `dataset` exit 2
/// naming the file and the line, not exit 1.
#[test]
fn f9_dataset_refuses_a_truncated_labels_file_naming_the_line() {
    let dir = temp_dir("f9-dataset-labels-trunc");
    let (ev, tk, lb) = regenerate(&dir);
    let bad = dir.join("bad");
    std::fs::create_dir_all(&bad).unwrap();
    let text = std::fs::read_to_string(&lb).unwrap();
    let truncated: String = text.chars().take(text.len() / 2).collect();
    let bad_labels = bad.join("labels.jsonl");
    std::fs::write(&bad_labels, truncated).unwrap();
    let ds = dir.join("ds");
    let (code, out, err) = run(&[
        "train",
        "dataset",
        "--events",
        ev.to_str().unwrap(),
        "--tasks",
        tk.to_str().unwrap(),
        "--labels",
        bad_labels.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.2",
        "--holdout",
        "0.2",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: dataset refuses a truncated labels.jsonl (never exit 1):\n{out}\n{err}"
    );
    assert!(
        err.contains("labels.jsonl"),
        "F9/B3: the refusal names labels.jsonl: {err}"
    );
    assert!(
        err.contains("line "),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3 (a): `fit` on a `tasks.jsonl` line that is valid JSON but not a record (`{}`) must exit 2
/// naming the file and the line, not exit 2 with no line.
#[test]
fn f9_fit_refuses_a_tasks_line_that_is_not_a_record_naming_the_line() {
    let dir = temp_dir("f9-fit-tasks-notrecord");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("tasks.jsonl")).unwrap();
    let mut lines: Vec<&str> = text.lines().collect();
    lines[0] = "{}";
    std::fs::write(ds.join("tasks.jsonl"), lines.join("\n") + "\n").unwrap();
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: fit refuses a `{{}}` tasks line:\n{out}\n{err}"
    );
    assert!(
        err.contains("tasks.jsonl"),
        "F9/B3: the refusal names tasks.jsonl: {err}"
    );
    assert!(
        err.contains("line 1"),
        "F9/B3: the refusal names the line: {err}"
    );
}

/// F9/B3 (a): `fit` on an `events.jsonl` cut to its first 20 lines must exit 2 naming the file,
/// never panic (exit 101) at probes.rs:111.
#[test]
fn f9_fit_refuses_an_events_log_that_lacks_the_datasets_tasks() {
    let dir = temp_dir("f9-fit-events-cut");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("events.jsonl")).unwrap();
    let cut: String = text.lines().take(20).collect::<Vec<_>>().join("\n") + "\n";
    std::fs::write(ds.join("events.jsonl"), cut).unwrap();
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: fit refuses an events.jsonl that lacks the dataset's tasks (never a panic):\n{out}\n{err}"
    );
    assert!(
        err.contains("events.jsonl"),
        "F9/B3: the refusal names events.jsonl: {err}"
    );
}

/// F9/B3 (a): `fit` on a `tasks.jsonl` whose task ids match no row must exit 2 naming probe (a)
/// and the counts, not exit 0 with the acceptance bundle id while probe (a) examines nothing.
#[test]
fn f9_fit_refuses_a_tasks_index_that_matches_no_row() {
    let dir = temp_dir("f9-fit-tasks-zeroed");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("tasks.jsonl")).unwrap();
    let zeroed: String = text
        .lines()
        .map(|l| {
            let mut v: serde_json::Value = serde_json::from_str(l).unwrap();
            v["task"] = serde_json::Value::String("0".repeat(64));
            serde_json::to_string(&v).unwrap()
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(ds.join("tasks.jsonl"), zeroed).unwrap();
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: fit refuses a tasks index that matches no row (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("post_action_events"),
        "F9/B3: the refusal names probe (a): {err}"
    );
}

/// F9/B3 (a): `fit` on a `tasks.jsonl` cut to its first line must exit 2 naming probe (a) and the
/// counts, not exit 0 while probe (a) examines 4 of 1129 rows.
#[test]
fn f9_fit_refuses_a_tasks_index_that_examines_too_few_rows() {
    let dir = temp_dir("f9-fit-tasks-head1");
    let ds = build_dataset(&dir, "7");
    let text = std::fs::read_to_string(ds.join("tasks.jsonl")).unwrap();
    let head1: String = text.lines().take(1).collect::<Vec<_>>().join("\n") + "\n";
    std::fs::write(ds.join("tasks.jsonl"), head1).unwrap();
    let b = dir.join("models").join("m4");
    let (code, out, err) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        b.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "F9/B3: fit refuses a tasks index that examines too few rows (never exit 0):\n{out}\n{err}"
    );
    assert!(
        err.contains("post_action_events"),
        "F9/B3: the refusal names probe (a): {err}"
    );
}

