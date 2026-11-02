# Automatic browser-session provider recovery

This prospective repair closes the manual preparation gap exposed by the two failed independent reviews. Historical closed admissions and failed requests are unchanged. Recovery is disabled unless a fresh browser admission explicitly supplies `transient_recovery` with version `exact-failed-zero-recovery@v1`, a finite retry allowance (1–10), and a 60–600 second backoff. Persistent feedback-session authority is also required. Retries consume ordinary session/campaign call and dollar budgets; they add no capacity and never extend the deadline.

## Implemented path

`engine-browser-transient.py` wraps the existing browser adapter. Only a received HTTP503 with one exact request identity, no model value, and an authenticated singleton provider record explicitly reporting failed status, zero tokens and zero charge qualifies. The existing official lookup/validator is reused. Missing, contradictory or charged records remain in the ordinary uncertain-effect stop path. The original HTTP receipt, lookup, exact wire request, original task digest and native source identity are retained before ordinary campaign/native settlement. This avoids converting an unknown charge into zero by assumption.

The ordinary `checked()` loop then creates a durable retry ticket and waits until its recorded backoff expires. A fresh request uses the same task and exact wire payload, including any previously admitted format-correction instruction. Every new request still passes ordinary pricing, reservation, cap, deadline, source and review checks. Semantic rejection is not eligible. Exhausted retry authority stops; unused format corrections cannot substitute for provider retries.

The cycle persists callback inputs before proposals and independent reviews. After interruption during a proven-settled retry wait, it checks the adapter's exact eligible ticket and resumes only that callback. A retained proposal is not regenerated. Completed stages reuse their recorded results through the original admission/stop/pending checks; changed accounting context does not turn a completed review into a new request. Existing launch/run locks prevent concurrent recovery. Empty pre-dispatch quarantine directories may be reused only after ticket verification; nonempty directories and symlinks are refused.

Crashes leaving dispatch or settlement intents unresolved still stop explicitly. The controller does not infer whether a request happened, poll indefinitely, replay unknown effects, reopen a closed historical grant, or manufacture an independent judgment. This is bounded recovery at a specifically authenticated failure boundary, not universal outage recovery.

## Verification

All new provider failures and judgments in tests are explicitly authored fixtures, not live model evidence. The focused suite covers exact proof, unknown/nonzero billing, semantic rejection, absent/invalid policy, changed request/source, budget/deadline exhaustion, persistent backoff, corrected-wire restart, duplicate launch and inert repeated recovery. The composed native test traverses actual adapter journals, native reservations/settlements and cycle receipts: malformed review → counted correction → confirmed-zero503 → interruption during backoff → fresh controller → exact retained review → ordinary selection. The proposal is unchanged, four calls are counted, and no execution group is used.

Independent source review by `handoff_audit` found and verified fixes for corrected-wire persistence and pre-dispatch quarantine handling. Extended testing also caught completed-stage callback input drift; completed-stage reuse was corrected and independently reviewed. Earlier failed test logs are preserved. One full-cycle test under the default tool sandbox lacked browser observations because Docker socket access was denied; the runtime was not weakened. Its rerun uses the already approved pinned isolated browser runtime via normal execution approval.

Frozen verification results:

- `python3 -m unittest discover -s scripts -p test_engine_browser_transient.py -v`: **7 passed**, 0.010s, no skips (`target/browser-transient-unit-final.log`).
- `TSR_BROWSER_ADAPTER_AUTHORED=1 python3 -m unittest discover -s scripts -p test_engine_browser_transient_native.py -v`: **1 passed**, 45.051s, including ordinary next selection after recovered review (`target/browser-transient-next-choice-fixed.log`).
- `PYTHONPATH=scripts TSR_BROWSER_ADAPTER_AUTHORED=1 TSR_BROWSER_SEALED_AUTHORED=1 python3 -m unittest test_engine_feedback_cycle.FeedbackCycle.test_return_review_rejection_choice_execution_reflection_without_bridge -v`: **1 passed**, 228.533s, via the approved isolated runtime (`target/browser-transient-full-cycle-authorized.log`). This complete authored cycle includes rejection, fresh proposal, execution, receipt-informed reflection and inert restart. The transient injection is covered separately by the composed test; this full cycle is compatibility evidence, not a live outage sample.
- Python compilation and `git diff --check` passed. No Rust changes or new dependencies.

Preserved failed/local diagnostic logs include `browser-transient-composed.log`, `browser-transient-next-choice.log` (completed-review context drift before fix), and `browser-transient-full-cycle.log` (default sandbox Docker denial). An accidentally discovered imported test suite was stopped by its identified local PID; the focused suite was then selected explicitly. No unknown provider effects were involved. At 16:14 UTC all final tests had completed; no live request or execution was pending.

## Live evidence and limits

No third provider request was made in this repair. The two preceding live reviews returned503 and were independently reconciled to zero charge; this does not establish present provider availability. Automatic transient recovery has local evidence only. The original validation deadline remains `2026-10-29T16:26:50.881922Z`; no admission was reset or retrofitted.

Campaign accounting remains **307 calls / 83 operations / $0.314198**, with no campaign unknowns. The combined preceding validation used five calls, no operations, and $0.005378. It still has **zero complete live action-return cycles**. Previous manual provider-recovery preparation and callback repairs remain disclosed interventions. Neither these tests nor automatic retry implementation demonstrate new desires, sustained TSR learning or comparative advantage.
