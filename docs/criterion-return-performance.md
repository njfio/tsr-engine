# Ordinary-return performance investigation

Offline authored fixture only, 2026-10-31. No provider calls or live authorization
changes. The fixed live packet, game artifacts and historical campaign are outside
this investigation.

## Baseline and diagnosis

The complete `OrdinaryCriterion.test_ordinary_return_to_preference_composition_and_restart`
fixture passed in **747.812 seconds**, using the same authored responses, real
isolated browser, native admission, later composition, retention and restart checks
as the prior integration. Python cProfile and per-CLI timings attribute **722.779
seconds to 556 native CLI calls**. Of those, 122 cycle requests consumed 442.661s,
12 previews 119.676s, and 47 full requests 69.698s. The 279 configuration reads
consumed only 4.247s. Repeated native projections dominate; Python serialization
and generic process startup do not explain most of the measured time.

Raw profile, timings, unchanged baseline executable, reproducer and final authored
fixture are retained under `target/criterion-performance/`.

An initial candidate avoided computing a continuation anchor when no unused
candidate existed. Its fixed-state cycle median was 5.760s versus 5.704s, not a
meaningful improvement on this state. That production change was reverted. Its
patch, binary and comparison outputs remain, and the added integrity regression
was retained. No timeout or validation relaxation was used as an optimization.

## Implemented candidate

`tsr-store` reuses **decoding only**, keyed by sequence and exact freshly queried
payload bytes. Both queries still execute the same SQL and visit all returned
rows. Changed bytes, invalid JSON, deleted rows, rollback and reused sequences
cannot return cached history. The cache retains only the latest query's rows;
returned Events are clones. All Engine chain, identity, CAS, source, review,
budget and Stop checks remain unchanged. Nothing is persisted or trusted by head
hash alone.

The memory tradeoff is another query's payload strings and decoded Event objects.
The preserved fixture has 69 events and 38,984 payload bytes; decoded heap size was
not measured. Alternating task queries may lose cache hits without changing
correctness.

## Fixed-state measurements

Five alternating old/new process pairs per command produced identical JSON after
excluding **only `elapsed_wall_ms`**. Raw stdout/stderr are retained. Event payloads,
CAS and editable source remained byte-identical. No OS-cache flush was performed;
these are warm, alternating measurements, not cold-start guarantees.

| Command | Original median | Decode-cache median |
| --- | ---: | ---: |
| Cycle request | 3.428s | 2.572s |
| Full project request | 2.210s | 1.640s |
| Criterion context | 0.419s | 0.312s |

The first cache comparison encountered the fixture's elapsed-budget boundary
mid-read and correctly refused changed inputs. Its failure remains in
`cache-comparison.log`. The successful `cache-comparison-2/` comparisons ran after
that boundary settled, with both binaries observing the expired/blocked budget.
No budget was extended or rewritten. Therefore these results alone do not prove
the active full-flow improvement; that requires the fresh matched fixture run.

Fixed state content hash:
`26aff17a443fd5d194753c11c64bae29ec6b12f9802e7c167dc848fd5c787f56`.
Original binary:
`43b6013d87851e5cca4cfaff98386d042096717fc25dd9f4fb2204e91e419ef9`.
Decode-cache binary:
`babed00f1ff248d6e8d171033b5855b536115e53725bd409bc7f705916cc1a14`.

## Verification status

Store regressions: 11 passed, including query interleaving, independent returned
values, pruning, rollback/reused sequence, altered payload with unchanged head,
invalid JSON and deleted rows. Native decoded-record integrity regressions: three
passed, including external broken-chain and CAS changes. Independent source
review found no blocking issue, conditional on final integration checks.

The broad workspace attempt completed 81 tests without reported failures and
one existing ignore, then stopped producing output at `d6_tier0`. After more than
six minutes at that binary, the owned command was interrupted (exit 130). A direct
30-second probe also produced no output; `subprocess.run` killed and waited for
that child. Neither result is a passed suite or a proven cache defect. The stalled
test uses scripted repositories; no real repository execution or paid effects
were pending. Raw logs are retained. Runtime binaries were excluded from the
workspace test PATH, per the repository's no-runtime test boundary; no images or
dependencies were acquired.

## Complete active-flow result

The fresh optimized authored flow passed in **632.198s**, versus **747.812s**:
**15.5% lower unadjusted total**, with the same 556 native calls and complete-flow
assertions. Native calls consumed 607.119s versus 722.779s. Cycle requests fell
from 442.661s to 326.340s; previews from 119.676s to 87.686s. The cycle body fell
from 746.572s to 558.240s (25.2%). Both binaries used the same debug build profile.

The first initialization call increased from 0.152s to **73.041s**, unexplained.
It remains in the headline total; the body timing is a separate diagnostic, not
an adjusted replacement. No concurrent benchmark or test ran during this sample.
These are one matched full-flow pair and warm fixed-state comparisons, not a
scaling study or cold-start guarantee.

Raw optimized evidence is in `target/criterion-performance/optimized/run.log`,
`summary.json`, `baseline-cli.jsonl` and `baseline.pstats` (the latter two names
come from the copied reproducer). The unchanged baseline evidence is in the
parent directory. All semantic model responses were authored fixtures; browser
execution was real and isolated. Neither run dispatched a paid model request.
Independent read-only review verified terminal OK, timings, fixed-state output
equivalence and the cache's narrow semantics, finding no blocking issue.

Final cycle regressions: **21 passed**. Criterion-return regressions: **6 passed**.
Ledger regressions: **4 passed**. Targeted `cargo clippy --offline -p tsr-store
--all-targets -- -D warnings`, scoped rustfmt and `git diff --check` passed.
The broad workspace result above remains incomplete. The final ledger build and
test startup were temporarily silent, then completed; no duplicate was launched.
A scoped process-inspection attempt was denied by the local sandbox, so retained
logs were used instead.

## Remaining limit

The full flow still takes minutes. Every query still reads its rows and every
native projection retains authoritative chain/CAS/source validation. The profile
identifies repeated native projections, not a complete Rust CPU flamegraph.
Further substantial reduction likely needs an explicitly scoped verified immutable
history/project view shared across nested helpers, with integrity and source
revalidation at effect boundaries. That is a larger change than this decode-only
optimization and was not implemented here. No timeout was raised, check removed,
or head-only cache introduced. This is not proof of adequate interactive latency,
spontaneous preference formation, live autonomy or causal TSR advantage.

Campaign ledger and frozen live manifest remain byte-identical. No live packet was
rebound and no paid authorization was consumed.
