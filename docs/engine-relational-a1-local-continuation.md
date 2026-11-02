# A1 local continuation: stage mismatch, preserved uncertainty

2026-10-22. The approved three-call, one-action continuation did **not** reach
possibility generation. The first normal-driver request unexpectedly selected
mechanical refinement. This is an integration failure, not evidence that Qwen32B
cannot act on the aspiration. No generated candidate, notebook action or relational
revision resulted.

## Frozen scope and terminal accounting

- A fresh SQLite backup of the original A1 synthetic import preserved its root
  `0edcee1cb98bdd5fc5c5673a24ce4ca14fd4745ccb8ec2acb0b12747a533a172`,
  both parent desires, the genuine external aspiration, and its origin hashes.
- Installed `qwen2.5-coder:32b`, digest
  `4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`;
  temperature 0, context 32768, maximum output 2048, timeout 240 seconds.
- One request dispatched: envelope
  `2183c3a504005db0a28bed502b257693b5954de1ee024d1c9b917699bc6d0332`,
  phase `refinement`. On detecting the mismatch, the owned driver was interrupted
  before a response was recorded or applied (exit 130). No automatic next call ran.
- Provider generation/usage remains **unknown**. The runtime logged HTTP 500 after
  cancellation, but the client received no response or token counts. The full
  reservation and consumed slot remain; cancellation is not a refund or proof of
  zero resource use. External charge was zero.
- Reconciliation verified the exact unchanged controller root and absence of
  application receipts. Two unused calls were explicitly expired. The owned runtime
  exited 0, its loopback port closed, and no matching driver/runtime remained.

Evidence: `target/relational-a1-local-live/` contains admission, original/after state,
request journal/envelope, stderr, reconciliation, expiration and terminal summary.
Original benchmark fixtures and previous failures remain unchanged.

## Diagnosed and corrected priority

The imported state simultaneously had `refinement_needed=true` and current applicable
`relational_guidance`. The driver let the former preempt the latter, asking for
single-parent decomposition before the newly generated relationship could inform
possibilities. The earlier mocked integration did not exercise both flags together.

Operational decomposition is optional; it is not action permission or a safety gate.
The driver now defers it while current relational guidance has an opportunity to
inform a direction. Frozen possibilities proceed to assessment. Due relational
reconsideration retains priority without first constructing an irrelevant refinement
schema. No controller flag is cleared and no fictitious refinement event is recorded.
Without applicable guidance, pending assessment or due relational review, the existing
decomposition behavior remains. Repeated renewed guidance can keep decomposition
deferred; this is an explicit scheduling choice, not guaranteed eventual refinement.

Verification: **82 Python driver tests passed**. The normal relational-to-possibility
test now includes the previously missed refinement flag. Additional controls cover
frozen assessment, absence/removal of guidance, due relational review and unchanged
state. A separate offline mock using the **exact preserved live input** now selects
`possibilities` and includes the unchanged aspiration in context; it makes no model
call and mutates no controller state (`offline-stage-check.json`). It supplies only
an explicitly labeled mock inability to terminate the test, not a candidate answer.

Independent review confirmed the priority defect and required these controls.
This correction does not establish live generation or semantic progress.

## Remaining validation

A new bounded admission of up to three local calls is needed for the intended
possibility → assessment/action → return-bound relational revision sequence. Do not
reuse or refund this expired admission. Use a fresh snapshot of the unchanged A1
input and the corrected driver. No changes to parent desires, predicates, authority,
model or constraints are needed. The live plan and semantic limits remain those in
[the return-cycle document](engine-relational-return-cycle.md).

The meaningful question is still whether a genuinely generated direction reflects
the relationship and whether its revision incorporates the actual limited return.
Notebook inclusion cannot establish social accountability. No provider-capability
conclusion or relational-fulfillment claim follows from this interrupted attempt.
