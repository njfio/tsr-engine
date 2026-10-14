**M4 r5b2: not done** — every case in the previous review now exits 2, but `fit` still exits 1 on a missing or non-UTF-8 `rows.jsonl`, a partial index still passes probe (a), and `bundle.rs`, a never-edit file, changed.

## (a) F9/B3 — NOT DONE
Named cases pass (exit 2, file and line or probe and counts, no panic):
- `fit`, events cut to 20 lines: probes.rs:121 refusal per row (was the panic at :111).
- Zeroed task ids, `dataset` and `fit`: "examined 0 of 1129 rows" (probes.rs:187).
- `head -1 tasks.jsonl`, `fit`: "examined 4 of 1129 rows, fewer than the held-out slice of 215" (:193).
- `{}` and `null` lines in tasks and events, `dataset` and `fit`: `<file> line 6: not a … record: missing field …`.
- Truncated labels.jsonl: "labels.jsonl line 28: EOF…". Empty, newline-only, truncated-mid and truncated-last files for tasks, events and labels: exit 2.
- Own cases (wrong-typed or missing index fields, a string for a label boolean, bad `rows.jsonl` lines): exit 2 at the right line.

Not done, from my own inputs:
- **Medium: `fit` exits 1 on structural bad input.** A missing `rows.jsonl`, `features.json`, `split.json` or `data-manifest.json`, or a non-UTF-8 `rows.jsonl`, gives "fit failed: io: …" and exit 1 (train_cmd.rs:407-408; `LearnError::Io` from bundle.rs:205). The brief says nothing exits 1 for bad input.
- **Medium: a partial index passes probe (a).** The gate is a count, `examined < held_out` (probes.rs:193). Cut the dataset's `tasks.jsonl` to the 229 fit-side tasks: 696 rows examined, none of the 215 held-out, `fit` exits 0 with id 22b26e5e…. A held-out-only index (76 lines) and a calibrate-only index (75 lines, 218 rows) also exit 0.
- **Low: wrong-typed `seq` accepted.** `"seq":"x"`, `1.5`, `null` or absent in an events wrapper is replaced by line position (dataset.rs:630, 636). `dataset` exits 0 with the same data root. `fit` used to refuse a line with no seq; it now exits 0.

## (b) Acceptance — DONE
`dataset --by time --calibrate 0.2 --holdout 0.2 --seed 7`, `fit`, `verify`, `report`. Bundle id 22b26e5e9bb4…a23. 152 rows: controller 0.157443 / 0.492302, generator 0.175364 / 0.577418; paired all −0.011781 [−0.036676, 0.015435] INCONCLUSIVE.

## (c) Diff and gates — NOT DONE (scope only)
- The diff touches train_cmd.rs, m4_train.rs, dataset.rs, probes.rs and two specs files, plus **bundle.rs (Low)**, which the brief lists under "Never edits": `read_events`, `load_dataset` and the `write_bundle` message (bundle.rs:192, :205, :463).
- fmt clean on 1.97 and 1.90; `cargo +1.90.0 clippy --workspace --all-targets -- -D warnings` exits 0.
- Tests: 301 passed, 2 failed, both Docker-only m0 (`selftest_is_reproducible_structurally_excluding_duration_ms_by_name`, `exec_resolves_the_session_task_and_refuses_without_one`). No other failure.
- One code commit, naming F9 and B3.

## New defects
- **Low:** the events-cut refusal is one 271 KB stderr line, prefixed "probe caught a leak" though nothing leaked.
- **Low:** `require_parseable_jsonl` (train_cmd.rs:179) still runs as a JSON-only pre-pass before the loader, so there are two parsers with different messages.

## What I ran
Acceptance `dataset`/`fit`/`verify`/`report`; about 70 input cases on `dataset` and `fit` in a temp dir; fmt on both toolchains; clippy 1.90; the workspace tests. Tree clean.
