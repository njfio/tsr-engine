**Lane 06r (M4 repair): not merge-ready.** The repair added a bug: the shipped isotonic calibrator is now non-monotone. `--by repo` still collapses the fixture into one all-fit unit, clippy 1.90 fails on the lane's own code, and item 10 was never done, so docs/m4-evidence.md still claims a conclusive win that the code no longer shows.

## Defects A1–A14, B1–B16

**A1 — FIXED.** decision.rs:97-119, 149-159 now index each task on its own. Repro: two sessions through the real `tsr hook` (SessionStart, Read) → `tsr forecast --kind propose_patch --args src/a.rs --p 0.9` → `hook` Edit, then `tsr train dataset --store`. Both rows have index 5, `generator_raw` 0.9 and `generator_missing` 0.

**A2 — PARTLY FIXED.** `make_split` (dataset.rs:146-194) now unions lineages correctly. But synth.rs:282-290 still builds clone groups from contiguous task indices with random lineages: all 22 groups span lineages, so the union is a single unit. `--by repo` for seeds 1–8 gives fit 1138, calibrate 0, held_out 0 every time. `fit` "accepts" it only because the split is degenerate (see R2).

**A3 — FIXED.** decision.rs:155 requires `at < index`. probes.rs:117-137 recomputes the generator p from the truncated log. Repro (throwaway test): I appended a generator forecast after the close that carries the commitment's version, and the rows were unchanged.

**A4 — FIXED.** probes.rs:163-211. I planted `label + 0.001·j` into `budget_fraction_used`, and the probe flagged it as `feature:budget_fraction_used`.

**A5 — FIXED.** dataset.rs:195-257. Ties of 2–5 tasks, with clone groups stripped or kept: `dataset` 0 and `fit` 0 in all eight runs.

**A6 — FIXED.** bundle.rs:226-282 and probes.rs:323-401. A forged split.json alone made `fit` exit 2 (split_digest disagrees). Forged with its digest recomputed and data-manifest updated, it still exits 2 with "boundary" ×76.

**A7 — PARTLY FIXED.** `dataset` runs the probes before it writes anything and names the row or feature (train_cmd.rs:330-347). `fit` still prints only probe names (bundle.rs:413-423). Repro: label planted into `test_runs_so_far`, ids and roots recomputed → "probe caught a leak: post_action_events, post_action_events, … label_reach". Several hundred names, and no row or feature.

**A8 — NOT FIXED.** dataset.rs:436-438 still drops tasks that have no index row without counting them. Repro: `--tasks` limited to the first 300 lines → 903 rows. Neither stdout, data-manifest.json nor the report mentions the 80 dropped tasks.

**A9 — FIXED.** synth.rs:487-491 forecasts from the same state the label is drawn from (synth.rs:322). Python over the dataset, mean generator p against label rate at decision positions 0–3: calibrated 0.756/0.734, 0.756/0.734, 0.742/0.683, 0.756/0.792; overconfident ≈0.955/0.63 at every position.

**A10 — FIXED.** synth.rs:403-415. Under the acceptance split every side holds all four families (rare: fit 36, calibrate 9, held_out 13 rows).

**A11 — FIXED.** features.rs:150-157 and 302-308. A real Edit with no forecast gives `generator_raw` None, logit 0 and `generator_missing` 1.

**A12 — FIXED.** bundle.rs:396-401. holdout-access.jsonl line counts: 222 after dataset → 444 after fit → 666 after verify --rebuild → 888 after report.

**A13 — FIXED.** lane.diff: the workspace Cargo.toml changes only the member line, and crates/tsr-cli/Cargo.toml gains one dependency line. `float_roundtrip` is now in crates/tsr-learn/Cargo.toml:20.

**A14 — FIXED.** features.rs:205-215 counts committed patches. Python over all 1138 rows: 0 mismatches against the committed ProposePatch count before each commitment.

**B1 — FIXED for `report`.** report.rs:245-283. A `--calibrate 0.1 --holdout 0.6` dataset and a 22-feature dataset both exit 2, and the 22-feature one also exits 2 on `apply`. `apply` accepts a foreign split, though (R4).

**B2 — FIXED.** bundle.rs:226-282. With all 470 fit labels flipped, `fit` exits 2 with "rows.jsonl: row … has id …, recomputed …".

**B3 — PARTLY FIXED.** A missing events.jsonl or tasks.jsonl now exits 2 naming the file (train_cmd.rs:367-377). An empty or truncated events.jsonl still panics at probes.rs:111 (exit 101). An empty tasks.jsonl silently turns off probe (a): a planted post-action leak on 5 rows is refused with the real tasks.jsonl, but with an empty one `fit` accepts it (id 78371121…).

**B4 — PARTLY FIXED.** In the text report an empty family prints `n=0 brier=n/a` and `mean=n/a`, and an uncovered floor prints `brier=n/a`. But empty reliability bins still print `mean_forecast=0.000000 observed_frequency=0.000000` (71 lines in the acceptance report; report.rs:193-195). In the JSON, the empty `rare` pair is `"mean": 0.0` (report.rs:489-496).

**B5 — FIXED.** report.rs:347-352. An independent recompute from `tsr train apply` plus rows.jsonl gives n=168 for both forecasters: controller 0.179570, generator 0.202899, the same as the report.

**B6 — FIXED.** report.rs:409-425. Independent recompute: the per-task mean is −0.021880 (the report's value) against a per-row mean of −0.023329. Overconfident `tasks=20` counts tasks.

**B7 — FIXED.** bundle.rs:84-133 returns `Applied { raw_p, calibrated: Option<Calibrated>, support }`. `tsr train apply` prints rare held-out rows as `uncalibrated support=…`. Formatting slip: see R8.

**B8 — PARTLY FIXED.** `verify` now refuses a forged `state`, `calibrator`, `seed` or `support_floor`, naming manifest.toml (bundle.rs:636-651). Three gaps remain:
- An edit to `code_commit`, whitespace or a comment still verifies with exit 0.
- A deleted head.json gives "io: No such file or directory" with no file name.
- `load_bundle` (bundle.rs:520-590) never checks the calibrator, so `report` and `apply` exit 0 and print the forged `calibrator 000000000000 features@v1`, which is the original B8 repro.

**B9 — FIXED.** A non-empty `--out` exits 2 (bundle.rs:364-374). `--rebuild` uses a temporary directory and leaves no `rebuild/` behind. `report` writes `<bundle>.report.{json,txt}` beside the bundle, naming the id and the split digest.

**B10 — FIXED.** Bins carry n, mean and frequency and print `n=`. Population prints `tasks=`, and `coverage` appears only in the floor sweep. (The empty-bin zeros are covered under B4.)

**B11 — FIXED in behaviour.** Rare rows in held_out print `uncalibrated` (report header: `uncalibrated 11`). The test at m4_train.rs:398 still passes on the header word alone.

**B12 — FIXED** (see A12).

**B13 — NOT FIXED.** Item 10 never ran. docs/m4-evidence.md:312-314 and 346-352 still quote the pre-repair numbers −0.114231, "conclusive". Line 422 still says `.tsr/store.db`. Step 5 at line 440 refits into an existing directory, which `fit` now refuses. The doc never says the controller's log loss is worse.

**B14 — NOT FIXED.**
- Interval bounds print raw, not through Proxy (report.rs:645).
- `interval n/a interval n/a` still prints twice (report.rs:646-649).
- The `--json` output has no clip field.
- `controller_logged` prints last rather than in name order (report.rs:568-576).

**B15 — NOT FIXED.**
- Standardization statistics are still in head.json, and the learning rate (head.rs:20) is recorded nowhere. features.json has only version, names, widths and absent.
- Support counts unlabelled rows: the index has 916 entries, 275 of them unlabelled (bundle.rs:300-309).
- In the floor sweep all 171 calibrate rows count themselves (report.rs:444).
- PAV now pools ties but no longer stays monotone (R1).

**B16 — NOT FIXED.** There are still no tests for "a different seed changes the id", "TSR_CODE_COMMIT leaves the id unchanged" or "no TSR term followed by a number". The behaviour itself is right: seed 8 → a3bbf478…; `TSR_CODE_COMMIT=deadbeef` → the same 43271a72…; grep finds no term followed by a number.

## Headline numbers

The acceptance commands of the original brief, on the fixture:

```
tsr train dataset --events fixtures/m4/events.jsonl --tasks … --labels … --by time --calibrate 0.2 --holdout 0.2 --seed 7 --out m4-data   # 1138 rows
tsr train fit m4-data --out models/m4-synthetic          # 43271a72941608e6…
tsr train report models/m4-synthetic --dataset m4-data   # seed 0
tsr train verify models/m4-synthetic --rebuild m4-data   # exit 0
```

On the same 168 rows, controller Brier is 0.179570 and generator Brier 0.202899. Log loss goes the other way: 1.107560 against 0.670951.

Paired difference (controller − generator), per task:

| Group | Tasks | Mean | 90% interval | Verdict |
|---|---|---|---|---|
| Overall | 58 (12 clusters) | −0.021880 | [−0.062228, 0.014044] | INCONCLUSIVE |
| overconfident | 20 | −0.049928 | [−0.116575, 0.002583] | INCONCLUSIVE |
| calibrated | 19 | +0.043673 | [0.009598, 0.078600] | conclusive (controller worse) |
| underconfident | 19 | −0.057908 | [−0.119640, 0.007844] | INCONCLUSIVE |

With `--seed 7` the overall interval is [−0.063087, 0.013995] and the overconfident interval [−0.112715, 0.001796].

So the original claim does not hold. The swarm rewrote m4_train.rs:520-601 (the test is still named `the_controller_beats_the_generator_on_held_out`) to assert INCONCLUSIVE overall and for overconfident, and a conclusive positive result for calibrated. Its doc comment says docs/m4-evidence.md "says so plainly", but it does not (B13).

These numbers also come from the non-monotone calibrator (R1). With the one-line PAV fix (run in tree-a, then reverted; isotonic.rs sha256 restored):
- Controller Brier drops to 0.168907, and log loss to 0.787067.
- Overall becomes −0.032028 [−0.069923, 0.001731], still INCONCLUSIVE.
- Overconfident becomes −0.069377 [−0.138606, −0.012168], conclusive.

The rewritten test would fail once R1 is fixed.

## New defects

**R1 (high).** The repair's tie pooling made PAV non-monotone and dependent on row order (isotonic.rs:50-56). The `continue` skips the merge loop. Repro: xs=[.1,.1,.7,.7,.7] with ys=[1,0,1,0,0] gives [(0.1,0.5),(0.7,0.333)], which decreases. With ys=[1,0,0,0,1] it gives [(0.7,0.4)]. The acceptance bundle's isotonic.json has 3 decreasing steps (1.0→0.5, 1.0→0.6, 1.0→0.333). The isotonic test at isotonic.rs:150 checks a single tie group only.

**R2 (high).** `fit` accepts empty calibrate and held_out sides (bundle.rs:376-384 checks only the mature count). It then writes an empty calibrator, which acts as the identity, and `Bundle::apply` returns `Calibrated { p = raw p }` for every family (bundle.rs:113-124). Repro: `--by repo --seed 1` → `fit` exit 0, `isotonic.json` `{"breakpoints":[]}`, `apply` prints `raw_p=0.932118 p=p=0.932118 calibrator=8a4670913…`.

**R3 (medium).** `cargo +1.90.0 clippy --workspace --all-targets -- -D warnings` (the DoD toolchain, installed here) fails: `too_many_arguments` at crates/tsr-learn/src/report.rs:1051, a lane test helper. The 1.97-only lints are not counted.

**R4 (medium).** `tsr train apply` never checks that the dataset belongs to the bundle (train_cmd.rs:420-453). With the `--holdout 0.6` dataset it exits 0 and prints 664 "held_out" rows, mostly the bundle's own training rows, with calibrated p.

**R5 (low).** Under `--by time`, a clone group whose members have different timestamps makes `dataset` refuse its own split. `make_split` places the whole unit at its earliest timestamp (dataset.rs:262-283), and the contiguity probe (probes.rs:279-311), which is stricter than the brief's rule, then rejects it. Repro: give tasks 10 and 300 one clone group → exit 2 "a Fit task (2026-11-28) follows a Calibrate task".

**R6 (low).** `--tasks`, required by the brief, is now optional (train_cmd.rs:287-296). Without it every task silently becomes its own lineage with family `unknown`.

**R7 (low).** `tsr train apply` prints `p=p=0.851852` and `raw_p=` instead of `raw p` (train_cmd.rs:469-481; Appendix A rule 3).

## What I ran

- `cargo build --workspace --all-targets` (1.97) → ok, after `cargo clean -p` of the five workspace crates (target-a held binaries built from `…/m4r/tree`; results identical).
- `cargo fmt --all -- --check` → 0 with both 1.97 and 1.90. `cargo +1.90.0 build --workspace` → ok. `cargo +1.90.0 clippy …` → fails (R3).
- `cargo test --workspace --no-fail-fast` → 242 passed, 2 failed. Both failures are lane 01 tests that need Docker (`docker` is on PATH but the daemon is down): m0_harness `selftest_is_reproducible…` and m0_forecast_default `exec_resolves…`. m0_reconstruct and m0_real_session pass.
- Fixture regeneration through `synth::generate_fixtures` into a temporary directory, then `cmp` with fixtures/m4 → all three files byte-identical.
- lane.diff → the only files touched outside lane 06 are the member line, the one dependency line, the Train arm of main.rs, and new specs/lanes briefs. Lane 01/02/04/05 files are untouched.
- The acceptance commands above, the report at seeds 0 and 7, and `apply --side held_out` with an independent Python recompute → numbers match.
- `--by repo` seeds 1–8 → dataset 0, fit 0, sides 1138/0/0. Tie files tie2–5 (stripped and kept) → 0/0. Clone group spanning time → dataset 2.
- Flipped labels, a forged split.json, a missing, empty or truncated events.jsonl and tasks.jsonl, and a non-empty `--out` → as stated above.
- Seven manifest.toml edits and two content-file edits → `verify`; forged calibrator → `report` and `apply`.
- Foreign datasets → `report` and `apply`; 300-row index → A8; holdout-access.jsonl line counts → A12.
- Throwaway test crates/tsr-learn/tests/zz_review_repair_a.rs (left in tree-a): PAV, A3, A4, A6, A7, B3, B15.
- PAV one-line fix → fit and report, then reverted.
