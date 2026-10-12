# M2 evidence — fixtures, the seeded generator, and the Mirror

This page records the commands that produced the M2 fixture corpus and the M2 commands' captured
output. Every block below is a command that was run in this workspace on branch `lane/reflect`,
followed by the output it printed. Nothing here is hand-edited.

## What the corpus is, and what it is not

`fixtures/m2/episodes/` holds **thirty synthetic episodes** — ten each in three task families
(`alpha`, `beta`, `gamma`). They are produced by a seeded generator,
`crates/tsr-cli/tests/m2_episodes.rs`, which drives the store API through the whole cycle:

> open → propose → forecast (generator, controller, operator) → commit → dispatch → execute →
> close → review

**They are synthetic.** No operator session produced them, and no number in them is a measurement of
a real agent. The real thirty episodes come from operator sessions after M3's batch runner. The
corpus exists so that `tsr reflect --family` has a curve to draw and so that the M2 commands have a
deterministic input to test against.

The three families differ in the generator's calibration: `alpha`'s generator is over-confident
(+0.15 on its raw probability), `beta`'s is unbiased (0.0), and `gamma`'s is under-confident
(−0.15). The controller and the operator carry no bias, so the generator's curve is the one that
moves between families.

## The generator is deterministic

The seed is `SEED = 0x5eed_2024_0000_0001` (`crates/tsr-cli/tests/m2_episodes.rs`). Every id, every
probability and every timestamp is a pure function of the seed and the episode index, so the corpus
is reproducible byte for byte.

```console
$ export PATH=/usr/local/cargo/bin:$PATH CARGO_HOME=$PWD/.cargo-home
$ cargo test -p tsr-cli --test m2_episodes
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.32s
     Running tests/m2_episodes.rs (target-fresh/debug/deps/m2_episodes-0c6b5dd344bda05a)

running 6 tests
test generator_curve_differs_across_families ... ok
test corpus_is_thirty_episodes_across_three_families ... ok
test fixtures_are_committed_and_match_the_generator ... ok
test generation_is_deterministic ... ok
test d6_fixture_compare_is_read_only ... ok
test store_holds_every_episode_for_the_cli ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 67.68s
```

The six tests are:

- `corpus_is_thirty_episodes_across_three_families` — thirty episodes, ten per family, each with the
  full event sequence.
- `generation_is_deterministic` — two runs from the same seed produce identical events.
- `fixtures_are_committed_and_match_the_generator` — the files on disk equal what the generator
  produces now.
- `generator_curve_differs_across_families` — the generator's Brier differs across families, and the
  over-confident family scores worse than the unbiased one.
- `store_holds_every_episode_for_the_cli` — `fixtures/m2/store/tsr.db` holds every episode's events
  and its head root is stable across a rebuild.
- `d6_fixture_compare_is_read_only` — a plain run (no `TSR_REGEN_FIXTURES=1`) compares the committed
  JSON and store against a fresh build and **fails** on a mismatch instead of silently refreshing
  them; it leaves `fixtures/m2/episodes/*.json` and `fixtures/m2/store/tsr.db` byte-identical.

## Regeneration is explicit

The corpus is regenerated only when `TSR_REGEN_FIXTURES=1` is set. Without it the test compares
read-only and never writes the committed files:

```console
$ TSR_REGEN_FIXTURES=1 cargo test -p tsr-cli --test m2_episodes
```

## Thirty episodes, ten per family

```console
$ ls fixtures/m2/episodes | wc -l
30
$ ls fixtures/m2/episodes | sed 's/-[0-9]*\.json//' | sort | uniq -c
     10 alpha
     10 beta
     10 gamma
```

Each file is one episode: the task id, its family, its index, its outcome, and the ordered events.
The first episode of the `alpha` family begins:

```console
$ head -c 400 fixtures/m2/episodes/alpha-00.json
{
  "task": "1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b",
  "family": "alpha",
  "index": 0,
  "outcome": "disputed",
  "events": [
    {
      "id": "4589fbbdf9342d35970dcbe9269dd820cf63b071f63680cc9358b251bc13d6bc",
      "schema_version": 1,
      "task": "1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b",
      "parents": [],
      "recorded_at": "2024-01-
```

## The store the CLI folds

`fixtures/m2/store/tsr.db` is **not committed** — `.gitignore` has `*.db`, so `git ls-files
fixtures/m2` shows only the thirty JSON episodes. The store is a derived artifact: run the generator
to rebuild it, then the `reflect`/`review` commands below read it.

```console
$ ls -la fixtures/m2/store/
total 624
drwxr-xr-x 3 root root     96 Oct 12 01:03 .
drwxr-xr-x 4 root root    128 Oct 11 18:47 ..
-rw-r--r-- 1 root root 638976 Oct 12 00:08 tsr.db
```

The store holds all thirty episodes in one file; there is no per-episode store. A task id from any
episode opens it, e.g. `alpha-00`'s task is
`1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b`.

## The Mirror — `tsr reflect <task>`

```console
$ tsr reflect 1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b --store fixtures/m2/store/tsr.db
Mirror — task 1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b
ledger root 7107da8033a57f3988ab786ed8b64d4fc8ed54f0670381e990bf2aba74464a95

Desire
  intent: the approved test passes without touching the schema
  criterion 9336872f1b30 [open] the approved test passes without touching the schema
  notes: none

Imagination
  candidate de1f6a69fdfd (generator) propose_patch — committed
  candidate 6f904c678006 (council) propose_patch — excluded by conflict
  candidate 0b1e5f37bbf3 (intuition) inspect — considered
    support (preferences served): 9336872f1b30
    tensions addressed: none yet

Belief
  committed: de1f6a69fdfd
  generator: raw p 0.4670; uncalibrated; support 1 episodes; propensity 1.0000
  controller: raw p 0.4034; uncalibrated; support 1 episodes; propensity 1.0000
  operator: raw p 0.3715; uncalibrated; support 1 episodes; propensity 1.0000

Return
  d2c473e3f7a1 executed exit=1 evidence=0436095cafff
  914441e6d5da label exit=- evidence=0436095cafff
    label disputed 

Residual
  Surprise (forecast residual per forecaster)
    forecast 35cd358cf453 residual -0.4670
    forecast fcf47932deec residual -0.4034
    forecast 8a1b7e0d5beb residual -0.3715
  Contrast (desired minus actual, per criterion)
    criterion b3bfa8e8731d unmet rejected the criterion went unmet evidence 
    criterion 14a407adf352 unmet rejected the criterion went unmet evidence 
    criterion 14a407adf352 unmet wrong_behaviour  evidence d2c473e3f7a1
```

## The Mirror as JSON — `tsr reflect <task> --json`

```console
$ tsr reflect 1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b --store fixtures/m2/store/tsr.db --json
{
  "task": "1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b",
  "ledger_root": "7107da8033a57f3988ab786ed8b64d4fc8ed54f0670381e990bf2aba74464a95",
  "desire": {
    "intent": "the approved test passes without touching the schema",
    "criteria": [
      {
        "id": "9336872f1b30",
        "text": "the approved test passes without touching the schema",
        "state": "open"
      }
    ],
    "notes": []
  },
  "imagination": {
    "candidates": [
      {
        "id": "de1f6a69fdfd",
        "source": "generator",
        "kind": "propose_patch",
        "excluded_by_conflict": false,
        "committed": true,
        "support": [],
        "tensions": []
      },
      {
        "id": "6f904c678006",
        "source": "council",
        "kind": "propose_patch",
        "excluded_by_conflict": true,
        "committed": false,
        "support": [],
        "tensions": []
      },
      {
        "id": "0b1e5f37bbf3",
        "source": "intuition",
        "kind": "inspect",
        "excluded_by_conflict": false,
        "committed": false,
        "support": [
          "9336872f1b30"
        ],
        "tensions": []
      }
    ]
  },
  "belief": {
    "committed": "de1f6a69fdfd",
    "forecasters": [
      {
        "forecaster": "generator",
        "raw_p": 0.4670024597241542,
        "calibrated_p": null,
        "calibrator": null,
        "calibration": "uncalibrated",
        "support_episodes": 1,
        "propensity": 1.0
      },
      {
        "forecaster": "controller",
        "raw_p": 0.4034315957489555,
        "calibrated_p": null,
        "calibrator": null,
        "calibration": "uncalibrated",
        "support_episodes": 1,
        "propensity": 1.0
      },
      {
        "forecaster": "operator",
        "raw_p": 0.37154197651788534,
        "calibrated_p": null,
        "calibrator": null,
        "calibration": "uncalibrated",
        "support_episodes": 1,
        "propensity": 1.0
      }
    ]
  },
  "returns": [
    {
      "event": "d2c473e3f7a1",
      "status": "executed",
      "exit_code": 1,
      "evidence_digest": "0436095cafff",
      "label": null,
      "label_reason": null,
      "supersedes": null
    },
    {
      "event": "914441e6d5da",
      "status": "label",
      "exit_code": null,
      "evidence_digest": "0436095cafff",
      "label": "disputed",
      "label_reason": null,
      "supersedes": null
    }
  ],
  "residual": {
    "surprise": [
      {
        "forecaster": "generator",
        "forecast": "35cd358cf453",
        "residual": -0.4670024597241542
      },
      {
        "forecaster": "controller",
        "forecast": "fcf47932deec",
        "residual": -0.4034315957489555
      },
      {
        "forecaster": "operator",
        "forecast": "8a1b7e0d5beb",
        "residual": -0.37154197651788534
      }
    ],
    "contrast": [
      {
        "criterion": "b3bfa8e8731d",
        "unmet": "rejected",
        "reason": "the criterion went unmet",
        "evidence": []
      },
      {
        "criterion": "14a407adf352",
        "unmet": "rejected",
        "reason": "the criterion went unmet",
        "evidence": []
      },
      {
        "criterion": "14a407adf352",
        "unmet": "wrong_behaviour",
        "reason": null,
        "evidence": [
          "d2c473e3f7a1"
        ]
      }
    ]
  }
}
```

## Rejecting a return — `tsr review <task> rejected`

The fixture corpus labels every return, so a rejection names the return it supersedes with
`--return`. The run below is against a scratch copy of the store, so the committed corpus is
untouched:

```console
$ tsr review 1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b rejected \
    --return d2c473e3f7a1fd3305752691bf8215d7d9c15c18a7827747ad716853fa3a6351 \
    --reason "regresses span offsets" --store .ev7.db
reviewed d2c473e3f7a1fd3305752691bf8215d7d9c15c18a7827747ad716853fa3a6351 label=rejected status=rejected record=ebc0b7945ebbd92b9f3e5fa15a14fb68dff39fb5d505d7dbb42dfc4008399144
```

## An operator note — `tsr note`

```console
$ tsr note 1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b \
    --text "operator saw the offsets drift" --store .ev7.db
noted e9284f482d238102ffb2863cd05a88e9bf35fd50e50525e0da0eb4f8b11d6d63 (observation=experience source=operator)
```

## Reconciling dispatches — `tsr task reconcile`

```console
$ tsr task reconcile 1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b --store .ev7.db
no pending dispatches
```

## The family curves — `tsr reflect --family`

`tsr reflect --family <f>` draws each forecaster's calibration curve over the family's episodes. The
generator's curve is the one that differs across families — that is the calibration bias the corpus
was built to carry:

```console
$ tsr reflect --family alpha --store fixtures/m2/store/tsr.db
Calibration curves — family alpha

  generator: n 10 brier 0.5038 log_loss 1.3975
    bin [0.0,0.1) count 0 mean_p - observed -
    bin [0.1,0.2) count 0 mean_p - observed -
    bin [0.2,0.3) count 0 mean_p - observed -
    bin [0.3,0.4) count 0 mean_p - observed -
    bin [0.4,0.5) count 3 mean_p 0.4723 observed 0.0000
    bin [0.5,0.6) count 2 mean_p 0.5434 observed 0.0000
    bin [0.6,0.7) count 0 mean_p - observed -
    bin [0.7,0.8) count 1 mean_p 0.7954 observed 0.0000
    bin [0.8,0.9) count 2 mean_p 0.8491 observed 0.0000
    bin [0.9,1.0) count 2 mean_p 0.9225 observed 0.0000

  controller: n 10 brier 0.3076 log_loss 0.8200
    bin [0.0,0.1) count 0 mean_p - observed -
    bin [0.1,0.2) count 0 mean_p - observed -
    bin [0.2,0.3) count 1 mean_p 0.2937 observed 0.0000
    bin [0.3,0.4) count 2 mean_p 0.3273 observed 0.0000
    bin [0.4,0.5) count 2 mean_p 0.4048 observed 0.0000
    bin [0.5,0.6) count 0 mean_p - observed -
    bin [0.6,0.7) count 4 mean_p 0.6895 observed 0.0000
    bin [0.7,0.8) count 1 mean_p 0.7385 observed 0.0000
    bin [0.8,0.9) count 0 mean_p - observed -
    bin [0.9,1.0) count 0 mean_p - observed -

  operator: n 10 brier 0.3557 log_loss 0.9370
    bin [0.0,0.1) count 0 mean_p - observed -
    bin [0.1,0.2) count 0 mean_p - observed -
    bin [0.2,0.3) count 0 mean_p - observed -
    bin [0.3,0.4) count 3 mean_p 0.3651 observed 0.0000
    bin [0.4,0.5) count 2 mean_p 0.4129 observed 0.0000
    bin [0.5,0.6) count 0 mean_p - observed -
    bin [0.6,0.7) count 0 mean_p - observed -
    bin [0.7,0.8) count 5 mean_p 0.7502 observed 0.0000
    bin [0.8,0.9) count 0 mean_p - observed -
    bin [0.9,1.0) count 0 mean_p - observed -
```

```console
$ tsr reflect --family beta --store fixtures/m2/store/tsr.db
Calibration curves — family beta

  generator: n 10 brier 0.3210 log_loss 0.8569
    bin [0.0,0.1) count 0 mean_p - observed -
    bin [0.1,0.2) count 0 mean_p - observed -
    bin [0.2,0.3) count 0 mean_p - observed -
    bin [0.3,0.4) count 4 mean_p 0.3340 observed 0.0000
    bin [0.4,0.5) count 1 mean_p 0.4175 observed 0.0000
    bin [0.5,0.6) count 0 mean_p - observed -
    bin [0.6,0.7) count 3 mean_p 0.6812 observed 0.0000
    bin [0.7,0.8) count 2 mean_p 0.7725 observed 0.0000
    bin [0.8,0.9) count 0 mean_p - observed -
    bin [0.9,1.0) count 0 mean_p - observed -

  controller: n 10 brier 0.3076 log_loss 0.8200
    bin [0.0,0.1) count 0 mean_p - observed -
    bin [0.1,0.2) count 0 mean_p - observed -
    bin [0.2,0.3) count 1 mean_p 0.2937 observed 0.0000
    bin [0.3,0.4) count 2 mean_p 0.3273 observed 0.0000
    bin [0.4,0.5) count 2 mean_p 0.4048 observed 0.0000
    bin [0.5,0.6) count 0 mean_p - observed -
    bin [0.6,0.7) count 4 mean_p 0.6895 observed 0.0000
    bin [0.7,0.8) count 1 mean_p 0.7385 observed 0.0000
    bin [0.8,0.9) count 0 mean_p - observed -
    bin [0.9,1.0) count 0 mean_p - observed -

  operator: n 10 brier 0.3557 log_loss 0.9370
    bin [0.0,0.1) count 0 mean_p - observed -
    bin [0.1,0.2) count 0 mean_p - observed -
    bin [0.2,0.3) count 0 mean_p - observed -
    bin [0.3,0.4) count 3 mean_p 0.3651 observed 0.0000
    bin [0.4,0.5) count 2 mean_p 0.4129 observed 0.0000
    bin [0.5,0.6) count 0 mean_p - observed -
    bin [0.6,0.7) count 0 mean_p - observed -
    bin [0.7,0.8) count 5 mean_p 0.7502 observed 0.0000
    bin [0.8,0.9) count 0 mean_p - observed -
    bin [0.9,1.0) count 0 mean_p - observed -
```

```console
$ tsr reflect --family gamma --store fixtures/m2/store/tsr.db
Calibration curves — family gamma

  generator: n 10 brier 0.1832 log_loss 0.5352
    bin [0.0,0.1) count 0 mean_p - observed -
    bin [0.1,0.2) count 3 mean_p 0.1723 observed 0.0000
    bin [0.2,0.3) count 2 mean_p 0.2434 observed 0.0000
    bin [0.3,0.4) count 0 mean_p - observed -
    bin [0.4,0.5) count 1 mean_p 0.4954 observed 0.0000
    bin [0.5,0.6) count 2 mean_p 0.5491 observed 0.0000
    bin [0.6,0.7) count 2 mean_p 0.6225 observed 0.0000
    bin [0.7,0.8) count 0 mean_p - observed -
    bin [0.8,0.9) count 0 mean_p - observed -
    bin [0.9,1.0) count 0 mean_p - observed -

  controller: n 10 brier 0.3076 log_loss 0.8200
    bin [0.0,0.1) count 0 mean_p - observed -
    bin [0.1,0.2) count 0 mean_p - observed -
    bin [0.2,0.3) count 1 mean_p 0.2937 observed 0.0000
    bin [0.3,0.4) count 2 mean_p 0.3273 observed 0.0000
    bin [0.4,0.5) count 2 mean_p 0.4048 observed 0.0000
    bin [0.5,0.6) count 0 mean_p - observed -
    bin [0.6,0.7) count 4 mean_p 0.6895 observed 0.0000
    bin [0.7,0.8) count 1 mean_p 0.7385 observed 0.0000
    bin [0.8,0.9) count 0 mean_p - observed -
    bin [0.9,1.0) count 0 mean_p - observed -

  operator: n 10 brier 0.3557 log_loss 0.9370
    bin [0.0,0.1) count 0 mean_p - observed -
    bin [0.1,0.2) count 0 mean_p - observed -
    bin [0.2,0.3) count 0 mean_p - observed -
    bin [0.3,0.4) count 3 mean_p 0.3651 observed 0.0000
    bin [0.4,0.5) count 2 mean_p 0.4129 observed 0.0000
    bin [0.5,0.6) count 0 mean_p - observed -
    bin [0.6,0.7) count 0 mean_p - observed -
    bin [0.7,0.8) count 5 mean_p 0.7502 observed 0.0000
    bin [0.8,0.9) count 0 mean_p - observed -
    bin [0.9,1.0) count 0 mean_p - observed -
```

The generator's Brier is 0.5038 in `alpha`, 0.3210 in `beta` and 0.1832 in `gamma`: the
over-confident family scores worst and the under-confident family best, which is the bias the
generator was seeded with. The controller and the operator are identical across families, as they
carry no bias.

## What is not here

- **No real session.** These episodes are synthetic; the real corpus arrives with M3.
- **No calibration.** Every forecaster reads `uncalibrated` — no head has been trained, so
  `calibrated_p` is `null` and `calibrator` is `null`.
- **No committed store.** `fixtures/m2/store/tsr.db` is derived and git-ignored; rebuild it with the
  generator before running the commands above.
- **`reflect --family` needs a `Scope::Family` preference no hook session has.** The family curve
  folds the episodes of one family, which requires a preference scoped to that family
  (`Scope::Family`). A hook session records only task-scoped preferences, so a session recorded by
  the hook has no family scope and `tsr reflect --family <f>` over it draws no curve. The command
  that shows it, over the hook's own session store:

  ```console
  $ tsr reflect --family alpha --store fixtures/m0/claude-session.db
  Calibration curves — family alpha

  no scored forecasts for family alpha
  ```

  The fixture corpus carries the family scope, so the curves above are drawn from it; a real hook
  session will need M3's batch runner to record the family scope.