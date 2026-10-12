**M4 r5b: not done** — the named empty, newline-only and truncated-line cases now exit 2 with file and line, but the guard only checks each line is JSON, so `fit` still panics at probes.rs:111 and `dataset` still exits 1 on inputs that are not data.

## (a) F9/B3 — NOT DONE
Named cases pass. `dataset` and `fit`, tasks and events: newline-only, blank-only, zero-byte, truncated last line, truncated middle line. All exit 2 with `<path> line N: …` (`require_parseable_jsonl`, train_cmd.rs:195; called :287, :292, :398).

Open, against "no input makes either panic or exit 1" and "an empty-input probe is refused, not skipped" (probes.rs untouched):
- `fit` panics, exit 101, at probes.rs:111. Repro: copy the acceptance dataset, replace its `events.jsonl` with `head -20 fixtures/m4/events.jsonl`, run `fit`.
- `fit` with a parseable `tasks.jsonl` that matches no row (task ids zeroed) exits 0 with the acceptance id 22b26e5e…; probe (a) examines nothing, silently. `head -1 tasks.jsonl` leaves 4 of 1129 rows examined, same exit 0.
- `dataset` exits 1 on a tasks.jsonl or events.jsonl line that is valid JSON but not a record (`{}`, `null`). `fit` exits 2 there, but its events message has no line number.
- A truncated `labels.jsonl` makes `dataset` exit 1.

## (b) F8 — DONE
I added two index rows with no events. `dataset` prints `uncommitted_tasks=2` (train_cmd.rs:377; dataset.rs:458, :572, :633). data-manifest.json has `"uncommitted_tasks": 2`, and report text and `--json` carry it. `fit` exits 0, bundle id unchanged.

## (c) F10/B4 — DONE
I dropped the held-out calibrated labels (18 rows). `report --json` gives `paired_by_family.calibrated = {"tasks":0,"mean":null,"interval":null,…}`. The text reads `calibrated n=0 clusters=0 mean=n/a interval n/a` (report.rs:78, :523-530, :710-724).

## (d) One label — DONE
Paired lines read `mean_brier=-0.011781`. `mean=brier=` and any other `x=y=` pattern appear 0 times in report and apply output.

## (e) Acceptance — DONE
`dataset --by time --calibrate 0.2 --holdout 0.2 --seed 7`, `fit`, `verify`, `report`. Bundle id 22b26e5e9bb4…a23. Controller 0.157443 / 0.492302, generator 0.175364 / 0.577418 on 152 rows. Paired all −0.011781 [−0.036676, 0.015435] INCONCLUSIVE. All identical to the previous review.

## (f) Diff and gates — DONE
- The diff touches train_cmd.rs, m4_train.rs, dataset.rs, report.rs and specs/. It also edits bundle.rs: a functional line in `load_dataset` (:332) plus test literals, where the brief allows a call-site change only. Low severity; report needs it to see the count.
- fmt clean (1.97 and 1.90). Clippy 1.90 `-D warnings` exits 0.
- Tests: 293 passed, 2 failed. Both are the Docker-only m0 tests (`selftest_is_reproducible_structurally_excluding_duration_ms_by_name`, `exec_resolves_the_session_task_and_refuses_without_one`).

## New defects
- **Medium:** `fit` panics (101) on a parseable `events.jsonl` that lacks the dataset's tasks.
- **Medium:** `fit` exits 0 with probe (a) skipping every row when `tasks.jsonl` matches none.
- **Low:** `dataset` exits 1 on `{}`/`null` lines and a truncated labels.jsonl.
- Fix direction: `fit` checks `events.jsonl` against the manifest's `source_root`; probe (a) refuses at zero rows examined; `dataset` maps record-shape errors to exit 2.

## What I ran
Acceptance `dataset`/`fit`/`verify`/`report` (text and `--json`). 24 input cases on `dataset` and `fit` in a temp dir. The F8 index, the label-drop report, the doubled-label grep, fmt, clippy 1.90 and the full workspace tests. `git status` is clean at 5a01eb4.
