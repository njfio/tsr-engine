# Prospective response identity durability

The Pond redesign interruption left a retained request and unknown reservation but no authenticated provider response ID. Its historical record is unchanged. A same-time billing entry is insufficient to establish identity; completing reconciliation requires an authenticated response ID or provider-side correlation to the retained request body. No provider outreach or retry was performed.

The shared release transport now journals a response identity record synchronously before reading a successful or HTTP-error response body. The record contains only allowlisted UUID identities, HTTP status, exact serialized request SHA-256 and byte count, local request identity, native root/snapshot/scope and admission hash. It excludes authorization headers, credentials and response text. The existing append-only journal fsync occurs inside the pending dispatch operation.

Missing or conflicting identities remain explicit. Identity is not billing, semantic acceptance or retry authority. Restart leaves an interrupted dispatch pending and refuses redispatch. An interruption before receiving/persisting headers can still leave an unknown; this change does not guarantee recovery of every network failure. The historical missing identity cannot be reconstructed by this fix.

Independent source review passed after adding guaranteed HTTP-error response closure when identity persistence fails. Authored offline verification: six identity fault tests and fifteen existing transport tests passed. Three adapter branch checks passed; five native-worker checks were initially skipped pending selection of the existing explicit fixture binary. These are local harness tests, not provider or game evidence. No campaign calls or operations were consumed.

Current historical blocker remains 347 calls / 101 operations, $0.403253 known and $0.453253 committed including the unresolved $0.05 reservation. See `pond-beautiful-interruption.md` for the exact pending request and evidence limitations.
