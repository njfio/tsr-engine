# Separate possibility composition and assessment

PRD §B8 describes Select → Tensions → Compose → typed candidates with support;
§B6/B7 and §B8 selection require forecasts before commitment. This change separates
the generative composition work from forecast/support form completion without
changing the shared controller, authority, named obligations or selection policy.
An intermediate proposal is not yet an admitted domain candidate or permission.

Use the existing local driver with `--two-stage`:

```sh
python3 scripts/engine-directions.py \
  --directory PATH_TO_RUN --model qwen2.5-coder:32b \
  --endpoint http://127.0.0.1:11435 --calls EXISTING_INITIAL_CALL_LIMIT \
  --two-stage --stop-file PATH_TO_OPERATOR_STOP
```

An existing run must retain its frozen model/endpoint/initial call configuration.
Previously granted named batches continue to apply. Both stages consume the same
started-call allowance, each with the existing 240-second timeout. No additional
calls are implied by enabling the flag. The legacy combined path remains available
for runs without a pending two-stage proposal.

## Stage boundaries

1. **Compose.** Generate two distinct bounded action possibilities (or the existing
   single-possibility edge case), with purpose and optional dependent continuation.
   No support IDs or forecasts are accepted here. Existing action scope, dependency,
   novelty and inability controls remain. `possibilities-N.json` preserves raw
   output; validation and a current-state check precede `possibilities_recorded`.
2. **Assess.** On the next separately budgeted call, the same installed model sees
   the immutable proposal and supplies only support and first/terminal forecasts.
   Wire digests bind the full proposal and each member. They are SHA-256 artifact
   references, not replacements for the domain's content-addressed obligation IDs.
   The response cannot contain replacement actions. Runtime joins predictions to
   the original actions in original order and rejects missing, duplicate, unknown,
   stale or changed bindings. Existing normalization and Rust validation then run.
3. **Return or reconsider.** Rust selects and executes only after complete valid
   assessment. Assessment syntax/forecast errors retain the frozen proposal for
   correction. Recognized controller selection refusals retire it with an audited
   `possibilities_rejected` event, preserving the failed offer and returning to
   composition. This avoids indefinitely reassessing the same refused actions.

An unchanged pending proposal survives restart by its receipt hash. Changed state
retires it as stale before composing again. An unresolved started request still
blocks automatic retry and uses the existing explicit unknown-abandonment process;
unknown generation is never treated as a failed or applied action. A pending
proposal cannot silently switch back to combined mode. One remaining slot may
freeze a proposal and exit `assessment_pending`; it cannot spend a second slot.

Either stage can report inability. Assessment inability must bind both state and
proposal. It stops without changing wants or claiming objective impossibility.
Operator stop controls still prevent dispatch or application. Candidate source
text remains untrusted and cannot grant authority.

## Verification

`python3 -m unittest discover -s scripts -p test_engine_directions.py`: 50 passing
tests. Added controls cover action preservation under assessment reordering,
changed/stale/duplicate identity, unknown support, contradictory subset forecasts,
meaningful alternative identity, interrupted assessment, frozen-response tampering,
one-slot pending behavior, and mocked full two-stage submission. A mocked
composition → assessment → controller abstention → fresh composition regression
covers the integration defect caught by independent review. These deterministic
fixtures are not live model-generation evidence. Python compilation and diff
whitespace checks also pass; independent read-only review found no remaining
blocker after the correction.

The original combined request 18 used a 4,980-byte system message and a 4,562-byte
schema. The separate composition request 19 uses 906 and 1,911 bytes respectively.
Typed desires, lineage, observations, history, uncertainty and residual context
remain; the reduction removes simultaneous forecast/support output requirements,
not original goals. Model and sampling settings remain unchanged.

Unknown abandoned request 13 and inability request 18 remain preserved. Corpus
accounting remains 151 qualified tasks; mature independent eligible episode count
is unknown. Notebook measurements are coverage/brevity, not semantic truth.

### Live result and remaining verification

Request 19 was the final slot of the existing ten-call batch. It returned
`generation_inability` in phase `possibilities`; no possibility set was frozen,
no assessment request was dispatched, and no controller action was applied.
The response is preserved in `target/support-live/run/inability-19.json`, SHA-256
`60313ef83dbf205bf7ea27d8ea52952fa30347aee13f8922b2709ab781550e93`.
The journal retains the original state root and unmet wants. The temporary
provider PID 66111 was terminated and its listener checked absent.

All ten batch slots are now dispatched, including abandoned unknown 13. This live
result verifies the separated-stage inability path, not useful composition or a
full live two-stage cycle. Separation alone did not resolve proposal generation.
A complete future live cycle needs at least two newly authorized calls (composition
and assessment), and additional correction would need its own bounded allowance.
Repeating this identical deterministic request is not justified by the result;
further work should first address generator search/diversity while preserving
immutable assessment and controller checks. No additional inference was dispatched.
