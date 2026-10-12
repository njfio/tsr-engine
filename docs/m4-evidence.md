# M4 (Calibrate) — exit evidence

This document is the M4 lane's exit evidence, re-captured verbatim from the final tip of
`lane/calibrate` after the repair. Every command below was run on the tip named in the next
section; every block is the command's own output, pasted unchanged. Where a number is
synthetic it is labelled synthetic.

## The tip

```
$ git log --oneline -1
14c43fa fix(m4): standardization stats and learning rate live in features.json; support counts only labelled fit/calibrate rows and never the row itself (B15)
```

The repair landed as a sequence of conventional commits on `lane/calibrate`:

```
14c43fa fix(m4): standardization stats and learning rate live in features.json; support counts only labelled fit/calibrate rows and never the row itself (B15)
e4398d1 fix(m4): clippy 1.90 saturating_sub in PAV merge loop (R1, F1)
8d1a70a fix(m4): clippy 1.90, seed-sensitivity tests, probe row/feature naming (F11, F15, B16, A7)
7fe5967 fix(m4): empty bins print n=0 and JSON null; operator scored on comparison rows; interval/order/clip printing; README documents apply (F10, B4, F12, F13, B14)
93a2d59 fix(m4): fit/dataset refuse what they must and accept what they made (F3, R2, F8, F9, B3, A8, R6)
8f339d7 fix(m4): verify refuses a non-canonical manifest and a code_commit edit; m4_corpus passes --tasks (F6, B8)
dd745fa fix(m4): load_bundle checks calibrator/state/canonical manifest; row id covers every field; report and apply exit 2 (F6, B8, F7, F14)
7bb5cc5 fix(m4): apply refuses a foreign dataset; print raw p/p/support; population names every family (R4, R7, B14)
ff08c2a fix(tsr-learn): correct PAV isotonic to be monotone and order-invariant
8afdf30 fix(m4): keep clone groups inside one lineage; pair forecasts by adjacency (F2, A2, F5)
```

## Build, format, lint

```
$ cargo build --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.95s
```

```
$ cargo fmt --all -- --check
(no output; exit 0)
```

```
$ cargo +1.90.0 clippy --workspace --all-targets -- -D warnings
    Checking tsr-learn v0.0.1 (/work/crates/tsr-learn)
    Checking tsr-cli v0.0.1 (/work/crates/tsr-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.93s
```

## The whole workspace test suite

```
$ cargo test --workspace --no-fail-fast
```

```
     Running unittests src/lib.rs (target/debug/deps/tsr_cli-bce67e544bd1cb3a)
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s
     Running unittests src/main.rs (target/debug/deps/tsr-8569f70ae62ee469)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/m0_forecast_default.rs (target/debug/deps/m0_forecast_default-9c67652a7ee035cd)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.16s
     Running tests/m0_harness.rs (target/debug/deps/m0_harness-b30139e0b1c55e38)
test result: oktest forecast_then_tool_call_yields_proposed_forecast_committed_and_no_second_forecast ... ok
test result: oktest forecast_cli_and_hook_agree_on_the_candidate_id ... ok
test result: oktest selftest_without_a_runtime_exits_3_and_writes_nothing ... ok
test result: oktest result: oktest result: oktest result: oktest selftest_sequence_through_the_fake_runner_has_19_events_and_holds_every_invariant ... ok
test result: oktest result: oktest no_proposal_after_a_tsr_exec_post_tool_and_the_second_run_reuses_its_commitment ... ok
test result: oktest selftest_is_reproducible_structurally_excluding_duration_ms_by_name ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.28s
     Running tests/m0_real_session.rs (target/debug/deps/m0_real_session-12748bf19c69eb48)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/m0_reconstruct.rs (target/debug/deps/m0_reconstruct-91bbaacb61a51a2e)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/m4_corpus.rs (target/debug/deps/m4_corpus-5e878acd500cc84f)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 63.59s
     Running tests/m4_probes.rs (target/debug/deps/m4_probes-33afa6b2ac97383b)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s
     Running tests/m4_train.rs (target/debug/deps/m4_train-1e95793bc20dfb27)
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 54.59s
     Running tests/policy_check.rs (target/debug/deps/policy_check-4c8b320ea3af5431)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running unittests src/lib.rs (target/debug/deps/tsr_domain-8e6445c3fa41ebe6)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/tsr_learn-e1505fd5c8114c34)
test result: ok. 112 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.10s
     Running unittests src/lib.rs (target/debug/deps/tsr_policy-ba3247ce7e46275e)
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/adversarial.rs (target/debug/deps/adversarial-5198535eed886606)
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running unittests src/lib.rs (target/debug/deps/tsr_store-74283e01879948c3)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

No suite failed. The two Docker-only `m0` tests ran here because Docker is present on this host;
on a host without Docker they are the only tests that do not run.

## The pipeline over `fixtures/m4/`

The synthetic corpus is `fixtures/m4/` (events, tasks, labels). It is regenerated byte-identically
by `TSR_REGEN_FIXTURES=1 cargo test -p tsr-cli --test m4_corpus fixtures_regenerate_byte_identically`.

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
rows=1129
data_root=0a0c5f97dd214be03bb7e5d1c00953e9f0188e054b29467252eccebd5032b047
training_root=bc2bddf3c12a38eca94e22afe7922ee95c85469479aceadb2fd6db1eea222118
split_digest=4692109b92a048a0ca02b952412e5d1726d5efa91a8127c37335bd9ca32ec8c1
unindexed_tasks=0
```

1129 rows over 380 tasks; the split is forward-time (`--by time`) with 20% calibrate and 20%
held-out. `unindexed_tasks=0` counts tasks the index names that have no row (A8). The dataset
directory holds `rows.jsonl`, `split.json`, `data-manifest.json`, `features.json`, `events.jsonl`,
`tasks.jsonl` and `holdout-access.jsonl` (the B9 held-out access log).

### 2. Fit the bundle

```
$ cargo run -q -p tsr-cli -- train fit target/m4ds --out target/m4-synth --support-floor 30
```

```
22b26e5e9bb4be87b491971df474777e39e91bfc44bbc608cebc0999a5158a23
```

The printed line is the bundle id: `tsr_domain::Hash::of` over each content file's digest and the
manifest's content fields — not the code commit, a timestamp or the evaluation report.

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
data_manifest = "0a0c5f97dd214be03bb7e5d1c00953e9f0188e054b29467252eccebd5032b047"
training_root = "bc2bddf3c12a38eca94e22afe7922ee95c85469479aceadb2fd6db1eea222118"
feature_schema = "features@v1"
code_commit = "unknown"
seed = 7
calibrator = "22b26e5e9bb4 features@v1"
support_floor = 30
split_digest = "4692109b92a048a0ca02b952412e5d1726d5efa91a8127c37335bd9ca32ec8c1"
source_root = "3f8827349aef6a03ee282c96308089689aeee9136b3e2ba7c25f551d9747a9a1"
state = "draft"
id = "22b26e5e9bb4be87b491971df474777e39e91bfc44bbc608cebc0999a5158a23"

[content_digests]
"features.json" = "c0a559b5809356e9c226206bc896e656cb3429842509e89d8ebb92a319503827"
"head.json" = "1dc0fc57c887654412b01e25d72123eb8bfda6ec54804be1bf7459e72d7ed32f"
"isotonic.json" = "04cab8baeeb7bec7f7ae13934b70a53f97152584474c9f13f0bd0c7e14506ccb"
"split.json" = "137d1ba81d0dc2764758b62c19505d5e07a78c3b25339f16be6976ad0936bcc9"
"support.json" = "bdfa2ad65f173f1e41ebbfc223c1361c3b6ec672efe631bdaf120eb1eda42983"
```

`code_commit` is `"unknown"` because `TSR_CODE_COMMIT` was not set for this run. The bundle
directory holds `manifest.toml`, `head.json`, `isotonic.json`, `features.json`, `split.json` and
`support.json`.

### 5. The report

```
$ cargo run -q -p tsr-cli -- train report target/m4-synth --dataset target/m4ds --seed 7
```

```
bundle 22b26e5e9bb4be87b491971df474777e39e91bfc44bbc608cebc0999a5158a23
split_digest 4692109b92a048a0ca02b952412e5d1726d5efa91a8127c37335bd9ca32ec8c1
calibrator 22b26e5e9bb4 features@v1
seed 7
scored 159
excluded 56
uncalibrated 7
unindexed_tasks 0
log_loss_clip 0.000000000001
support_floor 30

forecasters
  controller n=152 brier=0.157443 log_loss=0.492302
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=0
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=0
    bin [0.6,0.7) n=0
    bin [0.7,0.8) n=116 mean_forecast=0.747526 observed_frequency=0.767241
    bin [0.8,0.9) n=22 mean_forecast=0.894737 observed_frequency=0.909091
    bin [0.9,1.0) n=14 mean_forecast=0.916667 observed_frequency=0.785714
  generator n=152 brier=0.175364 log_loss=0.577418
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=8 mean_forecast=0.299433 observed_frequency=1.000000
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=28 mean_forecast=0.506249 observed_frequency=0.500000
    bin [0.6,0.7) n=0
    bin [0.7,0.8) n=32 mean_forecast=0.715397 observed_frequency=0.781250
    bin [0.8,0.9) n=25 mean_forecast=0.862453 observed_frequency=0.880000
    bin [0.9,1.0) n=59 mean_forecast=0.957375 observed_frequency=0.864407
  operator n=84 brier=0.183567 log_loss=0.573390
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=0
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=14 mean_forecast=0.500000 observed_frequency=0.571429
    bin [0.6,0.7) n=10 mean_forecast=0.672792 observed_frequency=1.000000
    bin [0.7,0.8) n=24 mean_forecast=0.750313 observed_frequency=0.625000
    bin [0.8,0.9) n=23 mean_forecast=0.855476 observed_frequency=1.000000
    bin [0.9,1.0) n=13 mean_forecast=0.926035 observed_frequency=0.615385

families
family calibrated
  controller n=52 brier=0.239252 log_loss=0.657159
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=0
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=0
    bin [0.6,0.7) n=0
    bin [0.7,0.8) n=43 mean_forecast=0.731648 observed_frequency=0.511628
    bin [0.8,0.9) n=4 mean_forecast=0.894737 observed_frequency=1.000000
    bin [0.9,1.0) n=5 mean_forecast=0.916667 observed_frequency=1.000000
  generator n=52 brier=0.156723 log_loss=0.462716
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=0
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=14 mean_forecast=0.500000 observed_frequency=0.000000
    bin [0.6,0.7) n=0
    bin [0.7,0.8) n=18 mean_forecast=0.710950 observed_frequency=0.611111
    bin [0.8,0.9) n=7 mean_forecast=0.858149 observed_frequency=1.000000
    bin [0.9,1.0) n=13 mean_forecast=0.937027 observed_frequency=1.000000
  operator n=22 brier=0.211831 log_loss=0.599490
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=0
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=6 mean_forecast=0.500000 observed_frequency=0.000000
    bin [0.6,0.7) n=0
    bin [0.7,0.8) n=12 mean_forecast=0.710950 observed_frequency=0.583333
    bin [0.8,0.9) n=2 mean_forecast=0.858149 observed_frequency=1.000000
    bin [0.9,1.0) n=2 mean_forecast=0.937027 observed_frequency=1.000000
family overconfident
  controller n=46 brier=0.161297 log_loss=0.522310
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=0
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=0
    bin [0.6,0.7) n=0
    bin [0.7,0.8) n=44 mean_forecast=0.739921 observed_frequency=0.863636
    bin [0.8,0.9) n=1 mean_forecast=0.894737 observed_frequency=0.000000
    bin [0.9,1.0) n=1 mean_forecast=0.916667 observed_frequency=0.000000
  generator n=46 brier=0.166223 log_loss=0.694455
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=0
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=0
    bin [0.6,0.7) n=0
    bin [0.7,0.8) n=0
    bin [0.8,0.9) n=0
    bin [0.9,1.0) n=46 mean_forecast=0.963125 observed_frequency=0.826087
  operator n=26 brier=0.197181 log_loss=0.633201
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=0
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=0
    bin [0.6,0.7) n=3 mean_forecast=0.650781 observed_frequency=1.000000
    bin [0.7,0.8) n=12 mean_forecast=0.789676 observed_frequency=0.666667
    bin [0.8,0.9) n=9 mean_forecast=0.892099 observed_frequency=1.000000
    bin [0.9,1.0) n=2 mean_forecast=0.950743 observed_frequency=0.000000
family underconfident
  controller n=54 brier=0.075381 log_loss=0.307988
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=0
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=0
    bin [0.6,0.7) n=0
    bin [0.7,0.8) n=29 mean_forecast=0.782609 observed_frequency=1.000000
    bin [0.8,0.9) n=17 mean_forecast=0.894737 observed_frequency=0.941176
    bin [0.9,1.0) n=8 mean_forecast=0.916667 observed_frequency=0.750000
  generator n=54 brier=0.201102 log_loss=0.588174
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=8 mean_forecast=0.299433 observed_frequency=1.000000
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=14 mean_forecast=0.512497 observed_frequency=1.000000
    bin [0.6,0.7) n=0
    bin [0.7,0.8) n=14 mean_forecast=0.721115 observed_frequency=1.000000
    bin [0.8,0.9) n=18 mean_forecast=0.864127 observed_frequency=0.833333
    bin [0.9,1.0) n=0
  operator n=36 brier=0.156463 log_loss=0.514242
    bin [0.0,0.1) n=0
    bin [0.1,0.2) n=0
    bin [0.2,0.3) n=0
    bin [0.3,0.4) n=0
    bin [0.4,0.5) n=0
    bin [0.5,0.6) n=8 mean_forecast=0.500000 observed_frequency=1.000000
    bin [0.6,0.7) n=7 mean_forecast=0.682225 observed_frequency=1.000000
    bin [0.7,0.8) n=0
    bin [0.8,0.9) n=12 mean_forecast=0.827563 observed_frequency=1.000000
    bin [0.9,1.0) n=9 mean_forecast=0.918102 observed_frequency=0.666667

paired controller minus generator
  all tasks=52 clusters=12 mean=brier=-0.011781 [interval_lo=-0.034651, interval_hi=0.016411] INCONCLUSIVE
  calibrated tasks=18 clusters=9 mean=brier=0.070844 [interval_lo=0.034464, interval_hi=0.125689] conclusive
  overconfident tasks=17 clusters=8 mean=brier=-0.002133 [interval_lo=-0.076820, interval_hi=0.052387] INCONCLUSIVE
  underconfident tasks=17 clusters=11 mean=brier=-0.108916 [interval_lo=-0.167237, interval_hi=-0.054255] conclusive

population
family calibrated tasks=63
family overconfident tasks=67
family underconfident tasks=66

floor sweep (calibrate slice)
family calibrated
  floor=0 coverage=59.000000 brier=0.081915
  floor=5 coverage=59.000000 brier=0.081915
  floor=10 coverage=57.000000 brier=0.082038
  floor=20 coverage=31.000000 brier=0.092053
  floor=30 coverage=15.000000 brier=0.122534
  floor=50 coverage=0.000000 brier=n/a
family overconfident
  floor=0 coverage=32.000000 brier=0.200386
  floor=5 coverage=32.000000 brier=0.200386
  floor=10 coverage=31.000000 brier=0.206493
  floor=20 coverage=21.000000 brier=0.218895
  floor=30 coverage=10.000000 brier=0.254400
  floor=50 coverage=0.000000 brier=n/a
family rare
  floor=0 coverage=0.000000 brier=n/a
  floor=5 coverage=0.000000 brier=n/a
  floor=10 coverage=0.000000 brier=n/a
  floor=20 coverage=0.000000 brier=n/a
  floor=30 coverage=0.000000 brier=n/a
  floor=50 coverage=0.000000 brier=n/a
family underconfident
  floor=0 coverage=44.000000 brier=0.195208
  floor=5 coverage=44.000000 brier=0.195208
  floor=10 coverage=42.000000 brier=0.189657
  floor=20 coverage=24.000000 brier=0.185503
  floor=30 coverage=13.000000 brier=0.276017
  floor=50 coverage=0.000000 brier=n/a
```

Empty reliability bins print `n=0` and no mean or observed value (F10/B4); the JSON report carries
`null` for them. The operator is scored only on the comparison rows where it recorded a forecast
(F12). The interval bounds are printed through `Proxy` (F13/B14). The log-loss clip is carried in
`--json` (B14). Forecasters are listed in name order: controller, generator, operator (B14).

### The headline comparison, as the code prints it

On the same 152 comparison rows, on this synthetic corpus:

| forecaster | n | Brier | log loss |
|---|---|---|---|
| controller | 152 | 0.157443 | 0.492302 |
| generator | 152 | 0.175364 | 0.577418 |
| operator | 84 | 0.183567 | 0.573390 |

The per-task paired difference (controller minus generator), clustered by lineage:

```
paired controller minus generator
  all tasks=52 clusters=12 mean=brier=-0.011781 [interval_lo=-0.034651, interval_hi=0.016411] INCONCLUSIVE
  calibrated tasks=18 clusters=9 mean=brier=0.070844 [interval_lo=0.034464, interval_hi=0.125689] conclusive
  overconfident tasks=17 clusters=8 mean=brier=-0.002133 [interval_lo=-0.076820, interval_hi=0.052387] INCONCLUSIVE
  underconfident tasks=17 clusters=11 mean=brier=-0.108916 [interval_lo=-0.167237, interval_hi=-0.054255] conclusive
```

**Plain statement.** On this synthetic corpus the controller beats the generator on both point
estimates — Brier 0.157443 against 0.175364, log loss 0.492302 against 0.577418 — but the per-task
paired difference is **INCONCLUSIVE overall**: the interval `[-0.034651, 0.016411]` spans zero. The
controller is conclusively *worse* on the calibrated family (mean +0.070844, interval entirely
above zero) and conclusively *better* on the underconfident family (mean -0.108916, interval
entirely below zero); the overconfident family is inconclusive. A negative mean favours the
controller. The point-estimate win and the inconclusive paired test are both reported as they are.

### `--by repo`

```
$ cargo run -q -p tsr-cli -- train dataset \
    --events fixtures/m4/events.jsonl --tasks fixtures/m4/tasks.jsonl --labels fixtures/m4/labels.jsonl \
    --by repo --calibrate 0.2 --holdout 0.2 --seed 7 --out target/m4-repo
$ cargo run -q -p tsr-cli -- train fit target/m4-repo --out target/m4-repo-models
```

```
training_root=a3dda9838b0d5b3699fcca39db8e209575984b91ea5af4ff5830cf666bd34432
split_digest=da61a8f7546fd0df32f56a780017593b11aa5c40610f62096ce66b17c19efdfb
unindexed_tasks=0
9f9746d51c37ff11e83c566994bd5bb644a5d3f7e07eed8ae42c0c5b6d613b2e
```

Under `--by repo` the split unit is the whole lineage (merged with every lineage a clone group
touches), so no lineage's tasks land on two sides. The side counts for seeds 1–8 are three
non-empty sides each, and `fit` accepts every one:

```
seed 1: {'fit': 269, 'calibrate': 69, 'held_out': 42}
seed 2: {'fit': 269, 'calibrate': 72, 'held_out': 39}
seed 3: {'fit': 258, 'held_out': 86, 'calibrate': 36}
seed 4: {'held_out': 69, 'fit': 238, 'calibrate': 73}
seed 5: {'fit': 262, 'calibrate': 80, 'held_out': 38}
seed 6: {'calibrate': 109, 'fit': 229, 'held_out': 42}
seed 7: {'calibrate': 109, 'held_out': 104, 'fit': 167}
seed 8: {'fit': 265, 'calibrate': 39, 'held_out': 76}
```

The `--by repo` report (seed 7) is:

```
paired controller minus generator
  all tasks=75 clusters=2 mean=brier=-0.032463 [interval_lo=-0.063735, interval_hi=-0.005100] conclusive
  calibrated tasks=26 clusters=2 mean=brier=-0.011700 [interval_lo=-0.015081, interval_hi=-0.008318] conclusive
  overconfident tasks=20 clusters=2 mean=brier=-0.042911 [interval_lo=-0.045299, interval_hi=-0.041318] conclusive
  underconfident tasks=29 clusters=2 mean=brier=-0.043873 [interval_lo=-0.119448, interval_hi=0.026663] INCONCLUSIVE

population
family calibrated tasks=55
family overconfident tasks=64
family underconfident tasks=54
```

### `apply`

```
$ cargo run -q -p tsr-cli -- train apply target/m4-synth --dataset target/m4ds --side held_out
```

```
row 03e256d85874bac623f4b0b17b2e73b8bf4a825b1ad2fe5067eaa1ffe8d4551f family calibrated raw p=0.188466 p=0.720000 (22b26e5e9bb4 features@v1) support=20
row 0440bef0688e3b1bd70ec9d9bb2b62e8ab763275ca4aab58964ac4738984bb33 family calibrated raw p=0.210504 p=0.720000 (22b26e5e9bb4 features@v1) support=14
row 044aa817ab4f8975044ecf6827bcb80557156961003d628a2967b8264166f80a family overconfident raw p=0.405317 p=0.720000 (22b26e5e9bb4 features@v1) support=22
row 04b741a266171492dd737139de7d7796c89f7b1b605d863c157851ed5867c710 family calibrated raw p=0.457489 p=0.720000 (22b26e5e9bb4 features@v1) support=36
row 0750dc8a96e0fd24a63092e4c462cc8f9f6540622a28dc382974069efa9c423d family calibrated raw p=0.630939 p=0.720000 (22b26e5e9bb4 features@v1) support=35
row 07775d11c5b531a8beeb2615ec5e2cd8977498dc25a31ba9831e79a8400053b5 family overconfident raw p=0.760440 p=0.720000 (22b26e5e9bb4 features@v1) support=18
row 08279b39558c06b66be87301d4f75111e1cfa8a5c1a4104a8a3d5b6043ceefea family calibrated raw p=0.826817 p=0.782609 (22b26e5e9bb4 features@v1) support=23
row 0c569247d24a8ce1ef6f91f00f40f60cbf1802aec51f0af9afc66f53f8f6e367 family underconfident raw p=0.907933 p=0.782609 (22b26e5e9bb4 features@v1) support=17
... (207 more rows) ...
rows=215
```

Each line prints `raw p=<value>` and `p=<value> (<calibrator version>)` with `support=<n>` (R7), and
the population lists every family in it (B14). `apply` refuses a foreign dataset exactly as
`report` does (R4).

### `verify` on a tampered manifest

A copy of the bundle with `code_commit` edited to `"deadbeef"`:

```
$ cargo run -q -p tsr-cli -- train verify target/m4-tampered
tsr: verify failed: bundle: invalid input: manifest.toml: code_commit "deadbeef" is not the current "unknown"
$ echo $?
2
```

`load_bundle` — used by `verify`, `report` and `apply` alike — refuses a `manifest.toml` that
differs from the canonical rendering of its fields, a calibrator other than the derived one, a
state other than `draft`, and names a missing file (F6/B8). A one-byte change to any content file
is refused naming the file.

### `report` on a foreign dataset

```
$ cargo run -q -p tsr-cli -- train report target/m4-synth --dataset target/m4-repo
tsr train report: invalid input: dataset split digest da61a8f7546fd0df32f56a780017593b11aa5c40610f62096ce66b17c19efdfb does not match the bundle's 4692109b92a048a0ca02b952412e5d1726d5efa91a8127c37335bd9ca32ec8c1
$ echo $?
2
```

`apply` refuses the same foreign dataset with the same message and exit 2 (R4).

### Row ids and provenance

The row id covers every field a row carries, not just the id: `fit`, `report` and `apply`
recompute it and refuse a mismatch naming the row (F7). A bad row is named by an id that appears
in `rows.jsonl` (F14). `report` and `apply` exit 2 — not 1 — on tampered inputs.

### The tests that carry these ids

`cargo test -p tsr-cli --test m4_corpus` (8 passed):

```
test two_real_forecasts_pair_both_with_the_commitment ... ok
test two_real_sessions_pair_each_commitment_with_its_own_generator_forecast ... ok
test every_task_satisfies_the_m0_invariants_as_its_own_session ... ok
test every_fixture_session_satisfies_the_m0_invariants ... ok
test the_corpus_has_the_shape_the_brief_names ... ok
test fixtures_regenerate_byte_identically ... ok
test b7_train_apply_prints_raw_and_calibrated_p_and_support ... ok
test by_repo_seeds_1_to_8_give_three_non_empty_sides ... ok
```

`cargo test -p tsr-cli --test m4_train` (24 passed) includes `f1_*` (isotonic monotone and
order-invariant), `f2_*`/`a2_*` (clone groups inside one lineage), `f3_*`/`f8_*`/`f9_*`/`r6_*`/`a8_*`
(refusals and counts), `f6_*`/`f7_*`/`f14_*` (provenance), `r4_*`/`r7_*` (apply), `f10_*`/`f12_*`/
`f13_*`/`b14_*` (printing), `f11_*`/`f15_*`/`b16_*`/`a7_*` (lints, seed sensitivity, probe naming).
`cargo test -p tsr-cli --test m4_probes` (4 passed) covers probes (a), (b), (c) and (d).

### What the operator runs

The operator's own log lives in `.tsr/tsr.db` — the store the hooks write, and the default of
`--store`. To fit a bundle from it into a new directory:

```
$ tsr train dataset \
    --store .tsr/tsr.db \
    --tasks .tsr/tasks.jsonl \
    --labels .tsr/labels.jsonl \
    --by time --calibrate 0.2 --holdout 0.2 --seed 7 --out .tsr/m4-data
$ tsr train fit .tsr/m4-data --out .tsr/m4-models
$ tsr train report .tsr/m4-models --dataset .tsr/m4-data
$ tsr train verify .tsr/m4-models --rebuild .tsr/m4-data
$ tsr train apply .tsr/m4-models --dataset .tsr/m4-data --side held_out
```

`--store` defaults to `TSR_DB` / `.tsr/tsr.db`; `--events <file>` is the alternative for a log
already exported to JSONL.

`--tasks` is required (F9/B3). An empty `tasks.jsonl` turns probe (a) off; an empty or truncated
`events.jsonl` panics with exit 2 naming the file. `fit` accepts an empty calibrate or fit slice and
labels raw output `Calibrated` (exit 2 naming the empty slice instead); it refuses a dataset whose
index names a task with no gated commitment, counting such tasks and continuing (F3/R2, F8).
`dataset` refuses its own split when a clone group spans time — under `--by time` a clone group
goes to the side of its earliest member, and probe (c) accepts exactly that (F9/B3).

### The test that asserts a verdict

`crates/tsr-cli/tests/m4_train.rs::the_controller_beats_the_generator_on_held_out` asserts the
verdict the corrected code gives, and it is the test the brief names. It parses the `paired
controller minus generator` block and asserts:

- overall (`all tasks=`): `mean < 0` (the controller is better on average) **and** the interval
  spans zero **and** the verdict is `INCONCLUSIVE`;
- `overconfident tasks=`: the same shape — `mean < 0`, interval spans zero, `INCONCLUSIVE`;
- `calibrated tasks=`: `mean > 0` and `lo > 0` — the interval is wholly above zero, so the
  controller is conclusively worse there.

The test was changed during the repair because the pre-repair code produced a conclusive win from a
non-monotone calibrator and a collapsed `--by repo` split; the corrected code gives the
inconclusive overall result above, and the test now asserts that. The change is recorded here so
the assertion is not mistaken for the pre-repair claim.
