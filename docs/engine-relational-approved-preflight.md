# Approved synthetic comparison: read-only cap preflight

2026-10-22. The user approved up to four CheaperInference `deepseek-v4.1-flash`
comparison calls, synthetic benchmark desires/instructions only, aggregate maximum
$1. This supersedes earlier pending-approval notes for this exact comparison. It does
not authorize changing account spending controls or accepting an overage.

## Observed results

At 15:47 UTC, authenticated read-only metadata using the existing configured key:

- `GET /v1/keys/self/task-budgets`: `open: null`; no applicable admin task limit.
  No historical budget records, key identifiers, credentials or account details were
  printed or retained in this report.
- Filtered `GET /v1/models?type=text&reasoning=true`: exact model listed, Chat
  Completions supported; context 1,048,576, published maximum output 393,216.
- Current USD/million rates: input 0.0825, output 0.33, cache read 0.00165,
  cache write 0.0825; list input/output 0.15/0.60. These rates alone do not bound a
  request's total billable reasoning. No live availability probe was sent.

The first unfiltered catalog read exceeded the old broker's 1 MiB metadata limit;
its result was discarded. The subsequent read used documented catalog filters and a
bounded 4 MiB read. This was metadata only, not a repeated inference attempt.

Evidence: `target/relational-preflight/approved-metadata-preflight.json`,
`approved-cap-control-read.json`, and `approved-openapi.json`. Secrets were used only
in memory by the existing credential path and never printed, stored or passed on a
command line. No inference, mutations, credential creation or account changes occurred.

## Concrete provider control

The official [OpenAPI specification](https://www.cheaperinference.com/api/openapi.json),
retrieved 2026-10-22, documents a per-key task budget. It reserves each admitted
inference request's worst-case charge before wallet reservation and refuses a request
that does not fit with `task_budget_exhausted`. Failed/non-billed requests settle to
zero. This is a server admission control rather than a retrospective client counter.
The budget applies to every request on the key, not only this comparison.

A budget has a lease; after expiry the key returns to normal limits. Request-level
maximum-token fields still lack the exact-model billing coverage needed to replace
this control. Generic rate ceilings limit unit prices, not total quantities. The
[provider documentation](https://www.cheaperinference.com/docs) also describes
server retries and settlement, so client timeout is not evidence of zero charge.

## Exact remaining authorization and implementation boundary

**Blocked, not denied:** opening a new temporary spending control is outside this
turn's explicit read-only/account-settings restriction. No suitable budget exists.
The concrete next authorization is to open a dedicated $1 task budget on the existing
key using `POST /v1/keys/self/task-budgets`, with a fresh opaque task UUID,
`limit_usd: "1.000000"`, and a 3,600-second lease; close only that budget after all
its requests are terminal. The schema currently identifies this client class as
`keak-code`; inspect the returned contract rather than asserting a different client.
No credential or global workspace policy change is required. This temporarily
constrains other traffic on the same key and must not override another task's budget.

Before any future inference, validate the returned $1 cap, task ownership, lease,
zero outstanding foreign reservations and current model identity/pricing. Use one
locked local four-attempt journal, exact synthetic allowlist, `x-ci-model-lock: 1`,
non-streaming requests and no client retry. Recheck the active budget and enough lease
for the provider's documented processing ceiling before each dispatch. Do not renew,
raise, replace, force-close or share a budget silently. On timeout/unknown settlement,
stop, retain the reservation and reconcile the original request; do not close away
its accounting or reclaim a call slot. All inference charges must fall under the
provider budget, including its reasoning/retry settlement behavior.

The external dispatcher and independent semantic assessment have not run. Their
implementation/verification follows only after the control is explicitly authorized
and its operational response verified. Current result: **0 of 4 inference calls,
$0 comparison spend; no new semantic outcome**. The prior local benchmark results
remain unchanged. This is a concrete cap-control dependency, not a need for another
unbounded token-price estimate or a different model.
