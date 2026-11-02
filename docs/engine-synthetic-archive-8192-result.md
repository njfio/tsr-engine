# A1 archive continuation with 8192 completion tokens

Fresh bounded admission on 2026-10-22: at most three requests to the same
CheaperInference `deepseek-v4.1-flash`, 8192 completion tokens including reasoning,
240 seconds per request and at most one local simulated event. Earlier truncated
call and expired slots remain unchanged. The previous request, state, model,
schema and criteria were compared before dispatch: only `max_tokens` changed.
The runner includes the independently reviewed frozen-proposal digest checks.

## Actual result

One possibility call completed with `finish_reason=stop`:

- 2,002 input tokens (1,792 reported cached), 1,693 completion tokens including
  1,519 reasoning, 3,695 total.
- Settled charge **$0.000789**; 9.384 seconds.
- Complete typed output contained two genuinely generated possibilities:
  register the `meaning` translation challenge, or record `access-review`.

Registration was admissible. Recording `access-review` was not: the corresponding
respondent is `night-reader`, and the current history has no route to that actor.
An available fictional response record does not replace this prerequisite.
The model was not asked to repair the output, and the valid alternative was not
selected by hand or used to excuse the invalid one.

Stopped before assessment. **No new action or revision; two unused calls expired.**
The complete fresh-process request equals the original request, including state
root `7ba7eda3615ab1a7c2e32588b2a9da3f38ccabf5eb58a220dd4ae7be286d17ea`.
The seed registration remains evaluator-authored setup, not generated behavior.
No temporary runtime, external actions, account changes or private-source exports.

Complete output was obtained under the higher allowance, but it used fewer than
2048 completion tokens this time. This does not isolate token allowance as the
cause or guarantee repeatability; provider generation and cache outcomes vary.
It is neither the earlier incomplete-output result nor explicit inability.

## Demonstrated boundary defect and repair

The Python archive boundary validated shape, identity, duplicates and withdrawal
citations but omitted registration, routing and withdrawn-state prerequisites.
Consequently the original `possibilities-validated.json` marker overstated its
checks. It is preserved alongside an appended sequence-rejection record. Rust's
transition validator already enforced these prerequisites; there was no execution
or authority bypass.

`engine-archive-contract.py` now checks every proposal independently against the
same current history: registration, matching decision revision, affected actor,
response citation and existing route, and withdrawal. One offered alternative
cannot supply another's prerequisite. The existing Rust gate remains authoritative.

Regression checks include response before route, route before registration, wrong
actor/challenge/response, withdrawn challenge, valid controls and offered alternatives
that must not become a sequential plan. Exact recorded output replay now rejects
before assessment and leaves the entire state unchanged.

`python3 -m unittest scripts/test_engine_archive.py scripts/test_engine_directions.py -q`:
**88 passed**. Existing mock HTTPError cleanup warning retained. No Rust code changed;
prior affected Rust result remains 207 passed / 5 ignored, not a newly run suite.
Independent review confirmed the semantic prerequisite violation and required
whole-offer preservation; the boundary fix received separate review.

Evidence: `target/synthetic-archive-live-8192/` includes the frozen admission/input,
exact allowlisted payload, started marker, metered receipt, typed proposals,
original shape-validation marker, appended sequence rejection, terminal closure,
regression log and `replay-invariants.json`. Prior `target/synthetic-archive-live/`
remains untouched.

## Next validation, not yet performed

The first proposal's rationale reflects contestability, but there is no selected
behavior, observed improvement or return-grounded relational revision to evaluate.
The three-stage behavioral endpoint remains open. Another attempt would require a
fresh admission; these two slots expired under the stop-on-invalid rule. A useful
next contract improvement would make the existing current-history prerequisite
relation explicit in the synthetic model input, without supplying a winner, dropping
invalid alternatives after generation or changing the desires. This turn did not
make another call or silently adjust that input.
