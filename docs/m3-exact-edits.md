# Exact edits and single-action proposals: offline implementation

The new `exact-edit@v1` adapter replaces model-authored whole-file regeneration with one bounded exact substitution. The controller validates a digest of the **complete inspected file**, requires a nonempty source span with exactly one match (including overlapping matches), and preserves every byte outside that span. Invalid UTF-8, missing/ambiguous spans, no-op edits, stale digests, unsafe paths, and oversized source/result are refused before writing. The controller rechecks the source immediately before application. Its serial arm owns the candidate copy exclusively: no candidate worker runs between validation and write. This is not an atomic compare-and-swap guarantee against an unrelated concurrent host writer; sharing a candidate copy with another writer remains unsupported.

## Implementation and compatibility

- `crates/tsr-batch/src/broker_v2.rs`: additive `Edit { path, expected_digest, old_text, new_text }`; existing path containment, policy gates, forecasts and budgets still apply. Patch budget charges the entire reconstructed file, as with legacy replacement, so using a short edit does not bypass the budget. The exact edit itself is hashed in the proposed event. Action history contains metadata, not the proposed source text.
- `scripts/m3-edit-broker.py`: a separate adapter exposing only inspect/test/edit/stop. The same schema and instructions serve both modes, request exactly one next action, distinguish the inspected file digest from the aggregate source-state digest, and prohibit changing tests or regenerating whole files. Strict decoding rejects multiple calls, malformed fields, duplicate JSON keys, non-finite forecasts, irrelevant populated fields and legacy `replace` actions. No action selection, retry or repair of malformed output is added.
- Existing full-file `Replace` remains available in the engine for historical adapters/replays. It is not exposed by the new adapter. The additive v2 action requires an engine that supports it; an older v2 engine will reject it. A new engine/adapter/schema baseline must be pinned before live use. No historical adapter, controller, manifest or engine binary was overwritten.
- The new adapter imports the frozen continuation adapter's transport, deadlines, reservations, settlement and content-free diagnostics, binding only the new encode/decode functions. Both files must be included in the next baseline. No new credentials or provider operations were added.

The final evaluator (`run.rs`), protected-test extraction, oracle receipts, containment implementation, historical outcomes and continuation halt policy are unchanged. Edits touching protected tests are still rejected at the existing final integrity check; this is not a new pre-apply semantic test filter. Source-precondition failures remain terminal driver errors, not automatic retry opportunities. The old live controller is not repointed to this adapter.

## Provider documentation checked

On 2026-10-17, public [CheaperInference API documentation](https://www.cheaperinference.com/api-reference) was inspected without credentials or inference requests. Its Chat Completions schema lists `tool_choice`, but does not declare `parallel_tool_calls`; additional fields are forwarded upstream. The separate Responses schema declares `parallel_tool_calls`. That is insufficient evidence of single-call control support for `deepseek-v4.1-flash` on the current Chat endpoint. We therefore retain `tool_choice: required` and strict exactly-one validation, strengthen the one-action instruction, and leave the unsupported/unverified setting absent. No endpoint migration or live compatibility probe occurred. Whether the model follows the stronger instruction is still unmeasured.

## Deterministic validation

`m3_exact_edits.rs` creates a tiny temporary Git repository from inert test-generated text. Its trusted Python bridge calls the actual new encode/decode functions with HTTP and credential access disabled. The Rust controller then applies edits and uses the unchanged evaluator with a non-executing Runner double. In **both record and rules modes**, it covers:

1. Valid repair, visible test, Stop, unchanged surrounding/test bytes, and a verifiable final-oracle receipt.
2. Stale digest, overlapping match, and duplicate match: no edit and no final oracle.
3. Protected inline test edit: rejected by the existing full-suite evidence guard, with no final oracle.
4. Multiple individually valid calls and malformed JSON: unavailable, no edit and no final oracle.

These are **14 synthetic scenario executions**, grouped in one integration test. They validate the pipeline, not real task performance. No candidate code or sandbox-escape payload ran. Unit tests also cover UTF-8/CRLF preservation, an actual source change between checks, symlink/traversal refusal, empty/missing/no-op spans and size limits. The initial unit test failed to compile because the edit helper did not yet exist; this was test-first feature development, not a claim that a prior implementation had been repaired.

Commands actually run (Rust toolchain 1.90.0):

| Command | Result |
| --- | --- |
| `cargo +1.90.0 test -p tsr-batch --offline` | 91 passed, 1 existing ignored |
| `cargo +1.90.0 test -p tsr-batch --lib exact_edit_preconditions_and_byte_preservation --offline` | Passed again after adding concurrent-change and symlink assertions |
| `python3 -m unittest discover -s scripts -p 'test_m3*broker.py' -v` | 20 passed |
| `cargo +1.90.0 check -p tsr-cli --offline` | Passed |
| `cargo +1.90.0 clippy -p tsr-batch --all-targets --offline -- -D warnings` | Passed |
| `rustfmt +1.90.0 --edition 2021 --check crates/tsr-batch/src/broker_v2.rs crates/tsr-batch/tests/m3_exact_edits.rs` | Passed |

Captured outputs are in `docs/m3-validation/m3-exact-edits-*.log`. Independent read-only review found no blocking defect. The review confirmed the compatibility baseline requirement and the lack of a provider-specific single-call guarantee. No workspace-wide test or hook pass is claimed.

Preservation audit: all 106 historical export files match the exact file set and SHA256 manifest. Of the 225 historical readiness inputs, only the deliberately extended `broker_v2.rs` source differs; the old evaluator, binary, scripts, inputs and recorded readiness remain intact. The old readiness manifest intentionally is not refreshed or presented as authorizing the new implementation.

## Bounded next live-validation proposal — not executed

First build the new engine to a separate output directory, freeze its binary plus the new adapter and inherited transport, pin the same sandbox image, fresh empty stores and exact current settled ledger prefix, and independently review a new one-shot controller manifest. This is a new experimental baseline, not a retry that can overwrite or complete prior failed arms.

Propose **one existing ProtocolError task, record then rules, serial**, at most **10 paid requests and 1,800 seconds per arm**, **20 requests and 3,600 seconds total**, with **$5 additional maximum inside the approved $100 aggregate ceiling**. The 3,600-second controller deadline must include setup, evidence export and reporting overhead; dispatch the second arm only if its full arm budget plus evidence-export margin fits the remaining time. Retain the existing conservative per-call reservation and refuse dispatch if it cannot fit either cap. Stop the entire pilot on the first malformed/multiple-call response, source-precondition failure, protected-test rejection, ambiguous billing, or missing required evidence; no retries or advancing past an integrity failure. Preserve all costs and failures. Do not expand to the second task until this separately reviewed pair demonstrates usable behavior.

The new pair can test proposal usability and produce paired receipts, but cannot establish representative M3 outcome quality. Live compatibility, success rate and operational M3 acceptance remain unproven. No paid run is launched by this change; no push, merge, publication or artifact deletion occurred. Historical aggregate spend remains $0.229582 with $0 outstanding.
