# A1 synthetic challenge/provenance adapter

This extends the existing deliberation adapter with an optional simulated archive.
It does not contact people or mutate a project. Its purpose is to test whether a
proposal conditioned on the genuine generated A1 aspiration chooses a relevant
operation and revises that aspiration after an actual simulated return.

The original rich A parent texts and the exact A1 relationship, aspiration and
discriminator are preserved in `engine-synthetic-archive-inputs.json`. Symbolic parent
IDs are mapped to the two unchanged texts' current obligation IDs. Origin typed
proposal SHA256 is `c9463da09b5118a2c64f2df3f266b992e729a1930f2a57b517e893e67f499ad3`;
response-body hash is `6809dc6f8515bd4298bc21593959f3388fbc3d485f9cba743c0f85e9ad6949a0`.
No raw provider response is stored here. The new frozen input hash is
`612f479aea815883e97d9b3de772ab3ffeb5056114d068770b94d648b565b200`.

## What the adapter observes

`archive.rs` validates an immutable evaluator-authored world and appends typed
register, route, response and evidence-backed withdrawal events. Registration does
not require insider endorsement. Routing names an admitted affected actor. A response
must cite a matching admitted fictional record and route. Acknowledgment is unresolved;
substantive response is only `addressed_in_simulation`. Withdrawal requires matching
challenge-author evidence; none is admitted in this live fixture, so its schema omits
that operation. It cannot erase history or improve a score by shrinking a denominator.

The pre-evaluation criteria are in `engine-synthetic-archive-criteria.md`. They measure
partial registration, routing and response behavior. One challenge deliberately lacks
substantive response evidence and cannot become addressed. These records are fictional
evaluator inputs, not real testimony. An `addresses` flag is an explicit simulated-world
classification, not a general semantic verifier. Rich desire fulfillment stays unknown.

Normal `direction-request`, two-stage possibility/assessment, `deliberate`, shared
kernel selection, frozen forecasts, committed returns and `direction-relate` are reused.
An observed event makes old guidance pending reconsideration; fresh revision needs the
current return and evidence key. Parents, scope and history are retained across restart.
Archive transitions reject notebook actions and vice versa. Related-return learning
transfers by archive operation kind, a coarse engineering assumption; it does not borrow
note-inclusion effectiveness. Operation cost remains the existing bounded action proxy
`0.01/max_cycles`, not measured human effort or provider spend.

## Runnable deterministic setup

From repository root, with installed Rust 1.90.0:

```sh
cargo +1.90.0 build --offline -p tsr-cli --bin tsr
TSR_ARCHIVE_RUN="$PWD/target/synthetic-archive-fresh/run" cargo +1.90.0 test --offline -p tsr-cli --lib build_synthetic_archive_fixture -- --ignored
./target/debug/tsr engine --directory target/synthetic-archive-fresh/run direction-request
python3 scripts/engine-archive-prepare.py target/synthetic-archive-fresh/run/initial-request.json target/synthetic-archive-fresh/prepared.json
```

The builder requires a fresh directory. It executes one **evaluator-authored seed**
registration through the controller, then imports the genuine historical A1 proposal
against that synthetic return. This is scenario setup, not model-discovered behavior.
It leaves alternatives including routing the registered challenge and registering the
other challenge; no next action or winning sequence is supplied. The marker file records
this provenance. Existing prepared evidence is `target/synthetic-archive-v1/`.

The preparer has no network/credential code. It reconstructs an allowlisted initial
possibility request for `https://api.cheaperinference.com/v1/chat/completions`, model
`deepseek-v4.1-flash`, from exact frozen synthetic fields only. It excludes private
reviews, history, repository source, paths and arbitrary extra fields. It pins input
hash, initial event, parents and aspiration and emits a payload hash with
`dispatch_enabled:false`. This is not dispatch authorization or proof that a stale
request is current; the eventual runner must verify the live ledger root before send
and before application. It deliberately refuses later states rather than serializing
them through a permissive exporter.

## Proposed three-call continuation (not run or authorized by this implementation)

Each call: same provider/model, conservative 2048 output tokens, 240-second timeout,
existing 16384 context ceiling; no retries on uncertain completion. Record actual
usage/cost and retain unknown reservations. This turn makes zero calls.

1. Generate one or two possibilities from the prepared initial synthetic context.
   Preserve inability and validate typed operation, distinct effects, state and scope.
2. Freeze returned possibilities and independently invoke assessment with those exact
   actions, current obligation IDs and the partial-effect contract. No silent changes.
   Reuse normal frozen-identity completion and controller validation. At most **one**
   already-admitted reversible local synthetic operation may execute if selected.
3. After the actual return, request relational revision using the original parents,
   genuine aspiration, exact simulated before/after/events and current return binding.
   No invented response evidence or claims about real people. Persist through normal
   `direction-relate`; observe whether revision changes subsequent possibility context.

Stop on inability, invalidity, no admissible action, uncertain application or budget;
expire unused calls. Assessment and revision allowlist envelopes must be materialized
from the actual preceding stage and checked before dispatch. No preauthored successful
responses or subsequent operations are included. A single offered possibility tests
selection against abstention, not comparative selection among alternatives.

Independent semantic review should separately judge action relevance to both parents,
whether revision incorporates the particular return, and whether any claim exceeds
simulated evidence. A successful sequence cannot establish that A1 caused the choice:
the vocabulary and partial predicates already favor registration/routing. That stronger
claim requires a matched guidance contrast, outside this small continuation. Neither
fixture tests nor future three-call success demonstrate social fulfillment or TSR's
qualitative claims.

## Verification

Focused tests cover ignoring, stale provenance, erased/forged dissent, wrong actor,
unsupported and acknowledgment-only responses, duplicate events, evidence-backed
withdrawal, restart, stale offers, preserved parents and return-driven reconsideration.
Python tests cover disjoint action schemas, frozen assessment identity, source drift and
allowlisted payload isolation. The ignored builder exercises normal persistence without
model calls. Independent review caught missing withdrawal evidence; it was fixed with
matching author/challenge/revision citations and regression coverage, then re-reviewed.

Verified commands on this change:

- `cargo +1.90.0 test --offline -p tsr-cli --lib`: **207 passed, 0 failed, 5 ignored** (45.78s). Local mock-server permissions only; no live providers/project targets.
- Focused archive tests: **2 passed, 0 failed, 1 ignored**; explicit fresh fixture builder separately **1 passed**.
- `python3 -m unittest scripts/test_engine_archive.py scripts/test_engine_directions.py -q`: **87 passed**. Existing mock HTTPError cleanup ResourceWarning retained; no failure.
- `cargo +1.90.0 clippy --offline -p tsr-cli --lib -- -D warnings`: passed.
- Offline binary build and rebuilt-binary `direction-request`: passed; prepared payload identical across restart, SHA256 `92bc03c77eed0734dc68839956bbc7229f7e3bb3f7760bfcd8ccab7d2a9bcad3`.
- `git diff --check`: passed. Changed Rust files formatted only; unrelated protected formatting untouched.

Full output: `target/synthetic-archive-v1/{cli-lib,clippy,python,fixture-build}.log`.
This is affected-library validation, not a new whole-workspace sweep or live relational result.
No model/provider requests, external charges or temporary provider services this turn.
