# Controller refusal and assessment repair

2026-10-22. Offline repair after the normal-driver live attempt documented in
`engine-normal-synthetic-live-result.md`. No provider calls, project executions,
protected-file edits, or changes to prior evidence. The historical $0.011843 receipts
and `uncertain_application` record remain intact. No unused allowance is reclaimed.

## Implemented boundary

The normal driver now distinguishes two positively recognized, normally exited Rust
controller refusals: no supported expected progress and stale direction offer. Its
adapter records a typed code, actual exit code and stderr hash/byte count. It does not
persist arbitrary error prose. Signals, timeouts, unrecognized exits and malformed
stdout carry separate failure metadata and remain uncertain after application intent.

A refusal is classified as `controller_refused` only when before/after state roots AND
complete authoritative file inventories match. Inventories cover SQLite and its
sidecars plus every CAS file, including additions/deletions, reject symlinks/nonregular
files, impose a size bound and check repeated stable reads. This establishes unchanged
authoritative storage under the bounded local contract, not arbitrary-process purity.

The immutable receipt binds the admission/implementation, request, assessment, apply
value and storage evidence. Terminal refusal binds the exact receipt filename/digest.
Restart validates that evidence and never reapplies. Missing/altered diagnostic,
receipt, status or storage blocks restart. A refusal with changed storage remains
uncertain with diagnostic evidence retained. An interrupt after a possible commit
cannot become a refusal merely because later reads happen to show an unchanged root.
Historical missing acknowledgments are not retrospectively reclassified.

Assessment schema and normalization now require nonempty, unique selected support IDs,
matching Rust's existing admission condition. There is no invented support, relaxed
progress gate, edited proposal or silent filtering. Honest inability is explicit when
the frozen batch cannot satisfy this contract. Exact effect bounds remain authoritative
against unsupported optimistic forecasts.

## Decision relevance and limits

The actual failed run proposed an extra route and a nonsubstantive acknowledgment.
Neither improved the current measurement. Its .75 forecast for extra routing conflicted
with the existing per-challenge Boolean measure. The request previously described
`registered + routed + substantive response` without explicitly saying each is Boolean.
The contract now spells out at-least-one indicators and that extra routes or ordinary
acknowledgments do not increase them. This clarifies existing semantics; it does not
choose a winning action, change either parent desire or turn quantitative coverage into
social fulfillment.

The proposals also followed the current aspiration's procedural routing/acknowledgment
framing. That is an observed correspondence, not proof that guidance caused the choice;
earlier paired evidence did not establish reliable guidance benefit. The model omitted
an already-admissible substantive record. This repair makes misassessment/refusal honest
and inspectable, but does not establish that generation will now discover useful
possibilities or that relational aspirations improve behavior.

## Checks and review

- Real-controller/fake-provider fresh-process suite: 12 methods passed, including full
  cycles/restart, optimistic but exactly nonproductive forecasts, pre-apply stale
  refusal, empty/stale assessments, missing/corrupt refusal evidence, storage drift,
  lost acknowledgment after real commit, interruption before apply and after return,
  unchanged controls and metering. These are evaluator-authored fixtures, not live
  model competence evidence.
- Complete Python engine regression suite: 212 run, 202 passed, 10 skipped (36.477s).
  Restricted execution first had 11 localhost bind permission errors; the same offline
  suite passed with permission for local fake HTTP servers. No hosted calls occurred.
- After final terminal-receipt hardening, affected real-controller suite rerun:
  12 passed (29.983s). Diagnostic/support unit suite: 3 passed. Diff whitespace clean.
- Independent review found two restart evidence gaps; both were repaired and tested.
  Final review found no remaining scoped blocker. No Rust production code changed.

Logs: `target/controller-refusal-repair/`. Earlier failed development checks are retained;
initial refusal classifier tests exposed the actual CLI prefix/exit contract (exit 2,
`tsr engine:`), which was corrected without weakening tests or controller behavior.

## Next meaningful validation

A fresh separately admitted normal-driver continuation from the same actual persisted
archive/A1 would discriminate whether explicit Boolean effects and truthful support
requirements yield useful generated alternatives, honest inability, or a verified
no-progress refusal. At most three calls (possibility, assessment, return revision),
one existing reversible simulated operation, unchanged parents/A1/authority and existing
stage bounds. No authored substitute, cached favorable candidate or forced selection.
Success requires an actual useful selected operation and a revision grounded in its
return; a clean refusal is procedural correctness, not culmination fulfillment. No such
new admission or live execution occurs in this repair turn.
