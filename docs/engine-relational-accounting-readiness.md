# Relational resource accounting and pending external preparation

2026-10-22. No live calls, runtime, project execution, external transmission or
credential access in this work. The four benchmark results and no-culmination
conclusion remain unchanged. External approval is **pending**, not granted.

## Accounting correction

`engine-directions.py` now retains provider-reported `prompt_eval_count`,
`eval_count`, `total_duration`, `load_duration`, `prompt_eval_duration` and
`eval_duration` in body-free metadata and terminal journal evidence. Counts are tokens;
durations are nanoseconds. Booleans, negatives and nonintegers are invalid, not zero.
Each started request records an input32768/output2048 token reservation. A completed
response with valid counts reconciles to reported counts; missing, partial or invalid
usage keeps the reservation. Exact reported counts remain distinct from independent
metering. Zero external charge never means zero local compute.

Any known count exceeding its bound is terminal, even if another count is missing
or a duration is invalid. The resource-limit receipt prevents automatic continuation
on restart. No call is refunded. Transport loss retains its started reservation;
existing uncertain-generation reconciliation does not recreate consumed allowance.
Malformed/inability/relational responses retain whatever valid resource data exists.
Historical artifacts are not rewritten; the prior benchmark's token usage stays
unknown, because the discarded fields cannot be reconstructed honestly.

Independent review found and corrected overrun retry and incomplete-usage masking
before completion. Tests:78 driver cases, including serialized relational usage,
inability, invalid/missing counts, partial known overrun, terminal stop and restart;
three external-preparation tests. All use mocks and local data, no inference.

## Offline external payload, not a dispatcher

`python3 scripts/engine-relational-external-prepare.py --pair A` (or B) prints a
prepared payload plus blocking readiness report. It has no network, credential,
provider-call, accounting-file or adoption capability. Its only input is the
hash-locked `docs/engine-relational-external-inputs.json`: exact evaluator-authored
synthetic parent texts and fictional premise. It reconstructs an explicit allowlist
and includes no repository source, real user desire, private review, prior model
outcome, store identity or operational receipt. Unknown pairs and changed inputs
fail before export. Nothing is transmitted.

Target preparation: CheaperInference `deepseek-v4.1-flash`, at most4 calls,
2048 output tokens each, proposed aggregate ceiling$1 and .25 reservation per attempt.
This is a request-specific plan, not reuse of the old pilot authorization/ledger.
The payload is different from the local full-controller request; any eventual
comparison must disclose that difference rather than attribute all effects to model.

**Cap is not currently guaranteed.** Historical list-price ceilings (.30 input and
1.20 output dollars/million) are not a current quote. The conditional calculation
assumes input UTF-8 bytes plus8192 framing tokens and only ordinary input/output
billing. Provider-side reasoning or other billable units, exact framing and maximum
billable completion are unverified. The old coding broker defaults to a$2 reservation,
which would refuse a$1 ceiling; it cannot simply be used unchanged.

After explicit approval, a bounded metadata-only pricing preflight would still need
to establish every chargeable unit and a conservative maximum <=.25 per attempt.
A future dispatcher must reserve before dispatch in one shared locked four-attempt
ledger, retain timeout/unknown reservations, prohibit implicit retries and refuse
prices or units it cannot bound. A client ledger does not guarantee that a provider
will honor its advertised billing. If a provider-enforced cap or sufficiently complete
billing contract cannot be established, **do not dispatch** under a hard-cap claim.
No pricing preflight, credential check or external request has occurred here.

## Rubric fairness review, separate from outcomes

Independent review finds the rubric fair for an initial generated relationship,
not a complete formal definition of the user's TSR meaning:

- Non-additivity is interpretive. A conjunction can sometimes express an important
  new relationship; grammar alone cannot decide. The observed outputs lacked an
  explained interaction, not merely a preferred grammatical form.
- Honoring a parent does not require repeating every qualifier. Omission weakens
  demonstrated contribution but is not automatically replacement.
- A discriminator measures investigability, not emergence itself. Poor discriminators
  did not alone cause these outputs to fail the substantive relationship judgment.
- One-shot prose cannot demonstrate evolution through action and return.

No categories, thresholds or outcomes were changed after observing results. Preserve
both independent assessments and their minor classification differences. These
limits should inform future preregistration, never convert these attempts into proof
of culmination or proof of its impossibility.
