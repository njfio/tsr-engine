# Generation and deadline root-cause review — 2026-10-22

Read-only independent reviews inspected the actual request/response artifacts and
source at `6580e6a`. No model call, notebook/project execution, altered desire,
authored runtime candidate or protected-file edit occurred. No reproducible contract
or synchronization defect was established, so no production or test code was changed.

## What the recorded generation actually says

`target/support-live/run/inability-32.json` reports inability with current information.
`inability-33.json` instead says:

> Unable to generate valid distinct possibilities that satisfy all standing wants without conflicts.

These are preserved typed inability records, not reconstructed explanations. Raw
transport response bodies are omitted under repository rules; validation metadata
and hashes remain. Both responses were complete (`done=true`, `done_reason=stop`),
with no tool calls. Independent review checked journal/request/response bindings.

The exact requests 32 and 33 have identical model, options, schema and system text.
They contain exactly two fresh messages: system and user. There is no assistant
conversation, previous inability field or prior inability explanation in request 33.
Thus “make the call stateless” does not by itself change this implementation.

The schema has no defaults or examples. It does not permit an empty directions
array: it requires two possibilities or an explicit inability object. Application
validation requires distinct first-action effects.
The inability branch is an honest fail-closed result, not a default empty answer.
Whether the easier branch biases this model is an untested hypothesis, not a reason
to remove abstention or count a forced malformed answer as progress.

The system explicitly says useful partial progress does not require simultaneous
full satisfaction, alternatives need not all be selectable improvements, and a later
controller assesses tradeoffs. The action contract allows deferred inspection of
unread evidence. The schema can express two distinct first effects: inspection and
composition of known evidence with a changed title length. This establishes
representability, not selected utility or real success. No witness was supplied to
the model or executed. Exactly two alternatives is not demonstrated to exclude all
feasible offers in this state.

Request 33's stated reason imposes a stronger all-wants/no-conflicts requirement than
its instructions. Model overinterpretation is consistent with this evidence, but
neither that cause nor the role of schema branching is experimentally isolated.
Historical effect/rejection summaries remain in the user context; misleading
conditioning by those summaries is likewise plausible but unproven. Reordering
context did not solve the observed inability. No new contract fix follows from this
review. The precise limit is this installed Qwen32B/configuration's repeated inability
to produce useful alternatives for this preserved conflicting-wants state, not a
proof of general model incapability or action infeasibility.

Read-only audit: `target/context-root-cause/request-audit.json`.
Source: `scripts/engine-directions.py`, `possibility_schema`, `possibility_context`,
`reconsideration_context`, and the two-stage request construction in `run`.

## Smallest materially different test — fresh admission required

A current-observation-only possibility projection would differ from the already
stateless conversation. It would omit historical return/rejection narratives from
only the possibility prompt, preserving them in immutable full context and in the
unchanged validator, reconsideration checks, assessment and controller. Keep exact
current authority, source/state binding, original/refined desires, known/unknown
notes, measurements, active scope, action rules, conflict and budget constraints.
Do not supply example actions, successful claim sets or a feasibility witness.

If separately admitted, bound this to at most two local calls: one possibility call
and, only if valid, its unchanged full-context assessment. Use the same pinned
Qwen32B digest `4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`,
temperature 0, context 32,768, output 2,048, and 240 seconds per call. No execution;
stop on inability/invalidity and expire the remaining slot. Freeze the projection
allowlist and compare every retained field against the original before dispatch.
This would discriminate sensitivity to historical narrative availability, not prove
reliability or causal attribution from one outcome. If inability persists, stop
this hypothesis rather than adding further hints. This plan is **not implemented,
admitted or dispatched**; all previous unused slots remain expired.

## Two-second stub deadline

The original parallel failure log shows all three HTTP replies complete, measured
150 tokens, and a terminal quarantine record. The outer resource guard then rejects
2,685 ms against a 2,000 ms admission. The stub has `delay=false`. It is not evidence
of malformed output, pin mismatch, cancellation, or provider incompletion.

`project_verifier_run` times all of `Verifier::evaluate`, including local freshness
checks, validated ledger/CAS reads, HTTP exchange, durable progress writes and
resource settlement. Complete HTTP receipt alone does not satisfy that elapsed-time
contract. Excluding bookkeeping to force acceptance would weaken the existing bound.

Independent isolated rerun passed (one test, 1.38 seconds total). A subsequent full
8-thread CLI run also passed: **197 passed, 3 ignored**, 44.42 seconds. Each fixture
has a separate temporary store, per-request lock and ephemeral listener; review found
no demonstrated shared lock or deterministic synchronization defect. The original
failure did not reproduce in this bounded rerun. Scheduling/storage contention is
plausible, but its exact contribution remains unknown. Deadline/assertions remain
unchanged; serial success is not presented as a repair or parallel-stability proof.

Original failure: `target/context-comparison-live/cli-tests.log`.
Parallel rerun: `target/context-comparison-live/independent-parallel-cli-tests.log`.
Source: `acceptance/reviewer.rs::project_verifier_run`,
`acceptance/reviewer/local.rs::Local::evaluate` and `local_tests.rs`.
If it recurs, the useful next diagnosis is test-local phase timing around validation,
transport and settlement, preserving the same deadline; no production optimization
is justified solely by this failure.

## Functional acceptance checklist

| Requirement | Current assessment | Evidence / remaining limit |
| --- | --- | --- |
| Standing interacting desires persist, including conflict and unmet wants | Implemented in bounded adapters | Shared obligation conservation, prospective losses, persistent state; request 33 retains all wants unchanged |
| Intuition produces genuinely useful directions under partial observation/conflict | **Not established for this case** | Requests 32/33 inability; representable actions are not model-generated success |
| Belief and expectation causally influence commitment | Implemented with bounded causal tests | Mechanism audit; operational calibration/support remains explicitly gated |
| Admitted action returns independently observed evidence | Implemented within scoped adapters | Real isolated repair and supplemental counterexamples; finite checks do not establish general semantics |
| Returns/refinements guide fresh next selection across restart | Implemented; live success is domain-bounded | Prior notebook continuations and shared-kernel controls; failed conflicting-wants continuation remains contrary evidence |
| Original wants and authority survive abstention/failure/closure | Verified for these records | Request state unchanged; explicit inconclusive closure retains unknown findings/costs and no label |
| Independent semantic acceptance without operator judgment | **Not generally delivered** | Typed verifier evidence fails closed; Mistral advisory errors preserved; manual review/domain checks still needed |
| Calibrated operational learning from mature independent episodes | **Not established** | 151 qualified corpus tasks do not establish mature eligible episode count; default fallback remains |
| Reliable unattended TSR cycle across admitted tasks | **Not accepted as complete** | Useful generation and independent semantic evidence are material limits, not solved by lifecycle/transport plumbing |

No missing user decision about the meaning of the original wants was identified.
The next optional experimental action needs a fresh bounded admission, while the
current evidence does not justify more code changes or another unchanged retry.
