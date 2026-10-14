**M4 r5a: done** — the data-root forge now exits 2, an untouched bundle verifies under any `TSR_CODE_COMMIT`, verify/report/apply agree on every tamper, and nothing outside the Owns list moved.

## P2 — DONE (report.rs:270-276, used by report and apply)
Acceptance corpus (`--by time --calibrate 0.2 --holdout 0.2 --seed 7`, `fit --support-floor 30` → 22b26e5e…). A throwaway test set all 215 held-out `generator_raw` to 0.99 and recomputed ids and roots with `row_id_of`/`data_root`. The forged dataset loads. Called directly, `report()` still gives "all" conclusive [−0.088582, −0.013317]. Through the CLI, `report` and `apply` both **exit 2**: "dataset data root 0fc90bc4… does not match the bundle's 0a0c5f97…" (both roots named; apply prints no rows). On the untouched dataset the headline is unchanged: controller 0.157443/0.492302, generator 0.175364/0.577418, n=152, all −0.011781 [−0.036676, 0.015435] INCONCLUSIVE. The seed-7 report equals docs/m4-evidence.md:196-385, apart from the new code_commit line.

## P1 — DONE (bundle.rs:710-715; train_cmd.rs:387-394; report.rs:591)
I fit a bundle with `TSR_CODE_COMMIT=08d8c2a`. With the variable unset, set to 08d8c2a and set to abc123: `verify` (including `--rebuild`), `report` and `apply` all exit 0. verify and report print `code_commit (recorded, not verified)=08d8c2a`. The `--json` key is plain `"code_commit"` (informational).

## F6/B8 — DONE (load_bundle bundle.rs:594-703; train_cmd.rs:384→715, 418, 494)
verify / report / apply exit codes:
- comment, whitespace, forged calibrator, `state="active"`: 2/2/2.
- missing head.json, one-byte isotonic.json change: 2/2/2, each naming the file.
- **code_commit edited cleanly, trailing newline kept: 0/0/0.** All three print `deadbeef` as recorded.
- code_commit edited via `lines().join("\n")`, single-quoted, or with only the newline dropped: 2/2/2 ("not the canonical rendering").

This matches the brief: the edit is refused only when the rendering changes, and all three commands give the same verdict.

## D — DONE
`git diff --stat 08d8c2a 23e9342` touches only train_cmd.rs, m4_train.rs, bundle.rs, report.rs and 2 files under specs/. fmt passes on 1.97 and 1.90. `cargo +1.90.0 clippy --workspace --all-targets -- -D warnings` exits 0 after a touch. `cargo test --workspace --no-fail-fast` gives **280 passed, 2 failed**: the Docker-only `exec_resolves_the_session_task…` and `selftest_is_reproducible…`. That is up 5 from 275, and all new p1_/p2_/f6_ tests pass.

## New defects
- **Low (test honesty), m4_train.rs:1397-1411, bundle.rs:1468.** The "edited code_commit line" tamper is refused only because `.lines().join("\n")` drops the trailing newline. A clean edit is accepted, as the operator decided. The 23e9342 message ("shown to refuse it") overstates this, and the docs run must not cite this test as evidence that a code_commit edit is refused.
- **Informational.** There are two commits, not one (ea387ec names P1, P2 and F6; 23e9342 is test-only). verify loads the bundle twice (train_cmd.rs:389).
- Not run: the new tests against 08d8c2a.

## What I ran
- Acceptance `dataset`/`fit` with the variable unset and set to 08d8c2a: same id both times.
- verify, report and apply under 3 environments.
- report at seeds 0 and 7, with seed 7 diffed against the doc.
- Throwaway crates/tsr-learn/tests/zz_review_r5a.rs (forge, plus a check-bypassing `report()`), then report/apply on the forge. Removed afterwards.
- 10 tampers × 3 commands.
- fmt, clippy 1.90, full tests.
- `git status` is clean at 23e9342.
