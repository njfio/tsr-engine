# Relational revision persistence and evidence partition

2026-10-22; offline only, no provider calls or simulated actions. The authoritative
run and all shadow outcomes remain unchanged. Implementation follows `5b70932`.

## Concrete representation repair

The request previously exposed admitted response catalog and event history in separate
lists but left their relationship implicit. `engine-archive-contract.py` now derives
`synthetic-response-evidence@v1` for normal synthetic requests, including revisions.
It binds state root and observation revision and partitions every admitted response
into recorded (exact matching event with zero-based index) or available-unrecorded.
Existing event validation rejects stale revisions, wrong challenges/respondents,
unrouted response records and duplicate events. Catalog identities must be unique.

Available does not mean executable prerequisites are met. Recorded is historical,
survives withdrawal, and does not imply current applicability, truth, fulfillment or
permission. Full parent meanings, original/current aspiration and fictional catalog
remain available; hypotheses are not censored or checked with keyword heuristics.
The projection is deterministic evidence, not an independent semantic verifier.

## Actual historical-prefix experiment

`target/relational-prefix-verification/verify.py` opened the source database read-only,
inserted its exact first 46 records into fresh databases with the same schema/triggers,
and copied CAS files. No deletion or rewriting of historical source events occurred.
Record 47 is the historical revision, excluded from the explicitly labeled prefixes.
All prefix records preserve identity, timestamps, payload and causation. Extra copied
CAS objects confer no state: only referenced prefix records are reconstructed.

Prefix root: `9954bfef196200ce7b3bd187a28e1dea80f1dff291a6178096efccdf19b8e19b`.
The actual access-review return is present; its original later revision is absent.

The exact S shadow response targets a different root and prior revision. Each disposable
prefix **refused it unchanged**, with identical authoritative storage afterward. It is
not valid to rewrite its binding and call that an original model result.

Three separately labeled offline fixtures then exercised `direction-relate`, normal
proposal projection and fresh-process reconstruction:

- no new insight: prior meaning verbatim, separate return commentary;
- contradictory interpretation: an explicitly fixture-authored challenge to a prior
  interpretation, not a fabricated external event;
- shadow-prose fixture: S's prose preserved, bindings adapted and documented as authored
  fixture data. The acknowledgment ambiguity remains visible and semantically unverified.

All three persisted, cleared the pending review, retained earlier lineage, produced
identical reconstructed state across restart, projected their guidance, and refused
replay without mutation. Parent desires, archive events, cycles, observations and
execution authority stayed unchanged. All 30 source SQLite/CAS files stayed unchanged.
The actual-prefix request is 16003 bytes, below the existing 16384-byte ceiling; no
limits were raised. Evidence and exact fixture files: `target/relational-prefix-verification/`.

These are integration checks, not three generated developments or proof of model
competence. Source snapshot and response hashes are preserved in the evidence; the
unmodified shadow result remains under `target/relational-development-comparison-final/`.

## Scoped functioning criteria

| Criterion | Status |
| --- | --- |
| Original parents, authority and append-only relational lineage survive revision/restart | Verified offline; prior live lineage retained |
| Actual return is separated from potential fictional response evidence | Implemented and deterministically checked |
| No-new-insight and genuine questioning remain possible | Verified with labeled fixtures |
| Rich relational guidance reaches normal possibility context | Integration verified; reliable behavioral benefit unestablished |
| Generated revision is substantively grounded and develops relational meaning | Partial evidence only; shadow S retains ambiguity and loses some original richness |
| Routine bounded proposal/action/return orchestration | Prior live cycle completed; this turn adds no behavioral evidence |
| Social fulfillment, general autonomous judgment, learned-calibration eligibility | Not established by this synthetic scope |

A functioning mechanism is not yet reliable semantic development. Typed partitions
cannot prevent a model from claiming an available acknowledgment occurred, or replacing
meaning with indicators. No language-based automatic acceptance claim is made.

## Smallest future validation, not authorized or dispatched here

One fresh, explicitly admitted provider revision call through the normal driver on a
new disposable copy of this actual pre-revision return: same pinned provider/model,
8192 completion tokens, 240 seconds, one call and zero actions. Freeze the exact prefix,
parent meanings, original/current lineage and typed evidence partition. Only disposable
state may receive a structurally valid revision; independently review grounding before
calling it semantic progress. Inability, truncation and ambiguity are preserved, with
no retry. This reuses a historical observed return; it does not establish adaptation to
a new event. No new world evidence, canned aspiration or desired next action is needed.

Success for that narrow check is normal-path generation/persistence/restart plus a
grounded distinction between actual recording and potential response content, with
relational meaning retained or explicitly questioned. No-new-insight is valid. A
later behavioral claim requires genuinely new admitted return evidence and subsequent
choice, rather than repeating a saturated archive scenario.

## Verification and independent review

- Focused relational/evidence regressions: 4 passed, including fresh-process persistence,
  replay refusal, no-new-insight, contrary interpretation and invalid event bindings.
- Broader offline Python engine sweep: 216 tests, 195 passed, 10 skipped, 11 errors.
  All 11 errors are `test_engine_run` stub-server setup failures: sandbox denied local
  socket bind (`PermissionError: Operation not permitted`). No provider call occurred.
  This is not a clean aggregate pass; log: `python-tests.log` in the evidence directory.
- Independent `/root/review` inspected implementation and actual prefix evidence: no
  blocker; noted only 381 bytes of request headroom. Future longer states may fail the
  existing ceiling; no broader payload capacity is claimed or silently granted.
- No Rust implementation changed. No protected files changed. Process listing was not
  attempted; the earlier sandbox denial was respected.
- Affected archive/synthetic normal-cycle suite: 15 passed in 28.465 seconds,
  including provider-fixture process-restart transitions; `affected-tests.log`.
- `git diff --check` passed. No live calls, inference charges or source adoption.
