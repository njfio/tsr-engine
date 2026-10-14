**M4 r5b4: not done** — every named case now exits 2, but a wrong-typed field in `data-manifest.json` (`"rows":"x"`, `"unindexed_tasks":"x"`) still exits 0 on fit, report, apply and verify.

## (a) NOT DONE — Medium
Done for `features.json` and `split.json`, and for `data-manifest.json` empty, truncated at 100 bytes, `null`, `[]`, `{}`: `parse_structural` (bundle.rs:223) maps the serde error to `Invalid("<file>: …")`. `fit`, `report`, `apply` and `verify --rebuild` all exit 2 naming the file, with no "fit failed" and no panic (108 runs: 3 files, 9 forms each, 4 subcommands). `dataset` reads none of the three (it writes them; `--out` onto a dir holding an empty `features.json` overwrites it, exit 0), so it has no case.
Not done: `data-manifest.json` is a `serde_json::Value` with a key allow-list (bundle.rs:249-273), so only the keys are checked, not the types. `rows`, `fit`, `calibrate`, `held_out`, `resolved`, `feature_schema`, `unindexed_tasks`, `uncommitted_tasks` set to `"x"` (also -1, null, 1e30) exit 0 on all four. `unindexed_tasks:"x"`/`-3`/`1e30` is reported as 0 (bundle.rs:372, 375 `as_u64().unwrap_or(0)`), so the A8 count can be erased silently. The tests have no wrong-typed row for this file (m4_train.rs:2862-2866 and the report/apply/verify twin at :2901). The hash fields (`data_root`, `training_root`, `split_digest`, `source_root`) are refused. Unknown fields are refused on all three files, and the acceptance chain below is unaffected: `write_dataset` (dataset.rs:737-750) writes exactly the 12 allow-listed keys, and `deny_unknown_fields` cannot reject a struct's own output.

## (b) DONE
`dataset` with `--holdout 0`, `--calibrate 0`, `--holdout 0.00001`, `--calibrate nan`, `--by repo`: exit 2, "the split leaves the held_out side empty (0 of 380 tasks)…" (dataset.rs:470-489, train_cmd.rs:305), nothing written. `--calibrate 0.5 --holdout 0.5` names `fit`. `fit`, on datasets the 66b1910 binary built with holdout 0 (that binary's `fit` exited 0, bundle ba66dc75…) and calibrate 0: exit 2, "the split leaves the held_out side empty (0 of 1129 rows)" (train_cmd.rs:377-409).

## (c) DONE
`fit --out <file>`, `dataset --out <file>`, a symlink to a file, `/dev/null`: exit 2, "--out <path> exists and is not a directory" (train_cmd.rs:218-226, called at :248, :356); the file is untouched. See Low defect for `--out <file>/sub`.

## (d) DONE
`dataset --by time --calibrate 0.2 --holdout 0.2 --seed 7` on `fixtures/m4`, `fit`, `verify --rebuild`, `report`: bundle 22b26e5e9bb4be87b491971df474777e39e91bfc44bbc608cebc0999a5158a23; 1129 rows; controller 0.157443 / 0.492302; generator 0.175364 / 0.577418; paired −0.011781 INCONCLUSIVE. All exit 0.

## (e) DONE, with two scope notes
`git diff --stat 66b1910 HEAD`: train_cmd.rs, bundle.rs, dataset.rs, m4_train.rs, m4_corpus.rs, m4_probes.rs (all on the Owns list; bundle.rs change is input loading only), specs/lanes/ (the director's brief and review commits), and `features.rs` (+2 lines, outside Owns; below). fmt clean; clippy `-D warnings` clean (1.90). `cargo +1.90.0 test --workspace --no-fail-fast`: 320 passed, 2 failed, 0 ignored; the two are the Docker-only m0 tests (`exec_resolves_the_session_task_and_refuses_without_one`, `selftest_is_reproducible_structurally_excluding_duration_ms_by_name`). `git status` clean, HEAD 8f1d347, branch r5b4.
`features.rs:23, :34`: `#[serde(deny_unknown_fields)]` on `AbsentFeature` and `FeatureSpec`. Not needed for the brief's forms: `version:5`, `names:"x"`, `absent:[{}]` already fail without it. It only makes `"seed":"x"` (a key `features.json` does not have) an error, and bundle.rs could do that by key check, as it does for the manifest. It does not break the chain (bundle's own `features.json` round-trips; verify passes).
The brief asks for one commit; there are two (de96622, then 8f1d347 for the unknown-field follow-up), both naming F9 and B3.

## New defects
- **Medium:** (a) — wrong-typed fields in `data-manifest.json` accepted, exit 0; `unindexed_tasks` silently read as 0. Fix: deserialize into a struct with `deny_unknown_fields` and add the row to both tests.
- **Low:** duplicate keys in `data-manifest.json` are accepted (last wins; `Value` parse); `features.json` and `split.json` refuse them.
- **Low:** `fit --out <file>/sub` and `--out <dangling symlink>` still exit 1 (`io: Not a directory` / `io: File exists`); same for `dataset --out`.
- **Low:** `features.json` `learning_rate: 1e308` makes `fit` exit 0 with `null` weights in head.json; `verify`/`report` then exit 2, "json: invalid type: null, expected f64", no file named. `-1` and `0` are also accepted. Value, not structure; older than this item.
- **Low:** the empty-side check is in `run_fit`, not `write_bundle`, so the library call still writes an empty-side bundle (deliberate, per the comment at train_cmd.rs:377-380); the dataset is loaded twice.
- **Low:** `load_bundle` still uses `serde_json::from_str(...)?` (bundle.rs:709-713), so a digest-valid but malformed bundle file would say "json: …" with no file name (exit 2 anyway). I did not forge one.

## What I ran
Acceptance chain; 108-run matrix (3 files × 9 forms × fit/report/apply/verify); 20 `data-manifest.json` key mutations; 348 runs of 29 own mutations on the three files (BOM, UTF-16, NUL, 200k-deep `[`, deep `{"a":`, trailing garbage, two documents, huge/negative numbers, duplicate keys, directory, symlink loop, `/dev/null`, 1 MB padding, CRLF): all exit 2 naming the file or are valid JSON, no exit 1, no panic, no timeout, except the Low items above; 12 share settings; 14 `--out` cases; fmt, clippy, workspace tests as above. Scratch in `/home/claude/r3/m4/rv54.*` (outside the tree).
