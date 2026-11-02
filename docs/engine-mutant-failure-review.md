# Corpus-mutant failure qualification after c40f186

## Original observation

The broad workspace sweep logged two failures in unchanged `tsr-corpus` tests:

- `caught_mutants_become_tasks`: descriptor `src/lib.rs:2:5: replace add -> i32 with 1`.
- `d9_each_mutant_carries_its_own_failing_tests`: descriptor
  `src/lib.rs:2:5: replace add -> i32 with 0`.

Both named fixture base tree `116c5cea7ba5c6536ee596cf095a13721427a5c5` and reported
that the mutation diff did not apply. Exact original output remains in
`target/shared-workspace.log`, around lines 1091–1103.

That text does **not** identify the failing operation. `reversal_diff` returns `None`
for materialization failure, patch process spawn/write/wait/application failure, failed
`git diff`, or empty diff output. Its caller maps every `None` to the same message.
`apply_patch` discards stderr, and scratch files are removed. Therefore the original
logs cannot distinguish patch mismatch from process/filesystem interference.

## Differential and concurrency checks

Baseline `31859c2` was extracted with `git archive` into the isolated
`target/mutant-review-baseline` directory and built offline with installed Rust 1.90.
No baseline source or tests were edited. The current source was `c40f186`.

`git diff 31859c2 c40f186 -- crates/tsr-corpus Cargo.toml Cargo.lock` is empty.
The only change to existing `tsr-domain/src/lib.rs` was exporting the new pure decision
module; corpus code does not call it. No new runtime initialization, environment mutation,
or shared mutable state was added to the corpus execution path. This is evidence against
an introduced coupling, not proof of the original failure's cause.

Each revision ran the **whole mutant test target three times with eight test threads**:

| Revision | Concurrent test runs | Result each run |
| --- | --- | --- |
| `31859c2` | 3 | 29 passed, 0 failed, 1 ignored |
| `c40f186` | 3 | 29 passed, 0 failed, 1 ignored |

The first baseline/current invocation pair overlapped at the process level. Subsequent
repetitions retained eight threads within each test process. These are bounded targeted
reproductions, not additional model runs or corpus mining. The earlier isolated serial
rerun also passed 29 tests with 1 ignored.

Logs:

- `target/mutant-review-current.log`
- `target/mutant-review-baseline.log`
- `target/mutant-review-c40f186-repeat-1.log` and `-2.log`
- `target/mutant-review-31859c2-repeat-1.log` and `-2.log`
- `target/shared-mutants-recheck.log` (earlier serial rerun)

An initial repetition harness refused to choose between multiple cached current test
executables; no current test ran in that invocation. The baseline repetitions finished.
The current repetitions then used the exact executable identified by Cargo's log. No
uncertain test execution was duplicated or counted as a pass.

## Conclusion and remaining limitation

No introduced regression was identified, and the failures did not reproduce on either
revision under these concurrent checks. **Their root cause remains unresolved.** It is
not established that they are environmental, pre-existing nondeterminism, or harmless.
The full workspace sweep remains 796 passed / 4 failed / 5 ignored, including these two
and the two documented M0 runtime-dependent failures; passing reruns do not erase it.

No production fix or protected-test change is justified by the available evidence.
If the failure recurs, the smallest diagnostic change is to preserve the failed stage,
subprocess exit/stderr and fixture diff before cleanup, rather than further blind reruns.
That diagnostic work is distinct from the verified bounded decision-kernel extraction.
