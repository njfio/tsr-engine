# Corrected sequential batch: accepted repair, adaptation unexercised

The single approved batch ran from 2026-10-28 09:40:39 UTC with a hard deadline of
11:09:37 UTC. It completed in about two minutes. The original run stopped at
`review_required`; independently supported acceptance was then recorded in a
separate review copy under the bundled authority.

Before launch, preparation exposed a grant dispatcher omission: the existing v2
validator was not reachable from `grant_calls`. No paid call or operation occurred
before that refusal. The one-line dispatch correction was independently reviewed,
tested with actual authored grant-chain append/reopen controls, and committed as
`5f943d5d`. Fresh release/admission/source pins were used, preserving both the
failed preparation and prior live failure. See `engine-information-grant-dispatch.md`.

Actual sequence:

1. Luna proposed a 2,438-byte replacement, exceeding the unchanged 2,048-byte limit.
   It was retained and rejected, never adopted.
2. One admitted fresh generation used precise size feedback and produced a
   1,335-byte replacement. No operator supplied replacement code.
3. A separate runtime review approved it. The existing sealed runtime executed one
   group; both immutable protected cases passed.
4. Independent assessor `/root/grant_review` checked the actual source and receipt
   against the public domain and requirements. `/root` faithfully transcribed that
   assessment through native acceptance in a separate copy. Its status is accepted
   and the scoped desire status is fulfilled. This is separate final assessment,
   not unattended self-certification.

The implementation covers distinct tags first, preferring prior available IDs,
then fills capacity with prior IDs before other available IDs. The assessor found
that this meets cardinality, uniqueness, availability, determinism, coverage and
maximum feasible retention under the stated distinct-ID domain.

No information read, relationship revision or second pursuit occurred. The engine
chose direct repair, and immediate success was a preregistered legitimate outcome.
The run therefore demonstrates bounded correction containment and a real accepted
repair, **not** outcome-driven TSR adaptation, culmination, or causal advantage.
Ordinary coding-agent behavior explains this result. The original behavioral
question remains open; no additional run is authorized.

Three requests (generator, one correction, reviewer), one program group, zero
information reads. Usage: 12,351 prompt tokens, 6,067 completion tokens, 18,418 total.
Settled cost: **$0.003161**; no campaign unknowns. Campaign totals: **226/239 calls,
62/64 groups, $0.184757/$5**. Unused allowances from this batch expire and do not
permit an extra attempt. The older Garden Trial 1 unknown remains separate.

Fresh-process replay returned `replayed_without_dispatch: true`, with no ledger
change. Protected bytes match frozen originals. Original durable state files and
prior failed-result evidence remain unchanged; read-only SQLite inspection created
only transient SHM/empty WAL sidecars. Separate acceptance survived fresh-process
inspection with the same identity.

Evidence index: `experiments/information-pursuit-live-2/result.json`.
Full local evidence: `target/native-information-live-2-dispatch/`.
Separate review copy: `target/native-information-live-2-acceptance/`.
No raw model output or credentials are committed. No limits, tests or semantic
criteria were weakened. Missing local lefthook was reported by Git; explicit local
checks and independent review, rather than unrun hooks, support the preparation.
