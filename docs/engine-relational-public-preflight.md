# Public provider preflight and relational restart checks

Retrieved2026-10-22. No credentials, inference, account changes or task payloads were
sent. Approval is still pending; no external dispatcher is enabled.

## Official public evidence

The authenticated `/v1/models` endpoint was inaccessible through the web reader.
The official OpenAPI describes an unauthenticated single-model route. A plain GET
succeeded at13:58:48UTC:
[public model](https://api.cheaperinference.com/public/models/deepseek-v4.1-flash).
Saved exact metadata: `target/relational-preflight/public-model.json`.

| Public field | USD per million tokens / value |
| --- | --- |
| Current input | 0.082500 |
| Current output | 0.330000 |
| Cache read | 0.001650 |
| Cache write | 0.082500 |
| Reference input/output | 0.150000 / 0.600000 |
| Context / maximum output | 1,048,576 / 393,216 tokens |

The record lists the exact model and Chat Completions support, reasoning and implicit
caching. This proves public catalog listing, not account access or current route
capacity. No live fill was tested. Top-level reference cache-read price is0.0015,
while scoped maker tiers list0.003; do not silently normalize that discrepancy.

[Official docs](https://www.cheaperinference.com/docs), retrieved2026-10-22:
reasoning is billed as output, sometimes absent from aggregate completion counts.
Parameters are model/route-specific. Routing is all-in without a separate surcharge;
fallback can change rates. Buffered responses include settled billing. Minimum wallet
funding is$5, not a documented per-request floor. I found no explicit request-minimum
fee specification. These are published statements, not an account-level quote.

[API reference](https://www.cheaperinference.com/api-reference), retrieved2026-10-22:
`min_discount_percent` constrains input/output rates relative to catalog list prices
and refuses unavailable supply. It is a rate ceiling, not a total-dollar cap.
`max_tokens` and `max_completion_tokens` are alternative output limits; matching
values are required if both are supplied. This does not document the exact DeepSeek
route's handling of every hidden reasoning token.

[OpenAPI](https://www.cheaperinference.com/api/openapi.json), retrieved2026-10-22:
public model schema includes cache prices, optional long-context tiers and maximum
output/context metadata. No `max_cost_usd` or `minimum_cost` field was found. Lack of
a named schema field is not proof that billing can never include another charge.
The [public market](https://www.cheaperinference.com/markets) corroborates the listing;
its displayed input rate is rounded. Use the exact public JSON for this snapshot.

## Conditional budget calculation and exact blockers

Our calculation, using reference rates and an assumed upper bound of32,768 input
and2,048 TOTAL billable output tokens per request:

`4 × (32768 × .15 + 2048 × .60) / 1,000,000 = $0.024576`.

Even substituting the catalog's393,216 maximum output yields$0.9633792 for four
requests, **if** that maximum bounds all billable output and the input assumption
holds. Neither arithmetic result proves those assumptions. In particular, the
catalog maximum is not evidence that a requested2,048 limit bounds hidden reasoning
on every route. Cache pricing and any applicable billing minimum also need a coherent
scope. Do not spend the difference as extra allowance.

Public data substantially narrows the uncertainty and supplies a useful rate-ceiling
mechanism. It still does not establish the complete hard$1 contract requested.
Before dispatch: fresh user approval; confirmed exact-model output/reasoning limit
semantics and all billable classes; compatible current rate scope; then reviewed
shared four-attempt reservations and terminal settlement handling. If these cannot
be established without inference, report the missing provider guarantee rather than
silently treating a cheap estimate as a hard cap. No additional payment, key creation
or account setting is authorized or required by this preflight.

## Independent local engineering completed

Reproduced a concrete no-op status bug: asserting active on an already-active
relational aspiration inserted a status entry, changed its evidence key and reopened
consideration despite no new evidence. Repeating a status also churned the state root.
The fix validates current root and known identity first, then returns unchanged view
for unchanged effective status. Real withdrawal/restoration remains append-only;
stale requests are still refused. No historical event is rewritten.

New deterministic regressions cover implicit-active no-op, repeated status across
restart, stale-root rejection, mixed valid/invalid batch atomicity and superseded
revision rejection after restart with latest-revision success. Seven relational
tests pass; one explicit benchmark builder remains ignored by default. All78 Python
driver tests and CLI library/test Clippy with warnings denied pass. Independent
read-only review found no remaining blocker in the fix. This is
useful independent engineering, not semantic generation or new weighting.

Useful work remains possible without inference: implementing and testing the
provider's published rate-ceiling and normalized billable-unit contract once its
model-specific semantics are established. Current substantive relational-generation
results remain unchanged and insufficient; no numerical or schema result replaces
that missing capability evidence.
