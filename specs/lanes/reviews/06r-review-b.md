**Lane 06 after repair, fresh review: not merge-ready.** The repair's change to PAV made the isotonic calibrator non-monotone, and the acceptance bundle's own `isotonic.json` shows it. `--by repo` puts all 380 tasks in fit, and `fit` then publishes an identity "calibrator". `docs/m4-evidence.md` still reports the pre-repair conclusive win.

## Brief items

1. **MET with gaps.** The pairing is task-local and the label cases resolve. Through the real hook, though, a generator forecast followed by an operator forecast never pairs (F5). Tasks with no index row are counted nowhere (`dataset.rs:437`).
2. **MET with gaps.** I measured the shape and the miscalibration on the acceptance dataset, at task level and at every decision position:

   | family | mean generator p | label rate |
   |---|---|---|
   | overconfident | 0.955 | 0.627 |
   | underconfident | 0.619 | 0.841 |
   | calibrated | 0.756 | 0.734 |

   `rare` has 12/4/4 tasks on fit/calibrate/held_out. Two problems: clone groups still span 2–6 lineages (`synth.rs:282-290`; F2), and the generator's state versions are ones the hook never writes (`synth.rs:519`; F5).
3. **MET with gaps.** The standardization statistics live in `head.json`, not in the spec. The learning rate is recorded nowhere (`head.rs:20`).
4. **NOT MET.** The repo split is degenerate (F2). `fit` refuses `dataset`'s own output (F8). The probes are not fail-closed (F9). A caught probe in `fit` names no row (`bundle.rs:413-423`).
5. **NOT MET.** The isotonic calibrator is non-monotone (F1). An empty slice yields a fabricated head or calibrator (F3).
6. **MET with gaps.** The support index holds unlabelled rows (916 entries, 641 labelled). In the floor sweep a calibrate row counts itself.
7. **MET with gaps.** See F3, F6 and F7.
8. **NOT MET.** Scoring runs on the broken calibrator, and `m4_train.rs:585-593` locks in a verdict that F1 produced. Also F10, F12 and F13.
9. **NOT MET.** See F4.

## Defects

**F1 — high (introduced by the repair): the isotonic calibrator is not monotone.**
- **Where:** `isotonic.rs:50-57`. A tied input is added to the last block with `continue`, so the merge loop never re-runs.
- **Minimal case:** `fit_isotonic([0.1,0.2,0.2,0.2],[1,1,0,0])` returns `[(0.1,1.0),(0.2,0.333)]`. I confirmed this with a throwaway test.
- **In the acceptance bundle:** `isotonic.json` has 30 breakpoints, including 0.4452→1.0 then 0.4473→0.5, and 0.6575→1.0 then 0.6830→0.333.
- **What `apply` prints:**
  - `raw_p=0.507334 p=p=0.828571`
  - `raw_p=0.648397 p=p=1.000000`
  - `raw_p=0.745489 p=p=0.629630`
- **Effect on the scores:** 52 of the 168 compared held-out rows get p = 1.0, and the controller's log loss is 1.108 (generator 0.671).
- **With a correct PAV** (pool ties, then merge), on the same bundle and data: Brier 0.1689 instead of 0.1796, log loss 0.787, and the overconfident paired difference becomes *conclusive* at [-0.140, -0.012]. The lane prints INCONCLUSIVE at [-0.117, 0.003].

**F2 — high: `--by repo` is degenerate.**
- **Repro:** `tsr train dataset … --by repo --calibrate 0.2 --holdout 0.2 --seed 1` (and `--seed 7`).
- **Result:** `{'fit': 380}` with calibrate 0 and held_out 0, exit 0. `training_root` equals `data_root`.
- **Cause:**
  - Synth draws a lineage per task while clone groups are blocks of indices (22 groups, spanning 2–6 lineages).
  - The union-find in `dataset.rs:178-193` merges all 12 lineages into one unit.
  - `round(1×0.2)=0` then empties both cut sides (`dataset.rs:294-299`).
- **Why the tests still pass:** the a2 tests pass on this split, and the split digests differ between seeds only because the seed is hashed in (`dataset.rs:335`).

**F3 — high: `fit` accepts an empty calibrate or fit slice and labels the result calibrated.**
- **Why:** there is only one minimum, over fit and calibrate together (`bundle.rs:378-384`).
- **Empty calibrate slice:** isotonic gets `[]`, and `apply_isotonic` returns its input (`isotonic.rs:105-107`).
  - `--calibrate 0`: `fit` exits 0 and `apply` prints `raw_p=0.841160 p=p=0.841160 calibrator=51a9fa0e2e48 features@v1`.
  - F2's repo bundle behaves the same way.
- **Empty fit slice:** with `--calibrate 0.8 --holdout 0.2`, `fit` exits 0. All weights are zero, raw p is 0.5 for every row, and every row is printed "calibrated" at 0.708268.

**F4 — high: `docs/m4-evidence.md` was not re-captured, and it contradicts the tip.**
- **What the doc shows:** `data_root=42af2d…`, bundle `18b623…`, `uncalibrated 0`, `population … support=66.000000`, and an overall result of "conclusive [-0.241970, -0.018035]".
- **What the tip prints:** `b7bd5f…`, `43271a…`, `uncalibrated 11`, `tasks=75`, and INCONCLUSIVE [-0.062228, 0.014044].
- **Missing runs:** there is no `apply`, `--by repo`, tampered manifest or foreign dataset.
- **Operator steps:** they use `.tsr/store.db` where the store is `.tsr/tsr.db`. Step 5 refits into the existing `models/m4-real`, which `fit` now refuses with exit 2.
- **Contradiction with the test:** `m4_train.rs:518` says the doc "says so plainly". It does not.

**F5 — high: generator forecasts lose their pairing on a real log.**
- **Real hook:** `tsr forecast --source generator`, then `tsr forecast --source operator` (§B7's approval forecast), then a PreToolUse hook. `train dataset --store` gives `generator_raw None operator_raw 0.4`. The hook stamps the generator forecast with state_version = commit index − 1.
- **Synthetic corpus:** synth stamps the generator forecast with the commit index even when an operator forecast follows (`synth.rs:519`).
- **Effect:** 607 of the 1138 fixture rows carry an operator forecast. On real data each of those would get `generator_missing=1` and drop out of the comparison set. That loss is not random.

**F6 — medium: the bundle's provenance can be forged.**
- **Where:** `load_bundle` (`bundle.rs:520-590`, used by `report` and `apply`) does not check `calibrator` or `state`.
- **Forged calibrator:** with `calibrator = "000000000000 features@v9"`, `verify` exits 2. But `apply` prints `calibrator=000000000000 features@v9` on every p, and so does the report header.
- **Other manifest edits:** `verify` exits 0 after editing `code_commit` and after adding comments, against the repair's "any byte".

**F7 — medium: rows.jsonl fields outside the row id are trusted.**
- **Where:** the row id covers only the commitment, the features, the label and the evidence (`dataset.rs:349-370`). Neither `fit` nor `report` checks `family`, `lineage` or `generator_raw` against the `tasks.jsonl` and `events.jsonl` in the same directory.
- **`family` edit:** renaming `rare` to `calibrated` in rows.jsonl → `fit` exits 0, the population shows `calibrated tasks=87`, and `uncalibrated 0`.
- **`generator_raw` edit:** setting held-out `generator_raw` to 0.99 → `report` exits 0, and the overall verdict flips to conclusive at [-0.117344, -0.002204].

**F8 — medium: `fit` and `dataset` refuse the tool's own output.**
- **A task with no gated commitment:** an index row for such a task makes `dataset` exit 0. `fit` then exits 2 with only `probe caught a leak: boundary` (`probes.rs:374-383`, `bundle.rs:420`).
- **A clone group spanning time:** `dataset` exits 2 ("Fit task … follows a HeldOut task"). The unit is ordered by its earliest timestamp (`dataset.rs:262-283`), but the probe checks contiguity per task (`probes.rs:279-312`).

**F9 — medium: the probe pass is not fail-closed.**
- **Empty `tasks.jsonl` in the dataset directory:** `fit` exits 0 with the same bundle id. Probes (a) and (c) skip every row (`probes.rs:108-110`, `292-294`).
- **Empty `events.jsonl`:** panic at `probes.rs:111`, exit 101.

**F10 — medium (introduced by the repair): numbers are printed for empty sets.**
- **Text report:** empty reliability bins print `n=0 mean_forecast=0.000000 observed_frequency=0.000000` (`report.rs:193-195`, `630-639`). That is 71 lines in the acceptance report.
- **`<bundle>.report.json`:** it writes `"mean": 0.0` for empty pairs (`report.rs:490-496`) and 0.0 for empty bins.

**F11 — medium (introduced by the repair): clippy fails on the lane's toolchain.**
- `cargo +1.90.0 clippy --workspace --all-targets -- -D warnings` fails on `too_many_arguments` at `report.rs:1051`.

**F12 — medium: the operator is scored outside the comparison set.**
- **Where:** operator and logged-controller pairs are taken from every scored row (`report.rs:340-346`).
- **Effect:** the operator's n=96 includes 8 rows from the uncalibrated `rare` family that the controller–generator comparison excludes.

**F13 — low: repair item 9 rendering leftovers.**
- `interval n/a interval n/a` (`report.rs:644-654`).
- Interval bounds are not printed through Proxy (`report.rs:645`).
- The JSON has no log-loss clip.
- `controller_logged` prints after `operator` (`report.rs:574`).
- `apply` prints `p=p=` and `raw_p=` (`train_cmd.rs:469-480`).
- `Calibrated.population` names only the row's family (`bundle.rs:122`).
- The README omits `apply`.

**F14 — low: wrong exit codes and an untraceable row name.**
- `report` and `apply` exit 1 on a tampered bundle or rows.jsonl (`train_cmd.rs:421-434`, `491-504`). The module's own contract says 2.
- `load_dataset` names a bad row by `Hash::of(commitment)`, an id that appears in no file (`bundle.rs:235`).

**F15 — low: the seed tests prove nothing.**
- **Under `--by time`:** seeds 7 and 8 give identical assignments, rows and head. The split digest and the bundle id still change, because the seed is hashed in (`dataset.rs:335`, `bundle.rs:175`).
- **Consequences:** `report` refuses the identical split built with seed 8 (exit 2). Combined with F2, the DoD's "different seed → different digest and id" tests prove nothing.

## What I ran

All runs used `target-b`. I forced a rebuild from tree-b, because the prebuilt binary carried `/m4r/tree/` paths.

- `cargo build --workspace` → ok.
- `cargo fmt --all -- --check` → 0.
- `cargo +1.90.0 clippy --workspace --all-targets -D warnings` → fails at `report.rs:1051`. With 1.97 the extra failures are 1.97-only lints in tsr-policy and tsr-learn.
- `cargo test --workspace --no-fail-fast` → all lane tests pass. `m0_forecast_default::exec_resolves…` and `m0_harness::selftest_is_reproducible…` fail because they need Docker, which is not reachable here.
- Acceptance commands:
  - `dataset` → exit 0, 1138 rows.
  - `fit` → `43271a72…`, exit 0.
  - `report` → exit 0. Read line by line: no "percent certain" and no TSR term followed by a number. Findings F10, F12 and F13.
  - `verify --rebuild` → exit 0, same id.
  - `report` with the same seed twice → identical.
- `train apply --side held_out` → 222 rows, 13 uncalibrated (`rare`). F1, F13.
- Python measurement of the corpus per family and per decision position, and of `isotonic.json` monotonicity → non-monotone.
- Throwaway tsr-learn tests (since removed):
  - The PAV minimal case → F1.
  - The report recomputed with a correct PAV.
- `--by repo` with seeds 1 and 7, then `fit`, `apply` and `report` → F2, F3.
- `--calibrate 0` and `--calibrate 0.8` → F3.
- Manifest edits under `verify`, `apply` and `report` → F6.
- rows.jsonl `family` and `generator_raw` edits → F7.
- An index task with no rows, and a clone group spanning time → F8.
- Empty `tasks.jsonl` and empty `events.jsonl` → F9.
- A two-forecaster session through the real `tsr hook` / `tsr forecast` → F5.
- `TSR_CODE_COMMIT=abc123 fit` → same id.
- Seed 8 under time, and `report` on that dataset → F15.
- A foreign split → exit 2.
- A tampered `head.json` or flipped label → `report` exits 1.
