# M2 evidence — fixtures and the seeded generator

This page records the commands that produced the M2 fixture corpus and their captured output. Every
block below is a command that was run in this workspace on branch `lane/reflect`, followed by the
output it printed.

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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.00s
     Running tests/m2_episodes.rs (target/debug/deps/m2_episodes-4b18fe78ca9a956e)

running 5 tests
test corpus_is_thirty_episodes_across_three_families ... ok
test generation_is_deterministic ... ok
test fixtures_are_committed_and_match_the_generator ... ok
test generator_curve_differs_across_families ... ok
test store_holds_every_episode_for_the_cli ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.84s
```

The five tests are:

- `corpus_is_thirty_episodes_across_three_families` — thirty episodes, ten per family, each with the
  full event sequence.
- `generation_is_deterministic` — two runs from the same seed produce identical events.
- `fixtures_are_committed_and_match_the_generator` — the files on disk equal what the generator
  produces now (the corpus is regenerated, not hand-edited).
- `generator_curve_differs_across_families` — the generator's Brier differs across families, and the
  over-confident family scores worse than the unbiased one.
- `store_holds_every_episode_for_the_cli` — `fixtures/m2/store/tsr.db` holds every episode's events
  and its head root is stable across a rebuild.

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

## The generator's curve differs across families

`tsr reflect --family <f>` draws each forecaster's calibration curve over the family's episodes. The
generator's Brier over the corpus, per family, is printed by the test itself:

```console
$ cargo test -p tsr-cli --test m2_episodes generator_curve -- --nocapture
generator Brier[alpha] = 0.136033
generator Brier[beta] = 0.103261
generator Brier[gamma] = 0.115490
```

`alpha`'s over-confident generator scores worse than `beta`'s unbiased one, which is the shape the
family bias was built to produce. The test asserts the difference and the alpha/beta ordering; it
does not assert a gamma/beta ordering, because ten episodes is too small a sample for the milder
under-confidence to order reliably.

## Regenerating the store

`fixtures/m2/store/tsr.db` is **not committed** — `.gitignore` has `*.db`, so `git ls-files
fixtures/m2` shows only the thirty JSON episodes. The store is a derived artifact: run the generator
to rebuild it, then the `reflect`/`review` commands below read it.

```console
$ export PATH=/usr/local/cargo/bin:$PATH CARGO_HOME=$PWD/.cargo-home
$ cargo test -p tsr-cli --test m2_episodes
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.00s
     Running tests/m2_episodes.rs (target/debug/deps/m2_episodes-4b18fe78ca9a956e)

running 5 tests
test corpus_is_thirty_episodes_across_three_families ... ok
test generation_is_deterministic ... ok
test fixtures_are_committed_and_match_the_generator ... ok
test generator_curve_differs_across_families ... ok
test store_holds_every_episode_for_the_cli ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.84s
$ ls -la fixtures/m2/store/
-rw-r--r-- 1 root root 569344 ... tsr.db
```

The store holds all thirty episodes in one file; there is no per-episode store. A task id from any
episode opens it, e.g. `alpha-00`'s task is
`1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b`:

```console
$ tsr review 1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b --store fixtures/m2/store/tsr.db
no returns await a label
```

## The corpus as a store the CLI can fold

The JSON episodes are the committed record; `fixtures/m2/store/tsr.db` is the same corpus as a
SQLite store, so the CLI can open it with `--store`. The store is **regenerated** by the test (it is
`*.db`, which `.gitignore` excludes), never committed:

```console
$ ls -la fixtures/m2/store/
-rw-r--r-- 1 root root 569344 ... tsr.db
```

```console
$ tsr reflect --family alpha --store fixtures/m2/store/tsr.db
```

> **Pending.** `tsr reflect` (task #5) had not landed when this page was written; the command above
> exits 2 with `not yet: reflect — B7 the Mirror view (M2)`. The capture is to be pasted here once
> the command exists. The corpus and the store it reads are complete and verified by the tests
> above.

## Reproducing the corpus

```console
$ export PATH=/usr/local/cargo/bin:$PATH CARGO_HOME=$PWD/.cargo-home
$ cargo test -p tsr-cli --test m2_episodes
```

The test regenerates `fixtures/m2/episodes/*.json` and `fixtures/m2/store/tsr.db` from the seed and
asserts they match what is committed. Nothing in the corpus is hand-written.
