**M4 r5b3: not done** — every named case now exits 2, but `fit` still exits 1 ("fit failed: json: …", no file named) on an empty, truncated or wrong-typed `features.json`, `split.json` or `data-manifest.json`.

## (a) DONE
Missing `rows.jsonl`, `features.json`, `split.json`, `data-manifest.json` and a non-UTF-8 `rows.jsonl`: exit 2, `fit refused: <file>: the file is missing…` / `…not valid UTF-8` (bundle.rs:204; dataset.rs:582, :628). Also exit 2: non-UTF-8 in the other structural files, and a directory in place of a file.

## (b) DONE
Tasks index cut to fit-side (229 lines), held-out-only (76), calibrate-only (75): exit 2, "examined 696 of 1129 rows but did not cover every split side: calibrate (0 of 218), held_out (0 of 215)" (probes.rs:233-250). One held-out task dropped: "held_out (212 of 215)".

## (c) DONE
`seq` "x", 1.5, null, absent, true, 2^64-1: exit 2, "events.jsonl line 8: `seq` must be an integer, found …" (dataset.rs:659-672), on `fit` and `dataset`.

## (d) DONE
`require_parseable_jsonl` is gone; `read_records` and `read_events_records` parse each file once. Events cut to 20, 3000, 8000 lines: longest stderr line 414, 746, 395 bytes (was 271 KB), text "probe refused: … holds no event for 1125 indexed rows, the first being task …".

## (e) DONE
`dataset --by time --calibrate 0.2 --holdout 0.2 --seed 7`, `fit`, `verify --rebuild`, `report`: bundle 22b26e5e9bb4…a23; 152 rows; controller 0.157443 / 0.492302; generator 0.175364 / 0.577418; paired −0.011781 INCONCLUSIVE.

## (f) NOT DONE — Medium
`fit` exits 1, `tsr: fit failed: json: …`, with no file named, when `features.json`, `split.json` or `data-manifest.json` is empty, truncated at 100 bytes, `null`, `[]`, `{}`, or has `"seed":"x"`. Cause: `serde_json::from_str(...)?` at bundle.rs:226-229 gives `LearnError::Json`; train_cmd.rs:365-367 maps it to exit 1. `report`, `apply` and `verify` exit 2 on the same files. Fix: map those three parses to `LearnError::Invalid("<file>: …")`.

Everything else of about 90 own cases across the five subcommands exits 2 with no panic (BOM, NUL, 200k-deep nesting, `event: null`, CRLF, bad shares).

## (g) DONE
Diff: train_cmd.rs, m4_train.rs, bundle.rs, dataset.rs, probes.rs, specs/. fmt and clippy 1.90 clean. Tests: 315 passed, 2 failed, both Docker-only m0 (`exec_resolves_the_session_task_and_refuses_without_one`, `selftest_is_reproducible_structurally_excluding_duration_ms_by_name`). Tree clean.

## New defects
- **Medium:** (f).
- **Low:** `dataset --holdout 0` then `fit` exits 0, a bundle with no held-out rows. The brief wants one row of each side; probes.rs:239 skips an empty side.
- **Low:** `fit --out <file>` and `dataset --out <file>` exit 1 (`io: Not a directory`).
- **Low:** the probe-refusal text at bundle.rs:471-483 is outside "input loading only", but (d) requires it.

## What I ran
Acceptance chain; about 90 cases in my own temp dir; fmt, clippy and `cargo +1.90.0 test --workspace --no-fail-fast` on `/home/claude/r3/m4/target`.
