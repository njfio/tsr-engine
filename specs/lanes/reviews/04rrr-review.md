**Lane 04rrr (M2, third repair): not merge-ready** — the CLI's behaviour is right on every item and the workspace is green apart from the two Docker-only tests, but besides the missing evidence doc, two reconcile tests still assert results the CLI cannot produce (item 3), and a few test comments still imply the CAS takes part (item 2); fix both in the same short run as item 7.

## Items

**(1) D11 residual sign: FIXED.**
- The generator now writes `f.raw.get() - truth` (`m2_episodes.rs:445`). The new test is `d11_fixture_surprise_residual_is_forecast_minus_observed` (`:770`).
- My own Python check reads the observed outcome from each episode's `outcome_recorded` label. All 90 Surprise residuals across the 30 files equal raw minus observed; 0 are wrong.
- The Mirror over alpha-00 prints `by generator residual 0.4670` for p=0.4670 with outcome 0.
- `TSR_REGEN_FIXTURES=1 cargo test -p tsr-cli --test m2_episodes` passes 8/8 and leaves the episode sha256s unchanged.

**(2) D1/N3 reconcile and the CAS: PARTLY.** The behaviour is fixed. Some comments are not.
- The code is clean: `ToolCheck` holds only the idempotent set, and `check()` always answers `Unknown` (`reconcile_cmd.rs:129-135`). `evidence_digest` and `from_log`'s keys and digests are gone.
- Scenarios on real stores through the CLI:
  - (a) RunApprovedTest, `cas/<digest>` present, no Executed: `unresolved retryable`, exit 0.
  - (b′) ProposePatch D→Executed→D: `unresolved blocked`, exit 1; the log ends `dispatched, executed, dispatched, unresolved`.
  - (c) round 1 `retryable` (exit 0); rounds 2 and 3 `refused` (exit 2).
  - (d) D→Executed→D with no cas: `unresolved retryable`.
- The limitation is stated in `docs/m2-evidence.md:368-376` (with the lane-01 change that lifts it) and in `reconcile_cmd.rs:70-74`.
- Residue (low). These comments and names still imply the CAS takes part:
  - `m2_reconcile.rs:772-773`: "The CLI's check, built from the log and the CAS".
  - `:916`: "With no digest in the CAS the outcome is unknown".
  - `:1028`: the name `d1_reconcile_resolves_idempotent_dispatch_with_digest_in_cas`.
  - `:1059-1060`: "the CAS digest in the log settles it".
  - `:1077`: "the CAS digest is the evidence".
  - `reconcile_cmd.rs:380`: "the CAS root is its sibling `cas/` directory".

**(3) D5 crash matrix: PARTLY.**
- `d5_matrix_rows_reconcile_the_store_their_crash_point_left` (`m2_reconcile.rs:679`) builds four different stores.
- I rebuilt each on disk and ran `tsr task reconcile`:

  | Row | CLI output | Matches the test |
  |---|---|---|
  | Intent | `no pending dispatches` | yes |
  | Send | `unresolved retryable` | yes |
  | Acknowledgement | `unresolved retryable` | yes |
  | None | `no pending dispatches` (Executed kept) | yes |

  Send and Acknowledgement differ only by a `cas/` file the CLI never reads. That follows from the operator's decision.
- Not done: "remove assertions the CLI cannot produce".
  - `reconcile_task_settles_every_pending_dispatch_once` (`:796-858`) feeds a hand-built `Fixed` reconciler answering `Executed` and asserts `Resolution::Executed` (`:843`). Over the same store the CLI prints two `unresolved retryable`. `Executed` is unreachable from the CLI.
  - `row_intent_non_idempotent_unknown_is_blocked` (`:345-355`) asserts `UnresolvedBlocked` for a `RunApprovedTest` candidate. The CLI gives `unresolved retryable` for that store.
  - Both `row_intent_*` tests (`:308-370`, "Row 1: crash at intent") reconcile a Dispatched store, which contradicts the matrix's own Intent row (nothing dispatched).
  - The module doc (`:3-7`) still says the intent row "is reconciled to at most one `Executed`".

**(4) D9 intent: FIXED.**
- `let intent: Option<String> = None` (`reflect_cmd.rs:276`).
- Over alpha-00 the Mirror prints `intent: (none recorded)` and `--json` gives `"intent": null`. The criterion text appears only on the `criterion 9336872f1b30 [open]` row.
- The tsr-domain change is named in `m2-evidence.md:184-190`.

**(5) N1 the last Stop decides: FIXED** (`resolve.rs:337-342`). Each log below was built on a real store, labelled with `tsr review <task> accepted` through the CLI, then resolved:
- Stop{operator} → Executed → Stop{done} → accepted: **True**.
- TaskClosed{operator} + accepted: Censored.
- A lone Stop{operator} + accepted: Censored.
- Stop{done} → Stop{operator}: Censored.
- See P1 for one mixed case.

**(6) N2: FIXED.**
- `main.rs` differs from d2b5f96 only by the M2 additions. The `Open`/`Status`/`Export`/`Forecast`/`Hook`/`Exec`/`Policy` arm bodies are identical. Reconcile is dispatched before the store opens (`:286`).
- I built d2b5f96's `tsr` and ran `task status foo` and `task export foo` on both binaries:
  - Missing store, no `TSR_DB`: identical stdout, exit 0, `.tsr/tsr.db` created.
  - `TSR_DB` at a missing path: identical, file created.
  - An M0-only store: byte-identical output.

**(6) R7: FIXED for lane tests.**
- After `cargo test --workspace`, the only new files in the tree are the ignored `target/{hook-unit,m0-forecast-default,m0-harness}` and `crates/tsr-cli/.tsr/session`.
- Those are written by main's `hook.rs:989`, `m0_forecast_default.rs:31` and `m0_harness.rs:39,:460`.
- No `m2_*` or tsr-ledger test writes into the tree; all other sha256s are unchanged.

**(6) R8: FIXED** (`write.rs:162-169`). A forged Contrast appended directly as Observed gives `Err(ContrastIdMismatch)` from `expand`. A genuine one appended directly still expands.

**(6) D4 message: FIXED** (`review_cmd.rs:537`). `tsr review --family alpha` exits 2 with `tsr: review needs a task id (or --stats to compute over a family)`.

**Previously fixed items still hold:** R1 (`m2_admission` 3/3); R2/R3 (see 5; `m2_resolution` 4/4); R4 (scenarios b′, c, d); R5 (missing store: exit 2, nothing created; `TSR_DB` works from an empty directory); R6 (see 1); D4 (`--stats` prints `labels=10 … 0.2857`); D9 propensity (code untouched, its test passes); D10 (10/10); D12 (`note` on an unknown task or missing store exits 2, creates nothing).

## New defects

**P1 (low): an operator cancellation after any hook Stop no longer censors.**
- `resolve.rs:337-341` lets a `TaskClosed{operator}` decide only "when no Stop was recorded", whatever the order.
- Repro: Executed → Stop{done} → TaskClosed{operator} → accepted resolves **True**. At 6eb8919 it was Censored, and §B7 says an operator stop for external reasons censors.
- Only the library's cancellation path (`record_cancellation` plus the caller's TaskClosed) writes this; no CLI command does.
- Fix: let the last close event, Stop or TaskClosed, decide.

No other regressions: `reflect --family` output is byte-identical to the doc; `review rejected --return` supersedes `62ad49a1c843` and adds an open preference; `note` and `reconcile` behave as documented.

## Evidence doc

Item 7 must change `docs/m2-evidence.md` as follows:
1. **Line 5** says "Nothing here is hand-edited". This is false: the Mirror blocks were hand-patched.
2. **Ledger root, lines 144 and 198:** the doc has `4bd309c2…6ef3e7`; the tip prints `a079b2f422f581d568aefb2364e2fbc60c3da13764c6d0a4d961ff69d452b1c9`.
3. **Label return row, lines 166 and 286:** the doc has `fd4069f0895b`; the tip prints `62ad49a1c843`. Re-capture both blocks. The sign text at lines 181-182 (+0.4670) is now correct.
4. **m2_episodes blocks, lines 33-50 and 76-90, and the list at 51-66:**
   - They show 7 tests; the tip has 8. `d11_fixture_surprise_residual_is_forecast_minus_observed` is missing.
   - Line 60 and lines 124 and 568 say the generator rebuilds `fixtures/m2/store/tsr.db`. A plain run builds only in a temp dir; the store appears only with `TSR_REGEN_FIXTURES=1` (`m2_episodes.rs:849-852`). The same applies to `README.md:152`.
5. **Workspace test block, lines 540-561:**
   - It is elided ("…") and claims 256 passed, `m2_reconcile` 21 and `m2_episodes` 7, with "no Docker-only test skipped".
   - The tip has 263 tests: `m2_reconcile` 20 and `m2_episodes` 8. On a host without Docker, 261 pass and the 2 Docker-only tests fail.
   - Quote the full output and name the host's Docker state.
6. **Missing commands:**
   - `git diff --stat main -- crates/tsr-cli/tests/m0_reconstruct.rs fixtures/m0` (empty).
   - The superseding second label (the Mirror after `review rejected` shows `supersedes 62ad49a1c843`).
   - The command that builds `.ev-d6/claude-session.db` for the block at line 580.
   - A reconcile over a pending dispatch, showing the limitation (`unresolved retryable`).
   - `rustc --version` beside the clippy block.
7. **Record and note ids, lines 350 and 358:** these are timestamp-derived. Re-capture them in the same run.

## Definition of done

**04rrr:**
- build, fmt: **MET**.
- clippy `-D warnings`: **MET for lane code.** On 1.97 only `filter_next` (`tsr-policy/gate.rs:411`) and `collapsible_match` (`exec.rs:482`) fire, both in main's untouched files and likely 1.97-only; with them allowed, exit 0.
- Workspace tests: **MET** (261/263; the 2 failures are Docker-only: docker CLI present, `docker info` exit 1).
- Protected files: **MET.** Empty diff against d2b5f96 for `fixtures/m0`, `hook.rs`, `exec.rs`, `store_path.rs`, `task.rs`, the m0 tests and `tsr-policy`; `lib.rs` adds only 4 `pub mod` lines; M0 arms identical.
- Residual sign test, limitation documented, Mirror never guesses, `task status`/`export` as on d2b5f96: **MET**.
- No CAS-decides code or comment: **PARTLY** (test comments, item 2).
- Four rows on different stores with CLI results: **MET**; but "remove assertions the CLI cannot produce": **NOT MET** (item 3).
- The last Stop decides: **MET** (P1 aside).
- No test writes into the tree: **MET** for lane tests (main's do, see R7).
- Evidence doc verbatim with the full summary: **NOT MET**.

**Original brief:** **MET** for admission; Surprise never expands while Contrast does, with lineage; the matrix (at most one Executed, blocked non-idempotent, UPDATE refused beside an append); `review rejected` with an Operator preference and supersede (needs `--return`, since every fixture return is labelled); `--json` five sections with `raw p`/`uncalibrated`; `--family` bins and Brier; no "percent certain" or TSR term with a number (grep over all reflect output); "synthetic" stated. **NOT MET:** the evidence doc quotes stale output.

## What I ran

All with `CARGO_TARGET_DIR=`, rustc 1.97.
- `cargo test --workspace --no-fail-fast`: 263 tests, 261 passed; failures `m0_forecast_default::exec_resolves_the_session_task_and_refuses_without_one`, `m0_harness::selftest_is_reproducible_structurally_excluding_duration_ms_by_name` (Docker).
- `git status --ignored` and a sha256 diff after the run: only main's test outputs (R7).
- Python check of the 90 residuals: 90/90 raw minus observed.
- `TSR_REGEN_FIXTURES=1 cargo test -p tsr-cli --test m2_episodes`: 8/8, episodes byte-identical.
- `tsr reflect` alpha-00 (text, `--json`) and `--family alpha|beta|gamma` diffed against the doc: family blocks identical; Mirror blocks differ by root and label id.
- `review rejected` (with and without `--return`), `note`, `task reconcile` on a scratch copy; `review --family alpha` with and without `--stats`; R5/D12 shell checks.
- Throwaway `crates/tsr-cli/tests/zz_r3.rs` (removed): reconcile a–d, crash rows and the two contested tests' stores via the CLI, resolution cases, R8.
- d2b5f96's `tsr` built by checkout + `cargo build -p tsr-cli --bin tsr` (copied to scratchpad; tip rebuilt identical): `task status/export` compared.
- `cargo fmt --all -- --check`: exit 0. `cargo clippy … -D warnings`: exit 101 (`gate.rs:411`); with the two lints allowed, exit 0.
- `git diff d2b5f96 4ade3c8` over the protected paths and `main.rs`.
- Clean-up: removed `zz_r3.rs`, `fixtures/m2/store`, `target/`, `crates/tsr-cli/.tsr`; tree sha256 equals the pre-review listing, `git status` clean.
