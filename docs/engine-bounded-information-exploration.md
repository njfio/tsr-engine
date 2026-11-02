# Explicit bounded information-exploration policy

The user approved allowing the engine to propose a small reversible exploration
when several directions plausibly serve its desires without supplied priority.
This local implementation does not authorize a paid run, activate a historical
state, invent a topic preference, or change action/spending limits.

The existing native inquiry path now supports `engine_exploration` origin. It uses
the same desire attention, independent review, possibility memory, source routing
and reader. There is no new selector or topic-specific branch. The engine proposes
its own question and record options; no preferred record is embedded in code.

A trusted local operator can append `exploration-policy` using the existing CLI:

```sh
target/debug/tsr-qualitative STATE exploration-policy POLICY.json
```

The policy object has current `state_root`, `enabled`, and `max_records` (1–4).
It defaults off when absent. This enables proposals under the policy, not execution.
Its event ID appears in `question-request` and is bound into inquiry context.
A model-origin exploratory inquiry declares:

- `origin: engine_exploration`, explicit generator attribution and uncertainty;
- `answer_source: corpus_assertions` with the exact configured corpus and rationale;
- `exploration` containing exact `policy` event ID, allowlisted `considered_records`,
  `possible_information_benefit` and `why_no_user_priority_needed`.

The existing independent review must assess plausible benefit, active desire
contribution, appropriate source and origin, and whether priority is actually a
user-only decision. Nonempty strings do not prove benefit. Unknown or rejection
leaves the inquiry unadmitted. A consequential/personal-preference/authority
question cannot become an exploratory corpus read merely by enabling this policy.
Source classification and origin appropriateness remain fallible semantic judgments.

Before a read, native code checks the current policy and inquiry under the existing
write lock. Only reviewed considered IDs may be read, within `max_records`. This
is a **per-read** bound, not a new cumulative budget. Existing campaign/admission,
reader byte/time/path limits and replay checks remain separate. Policy changes,
withdrawal or parent drift invalidate old exploratory guidance without deleting it.

Ordinary `model_proposed` questions remain available through the earlier reviewed
information path. Default-off blocks the explicit exploration path; it is not a
semantic classifier that detects every exploratory question mislabeled ordinary.
Reviewers must inspect the full inquiry and assess that distinction. No automatic
reader invocation, user message or provider export is introduced here.

## Local evidence

Authored controls cover policy-off, empty benefit, unknown benefit review,
user authority/preference exclusion, equally plausible alternative records,
per-read bound, second-record choice (no prescribed first winner), stale desire,
policy revocation, unchanged view, replay and reopened persistence. These prove
mechanisms, not model generation, improved decisions or culmination.

The previous abstention and all frozen live evidence remain unchanged. No current
historical store was activated; only temporary authored fixtures enabled policy.
Existing provider-export contracts remain frozen and need prospective preparation
before any new live validation.

## Exact next validation scope, not authorization

Using the existing six-stage reviewed path would require **six total Luna calls
and one bounded inspection** of the same fictional corpus: exploration question,
independent inquiry/source review, record choice, receipt-informed reflection,
independent reflection review, then preview-only next proposal. Use one record per
read for this prospective policy; retain abstention at every relevant stage. Both
reviews remain included. All source inputs remain non-private and frozen, with
only newly generated advisory state eligible for reviewed isolated persistence.

At campaign **196/200 calls, 57/60 groups, $0.162874/$5**, that complete envelope
needs a minimum call-cap change from **200 to 202**, plus fresh approval of the
six-call/one-read run. The dollar and operation caps need no increase. Same
CheaperInference gpt-6-luna, 8192 completion tokens, 240 seconds and 32 KiB per call;
$0.30 conservative allowance with fresh pricing and honest unknown reservations.
No call or inspection is authorized by this document. A shorter run would omit
part of that proof and must not be described as a completed six-stage loop.

Verification: 24 affected qualitative Rust tests passed. Independent source review
found one contradictory origin instruction; it was corrected and re-review found
no blocking issue. Tests are authored mechanism controls, not live model evidence.
The campaign remains unchanged at 196 calls, 57 groups and $0.162874.

Final verification: the pending CLI build completed successfully in 4m 10s; the
pending read-only listing also exited 0. No process termination or duplicate build
was needed. The cause of the delay was not established. Ten affected Python tests
passed against the resulting binary. A separate temporary-store fresh-process CLI
check passed policy enable/disable, stale replay rejection, persisted state and
unchanged authority, with no file inspection or provider call. No user action is
needed to recover the executor. Independent review remains passed.
