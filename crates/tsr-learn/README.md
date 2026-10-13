# tsr-learn

The calibration half of milestone M4 (Calibrate) of the TSR Decision Engine
(`docs/PRD.md` §B8, §B9, §B11 M4, §B12).

`tsr-learn` depends on `tsr-domain` and `tsr-store` only — never on `tsr-cli` — and is
**read-only** on the log. The one exception is the synthetic corpus generator
(`src/synth.rs`), which appends to its own fresh store; nothing else writes to a store.

It holds:

- `decision.rs` — the common decision event (`task_accepted_in_budget@v1`) as a pure view over
  the events: one row per commitment of a gated action, with its task, the commitment's event
  id and 0-based index, the action kind, and every forecast on the common event paired to it.
- `synth.rs` — the seeded, clock-free synthetic corpus generator (see below).
- `features.rs` — `features@v1`, the versioned feature spec carried in the bundle.
- `dataset.rs` — the dataset, the split and the data manifest.
- `probes.rs` — the three leakage probes (post-action events, label reach, the boundary).
- `head.rs` — the Tier 1 logistic head (hand-rolled, no ML framework).
- `isotonic.rs` — pool-adjacent-violators isotonic calibration.
- `support.rs` — the support count and the floor.
- `bundle.rs` — content-addressed bundles: fit, manifest, id, load, verify.
- `report.rs` — the controller against the generator on held-out rows.

## Usage

All of this is driven through the `tsr train` CLI (wired in `crates/tsr-cli/src/train_cmd.rs`).
Every printed number goes through `tsr_domain::Proxy` with its engineering name.

```
tsr train dataset (--store <db> | --events <events.jsonl>) --tasks <tasks.jsonl> \
    --labels <labels.jsonl> --by time|repo --calibrate <f> --holdout <f> --seed <n> --out <dir>
tsr train fit <dataset-dir> --out models/<name> [--support-floor <n>]
tsr train verify <bundle> [--rebuild <dataset-dir>]
tsr train report <bundle> --dataset <dir> [--seed <n>] [--json]
tsr train apply <bundle> --dataset <dir> [--side held_out|fit|calibrate|all]
```

### `tsr train dataset`

Builds the dataset, its split and its manifests from the log.

- `--store <db>` or `--events <events.jsonl>` — the log. `--events` reads the `tsr task export`
  line format (`{"seq": <n>, "event": {...}}`).
- `--tasks <tasks.jsonl>` — the task index: JSONL of
  `{ task, family, lineage, timestamp, clone_group?, tool_ms_ceiling? }`.
- `--labels <labels.jsonl>` — the labels input: JSONL of
  `{ task, criteria_met, diff_label, reviewer, recorded_at }`. `diff_label` is
  `accepted | rejected | disputed | null`; `null` means never reviewed. A record whose
  `reviewer` is not the operator is refused with exit 2.
- `--by time|repo` — the split rule. `time` is forward-time by the index timestamp; `repo`
  assigns whole lineages by a seeded shuffle. A task, and a clone group when the index names
  one, never crosses a boundary.
- `--calibrate <f>` / `--holdout <f>` — the calibrate and held-out shares of the tasks.
- `--seed <n>` — the split seed.
- `--out <dir>` — the dataset directory.

Writes `rows.jsonl`, `split.json`, `data-manifest.json`, `features.json`, `events.jsonl` and
`tasks.jsonl` into `<dir>`. A row's id is `tsr_domain::Hash::of` over the commitment's event id,
the feature vector and the resolved label with its evidence ids. The data root hashes every row
id in canonical order; the training root the fit and calibrate rows. The split is made before
any fitting and has three sides: `fit` (the logistic head), `calibrate` (isotonic, the support
table, the population) and `held_out` (reports only).

Three leakage probes run and a caught probe exits 2 naming the probe and the row:

- **post-action events** — every row's features are recomputed from the log truncated before
  the commitment; any difference fails.
- **label reach** — extractors receive no label, by type, and a single feature that perfectly
  separates the fit slice's labels fails, named.
- **the boundary** — no task, clone group or (under `repo`) lineage on two sides, and under
  `time` every held-out task is later than every fit and calibrate task.

Every read of a held-out row appends a line to `<dir>/holdout-access.jsonl`; the split digest
covers assignments only, so logging never changes it.

### Input rules

Every input is validated before anything is written, and a refusal exits 2 naming the file and
the field:

- `--tasks` is required: without it every task silently becomes its own lineage with family
  `unknown`, so the flag is refused rather than defaulted.
- A `--labels` record whose `reviewer` is not the operator is refused; `diff_label` must be
  `accepted`, `rejected`, `disputed` or `null`.
- A malformed JSONL line, a wrong field type, a bad hash or a duplicate task id is refused
  naming the file and line.
- `--by` must be `time` or `repo`; a share that leaves any side empty is refused.
- A `--store` whose header is not a TSR store, or whose body is corrupt, is refused naming the
  store.

### `tsr train fit`

Fits a bundle: the logistic head on the fit slice, then isotonic calibration of the common
event over the head's outputs on the calibrate slice. Re-runs every probe and refuses when one
fails. Exits 2 with the counts when fit and calibrate hold fewer than 150 tasks with resolved
labels; a family with fewer than 30 stays outside the calibrated population.

`--support-floor <n>` records the operator's floor in the bundle (default 30).

Prints the bundle id: `tsr_domain::Hash::of` over each content file's digest and the manifest's
content fields — not the code commit, a timestamp or the evaluation report — so the same data
and seed give the same id.

### `tsr train verify`

Recomputes every digest and refuses a mismatch. Exits 0, or 2 naming the file whose digest
differs. With `--rebuild <dataset-dir>` it retrains and compares ids. The manifest's
`code_commit` is recorded provenance, not a verified property: `verify` prints it labelled
`code_commit (recorded, not verified)=<value>` and accepts any value.

### `tsr train report`

Scores, on held-out rows with a resolved label, the controller's calibrated `p` against the
generator's raw `p` for the same commitment: Brier, log loss (clip named and printed) and
reliability (10 equal-width bins with counts) per forecaster, per family and for the bundle.
The operator's raw `p` is scored beside them where recorded. The paired difference, controller
minus generator, is taken per task with a 90% cluster-bootstrap interval over repository
lineage (2000 resamples, seeded), `INCONCLUSIVE` when it spans zero, and `interval n/a` with
fewer than two lineage clusters. Uncalibrated rows are excluded from the pair and counted.

A calibrated number prints as `p` with its calibrator version (the bundle id's first 12 hex and
`features@v1`), a raw one as `raw p`, a missing calibrator as `uncalibrated`. Families and
forecasters are listed in name order, never by score.

### `tsr train apply`

Applies the bundle to a dataset's rows and prints one line per row, in row-id order:

```
row <row id> family <family> raw p=<value> p=<value> (<calibrator version>) support=<n>
...
rows=<n>
```

`raw p` is the generator's uncalibrated probability; `p` is the controller's calibrated one,
printed with its calibrator version (the bundle id's first 12 hex and `features@v1`), or
`uncalibrated` when the row's family is outside the calibrated population. `support=<n>` is the
number of calibrate-slice rows that back the calibrator for the row's family. `--side` selects
which rows to apply to: `held_out` (the default), `fit`, `calibrate` or `all`.

It refuses a foreign dataset (a split digest or feature spec that is not the bundle's) and a
tampered bundle or row, exiting 2 and naming the file or the row id.

## The bundle layout

`tsr train fit` writes a bundle directory (e.g. `models/<name>`) containing:

| file | content |
|------|---------|
| `manifest.toml` | the data-manifest digest (data and training roots), feature-schema version, code commit (`TSR_CODE_COMMIT` or `"unknown"`), seed, calibrator, thresholds (the support floor), the split digest, the source root, the state (`draft`) and the id |
| `head.json` | the logistic head: weights, bias, the standardization statistics and the L2 penalty |
| `isotonic.json` | the isotonic breakpoints |
| `features.json` | the `features@v1` spec |
| `split.json` | the split assignments and its digest |
| `support.json` | the support floor, the calibrated population and the support index |

A bundle is written once, as `draft`; the evaluation report sits beside it naming the id. State
transitions and git tags are M5's. Coefficients and breakpoints live only in the bundle —
nothing writes a fitted parameter to the log or the store.

## The synthetic corpus

`src/synth.rs` generates `fixtures/m4/` deterministically. Regenerate with:

```
TSR_REGEN_FIXTURES=1 cargo test -p tsr-learn --lib synth::tests::regen -- --nocapture
```

`crates/tsr-cli/tests/m4_corpus.rs` regenerates into a temp dir, asserts byte-identity with
`fixtures/m4/`, and asserts every fixture session satisfies all of `tsr_cli::reconstruct`'s
invariants. The true acceptance model, each family's distortion, the seed and the counts are
documented in the module header of `src/synth.rs`.

Every number produced from this corpus is **synthetic**. The real exit evidence — controller
Brier below generator Brier on the held-out part of M3's mined corpus, and the support floor
set — waits for lanes 04 and 05 to merge and for the operator's runs. `docs/m4-evidence.md` is
generated by `scripts/m4-evidence.sh` and shows the synthetic corpus's numbers as the binary
prints them.