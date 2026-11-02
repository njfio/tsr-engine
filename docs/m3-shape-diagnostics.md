# Content-free response diagnostics

Implemented in the separate `scripts/m3-shape-diagnostics-broker.py`. The halted run's frozen adapter and controller are unchanged; no launch points to the new adapter. This work made no provider calls, accessed no credentials, and introduced no retry.

The observer records only a version number, fixed type/category strings and booleans. It distinguishes missing, non-array, zero, one and multiple choices/calls; missing or wrong function names through type and expected-name match; legacy function-call, content and top-level output presence/types; allowlisted finish reasons versus absent/null/nonstandard values; refusal type/truthiness; and missing/non-string/invalid-JSON/valid-JSON argument structure. Unknown function names, finish reasons, argument values, source text, content and refusal text are never copied. Counts are bucketed rather than unbounded. Metadata has no authority over validation.

The trace is attached after billing settlement and immediately before strict decoding. Existing failure codes and all action, scope, forecast and usage checks remain unchanged. Existing transport JSON errors remain distinct from argument JSON failures through the diagnostic stage and argument-parse category. The observer does not persist provider response bodies or change successful proposals. It can describe future responses only; it cannot reconstruct the old sixth response.

## Validation

- `python3 -B scripts/test_m3_shape_diagnostics.py -v`: **8 passed**. Covers metadata categories, privacy canaries, malformed structures, nonmutation, unchanged function ASTs, and a mocked settled format failure that emits one catalog request and one completion request with no redispatch. Both requests are fake; credentials are an inert mocked constant.
- `python3 -B scripts/test_m3_response_shape.py -v`: **8 passed** against the unchanged frozen decoder.
- Replaying the new observer test against the frozen adapter fails because `response_shape` is absent; captured separately as a baseline feature-absence check, not evidence of an old decoder defect.
- Exact **68-file export** and **91 frozen input hashes** remain unchanged. Hashes and counts are in `m3-shape-diagnostics-verification.json`; command output is in `m3-validation/m3-shape-diagnostics-*.log`.

## Risks and live-execution limits

The observer repeats JSON argument parsing in memory; processing is bounded by the existing 1 MiB HTTP response cap, and malformed/recursive JSON is classified without retaining text. It introduces small parsing overhead. Structural metadata cannot prove provider assertions or establish a semantic reason for refusal/truncation when the provider omits indicators. Alternate envelope flags identify structure, not protocol compatibility. Strict decoding may still classify other malformed shapes as an internal error; this work does not broaden acceptance.

The adapter is a deliberate versioned copy; future changes require checking for drift from its frozen predecessor. It inherits the prior provider/budget configuration and is not a new launch authorization. Independent review is required for any future wiring, freeze and execution plan. The reviewed continuation specification is in `m3-format-failure-continuation-policy.md`; it is not implemented. It allows only fully settled, verified format failures to preserve an unavailable result and consider the next unattempted manifest arm. Global billing, containment, evidence and budget failures still halt. Independent review identified that choice-count errors precede refusal/truncation checks; the policy was narrowed to keep all choice-count errors globally halting because multi-choice metadata cannot prove those exclusions. Live behavior of the new metadata has not been validated, and no new paid attempt is requested by this report.
