# Dated DeepSeek / GPT Luna shadow comparison — 2026-10-25

Four explicitly approved requests ran on compatibility commit `3c48167`; this is
not B9.1 Trial 11. Exact order: `deepseek-v4.1-flash-1017`, `gpt-6-luna`,
`gpt-6-luna`, `deepseek-v4.1-flash-1017`. Zero generated-code execution or adoption.
The frozen synthetic messages were identical across requests. Only model ID and
the documented limit parameter differed: `max_tokens:8192` for DeepSeek,
`max_completion_tokens:8192` for Luna. Each used a 240-second local deadline,
stream false, model lock, and otherwise omitted sampling/reasoning overrides.
Defaults and tokenizers need not be equal. No fifth/probe call was made.

Ordinary requested limits were supported by [DeepSeek Chat API](https://api-docs.deepseek.com/api/create-chat-completion/),
[OpenAI Chat API](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create/)
and [CheaperInference forwarding documentation](https://www.cheaperinference.com/docs#parameters).
This is not a guaranteed billing cap or proof of universal provider compliance.
No ignored limit was observed for these four requests. Qwen remains excluded.

Official catalog prices checked before dispatch were $0.075/$0.300 per million
input/output for [DeepSeek 1017](https://api.cheaperinference.com/public/models/deepseek-v4.1-flash-1017)
and $0.065/$0.325 for [Luna](https://api.cheaperinference.com/public/models/gpt-6-luna).
Actual charges below are provider-settled amounts, not recomputed from these advertised
rates; the retained usage includes cache details, and advertised-rate multiplication
does not reproduce every settled amount. The exact cause of that difference is not
established by these receipts alone. The charge ledger has not been altered to fit an estimate.

Canonical admission hash:
`375d9ff92f0a0ed9ba014f24913883102f9e981e6a942760fe61082ff307a8f2`.
Evidence directory: `target/b91-1017-luna-shadow/` (manifest, journal, typed
candidates, blind reviewer inputs and summary). Independent readiness review checked
all implementation/binary/body/validation hashes and scope before dispatch. Fresh
validation controls ran before every call. No expired historical admission was reused.

| Call | Model | Prompt | Completion | Reasoning subset | Seconds | Settled USD | Outcome |
|---|---|---:|---:|---:|---:|---:|---|
| 1 | DeepSeek 1017 | 2492 | 303 | 0 | 15.327 | 0.000416 | malformed JSON |
| 2 | Luna | 2371 | 869 | 504 | 17.705 | 0.000548 | typed candidate |
| 3 | Luna | 2371 | 1229 | 846 | 17.714 | 0.000478 | typed candidate |
| 4 | DeepSeek 1017 | 2492 | 845 | 0 | 19.287 | 0.000392 | malformed JSON |

Totals: **9726 prompt + 3246 completion = 12972 tokens; $0.001834 settled;
zero unknown charges; four calls, zero executions/adoptions**. Reasoning is already
included in completion. Call 2 reports 2369 cache-write tokens; call 3 reports 2369
cached prompt tokens; call 4 reports 2432 cached prompt tokens. All responses ended
with `finish_reason:stop`. All slots are consumed; no retries or extra calls remain.
Both malformed responses failed parsing at content, so their assertions and semantic
quality are unknown under the retention policy. Raw bodies/reasoning are not retained.

## Blinded independent findings

The reviewer saw only Aster/Fern/Maple/Rowan candidate files, frozen task and rubric;
model/order/billing were withheld. Labels were unblinded after judgments:

- **Fern (Luna call 2):** correct empty-input result and exact vector for a signed
  nonempty list; requires rejection of whitespace, leading/trailing empty fields,
  malformed text and positive overflow. Nonvacuous both-aim coverage, no inversion.
  Both declared IDs match root Rust test functions and do not collide with existing
  names. It correctly notes that a failed case need not identify its failed assertion.
- **Maple (Luna call 3):** correct empty-input result, explicit plus/minus signed
  vector, and exact `Err("invalid")` expectations for interior/trailing empty fields,
  whitespace and positive overflow. Both-aim coverage and correct case identities.
  Its malformed-field checks do not establish coverage of arbitrary malformed text.
- **Aster/Rowan (DeepSeek calls 1/4):** no typed candidate; semantic quality unknown.

Both Luna experiments would distinguish the supplied field-filtering implementation
through empty-field cases, according to source review. This is a prediction, not an
observed test execution. Coverage is finite: neither proves all nonempty behavior,
negative overflow, successful boundaries or leading-zero behavior. No social/general
TSR fulfillment, reliable causal advantage or broad autonomy follows.

Recommendation: Luna is the best-supported next candidate among these two for a
separately admitted bounded B9.1 trial. This does not switch the production model or
license another call. Two samples per model, one synthetic state, cache effects,
provider defaults and lack of execution limit generalization. Keep the current
independent review and actual-return verification gates.

Fresh-process replay refused the existing journal before dispatch and left its hash
unchanged. No source/acceptance edits or historical trial rewrites occurred.
Compatibility was independently reviewed and 21 focused tests passed in 35.221s.

Final full offline suite on this implementation: **627 tests in 186.663s,
610 passed and 17 skipped, zero failures** (`target/requested-model-full-python.log`).
Existing HTTP-error/SQLite cleanup ResourceWarnings remain. No new Rust sweep is
claimed. Native validation paths are covered by the Python integration suite. No
new software, protected edit, credential/account change, push or deployment occurred.
