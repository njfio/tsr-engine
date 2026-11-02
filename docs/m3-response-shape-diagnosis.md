# Offline diagnosis of the sixth response

The sixth response was invalid for the adapter's required envelope. **The exact response cannot be recovered from the retained evidence. No adapter defect has been demonstrated, and no parser or retry policy was changed.** This diagnosis made no provider calls and accessed no credentials.

## Evidence and limits

The premise that a response body was preserved is incorrect. `AGENTS.md:176` prohibits logging or committing provider response bodies. The frozen adapter parses the HTTP JSON in memory, settles billing, then decodes it; its persisted trace contains only scalar operational metadata. The exported artifacts and current run/state directories contain the trace, ledger, committed-action evidence and process results, not a saved sixth response. This is a retention limit, not a permission or approval denial. No unrelated session paths were searched.

The matching reservation is `d9d3943d-b1d8-45ba-9175-373f1e25351d`, provider request ID `5a19108f-53ca-42cc-a3cb-7f06337e0705`. Its diagnostic is `stage=decode`, `code=tool_shape`, `settled=true`, elapsed 5240 ms. Billing records 5098 prompt tokens, 201 completion tokens, 5299 total tokens and $0.000736 for this response. The full attempt remains $0.005842; aggregate remains $0.214469 with nothing outstanding.

Required by `scripts/m3-progress-validation-broker.py:103`: one choice whose `message.tool_calls` contains exactly one call whose nested `function.name` is `propose`. Arguments then must be JSON with exactly `action`, `path`, `id`, `contents`, and `forecast`; allowed action, field types, scope and usage checks follow. The request uses `tools` and `tool_choice=required`, non-streaming chat completions, with a 16384 output-token cap.

Observed from the control flow: JSON transport parsing and billing settlement succeeded; choice count was one; no truthy `message.refusal`, `finish_reason=content_filter`, or `finish_reason=length` triggered the earlier guards. The expected single named tool call was absent or malformed. Argument parsing and action validation were not reached. These are code-path inferences, not inspection of a saved response body.

The trace cannot discriminate missing/empty calls, multiple calls, a wrong or missing function name, or an alternate envelope. It also does not identify whether the model or provider serialization produced the mismatch. An explicitly signaled length truncation would have produced `output_truncated`, not `tool_shape`. The reported 201 output tokens are below the cap, but missing/nonstandard finish metadata or unreported truncation cannot be ruled out. No actual finish reason was retained.

Legacy `message.function_call`, JSON in `message.content`, and a top-level `output` array are not supported by this adapter. Synthetic examples of each produce `tool_shape`. There is no evidence any occurred in the live response, nor evidence that a supported envelope was incorrectly rejected. Automatically coercing these formats or selecting one of multiple calls would change the protocol; it is not justified as a repair of this incident.

## Why the other three arms stopped

At `scripts/m3-progress-validation-controller.py:106`, the serial controller admits only one result with outcome `accepted` or `rejected`. The first arm returned `unavailable`, triggering `arm unavailable; preserve and stop`. The exception handler outside both loops writes `halted-review-required` and exits. Therefore the other three arms never started.

This is a conservative global stop policy chosen for the pilot, not a causal dependency between independent tasks. Billing was settled and the arm process exited cleanly; neither required classifying the remaining arms as failed. The existing one-shot state guard also prevents restarting this controller. No guard was bypassed.

## Deterministic regression and review

`scripts/test_m3_response_shape.py` generates inert synthetic response fixtures in memory. It does not replay or claim to reconstruct the live sixth response. Eight tests cover a valid single proposal; missing, empty, multiple, wrong-name and missing-function calls; alternate envelopes; refusal/truncation precedence; absent finish metadata; choice and argument errors; unchanged action validation; and the exact retained diagnostic field set. Credential and transport entry points are mocked to fail if called.

Command: `python3 -B scripts/test_m3_response_shape.py -v`. Result: **8 passed**. Captured output: `docs/m3-validation/m3-response-shape-tests.log`. These are characterization regressions against unchanged code, not a fail-first proof of an implementation defect. No fix was warranted. Independent review agreed that the retained evidence does not establish a parser bug or narrower response shape.

The exact 68-file export and 91 frozen inputs were checked again and remain unchanged. The frozen adapter, evaluator and controller remain unchanged.

## Smallest next step and separate recovery proposal

The smallest useful next code change is an offline-tested, separately versioned diagnostic adapter that records only bounded shape metadata: allowlisted finish-reason category, choice and tool-call count categories, expected-name-match boolean, and legacy-envelope presence flags. Never retain names, arguments, content, reasoning, raw bodies or credentials. This would distinguish future failures; it cannot repair missing historical evidence. This diagnostic change has not been implemented or used in a paid run here.

A separate future recovery policy could classify settled protocol failures as terminal for that arm, then continue independent arms only after ledger settlement, worker/container quiescence, remaining budget and unchanged-input checks. Ambiguous billing, identity/integrity errors, or unresolved processes must still stop the cohort. Keep the failed arm in the report and do not count an incomplete pair as a completed comparison. If a fresh retry is explicitly authorized, cap it at one separately identified attempt within a fixed dollar/call budget and retain the original unavailable result; do not concatenate hidden retries or loosen action validation. This is a proposal only, not an implemented policy or a request to resume this run.
