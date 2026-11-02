# Bounded local verifier transport

The normal `project verifier` interface now has `local-configure`, `local-run`,
`local-cancel` and `local-reconcile`. It never starts a server, pulls a model,
executes candidate code or adopts a report. Existing explicit operator adoption
remains separate; default acceptance is unchanged.

Configure an admitted verifier before freezing a request. Configuration version is
`local-verifier@v1`, with fields `endpoint`, `model`, `digest`,
`generator_model_digest`, `context_tokens`, `output_tokens`. The endpoint must be
literal `http://127.0.0.1:PORT` without path or credentials. The model digest must
match the admission artifact and an installed local GGUF entry in `/api/tags`
before and after the separate `/api/chat` invocation. No DNS, proxy settings,
redirects, remote model entries, model pulls or tool calls are supported.

The artifact pins **model weights**, not transport code; use the repository commit
for implementation provenance. `generator_model_digest` is **operator-declared**,
not an authenticated generator receipt or proof of error independence. Distinct
invocation identity is required by the review admission. The reviewer receives
only frozen requirements, allowlisted source/protected evidence and coverage map,
with a separate fixed verifier system instruction. It receives no generator
conversation, private review verdict or repair rationale. Source content remains
untrusted data. Complete criterion coverage and typed findings are enforced by the
existing report validator; that validates structure, not semantic truth.

## Budgets and recovery

An active prospective project budget is required. Before any generation POST,
reserve one call and `context_tokens + output_tokens` in the shared accounting
ledger, then record dispatch intent. Context is bounded conservatively by UTF-8
bytes plus fixed overhead; no tokenizer estimate is presented as measurement.
Runtime-reported paired prompt/output counts and positive duration are recorded
separately from host elapsed time. Invalid/missing token counts retain the token
reservation. Local external charge is zero, not zero compute. Calls are never
reclaimed. The deadline is the minimum of admission timeout, admission expiry and
remaining project wall budget. Output is bounded to 16 KiB; HTTP capture to 68 KiB.

Complete and partial HTTP response **hashes**, bounded byte counts and completeness
are recorded in `local_progress`; raw response bodies are never persisted, following
AGENTS.md. These metadata are never instructions, accepted proof or generator input. Request envelopes, roles, pinned digest, timestamps,
usage and quarantine records remain inspectable through `verifier inspect`.

Cancellation appends an operator request and closes the connection when observed
(the transport polls at most every 50 ms during socket IO, plus local ledger work).
It does **not** prove that the server stopped inference. A canceled/invalid/stale
result cannot be adopted. Timeout or lost response preserves unknown completion
and its reservation. A process-held advisory file lock prevents reconciliation
while the transport is active. After a dead process, `local-reconcile --request ID
--reason TEXT` settles from recorded usage, or retains the reservation with unknown
execution after dispatch intent. Before dispatch it records not-sent, still without
reclaiming the call/slot. Reconciliation never recovers or invents a valid report;
unknown elapsed time is null. A new request consumes a new slot. Prior receipts
are immutable. A received-but-unapplied report remains quarantined, not accepted.

The Unix file lock uses the already-locked/cached rustix dependency to preserve the
repository Rust 1.85 minimum (std file locking requires 1.89); no new software was
downloaded. Other platforms fail closed for this transport.

The loopback service and host ledger are trusted infrastructure. Catalog checks do
not authenticate a malicious daemon or eliminate a model-tag change that occurs
and is reverted between checks. Socket cancellation is not an OS resource governor.
These are bounded advisory evaluations, not unattended acceptance authority.

## Minimal next shadow evaluation (not dispatched or admitted by this document)

Use two fresh disposable copies of the existing **synthetic invoice** fixture:
one frozen rejected candidate and one frozen accepted candidate, with their source
hashes, requirements and protected observed-test bindings checked independently.
Do not reopen historical closed episodes or replay stale test receipts as current.
If current compatible test evidence is unavailable, mark it unknown; this shadow
exercise must not run candidate code to fill that gap without separate admission.

Use installed `mistral:latest`, recorded digest
`61e88e884507ba5e06c49b40e6226884b2a16e872382c2b44a42f2d119d804a5`,
subject to fresh catalog verification at dispatch. Generator was
`qwen2.5-coder:32b`, digest
`4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`.
The installed catalog is preserved at `target/guidance-live/installed-models.json`.
These are distinct weights/families; Mistral's structured review competence has
**not** been established, and distinct models still may share errors. If absent or
incompatible, stop; do not download or silently substitute the generator model.

Exactly one request per copy, **two total**, each at most 240 seconds, context
16,384 tokens and output 2,048 tokens; reserve 18,432 tokens per copy (36,864 total),
local external charge zero, zero patch bytes and zero execution actions. Each fresh
project lifecycle allows one model call, 18,432 model tokens, 300,000 ms wall and
one ms tool ceiling solely because the existing admission requires a positive
ceiling; that is not execution permission. Freeze review admission/configuration
and request separately for each source. `local-run` performs one POST with no retry.

Preserve both outcomes including inability, malformed output and unknowns. Compare
per-criterion findings and evidence limitations against a separate operator semantic
review blinded to generator rationale. Do not adopt either report during this
shadow comparison. No human effort is inferred from elapsed time; no mature-data,
calibration, unattended-acceptance or general correctness claim follows. The next
question is whether this distinct installed model supplies useful independent
findings within the existing contract, not whether its output can approve itself.

## Implementation verification

No live model/verifier calls or candidate executions were made in this implementation
turn. Tests use fabricated loopback HTTP responses and disposable synthetic engine
state; the loopback stub tests needed sandbox escalation solely to bind ephemeral
ports. CLI library suite: **178 passed, 3 existing ignores**. After adding two final
boundary regressions, focused `local_` suite: **11 passed**. Checks:

```
cargo +1.90.0 test -p tsr-cli --lib --offline
cargo +1.90.0 test -p tsr-cli --lib local_ --offline
cargo +1.90.0 clippy -p tsr-cli --all-targets --offline -- -D warnings
cargo +1.90.0 build -p tsr-cli --offline
```

Coverage includes isolated input/role provenance, digest/endpoint rejection,
quarantine/no automatic adoption, stale source, replay, timeout, cancellation during
IO and after response, malformed/truncated/injected reports, invalid/missing usage,
partial HTTP metadata, exhausted budget and crash reconciliation before dispatch,
after uncertain dispatch and after recorded response. Restart never refunds unknown
work. Independent review found and verified fixes to late cancellation, concurrent
provenance reads, body-retention prohibition and compatible locking. Logs and review
summary: `target/local-verifier-transport/`. Protected files remain unchanged.

## Typed admission update

New local invocations now require `verifier-admission@v2` and per-criterion typed
prerequisites; the local transport configuration remains `local-verifier@v1`.
See [typed evidence and advisory closure](engine-verifier-evidence.md) before
preparing another admission. Historical v1 reports remain unchanged and do not
retroactively satisfy the new contract.
