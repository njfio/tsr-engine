# Precise native correction diagnostics — local repair

This prospectively repairs the feedback weakness observed in live4. Its failed
verdict, rejected responses, closed grant and campaign accounting remain unchanged.
No model request or campaign inspection ran during this repair.

## Native source of truth

`crates/tsr-cli/src/engine/qualitative/contract.rs` now emits bounded structured
`native-shape-diagnostic@v1` errors for unsupported/missing fields, object/array/text
types and length bounds. The same native `Shape` both validates and supplies the
expected fields. For all three retained live4 outputs it reports:

- actual invalid path `$.protocol`;
- valid placement `$.response.protocol`;
- allowed/required root fields `basis`, `generator`, `response`.

The `$` denotes the complete output object, avoiding confusion with its nested
`response` member. `stage` has no valid output placement. Unknown model field names
are withheld and identified by ordinal at the containing object; response values
are never echoed. Known native field names may be located in several union branches;
locations describe structure, not which meaning the model should choose.

Identity, lineage, provenance and semantic validation remain unchanged. Ambiguous
union failures remain terminal for automatic correction rather than guessing an
intended variant. Their native diagnostics retain bounded branch details locally.
This does not make every validation failure retryable.

## Producer and correction boundary

`engine-exploration-export.py` derives an explicit root/nested output layout from
the native contract for all six stages. It distinguishes request metadata from
output shape without providing candidate content or a semantic pathway.
`engine-bounded-correction.py` validates the diagnostic version, permitted codes,
paths, field lists, scalar types and bounds, then includes it in the fresh correction
message. Full original task/context/schema remains unchanged. No JSON coercion,
field deletion, identifier repair or model-output rewrite occurs.

Existing default-zero, explicitly admitted shared correction limits, counting,
settled-charge eligibility and fail-closed restart rules remain. Unknown effects,
semantic rejection and stale identities cannot trigger format correction. JSON-object
mode remains syntax guidance, not provider-enforced native schema.

## Evidence and tests

`scripts/testdata/qualitative-live4-rejections.json` contains the three exact safe
retained content strings with source labels and hashes. They contain only the frozen
fictional corpus interpretation, no wrappers, reasoning or credentials. They remain
rejected regression inputs, never admitted outcomes or valid-answer templates.

Tests use these actual failures against the native validator, then inspect serialized
correction feedback and verify unchanged inputs. All six stages are checked for
explicit unsupported-field paths, allowed fields and privacy exclusions. A separate
**authored** fake-provider flow injects the same reflection-envelope defect through
the real serializer/strict transport/native validator, then supplies a fresh valid
fixture response; it completes in seven counted requests and one temporary-fixture
read, with the rejection retained and no-repeat replay. This is deterministic wiring
evidence, not live recovery or evidence that model meaning is stable across retries.

Independent review passed; its minor scalar diagnostic validation concern was also
addressed with negative tests. No protected acceptance criteria or original desire
meaning changed. Current campaign remains **205/220 calls, 58/60 groups, $0.167815/$5**,
with the older Garden unknown separate. Another live test would require a new
explicit admission; none is created here.

Verification completed: **61 Python tests: 60 passed, one skipped** (explicit native
fake-worker binary prerequisite); **22 qualitative Rust tests passed**. Native CLI
build, changed-file rustfmt and diff checks passed. Campaign ledger hash and all 69
previously snapshotted historical files remain unchanged. No full-workspace pass is
claimed.

Commands:

```sh
PYTHONPATH=scripts python3 -m unittest test_engine_output_release test_engine_bounded_correction test_engine_synthetic_transport test_engine_response_evidence test_engine_exploration_contract test_engine_exploration_validation test_engine_b91_campaign -q
cargo +1.90.0 test --offline -p tsr-cli --lib engine::qualitative -- --test-threads=2
```
