# B9.1 Trial 4 — bare relational payload

2026-10-24, user-approved trial on verified `48d1c44`. Fresh admission
`41dda18ca2a7df655a99c181b86d89bd3e2cd9507a992f19d30d0906d5f4e6c0`
was independently checked: six calls/four groups, synthetic-only, one hour ending
13:08:47 UTC, 8192 completion tokens/240 seconds per call, 240000ms aggregate tools.
The installed pinned worker image was present; no download occurred. A fresh Python
process confirmed the updated Homebrew bundle excluded emSign ECC Root CA - G3 with
certificate/hostname checks enabled. Read-only official provider metadata confirmed
the exact approved model. No settings or credential changes occurred in this trial.

## Actual result

The normal driver ran one evaluator-authored seed execution group. Expected failure
occurred: empty-input check passed, existing-behavior check failed. Both original
desires remain represented and open, semantic completeness unresolved. The frozen
external evaluator's 0.1 forecast precedes the exact action/return; Surprise and
Contrast are separately recorded. This is not learned calibration.

The first model response at `relation_after_seed` completed normally (`stop`) with
six root fields: parents, relationship, aspiration, discriminator, revises (null),
and catalysts (empty). It omitted the mandatory outer `relational` envelope.
The early validator correctly rejected it as `missing_or_ambiguous_envelope`.
Absence of development is permitted for an initial aspiration; this is **not**
Trial 3's misplaced-development fault. No automatic wrapping or repair occurred.

No generated proposal was admitted. There was no second-aim experiment, generated
boundary patch, unchanged-experiment re-verification, revision, or final proposal.
**B9.1 remains incomplete.** Relationship/aspiration text is deliberately not retained
in structural quarantine, so independent review cannot judge its semantic quality
or infer causal benefit from the field names. The changed lineage instructions were
not validated end-to-end by this failed response.

## Accounting, terminal state and replay

- One call, request `fd8fdd93-054d-4b1c-915b-3c31b52b5c15`.
- 1232 prompt + 689 completion = **1921 tokens**, with 488 reasoning tokens included
  in completion; transport elapsed 4.800 seconds.
- **$0.000448 settled**; no unknown cost/usage requests or unresolved effects.
- One seed group, **14772ms** tool charge; 191 evaluator-authored patch bytes.
- Five unused calls/three groups expired; STOP prevents continuation. Native episode
  remains open/unaccepted; no fabricated closure, maturity or human-review effort.
- Two fresh-process read-only inspections agree: 27 events, 13 completed effects,
  no unresolved effects, dispatch disabled. Journal/quarantine hashes did not change.
- Earlier trial journal/STOP/quarantine hashes match the captured pretrial values.
- Owned driver process terminated; no persistent service or live continuation.

Evidence is under `target/b91-live-trial4/`: driver/provider journals, structural
quarantine, terminal status/request/budget/evaluator-forecast/summary, STOP and replay
check. Admission, launch log and prior-hash inventory are sibling files.

## Offline interface hardening

Independent review found no logically contradictory instruction: the old system text
already mentioned the envelope. It did find a presentation weakness: three generic
pseudo-JSON alternatives were followed by detailed inner-object instructions, without
an explicit per-stage top-level schema. This is a plausible source of ambiguity, not
proof of why this model omitted the wrapper.

The offline correction adds `response_contract.envelope_schema` and explicit JSON paths
for every stage. A single mapping supplies both that schema and the existing validator's
stage-kind checks. Initial/revision contracts still describe the payload at the named
path. Final proposal-only retains its three allowed branches and cannot execute.
The schema describes only the envelope; inner contracts/native validation remain
mandatory. No provider structured-output feature is assumed, no canned aspiration is
provided, and no failed output is retroactively admitted. There is no claim that this
clarification guarantees future compliance. A fresh live trial requires new approval.

Offline verification of the clarification:

- Focused wire/envelope/lineage/diagnostic suites: **31 passed** (20.364 seconds).
- Explicit cfg(test) native six-stage fixture: **passed** (63.972 seconds), including
  actual Rust/Python initial/revision validation and terminal replay. Six authored
  fake provider replies/four fake worker-return groups, zero target commands.
- Independent read-only code review: no blocking findings. Native semantic/admission
  checks stay required; this is not a provider-enforced schema or live success.
- `git diff --check` passed. Trial 1–4 terminal journal/quarantine hashes remain unchanged.
- Full offline Python discovery: **591 total, 574 passed, 17 opt-in skips**, zero
  failures/errors, 126.320 seconds (`target/b91-trial4-offline-python.log`).
  No Rust production code changed in this follow-up; no new full Rust sweep was run.
