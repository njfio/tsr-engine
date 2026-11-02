# Independent review: integrated synthetic campaign 1

Reviewed 2026-10-26 by `/root/b91_full_review`, distinct from Luna generation and synthetic evaluator authorship. Scope: Trials 12–16, frozen B9.1 criteria. This is a partial result, **not a completed or accepted B9.1 run**.

## Observed progress and failures

| Trial | Actual reached stage | Terminal limitation |
|---|---|---|
| 12 | Seed failure, generated guidance and experiment, real experiment return | HTTP error before return revision |
| 13 | Real seed failure | Initial output rejected for duplicate JSON key; no adoption |
| 14 | Seed failure, generated guidance and experiment, real experiment return | HTTP error before return revision |
| 15 | Above plus independently reviewed return-grounded relational revision | HTTP 503 before any generated boundary patch |
| 16 | Real seed failure | Initial generation HTTP 503; no proposal |

Driver/provider journals record 12 dispatched requests and eight completed execution groups across these trials. Failures remain part of the record. Trial12/14/15 billing failures were separately reconciled append-only; original provider journals still retain their initial unknown outcomes. The reconciled status through Trial15 records 11 calls, seven groups and $0.004991 known cost. Trial16 reconciliation was pending at this review; no zero-cost inference is made from absent usage.

All three generated experiments had exact declared/native Rust test identities, distinct from protected tests, and executable assertions relevant to the second aim. Their real results passed empty-input cases and failed nonempty-preservation cases. The observations establish named-case outcomes, not every assertion's execution, actual returned values, or universal coverage. Early failure may prevent subsequent assertions. Source-based predictions of empty-field dropping are inference, not observed assertion traces.

Trial15's revision explicitly incorporated protected and supplemental pass/fail observations while preserving the original meaning. It distinguished source prediction from observation and left actual values, failing assertion, and universal coverage unknown. That is a grounded update to guidance. It did not invent a successful repair or declare either parent fulfilled.

## Semantic qualification and incomplete endpoint

Initial aspirations consistently expressed an exactly-empty exception alongside nonempty preservation. They are coherent, authority-preserving relationships, but predominantly conjunction/restatement of the requirements. These outputs do not establish the user's richer non-additive culmination goal.

Both original desires remain open after expected failure. A generated experiment for the second aim and observed return are demonstrated. No trial produced a boundary patch, its passing receipt, unchanged-experiment re-verification, independent final acceptance, or final next proposal. No complete cycle, improved next choice, universal preservation, calibrated competence, or general autonomy is established.

## Human/agent intervention boundary

Seven independent review files were authored: two each for Trials12 and14, three for Trial15. Main manually resumed the normal driver at these review gates, reconciled exact failed requests, and admitted fresh trial identities after terminal failures. The reviewer supplied no candidate, test rewrite, repair, provider call, or execution. This is operator-assisted orchestration, not unattended completion. No prior output was adopted into a fresh trial by this review.

## Evidence and recommendation

- `target/b91-integrated-trial{12,13,14,15,16}/driver.jsonl` and `provider.jsonl` record actual stages, effects and failures.
- Corresponding `*-reviews/*.request.json` and `*.review.json` bind independent judgments to source/state and subject hashes.
- Trial15 revision request `7bb97370115029548ce823cf9ef3c498bd247fd1338ccda9c13efd64004bfccc.request.json` contains the grounded revision and exact return basis.
- `target/tsr-integrated-campaign-1/reconciled-status-trial15.json` supports the settled accounting boundary above.

Stop redundant live attempts while the actual provider availability failure persists; finish exact Trial16 reconciliation without replaying its request. After recovery and within remaining authorization, a fresh trial can test the still-missing patch→verification→independent acceptance→next-proposal sequence. Do not relax frozen criteria or count another partial prefix as completion. These records do not by themselves justify a controller repair or a claim that relational guidance improves selection.
