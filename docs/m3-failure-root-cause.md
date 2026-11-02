# Offline continuation failure assessment

No unambiguous local adapter or patch-application correctness defect was established. This change adds characterization tests and this assessment; production code, the frozen live baseline, prior failures, and eligibility rules remain unchanged. No provider calls, retries, candidate execution, deletion, push, or publication were performed for this assessment.

## Multiple tool calls

The frozen `scripts/m3-continuation-broker.py:encode` sends `tool_choice: "required"`, declares exactly one function (`propose`), and does **not** send `parallel_tool_calls`. Its instructions request one bounded action. Requiring a tool and defining one function does not itself establish one invocation per response. The seventh ProtocolError/rules response's retained structural metadata records multiple calls, with `finish_reason=tool_calls`; it does not retain their exact count or contents. Strict decoding correctly refuses that response instead of selecting or executing a subset.

The retained `docs/m3-provider-catalog.json` explicitly says tool support is not established by the catalog. Neither that catalog nor the successful earlier stop pilot establishes support for `parallel_tool_calls: false` on this provider/model. Compatibility cannot be verified from available offline evidence. No live probe or documentation fetch was made.

Smallest proposed transport change, contingent on authoritative provider documentation: add its supported single-call control (possibly `parallel_tool_calls: false`) while retaining exactly-one validation. Test the serialized request and rejection of zero/multiple calls deterministically. Do not silently retry, choose a call, or reinterpret multiple proposals. A forced function name or strict argument schema alone is not a cardinality guarantee. This proposal is **not implemented** pending capability evidence and review.

## Changed protected tests

`broker_v2::Proposal::Replace` carries `path` and `contents`. The controller serializes that normalized proposal for its argument hash, then applies `ScriptedEdit::new(path, contents)`. `ScriptedEdit::apply` writes the entire new contents; it is not a diff or merge interface. The tool schema exposes `contents` only as a string, while its system instruction prohibits changing tests. It does not explicitly describe whole-file semantics in the field description.

The final retained source is 4,179 bytes versus 19,369 bytes in the original base. Of 20 original test declarations, 19 are absent; the surviving named test is also rewritten. Thus the submitted content changes tests, rather than merely demonstrating an unexpected workspace mismatch. The evidence cannot distinguish accidental omission while reconstructing a whole file from a deliberate test change; no intent is inferred.

The final source reconstructs the normalized second and third replacement proposals exactly. Hashing compact UTF-8 JSON in Rust field order (`action`, `path`, `contents`), with the domain's little-endian u64 length prefix, yields:

`1d018dbfa471778e8012b10eb11f0b0bf813734e0cf0d7b532d4b9c9585cc1e2`

That matches proposed-event arguments at sequences 44 and 50. The length-prefixed source hash is `3e183697f5f618db675d357af4d639560f5c0dc5647baf5667233c77d6c9298b`, matching inspection events 49 and 55. Source SHA256 is `ecf5b7e69b756849667163285ca444db4a96f9f10a81799c6d459b5fe553d76b`. The first replacement at sequence 12 has a different hash and cannot be recovered from retained payloads. This reconstructs normalized proposals, not provider HTTP bodies, forecasts, or reasoning.

The protected-test comparison in `run::tamper_reason` correctly rejected changed full-suite test evidence before final-oracle execution. Neither this rejection nor the two preceding failed visible tests is a final-oracle result.

Smallest proposed usability change: describe `contents` as the **complete UTF-8 file**, requiring preservation of unrelated and test bytes and prohibiting elisions. This may reduce misunderstanding but cannot guarantee compliance; it changes a frozen prompt/schema baseline and is proposed for review, not applied here. A larger versioned interface could support bounded edits with an exact old-text/source-hash precondition. Any pre-apply protection should reuse the trusted snapshot/parser and retain final integrity checks in both modes. Do not restore omitted tests automatically, weaken the protected projection, or turn integrity failures into retry-eligible outcomes.

## Offline validation

Added `scripts/test_m3_failure_contracts.py` verifies the actual frozen request configuration, rejects two individually valid calls, and binds the retained final source to both proposal and inspection hashes. It also checks all 106 exported files against their frozen SHA256 manifest. Its local BLAKE3 helper dependency is `target/m3-blake3-checksum`, already built for the terminal audit; no download is performed.

Added `crates/tsr-batch/tests/m3_replacement_contract.rs` demonstrates full-file replacement and byte-preserving complete replacement with inert temporary text. These are characterization tests, not a claimed fail-first production repair. Candidate source is never executed.

Commands and results:

- `python3 -m unittest discover -s scripts -p test_m3_failure_contracts.py -v`: 3 passed.
- `cargo +1.90.0 test -p tsr-batch --test m3_replacement_contract --offline`: 2 passed.
- `cargo +1.90.0 test -p tsr-batch --lib inline_test_module_helpers_and_macros_are_frozen --offline`: 1 passed.
- `cargo +1.90.0 test -p tsr-batch --lib unlisted_tests_are_frozen_before_and_after_execution --offline`: 1 passed.

All 225 readiness-manifest inputs were separately rehashed and remain unchanged. Independent read-only review found no blocking issue; its manifest-set observation was addressed by asserting exact export file-set equality as well as all hashes. No full-workspace test or hook pass is claimed. Historical evidence and costs remain as reported in `m3-continuation-final.md`: aggregate $0.229582, outstanding $0, zero completed pairs and zero verified acceptances. M3 operational acceptance is still unmet. The remaining Route/rules arm stays unattempted under the halt policy; no tool denied an action during this assessment. Provider capability confirmation and any new live baseline/run remain pending separate review, not a reason to relax validation.
