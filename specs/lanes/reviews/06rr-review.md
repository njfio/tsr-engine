**Lane 06rr (M4, second repair): not merge-ready** — the core repairs hold (monotone PAV, a real three-way `--by repo`, real-hook pairing, an honest headline), but `fit`'s fail-closed guard only catches zero-byte files, `verify` now fails an untampered bundle when `TSR_CODE_COMMIT` differs, and the evidence doc's prose states the old defects as current behaviour; headline: on the same 152 held-out rows the controller beats the generator on Brier (0.157443 vs 0.175364) and log loss (0.492302 vs 0.577418), but the per-task paired difference is INCONCLUSIVE overall (−0.011781, 90% [−0.036676, 0.015435]).

## Items

**(1) R1/F1 — FIXED** (isotonic.rs:51-83). 3000 seeded random inputs, half heavily tied: always non-decreasing, unchanged under 3 permutations, and equal to a reference PAV. Hand case `[.1,.2,.2,.2]/[1,1,0,0]` → `[(0.2,0.5)]`. The acceptance isotonic.json has 0 decreasing steps of 10, as do all 8 repo bundles; no row prints p=1.000000.

**B15 — FIXED.** features.json holds the standardization stats and `learning_rate 0.5`. support.json has 637 entries = 492 fit-labelled + 145 calibrate-labelled (bundle.rs:344-353). The floor sweep subtracts the row itself (report.rs:454).

**(2) F2/A2 — FIXED.** All 22 clone groups sit inside one lineage. `--by repo` seeds 1–8 give three non-empty sides (rows fit/cal/held from 494/332/303 to 823/198/108); `fit` exits 0 on each, 8 distinct ids.

**F5 — FIXED** (decision.rs:155-183). Real hook: Read, then `forecast --source generator --p 0.9`, `--source operator --p 0.4`, PreToolUse → one row, `gen 0.9 op 0.4`. Synth stamps state_version = index+1 exactly as the hook does (1745/1745).

**Fixtures — FIXED.** `TSR_REGEN_FIXTURES=1` rewrote fixtures/m4 with unchanged sha256.

**(3) F3/R2 — FIXED** (bundle.rs:475-497). `--calibrate 0` → `fit` exit 2 naming "calibrate slice". `--calibrate 0.8 --holdout 0.2` → exit 2 naming "fit slice".

**F8 (task with no gated commitment) — PARTLY.** An index row for a task with no events → `dataset` 0, `fit` 0, but the task is silently dropped and counted nowhere (dataset.rs:447-456 says "Count such tasks"; the code does not).

**F8/R5 (clone group spanning time) — FIXED.** Index rows 10 (Feb) and 300 (Nov) in one group → `dataset` 0, `fit` 0. Probe (c) still catches a forged time split ("a Fit task (2026-12-31) follows a HeldOut task").

**F9/B3 — PARTLY.** The guard is `metadata.len()==0` (train_cmd.rs:353-364), so zero-byte files exit 2 naming the file. Everything else gets through:
- A `tasks.jsonl` that is one newline → `fit` **exit 0, id 22b26e5e… (the acceptance id)**. Probes (a) and (c) skip every row (probes.rs:108-110).
- An `events.jsonl` that is one newline → panic at probes.rs:111, **exit 101**.
- A truncated `events.jsonl` → **exit 1**, "json: EOF while parsing", with no file named.
- `dataset`: truncated events → exit 1; empty tasks/events → exit 0 with 0 rows.

**A8 — FIXED.** A 300-line index gives `unindexed_tasks=80` in stdout, in data-manifest.json and in the report.

**R6 — FIXED.** Omitting `--tasks` → clap exit 2.

**(4) F6/B8 — PARTLY.**
- `load_bundle` refuses a comment, a whitespace edit, a forged calibrator, `state="active"`, a missing head.json (named) and a one-byte isotonic.json change (named). verify, report and apply all exit 2 (bundle.rs:657-683).
- An edit to `code_commit` is refused only by `verify`, and only by comparing it with the verifier's environment (bundle.rs:708-714; see P1). `report` and `apply` exit 0 on it.

**F7 — FIXED as specified** (dataset.rs:360-395). Edits to six row fields each give exit 2 from fit, report and apply, naming the row. See P2.

**F14 — FIXED.** Exit 2, and the row id shown is the one in rows.jsonl.

**(5) R4 — FIXED.** A `--holdout 0.6` dataset → `apply` exit 2, "split digest … does not match".

**R7 — FIXED.** Output reads `raw p=0.188466 p=0.720000 (22b26e5e9bb4 features@v1) support=20`. Rare rows read `uncalibrated`.

**B14 population — FIXED.** `Calibrated.population` joins every family (bundle.rs:123-128).

**(6) F10/B4 — PARTLY.**
- Empty bins print `n=0` alone, and report.json has 77 `null`s.
- An empty *pair* is still `"mean": 0.0` in the JSON (report.rs:77, 501-508). Repro: drop the labels of the 24 held-out calibrated tasks, then `report --json` gives `{'tasks': 0, 'mean': 0.0, …}` for `calibrated`.

**F12 — FIXED** (report.rs:354-359). Operator n=84, all inside the 152 comparison rows. The old basis gave 91.

**F13/B14 — FIXED.** One-lineage report: `interval n/a` once per line. Bounds print as `interval_lo=`/`interval_hi=`. `--json` has `log_loss_clip`. Forecaster order checked in code only (report.rs:583-590): the fixture has no logged-controller rows.

**README documents apply — PARTLY** (P4). crates/tsr-learn/README.md:36 gives `apply … --task <id> --candidate <id>` (exit 2 "unexpected argument '--task'"; the flag is `--side`); :112 calls `raw p` "the generator's"; :112-113 says support counts "calibrate-slice rows"; :124 still puts the standardization stats in head.json.

**(7) F11 — FIXED.** `cargo +1.90.0 clippy --workspace --all-targets -D warnings` exits 0 after touching the sources to force a re-check.

**F15 — FIXED.** Under repo, the seeds give different sides and different ids.

**B16 — FIXED.** Seeds 7/8 → 9f9746d5…/ae477318…. `TSR_CODE_COMMIT=08d8c2a` → same 22b26e5e…. No "percent certain" or B2 term followed by a number in report, `--json` or `apply --side all`.

**A7 — FIXED.** I planted the label into `test_runs_so_far` on fit rows and recomputed ids, stats and roots. `fit` exits 2 naming row ids for `post_action_events` and `feature:test_runs_so_far` for label reach.

**(8) F4/B13 — PARTLY.** The report block (docs/m4-evidence.md:196-385) is byte-identical to `report --seed 7`; the headline table, paired block and plain statement are correct; the `--by repo` block reproduces; the test change is explained (:571-586); `.tsr/tsr.db` and a new directory are used. But the prose around the captures is wrong (P3).

## Headline

`tsr train report` on the acceptance corpus (seed 0 default; the doc and test use seed 7). 1129 rows; bundle 22b26e5e…; scored 159, excluded 56, uncalibrated 7 (rare). On the same 152 rows: controller brier 0.157443 / log_loss 0.492302; generator 0.175364 / 0.577418; operator (n=84) 0.183567 / 0.573390.

Paired controller − generator, per task:

| group | tasks/clusters | mean | interval, seed 0 | interval, seed 7 | verdict |
|---|---|---|---|---|---|
| all | 52/12 | −0.011781 | [−0.036676, 0.015435] | [−0.034651, 0.016411] | INCONCLUSIVE |
| calibrated | 18/9 | +0.070844 | [0.035615, 0.122933] | [0.034464, 0.125689] | conclusive, controller worse |
| overconfident | 17/8 | −0.002133 | [−0.072803, 0.051209] | [−0.076820, 0.052387] | INCONCLUSIVE |
| underconfident | 17/11 | −0.108916 | [−0.164347, −0.051151] | [−0.167237, −0.054255] | conclusive, controller better |

The controller does not demonstrably beat the generator, and the doc says so. Under `--by repo --seed 7`, overall is −0.032463 [−0.063735, −0.005100] "conclusive", but it rests on **2** clusters and the controller's log loss is worse there (0.688397 vs 0.540648).

## New defects

**P1 — medium: verify is environment-dependent** (bundle.rs:708-714; introduced by 8f339d7). An untampered bundle fails `verify` whenever the verifier's `TSR_CODE_COMMIT` differs from the fitter's. Repro: `TSR_CODE_COMMIT=08d8c2a fit` → 22b26e5e…; `verify` with it unset → exit 2 "code_commit \"08d8c2a\" is not the current \"unknown\"". `TSR_CODE_COMMIT=abc123 verify` of the acceptance bundle also → 2. No bundle can be verified from a later commit, while report and apply still accept a `code_commit` edit.

**P2 — medium: report and apply do not anchor the dataset to the bundle** (report.rs:251-289). They check the split digest and feature spec, never `dataset.data_root` against the bundle's `data_manifest`. Repro: set held-out `generator_raw` to 0.99 and recompute ids and roots with the public `row_id_of`/`data_root` → `report` exits 0 and overall flips to "conclusive" [−0.088582, −0.013317].

**P3 — medium: the evidence doc's prose is false** (introduced by 936802f/08d8c2a). docs/m4-evidence.md:564-569 states the pre-repair defects as current behaviour: `fit` "accepts an empty calibrate or fit slice and labels raw output Calibrated", `dataset` "refuses its own split when a clone group spans time", an empty events.jsonl "panics with exit 2". The binary contradicts all three. :538-542 lists `f1_*`, `f2_*`, `f6_*`, `r4_*`, `f10_*` tests in m4_train that are not there; :487 says apply prints the population (it does not). The 9ca7482 statement that the real exit evidence waits for lanes 04/05 and the operator (old :6, :450-452) and the corpus model table (old :26-29) were dropped.

**P4 — low: the README's apply section describes a CLI that does not exist** (README.md:36, 112-113, 124; introduced by 7fe5967).

**P5 — low, pre-existing or informational.**
- The paired lines print `mean=brier=-0.011781` (a doubled label, the `p=p=` pattern; report.rs:686).
- On rust 1.97, the lane's own code trips `sort_by_key` (dataset.rs:535) and `collapsible_match` (decision.rs:122, features.rs:325). The 1.90 DoD is unaffected.

## Definition of done

- `cargo build --workspace`: **MET**.
- fmt (1.97 and 1.90): **MET**.
- clippy 1.90: **MET**. On 1.97 it fails on main's gate.rs:411 and on the lane's three lints in P5.
- `cargo test --workspace --no-fail-fast`: **MET**. 275 passed; the 2 failures are Docker-only m0 tests (`selftest_is_reproducible…`, `exec_resolves…`; daemon down).
- Protected files byte-identical to d2b5f96: **MET** (empty diff). Outside lane files only the member line, dependency line and Train arm changed.
- Each item has a test naming its ids that failed before its fix: **CAN'T TELL**. The tests exist, but I did not run them on 9ca7482.
- Isotonic non-decreasing and order-independent: **MET**.
- `--by repo` 1–8, the real-hook pairing and byte-identical regeneration: **MET**.
- fit refuses empty slices and accepts dataset's own output: **MET**. The rowless tasks it drops go uncounted.
- Empty or truncated inputs exit 2 naming the file: **NOT MET** (F9/B3 above).
- load_bundle refuses any manifest edit and a forged calibrator or state: **NOT MET** (a `code_commit` edit is accepted by report and apply).
- Edited row fields refused: **MET**.
- apply refuses a foreign dataset; prints raw p, calibrated p or uncalibrated, and support: **MET**.
- No number printed for an empty set: **NOT MET** (JSON pair `"mean": 0.0`).
- Evidence verbatim, with the headline as printed: **MET** for the captures and the headline. The surrounding prose fails (P3).

Original brief: **MET** for the decision view, labels and probes (lane tests plus my (b)/(c) forges), forged split → fit 2, determinism (roots, id, `verify --rebuild`, same-seed report identical, seed changes id), the 150 minimum ("86 tasks…, need 150"), rare `uncalibrated`, one-byte change → verify 2 naming the file, comparison as computed, INCONCLUSIVE on a straddle, holdout-access.jsonl, no forbidden strings and calibrator version printed. **NOT MET**: the evidence no longer states that the real exit evidence waits for lanes 04/05 and the operator.

## What I ran

- build → ok; `cargo test --workspace --no-fail-fast` → 275 passed, 2 failed (Docker).
- fmt 1.97/1.90 → 0; `cargo +1.90.0 clippy …` → 0; 1.97 clippy → 101.
- Acceptance dataset/fit/report (seeds 0 and 7, `--json`)/`verify --rebuild` → 0/22b26e5e…/0/0; seed-7 report diffed against the doc → identical.
- `--by repo` seeds 1–8 → dataset, fit, isotonic monotonicity.
- Real-hook two-forecast session → `task export`, `dataset --store`.
- `TSR_REGEN_FIXTURES=1 … fixtures_regenerate_byte_identically` + sha256 → identical.
- Item 3 cases: `--calibrate 0`/`0.8`, rowless index task, clone group spanning time, no `--tasks`, 300-line index, empty/newline-only/truncated tasks and events for `dataset` and `fit`.
- 7 manifest/content tampers × verify/report/apply; `TSR_CODE_COMMIT` combinations; 6 row-field edits × fit/report/apply; apply/report on a foreign dataset.
- Labels-dropped `--json` report, single-lineage report, forbidden-term grep, 150-task minimum.
- Throwaway crates/tsr-learn/tests/zz_review_06rr.rs (PAV, recomputed-id forge, label-in-feature forge, time-split forge) → removed; git status clean at 08d8c2a.
