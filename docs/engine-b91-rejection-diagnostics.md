# B9.1 rejected-response diagnostics

This offline repair follows Trial 2 (`d1ff7d6`). It does not reconstruct the lost
candidate, establish its exact defect, reopen that trial, or consume new inference
or execution allowance. Trial 2 remains incomplete: one seed execution group, one
provider request, $0.000457 settled, with unused allowance expired.

## Boundary and evidence

All six stages now check the response envelope before binding: relation after seed,
experiment, relation after experiment, boundary patch, relation after verification,
and final proposal only. The final stage still accepts each existing proposal kind
without granting execution. Checks distinguish root/payload type, missing or ambiguous
envelopes, wrong stage, unsupported fields, and missing fields. Existing binder and
Rust validation remain authoritative; this is not a permissive replacement schema.

Each rejection has a fixed stage/code and a source/request-bound quarantine digest.
Malformed wire JSON and malformed message-content JSON are distinguished. Duplicate
keys, nonfinite values, model/response mismatch, output truncation, resource stops,
transport uncertainty, stale state, binding rejection and native rejection retain
separate classifications. Native rejection does **not** claim a specific provenance
fault when the native interface has not supplied a safe typed reason. Source/parent,
review, provenance and authority checks are not relaxed.

The driver durably records structural evidence immediately after transport, before
settlement/binding. Rejected responses also have an exclusive, mode-0600 file under
`quarantine/`, separate from admitted proposals. It contains schema-key names from a
fixed allowlist, types, counts, string lengths and hashes; arbitrary keys are hashed,
and arbitrary text, source bodies, reasoning and exception messages are not copied.
This follows AGENTS.md's prohibition on logging provider response bodies. Output is
bounded to 128 nodes, depth 6, 32 fields/object, 8 items/array and 32KiB/report;
bounded flags disclose omitted structure. This evidence can identify envelope and
shape defects, but cannot recover arbitrary wording or prove semantic validity.

Quarantine is not read into model context or guidance. Failure to write it leaves
an unreconciled dispatch and blocks restart rather than retrying. Interruption after
writing quarantine but before recording an outcome has the same conservative behavior.
A received response with unknown usage/cost remains resource-stopped; a transport
exception remains unknown, never zero-cost. No retrospective edits to receipts occur.

## Verification and review

Authored fixtures cover all six stage envelopes, unsupported fields/forged bindings,
stale state, native rejection, private strings and keys, nested size bounds, symlink
refusal, malformed wire/content JSON, truncation, unknown transport, immutable
quarantine, interruption and no redispatch/no context leakage. Existing native
cross-language fixtures exercise all six stages and four fake worker returns, plus
metered invalid response and unknown-charge restart. They execute no target commands
and are not live B9.1 evidence.

Independent review found and corrected a traversal issue: exhausted sibling loops
could exceed the advertised node budget. The corrected traversal and interruption
regression were re-reviewed with no remaining blocking finding.

Logs: `target/b91-diagnostics/{affected,diagnostics-review,native,python-full,python-final}.log`.
The initial affected run exposed the lost legacy “state drift” diagnostic; wording
was restored while retaining the typed code. No assertions were weakened.

## Next validation

A future **freshly approved** trial should freeze the same synthetic scenario and
acceptance, bind the new implementation in its admission, and stop on its first
invalid/unknown result. Its first rejected response will now identify a bounded
stage/schema category and structural evidence rather than collapse into a generic
edit error. A successful response must still traverse normal binding, independent
review and separately admitted execution. This is a materially improved diagnostic
trial, not evidence that model generation has improved. No retry is authorized by
this document. The prior six-call/four-execution envelope would require fresh approval;
Trial 2's expired slots cannot be reused.

Final verification (2026-10-23):

- `PYTHONPATH=scripts python3 -m unittest discover -s scripts -p 'test_*.py'`:
  **584 total, 568 passed, 16 opt-in skips, zero failures/errors**, 124.728 seconds.
  No runtime/provider/target opt-in was enabled for discovery.
- Focused envelope/transport/diagnostics: 34 passed before the independent-review
  additions; the expanded diagnostic suite's eight tests passed, and all are included
  in final discovery.
- `TSR_B91_TEST_WORKER_BIN="$PWD/target/debug/deps/tsr_cli-62904bb7a7cdafee"`
  with `test_engine_b91_driver.NativeSequence.test_full_native_sequence` and
  `test_unknown_charge_keeps_native_token_usage_and_blocks`: passed. The third
  test initially exposed a new assertion's incorrect tuple index, not a controller
  failure. After fixing that index,
  `test_known_invalid_provider_failure_is_metered_and_not_retried` passed separately
  in 5.910 seconds. The initial failure log is retained. These use cfg(test) worker
  receipts and execute zero target commands.
- `git diff --check`: passed. Independent reviewer cleared the corrected changes.
- Trial1 driver/provider/STOP hashes match the previously recorded baseline. Trial2
  records were only read; no quarantine or retrofit was added to that historical run.

No Rust implementation changed, and no new full Rust suite was run for this Python
boundary repair. Existing skipped tests are not represented as passing.
