# Contrast-derived preference: native ledger integration

2026-10-31. Offline follow-up to `5553638b`; no paid evaluation or change to its
frozen packet. Sources are the checked-in PRD, implementation and recorded author
clarifications, not an independently inspected original book.

## Audit finding

The first bridge created an actual canonical `tsr_domain::Preference` and a native
Generator `Expanded` event. It did **not** use `Ledger::expand`. Its Council
`GeneratedPreferenceAdmission` was replayed by the project fold; that made a node
available for relational composition, but not in the generic active ledger.
Short generated nodes remained inactive there and oversized nodes were omitted.
Calling that complete general new-preference integration would have been wrong.

PRD B2.1 (line 177), B6 and B8 (line 403) require generated wanting to wait for
review, retain lineage and feed future generation as guidance. They do not require
Operator impersonation. The generic ledger's Operator-only `StatusSet(Open)` rule
was an implementation limitation, not a required philosophical distinction.

## Prospective integration

- `PreferenceAdmissionReview` and `ReviewedPreferenceAdmission` are additive native
  domain records. They bind one exact expansion, preference, Contrast and originating
  return; proposal, source snapshot, requirements, request and response bindings;
  actual reviewer/generator identities and approval decision.
- `Ledger::propose_generated` stages the canonical Generator expansion from known
  nonempty same-task Contrast. This additive API supports the reviewed project's
  existing text capacity. Legacy `Ledger::expand` remains unchanged.
- The controller authenticates independent review through the existing cycle. It
  stores the exact typed attestation in CAS and appends a Council evidence observation
  with its digest. `Ledger::admit_reviewed` checks the current head and exact prior
  evidence before appending the Council admission. All project writes occur inside
  the existing native transaction.
- Writer and generic fold share a prefix-only validator. Foreign/future references,
  unknown/empty or altered Contrast, mismatched evidence, nonapproval and identical
  reviewer/generator identity are rejected. This validates controller-attested
  evidence association; it does not authenticate a provider from prose or make a
  reviewer correct merely because a name differs. Transport authentication and
  current-source applicability remain the controller's responsibility.
- The fold reconstructs and opens the reviewed Generator node with Council history.
  It does not rewrite `StatusSet(Open)` semantics or broadly admit old oversized
  Operator records. Conflicting existing identity/provenance is rejected. Invalid
  raw admissions do not poison later valid ones; a successfully applied admission
  cannot be replayed to reopen a subsequently closed node.
- The project fold now requires actual native ledger eligibility as well as its
  current contributor checks. Open or Fulfilled contributors can participate in
  composition; unfinished-preference selection stays Open-only. A separate bounded
  source-bound status projection exposes recorded ledger status without confusing
  it with unknown semantic fulfillment. Withdrawn/out-of-scope/superseded/overlapping
  nodes and descendants remain unavailable, with lineage retained. It no longer independently grants eligibility from
  a project guidance record. Historical bridge-only records are not retroactively
  upgraded; they remain preserved and ineligible. This patch does not implement a
  migration for legacy bridge-only nodes. No live bridge nodes were created before
  this repair; the prior bridge evidence was authored offline testing.

## What survives and what it means

Both expansion and reviewed admission persist in the same append-only native store.
Reopening that store reconstructs the same `CollectiveReality`, active view and
Council status history. Generic selection can consume the generated preference when
its existing Tune admits Generator provenance. The integration does not change Tune
or other source permissions to force selection.

A new session over that same authenticated store sees the native identity. Browser
projection additionally requires journal ownership or exact frozen public import
pins, so a random fresh session cannot export unrelated stored content. A blank
project is not a global-memory import, and comparative forks remain isolated.

Original acceptance aims, requirements, permissions, recorded meanings and all
contributing parent history remain unchanged. Generated wanting has native identity,
lineage and eligibility; it is not a new user command. No new fulfillment predicate,
minimum obligation, action route or permission is invented. Browser executable
attention still requires an authorized goal/action contract; generated contributors
influence later relational composition and pursuit through that contract. Being
open in the ledger is not proof of fulfillment, player preference or autonomous
emergence. Successful combination without Contrast remains a separate unresolved
extension.

## Verification

The broad native project suite passed 180 checks, with three existing ignored
cases (`ledger-project-conclusive.log`, 250.23 seconds). The domain/ledger suite
passed 55 checks, including eleven new reviewed-admission regressions
(`ledger-domain-conclusive.log`). The prior adapter pass had two authored successes
(`ledger-adapter-conclusive.log`, 235.379 seconds).

After the final contributor-status correction, all seven affected native bridge
checks passed again (`ledger-status-native.log`, 33.08 seconds), including
fulfilled contributors, descendants, exclusion statuses and restart. These repeats
are not additional distinct checks. Domain and ledger Clippy checks passed with
`-D warnings` (`ledger-clippy.log`). CLI Clippy still reports the same four errors
in unchanged qualitative modules (`ledger-cli-clippy.log`); full clean lint is not
claimed. `git diff --check` passed.

The initial integration test failure was a linear-event-chain mismatch, corrected
without weakening reference validation. A development test compile failed because
it used nonexistent `Hash::ZERO`; the fixture now uses `Hash([0; 32])`. Both failures
are retained in development logs. Independent source review caught and verified
corrections for origin collisions, invalid admission poisoning, untrusted Intuition
returns, future-reference ordering and fulfilled-contributor composition. The
controller's ledger events use actual Unix-millisecond timestamps; causal order is
still the event sequence, not the clock.

Exact accepted code/binary pins are `ledger-accepted-pins-v3.json`; source review
records are `ledger-independent-review.json`. The final status negative test initially confused the cycle-view and full-request
interfaces, causing two fixture errors before the target guard. Both are preserved
in `ledger-status-adapter.log` and `ledger-status-adapter-followup.log`. The corrected
test uses the real `web.project(request, public)` export boundary with a passing
positive control and an exact rejection assertion. Runtime validation was not changed
to accommodate the fixture. The corrected adapter case passed in 85.776 seconds
(`ledger-status-projection-final.log`); the other final recovery case had already
passed. Across the broad and final affected verification, 237 distinct tests passed
(180 project + 55 domain/ledger + 2 adapter), three were ignored. Repeats are not
counted again. No live spontaneous preference formation is claimed. Development
failures and independent-review findings are retained separately in
`target/contrast-preference-repair/`. No fixture output is live TSR evidence.


Campaign ledger and denied manifest remained byte-identical throughout. Audited
accounting is unchanged: 395 calls / 129 groups / $0.644668 known, plus the retained
historical $0.05 uncertain reservation estimate (not a verified maximum). No paid
launch was retried, no frozen admission changed, and no game artifact was edited.
All verification used authored local fixtures. The paid evaluation remains blocked
pending the separate requested authorization.

Git's existing hook reports `lefthook` unavailable in PATH; explicit recorded test
and lint commands, not an executed hook, are the verification evidence. No dependency
was installed. Existing unrelated untracked artifacts were preserved.
