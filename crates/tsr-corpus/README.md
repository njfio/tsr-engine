# tsr-corpus

The corpus half of milestone M3 (PRD §B9, §B11 M3): turn a repository's history and its mutants into
a corpus of tasks, and split that corpus without leaking the holdout.

## The Runner split

Everything that would run a repository's code goes through the `Runner` trait:

```rust
fn run(&self, snapshot: &Path, argv: &[String], limits: &Limits) -> RunResult
// RunResult { exit, stdout_digest, stderr_digest, duration_ms }
```

- **`ScriptedRunner`** answers from a fixture script. Every test uses it. It never touches a
  repository.
- **`ExecRunner`** invokes the `tsr` binary on `PATH` as `tsr exec -- <argv>`, so every command runs
  in the sealed sibling sandbox the PRD requires (PRD §B4). Its integration tests are `#[ignore]`d
  with the reason `needs tsr exec and a container runtime; operator runs it`.

**Nothing in `tsr-corpus` runs a repository's tests on the host.** `mine` reads git history through
`std::process::Command` (it reads history, it executes nothing from the repository); every *suite*
run goes through the `Runner`.

## Commands

### `tsr corpus mine <repo> --out <dir> [--since <rev>] [--suite <cmd>]`

Walks the repository's first-parent history. For each commit with a parent it runs the suite at the
parent and at the commit through the `Runner`, and keeps the tests that flipped red→green. A task is
`{ id: blake3 of (repo, parent, commit), parent_snapshot: digest, flipped_tests, hidden_diff: CAS
digest of the reference diff, family (from the paths touched: crate name, or a declared mapping),
lineage: repo, timestamp }`. Writes `tasks.jsonl` and the CAS under `--out`.

The agent is meant to see a shallow clone at the parent only; the oracle is "flipped tests pass and
the full suite does not regress", encoded as the task's `OutcomeDef`.

### `tsr corpus mutants <repo> --out <dir>`

Drives `cargo mutants --list --json` and the kill run through the `Runner`. Each mutant killed by
the suite becomes a task `{ mutated snapshot digest, failing tests, known reversal diff in the CAS,
family = (mutation operator, crate), lineage, timestamp }`. The real cargo-mutants 24.x JSON shapes
are captured under `fixtures/m3/`.

### `tsr corpus split <dir> --by time|repo --holdout <fraction> --seed <n>`

Forward-time by task, or lineage-separated by repository. Near-clone deduplication by diff
similarity (normalized diff shingles, Jaccard ≥ 0.8 → one clone group, never split across the
boundary). Writes a split manifest with the clone groups and the rule used. Leakage probes run as
tests: a task whose hidden diff appears in another split's parent snapshot, a test added after the
task's timestamp, a post-action log — each must be caught. Every holdout access is logged to the
manifest (the PRD's "holdout access is logged" rule).

## Operator commands

These need a host with Docker (or Podman rootless), the `tsr` binary on `PATH`, and lane 01's
`tsr exec`:

```sh
git clone --filter=blob:none https://github.com/njfio/rts /work/rts
tsr corpus mine /work/rts --out /work/corpus-rts --suite "cargo test --offline"
tsr corpus mutants /work/rts --out /work/corpus-rts-mutants
tsr corpus split /work/corpus --by time --holdout 0.2 --seed 1
```

See `docs/m3-evidence.md` for the captured output of every command.