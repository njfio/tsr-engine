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
(never reviewed, or an operator stop) and some still open. Everything is drawn from one
`SplitMix64` seeded with `SEED`; no clock is read.

### Regenerating the fixtures

```
$ TSR_REGEN_FIXTURES=1 cargo test -p tsr-learn --lib synth::tests::regen -- --nocapture
```

<!-- CAPTURE:regen -->

### The corpus test

```
$ cargo test -p tsr-cli --test m4_corpus
```

<!-- CAPTURE:m4_corpus -->

## The pipeline over `fixtures/m4/`

### 1. Build the dataset

```
$ cargo run -q -p tsr-cli -- train dataset \
    --events fixtures/m4/events.jsonl \
    --tasks fixtures/m4/tasks.jsonl \
    --labels fixtures/m4/labels.jsonl \
    --by time --calibrate 0.2 --holdout 0.2 --seed 7 --out target/m4ds
```

<!-- CAPTURE:dataset -->

### 2. Fit the bundle

```
$ cargo run -q -p tsr-cli -- train fit target/m4ds --out models/m4-synth --support-floor 30
```

<!-- CAPTURE:fit -->

### 3. Verify the bundle

```
$ cargo run -q -p tsr-cli -- train verify models/m4-synth
$ cargo run -q -p tsr-cli -- train verify models/m4-synth --rebuild target/m4ds
```

<!-- CAPTURE:verify -->

### 4. Report the controller against the generator

```
$ cargo run -q -p tsr-cli -- train report models/m4-synth --dataset target/m4ds --seed 7
```

<!-- CAPTURE:report -->

### 5. The bundle manifest

```
$ cat models/m4-synth/manifest.toml
```

<!-- CAPTURE:manifest -->

## The tests

```
$ cargo test -p tsr-learn
$ cargo test -p tsr-cli --test m4_corpus
$ cargo test -p tsr-cli --test m4_train
```

<!-- CAPTURE:tests -->

## What the operator runs

These are the commands the operator runs **once lanes 04 and 05 have merged**, against the
real store and the real mined corpus. They are the M4 exit evidence; nothing in this document
substitutes for them.

1. Build the dataset from the real store (lane 04's ledger and lane 05's oracle verdicts feed
   the labels input through the merge-time adapter):

   ```
   tsr train dataset --store .tsr/tsr.db \
       --tasks <mined-corpus>/tasks.jsonl \
       --labels <mined-corpus>/labels.jsonl \
       --by time --calibrate 0.2 --holdout 0.2 --seed 7 --out models/m4-dataset
   ```

2. Fit the bundle, recording the support floor:

   ```
   tsr train fit models/m4-dataset --out models/m4 --support-floor <n>
   ```

3. Verify the bundle:

   ```
   tsr train verify models/m4
   tsr train verify models/m4 --rebuild models/m4-dataset
   ```

4. Report the controller against the generator on the held-out part of M3's mined corpus —
   the M4 exit criterion is controller Brier below generator Brier here:

   ```
   tsr train report models/m4 --dataset models/m4-dataset --seed 7
   ```

5. Set the support floor from the floor sweep the report prints (coverage and controller Brier
   per family at floors 0, 5, 10, 20, 30 and 50), then re-fit with the chosen `--support-floor`.

At merge the operator also wires `tsr eval calibration` to the same function as
`tsr train report` (lane 05 owns the Eval arm).