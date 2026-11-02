# Requested exact-model comparison readiness — 2026-10-25

The user requested `deepseek-v4.1-flash-1017` and `gpt-6-luna`; the bounded scope is
two synthetic shadow requests each, no execution/adoption. **No inference request
has been dispatched for this comparison.** No aliases or production model selections
were changed. Preparation remains pending output-limit compatibility, not price.

| Exact public catalog ID | Input / output USD per million | Context / advertised max output |
|---|---|---|
| deepseek-v4.1-flash-1017 | 0.075 / 0.300 | 1,048,576 / 1,048,576 |
| gpt-6-luna | 0.065 / 0.325 | 1,050,000 / 128,000 |

Official metadata: [DeepSeek dated model](https://api.cheaperinference.com/public/models/deepseek-v4.1-flash-1017)
and [GPT Luna](https://api.cheaperinference.com/public/models/gpt-6-luna).
Both advertise Chat Completions, structured output and reasoning. Advertised maximum
capacity is not a per-request enforced limit or a competence measurement. Prices fit
the user's inexpensive tier. Local public snapshot: `target/requested-model-readiness.json`.

[OpenAI's Chat Completions contract](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create/)
defines `max_completion_tokens` as including visible output and reasoning.
The existing runner/transport currently admits `max_tokens` for its finite old model
allowlist; adding Luna needs an explicitly pinned, tested control change rather than
a model-string substitution.

[CheaperInference parameters](https://www.cheaperinference.com/docs#parameters) say
common fields are forwarded but support and interpretation are model-specific and
unsupported parameters may be ignored or rejected. Its [API reference](https://www.cheaperinference.com/api-reference)
accepts `max_completion_tokens` as an alternative, requiring equal limits if both
fields are supplied. This does not independently establish exact-route enforcement.
[DeepSeek's Chat API](https://api-docs.deepseek.com/api/create-chat-completion)
describes `max_tokens` as the generation maximum, while its current
[thinking guide](https://api-docs.deepseek.com/guides/thinking_mode/)
does not establish the dated broker ID's total reasoning-plus-answer interpretation.
No unsupported flag or empirical cap probe was attempted.

Remaining technical prerequisite: obtain sufficient official route-specific documentation
for the intended total bound, or explicitly approve a provider-specific control and
its uncertainty for a bounded shadow attempt. This is not a request to raise the
four-call allowance or switch providers. Under the parent instruction to stop if
output bounds cannot be established, live dispatch remains **pending**, not denied
or uncertain; there are no new charges or consumed slots. Compatibility code/model
allowlist changes have not been made while this prerequisite is unresolved.

## Resolved scope of assurance

The parent clarified that this trial requires ordinary documented requested-token
bounds, not mathematical proof of broker enforcement or a guaranteed billing cap.
Under that standard the documented controls suffice: DeepSeek's Chat API specifies
`max_tokens` as its generation maximum; OpenAI specifies `max_completion_tokens`
including visible output and reasoning; CheaperInference documents forwarding both.
There is no retained evidence of an ignored limit for either newly requested model.
The Qwen report is not evidence about these models. No extra probe is necessary.

The narrowly updated shadow integration uses `max_tokens:8192` for the exact dated
DeepSeek ID and `max_completion_tokens:8192` for exact Luna. It forbids the wrong or
both limit fields rather than silently translating them at dispatch. Messages and
synthetic source remain identical. Model defaults and tokenizers need not be equal.
Per-call 240-second local deadline, four-call admission, no retry on uncertainty and
post-return overrun stop remain. Reported usage and actual bill remain distinct;
provider misbehavior is still possible and is not hidden by these requested limits.
No premium model, Qwen or production model selection is enabled by this change.

The authorized four-request comparison has now completed without a probe or overrun;
see [actual outcomes](engine-1017-luna-shadow.md). The earlier pending status above
records the pre-clarification checkpoint, not a current unspent allowance. No production
model selection changed, and all four approved slots are consumed.
