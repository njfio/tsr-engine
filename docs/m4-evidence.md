# M4 (Calibrate) — lane 06 evidence

> **Every number in this document is SYNTHETIC.** It is produced from the seeded synthetic
> corpus in `fixtures/m4/` (`crates/tsr-learn/src/synth.rs`), not from any real task log.
>
> **The real exit evidence waits.** The M4 exit criteria — controller Brier below generator
> Brier on the held-out part of M3's mined corpus, and the support floor set — cannot be
> produced by this lane. They wait for lanes 04 (`tsr-forecast`, the ledger) and 05
> (`tsr corpus`/`tsr eval`, the mined corpus) to merge, and for the operator's runs. The
> commands the operator will run are in [What the operator runs](#what-the-operator-runs).

This document quotes every command run for lane 06 (the calibration half of M4) and its
captured output, over `fixtures/m4/`. It is the evidence for `docs/PRD.md` §B8, §B9, §B11 M4
and §B12.

All commands were run from the workspace root with:

```
export PATH=/usr/local/cargo/bin:$PATH CARGO_HOME=$PWD/.cargo-home
```

## The synthetic corpus

`crates/tsr-learn/src/synth.rs` is a seeded, clock-free generator that drives
`tsr_store::Store::append` through the honest M0 sequence and writes `fixtures/m4/`. Its true
acceptance model, each family's distortion, the seed and the counts are documented in the
module header; the summary is:

| family | base(f) | distortion(f) | p_true at 0 insp | p_gen at 0 insp |
|--------|---------|---------------|------------------|-----------------|
| `overconfident` | -0.40 | +2.20 | 0.401 | 0.900 |
| `calibrated` | 0.00 | 0.00 | 0.500 | 0.500 |
| `underconfident` | 0.85 | -0.85 | 0.701 | 0.300 |
| `rare` | -0.85 | -0.85 | 0.300 | 0.300 |

Three families of 120 tasks plus a fourth of 20 (`rare`) — 380 tasks; twelve lineages
(`lineage-00` … `lineage-11`); two to four gated decisions per task; some tasks censored
(never reviewed, or an operator stop) and some still open. Families are **interleaved in
time**, so a forward-time split sees all four families on every side. Everything is drawn
from one `SplitMix64` seeded with `SEED`; no clock is read.

### Regenerating the fixtures

```
$ TSR_REGEN_FIXTURES=1 cargo test -p tsr-learn --lib synth::tests::regen -- --nocapture
```

```
running 1 test
TSR_REGEN_FIXTURES=1: rewrote /work/crates/tsr-learn/../../fixtures/m4
test synth::tests::regen ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 63 filtered out; finished in 5.58s
```

The generator is deterministic: after this run `git status --short fixtures/m4/` is empty
(the fixtures are byte-identical to the committed ones).

### The corpus test

```
$ cargo test -p tsr-cli --test m4_corpus
```

```
test every_fixture_session_satisfies_the_m0_invariants ... ok
test the_corpus_has_the_shape_the_brief_names ... ok
test fixtures_regenerate_byte_identically ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.04s
```

## The pipeline over `fixtures/m4/`

### 1. Build the dataset

```
$ cargo run -q -p tsr-cli -- train dataset \
    --events fixtures/m4/events.jsonl \
    --tasks fixtures/m4/tasks.jsonl \
    --labels fixtures/m4/labels.jsonl \
    --by time --calibrate 0.2 --holdout 0.2 --seed 7 --out target/m4ds
```

```
dataset target/m4ds
rows=1138
data_root=42af2d8ce3e9a6f7b97159a16d85a22761f1318bf1f57df88483dc5fba397a9c
training_root=21fe3aea6e82c2a8449ab76f12b4b157088af71c855b66fe2898ec29a8450b92
split_digest=efef2a83a24671353358c757d30390eab66c03fba542f172c1e6e9163bde43f5
```

1138 rows over 380 tasks; the split is forward-time (`--by time`) with 20% calibrate and 20%
held-out. The dataset directory holds `rows.jsonl`, `split.json`, `data-manifest.json`,
`features.json`, `events.jsonl`, `tasks.jsonl` and `holdout-access.jsonl` (the B9 held-out
access log).

### 2. Fit the bundle

```
$ cargo run -q -p tsr-cli -- train fit target/m4ds --out target/m4-synth --support-floor 30
```

```
18b623ec948b2f08526c1492939ea205616bb0009cb76bddb0db30ebea5abd13
```

The printed line is the bundle id: `tsr_domain::Hash::of` over each content file's digest and
the manifest's content fields — not the code commit, a timestamp or the evaluation report.

### 3. Verify the bundle

```
$ cargo run -q -p tsr-cli -- train verify target/m4-synth
$ cargo run -q -p tsr-cli -- train verify target/m4-synth --rebuild target/m4ds
```

```
bundle target/m4-synth verified
bundle target/m4-synth verified
```

`--rebuild` retrains from the dataset and compares ids; both exit 0.

### 4. The bundle manifest

```
$ cat target/m4-synth/manifest.toml
```

```
data_manifest = "42af2d8ce3e9a6f7b97159a16d85a22761f1318bf1f57df88483dc5fba397a9c"
training_root = "21fe3aea6e82c2a8449ab76f12b4b157088af71c855b66fe2898ec29a8450b92"
feature_schema = "features@v1"
code_commit = "unknown"
seed = 7
calibrator = "18b623ec948b features@v1"
support_floor = 30
split_digest = "efef2a83a24671353358c757d30390eab66c03fba542f172c1e6e9163bde43f5"
source_root = "8e18314439ef064fd4645974e23f2e9903d40018b9c8f6bc2d907c412ad8209a"
state = "draft"
id = "18b623ec948b2f08526c1492939ea205616bb0009cb76bddb0db30ebea5abd13"

[content_digests]
"features.json" = "5ca086adf6a7bd7a34bc4bb624bbd8ff76af41b6a0d07f2661d6ffe0021a5711"
"head.json" = "9010470cab54c4f98722c400d4d9c52edc2d453c8e16057c0aa1da8b8ab3f835"
"isotonic.json" = "07ae27f59052b5ae4e5ac381b35a2e009a30d33ecacca34c019251232f20a1fb"
"split.json" = "edeadf271b191df9dba4e0460e95af6723c4ecc882b12662a299f30ac23977fc"
"support.json" = "c32dbe05a61bf350d673da0e35ff5541b2d374cde72adf62a05306242cba4c41"
```

`code_commit` is `"unknown"` because `TSR_CODE_COMMIT` was not set for this run. The bundle
directory holds `manifest.toml`, `head.json`, `isotonic.json`, `features.json`, `split.json`
and `support.json`.

### 5. The controller against the generator on held-out

```
$ cargo run -q -p tsr-cli -- train report target/m4-synth --dataset target/m4ds --seed 7
```

```
bundle 18b623ec948b2f08526c1492939ea205616bb0009cb76bddb0db30ebea5abd13
calibrator 18b623ec948b features@v1
seed 7
scored 179
excluded 43
uncalibrated 0
log_loss_clip 0.000000000001
support_floor 30

forecasters
  controller n=179 brier=0.168178 log_loss=1.191813
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=0.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=0.000000
    bin [0.6,0.7) coverage=0.000000
    bin [0.7,0.8) coverage=88.000000
    bin [0.8,0.9) coverage=61.000000
    bin [0.9,1.0) coverage=30.000000
  generator n=179 brier=0.207106 log_loss=0.720339
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=2.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=10.000000
    bin [0.6,0.7) coverage=0.000000
    bin [0.7,0.8) coverage=24.000000
    bin [0.8,0.9) coverage=51.000000
    bin [0.9,1.0) coverage=92.000000
  operator n=96 brier=0.237200 log_loss=0.706277
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=0.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=3.000000
    bin [0.6,0.7) coverage=9.000000
    bin [0.7,0.8) coverage=11.000000
    bin [0.8,0.9) coverage=25.000000
    bin [0.9,1.0) coverage=48.000000

families
family calibrated
  controller n=58 brier=0.156514 log_loss=0.461473
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=0.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=0.000000
    bin [0.6,0.7) coverage=0.000000
    bin [0.7,0.8) coverage=33.000000
    bin [0.8,0.9) coverage=18.000000
    bin [0.9,1.0) coverage=7.000000
  generator n=58 brier=0.151745 log_loss=0.456502
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=0.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=4.000000
    bin [0.6,0.7) coverage=0.000000
    bin [0.7,0.8) coverage=10.000000
    bin [0.8,0.9) coverage=14.000000
    bin [0.9,1.0) coverage=30.000000
  operator n=24 brier=0.267890 log_loss=0.747887
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=0.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=2.000000
    bin [0.6,0.7) coverage=0.000000
    bin [0.7,0.8) coverage=4.000000
    bin [0.8,0.9) coverage=6.000000
    bin [0.9,1.0) coverage=12.000000
family overconfident
  controller n=62 brier=0.206992 log_loss=0.585080
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=0.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=0.000000
    bin [0.6,0.7) coverage=0.000000
    bin [0.7,0.8) coverage=46.000000
    bin [0.8,0.9) coverage=13.000000
    bin [0.9,1.0) coverage=3.000000
  generator n=62 brier=0.310556 log_loss=1.192908
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=0.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=0.000000
    bin [0.6,0.7) coverage=0.000000
    bin [0.7,0.8) coverage=0.000000
    bin [0.8,0.9) coverage=0.000000
    bin [0.9,1.0) coverage=62.000000
  operator n=35 brier=0.263693 log_loss=0.775097
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=0.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=0.000000
    bin [0.6,0.7) coverage=5.000000
    bin [0.7,0.8) coverage=7.000000
    bin [0.8,0.9) coverage=10.000000
    bin [0.9,1.0) coverage=13.000000
family underconfident
  controller n=59 brier=0.138856 log_loss=2.547357
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=0.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=0.000000
    bin [0.6,0.7) coverage=0.000000
    bin [0.7,0.8) coverage=9.000000
    bin [0.8,0.9) coverage=30.000000
    bin [0.9,1.0) coverage=20.000000
  generator n=59 brier=0.152818 log_loss=0.483105
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=2.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=6.000000
    bin [0.6,0.7) coverage=0.000000
    bin [0.7,0.8) coverage=14.000000
    bin [0.8,0.9) coverage=37.000000
    bin [0.9,1.0) coverage=0.000000
  operator n=37 brier=0.192233 log_loss=0.614187
    bin [0.0,0.1) coverage=0.000000
    bin [0.1,0.2) coverage=0.000000
    bin [0.2,0.3) coverage=0.000000
    bin [0.3,0.4) coverage=0.000000
    bin [0.4,0.5) coverage=0.000000
    bin [0.5,0.6) coverage=1.000000
    bin [0.6,0.7) coverage=4.000000
    bin [0.7,0.8) coverage=0.000000
    bin [0.8,0.9) coverage=9.000000
    bin [0.9,1.0) coverage=23.000000

paired controller minus generator
  all tasks=61 clusters=12 mean=brier=-0.114231 [-0.241970, -0.018035] conclusive
  calibrated tasks=58 clusters=10 mean=brier=0.004769 [-0.007773, 0.019179] INCONCLUSIVE
  overconfident tasks=62 clusters=9 mean=brier=-0.103564 [-0.203953, -0.025500] conclusive
  underconfident tasks=59 clusters=10 mean=brier=-0.013961 [-0.051665, 0.016896] INCONCLUSIVE

population
family calibrated support=66.000000
family overconfident support=67.000000
family underconfident support=67.000000

floor sweep (calibrate slice)
family calibrated
  floor=0 coverage=51.000000 brier=0.096063
  floor=5 coverage=51.000000 brier=0.096063
  floor=10 coverage=51.000000 brier=0.096063
  floor=20 coverage=49.000000 brier=0.096578
  floor=30 coverage=48.000000 brier=0.098152
  floor=50 coverage=11.000000 brier=0.106622
family overconfident
  floor=0 coverage=62.000000 brier=0.207736
  floor=5 coverage=62.000000 brier=0.207736
  floor=10 coverage=62.000000 brier=0.207736
  floor=20 coverage=62.000000 brier=0.207736
  floor=30 coverage=46.000000 brier=0.226142
  floor=50 coverage=11.000000 brier=0.200360
family underconfident
  floor=0 coverage=58.000000 brier=0.150752
  floor=5 coverage=58.000000 brier=0.150752
  floor=10 coverage=58.000000 brier=0.150752
  floor=20 coverage=55.000000 brier=0.148005
  floor=30 coverage=36.000000 brier=0.149970
  floor=50 coverage=6.000000 brier=0.257719
```

**Reading the synthetic result.** On the held-out slice the controller's Brier (0.168178) is
below the generator's (0.207106); the paired per-task difference, controller minus generator,
is `-0.114231` with a 90% cluster-bootstrap interval `[-0.241970, -0.018035]` over 12
lineages — **conclusive** (the interval does not span zero). The gain is concentrated in the
`overconfident` family (`-0.103564`, conclusive), which is exactly the family the generator
was built to be wrong about; the `calibrated` and `underconfident` families are
INCONCLUSIVE, as they should be. `rare` (20 tasks) is below the 30-task family floor and is
outside the calibrated population, so it contributes no calibrated row.

The `operator` forecaster is scored beside them where a forecast was recorded (n=96). The
`log_loss_clip` is `1e-12`, printed so the reader knows the clip. `excluded 43` counts the
held-out rows with no resolved label (censored, unresolved, or no index row); `uncalibrated
0` counts rows whose family is outside the population.

The floor sweep prints, per family on the calibrate slice, coverage and controller Brier at
floors 0, 5, 10, 20, 30 and 50, so the operator can set the floor from evidence. Support is
compared with the floor and reported; it never weights, scales or orders anything.

## The test suites

```
$ cargo test -p tsr-learn
```

```
     Running unittests src/lib.rs (target/debug/deps/tsr_learn-99bdf86ab06c5827)
test result: ok. 64 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.62s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```
$ cargo test -p tsr-cli --test m4_corpus
```

```
test every_fixture_session_satisfies_the_m0_invariants ... ok
test the_corpus_has_the_shape_the_brief_names ... ok
test fixtures_regenerate_byte_identically ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.04s
```

```
$ cargo test -p tsr-cli --test m4_train
```

```
test fit_refuses_fewer_than_150_mature_tasks ... ok
test a_non_operator_label_is_refused_with_exit_2 ... ok
test the_controller_beats_the_generator_on_held_out ... ok
test a_family_below_30_tasks_is_uncalibrated ... ok
test a_one_byte_bundle_change_is_refused_naming_the_file ... ok
test a_different_seed_gives_a_different_split_digest ... ok
test a_paired_interval_that_spans_zero_is_inconclusive ... ok
test held_out_reads_are_logged ... ok
test the_report_uses_engineering_names_and_a_calibrator_version ... ok
test the_train_pipeline_runs_end_to_end ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.51s
```

## What the operator runs

These are the commands that produce the **real** M4 exit evidence. They cannot run until
lanes 04 (`tsr-forecast`, the ledger) and 05 (`tsr corpus`/`tsr eval`, the mined corpus) have
merged, and the operator has a real store and a real labels file. The merge-time adapter feeds
lane 04's outcome records and lane 05's oracle verdicts into the labels input; this lane
writes neither.

```sh
export PATH=/usr/local/cargo/bin:$PATH CARGO_HOME=$PWD/.cargo-home

# 1. Build the dataset from the real store (or from an exported events file).
#    --tasks is the task index (lane 05's `tsr corpus` output shape);
#    --labels is the review labels (criteria_met, diff_label, reviewer, recorded_at).
cargo run -p tsr-cli -- train dataset \
    --store .tsr/store.db \
    --tasks corpus/tasks.jsonl \
    --labels corpus/labels.jsonl \
    --by time --calibrate 0.2 --holdout 0.2 --seed 7 \
    --out target/m4-real

# 2. Fit the bundle. --support-floor is the operator's floor (B12, open); default 30.
cargo run -p tsr-cli -- train fit target/m4-real --out models/m4-real --support-floor 30

# 3. Verify the bundle (recomputes every digest; --rebuild retrains and compares ids).
cargo run -p tsr-cli -- train verify models/m4-real
cargo run -p tsr-cli -- train verify models/m4-real --rebuild target/m4-real

# 4. Report the controller against the generator on held-out. The exit evidence is
#    controller Brier below generator Brier, with a conclusive paired interval.
cargo run -p tsr-cli -- train report models/m4-real --dataset target/m4-real --seed 7

# 5. Set the support floor from the floor sweep the report prints, then refit.
cargo run -p tsr-cli -- train fit target/m4-real --out models/m4-real --support-floor <n>
```

The operator also runs the same report with `--by repo` (a repository holdout) to check the
second §B9 holdout, and reads `<dir>/holdout-access.jsonl` to confirm every held-out read was
logged.

## What this lane does not claim

- Every number above is synthetic. None of it is evidence about the real generator.
- The real exit evidence — controller Brier below generator Brier on the held-out part of
  M3's mined corpus, and the support floor set — is **not** in this document. It waits for
  lanes 04 and 05 to merge and for the operator's runs above.
- The compose step, Intuition candidates admitted and the two-desires test are not this
  lane's; they need lane 04's ledger and follow in lane 07.
