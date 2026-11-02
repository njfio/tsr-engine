# Offline proposal contract audit

The v1 schema and decoder disagree on inactive strings, forecast bounds/nulls, and edit preconditions. The new `scripts/m3-edit-v2-broker.py` makes those expressible constraints explicit and adds content-free validation diagnostics. It preserves accepted actions and normalized proposals; it does not relax the decoder. No provider calls, retries, candidate execution, deletion, push, or publication occurred for this audit.

## Historical failure remains unresolved

The frozen pilot received one `propose` function call with a valid JSON object and failed with `proposal_type`. In v1 that code covers non-string string fields, invalid action names, and nonempty inactive fields. The exact field cannot be recovered: the response body was intentionally not retained. Missing/extra fields use `proposal_fields`; null forecasts on test/edit use `forecast_missing`. Those demonstrated gaps must not be represented as the known cause of this historical failure.

The prior result remains unavailable, rules unattempted, zero completed pairs and no M3 acceptance. Historical aggregate settled cost remains $0.229744 with $0 outstanding; this offline work adds no charges.

## Exact contract

All seven fields are required for every variant: `action`, `path`, `id`, `expected_digest`, `old_text`, `new_text`, `forecast`. There are no optional fields. Every non-forecast field is a string, never null. Inactive strings must be exactly empty strings.

| Action | Active strings | Forecast |
| --- | --- | --- |
| inspect | exact allowed path | null or finite number in [0,1] |
| test | exact named test ID | finite number in [0,1] |
| edit | exact allowed path, lowercase 64-hex digest, nonempty old span, replacement span | finite number in [0,1] |
| stop | none | null or finite number in [0,1] |

An empty replacement is legal deletion. Unicode spans are legal. The replacement must differ from the old span. The schema uses action-specific `oneOf` branches for inactive strings, path/test scope, digest shape, nonempty old span, and required numeric test/edit forecasts. Explicit instructions and four shape examples state the same rules. Examples are labeled inert placeholders, requiring inspected source/digest before an edit.

Cross-field old/new inequality remains a decoder check. Serialized byte limits, current source digest, unique span match, and other engine preconditions remain runtime checks; JSON Schema does not establish these properties.

## Diagnostics and integration boundary

`ValidationFailure` reports only a fixed schema field path, expected JSON type, actual JSON type, and finite reason enum. Missing values use the `missing` type. Unknown and duplicate keys use `/arguments/*`, never the provider's key name. Parser exception messages and argument values are excluded. The wrapper adds this detail to the existing content-free transport diagnostic after the existing settlement logic; it introduces no retry.

For example, a null inspect path yields `{ "version": 1, "path": "/arguments/path", "expected_type": "string", "actual_type": "null", "reason": "type" }`.

This new adapter is intentionally not wired into any frozen controller. Actual provider/model support for `oneOf` is unverified, and no compatibility claim follows from local tests. Any future live use needs a separately reviewed baseline and resolution of the execution pause; the previous frozen launch must not be replayed as this version.

## Validation performed

- `python3 -m unittest discover -s scripts -p test_m3_edit_v2.py -v`: 6 tests passed. A 266-case matrix compares both acceptance and normalized output with the frozen decoder and independently checks emitted schema constraints. The same 266 cases run through the actual transport's fake-provider request, reservation, settlement, decode, and diagnostic path. HTTP, credential lookup, and budget availability are stubbed; temporary real ledger writes are used. This is not a live provider test or a Rust evaluator end-to-end run.
- Tests cover every action, all seven missing-field cases per action, null/type substitutions, inactive nonempty strings, unknown keys, forecast boundaries, digest case/length, legal deletion and Unicode edits. Additional checks cover examples, no-op edits, NaN, malformed/duplicate/deep JSON, malformed envelopes/usage, and exact diagnostic types/paths. Sentinel keys, values, and credentials must be absent from diagnostics.
- The independent schema evaluator covers the exact emitted keyword subset. A third-party JSON Schema implementation was not available; these tests do not certify vendor schema support.
- `python3 -m unittest discover -s scripts -p 'test_m3*broker.py' -v`: 20 existing broker tests passed.
- Independent read-only review by `/root/review` found no blocking defect in schema, acceptance parity, diagnostics, or settlement wrapping. Suggested deletion, Unicode, forecast and envelope/usage cases were included in final tests.
- SHA-256 preservation checks passed for all 535 frozen inputs, all 91 exact-live export files, all 106 prior continuation export files, and all 68 progress-validation export files. See `m3-proposal-contract-preservation.json` and the two new test logs under `m3-validation/`.

M3 operational acceptance remains unmet. This audit fixes demonstrated contract mismatches offline; it neither proves the historical failure's cause nor supplies a successful real-corpus paired outcome.
