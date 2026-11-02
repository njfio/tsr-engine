# Bounded cap investigation and process lifecycle verification

2026-10-22. No inference, authentication, credentials or task payload transmission.
The cap investigation is now stopped at the explicit documentation gap below.
Approval remains pending and the external dispatcher remains disabled.

## Official schema findings

Retrieved official [OpenAPI](https://www.cheaperinference.com/api/openapi.json)
without authentication at 14:03:50 UTC. Exact relevant fields are saved in
`target/relational-preflight/request-limit-schema.json`.

- `max_tokens` is an integer; `max_completion_tokens` is an alternative output limit.
  Supplying both requires equal values.
- `reasoning=false` disables thinking only where supported. `reasoning.max_tokens`
  is a manual budget only where supported; the documented concrete constraints
  concern Claude, not this DeepSeek model.
- `reasoning_effort` supports generic values including none, with model-dependent
  support; conflicting effort aliases fail 400.
- `min_discount_percent` imposes per-token rate ceilings, with 503
  `min_discount_unavailable` rather than above-ceiling fallback.

There is no exact-model guarantee here that DeepSeek 4.1 Flash accepts disable/manual
budget options or that its aggregate billable reasoning obeys the requested output
limit. The public model advertises automatic reasoning support, not that guarantee.
Do not try an undocumented field and infer safety from its acceptance.

[Official docs](https://www.cheaperinference.com/docs), reviewed 2026-10-22, distinguish
reasoning billing from visible output and make parameter interpretation route-specific.
They document a gateway retry for network failures and selected transient statuses,
followed by eligible route fallback. Failed requests do not settle a charge; successful
buffered responses include settled billing. Client timeout is not proof the gateway
failed: its documented processing limit can exceed the client's. Retain reservations
and stop on uncertain completion. Explicit rate-ceiling errors include 400
`invalid_min_discount`, 400 `list_price_unknown`, 503 `min_discount_unavailable`.
No authenticated supply or usage check was performed.

**Finite blockers:** fresh paid-call approval; a documented exact-model bound covering
all billable input/output/reasoning/cache units (or provider-enforced total monetary
cap); then bounded dispatcher integration with locked reservations and settlement.
The small conditional estimate in the earlier preflight remains an estimate. Public
documentation reviewed here does not establish the requested hard $1 guarantee.
No support contact, account/security change or repeated live experiment will be used
to work around that gap.

## Complete deterministic lifecycle

New `crates/tsr-cli/tests/relational_lifecycle.rs` invokes a distinct `tsr` process
for each operation against a temporary fixture store:

1. Initialize fixture desires; actual local notebook inspection creates a return.
2. Submit an authored fixture relationship as provisional qualitative guidance.
3. Restart/read and verify persistence; reject an unknown claimed return unchanged.
4. A second actual notebook inspection produces measured adapter return; old guidance
   becomes inactive pending reconsideration, with semantic discriminator still unknown.
5. Submit a revision; preserve the old revision exactly.
6. Withdraw, restart, restore; repeated unchanged statuses preserve exact root/view.
7. Verify original desire texts, identities and authority throughout.

This executes the bounded in-memory notebook adapter, not external code, a project,
or a model. Fixtures are clearly labeled authored test data. It demonstrates the
controller lifecycle across OS process boundaries, not generated emergence or semantic
fulfillment. Unknown claimed returns never create observations, criteria or revisions.
Historical proposal evidence remains immutable while real fixture returns accumulate.

No new goals, weights, provider allowances or production behavior were added for
this test. Useful deterministic engineering can continue where a concrete defect is
found; neither cap uncertainty nor absent semantic generation warrants speculative
features or an additional benchmark campaign.

## Verification

- Focused subprocess integration test: 1 passed, 0 failed (2026-10-22).
- Focused Clippy with warnings denied and Rust formatting check passed.
- Independent code review found no blocker. Its suggested stronger rejection check
  was added: an unknown return submitted while reconsideration is due must fail
  specifically with stale relational evidence. The strengthened test passed.
- No production change or broader suite claim; this is fixture-only lifecycle evidence.
