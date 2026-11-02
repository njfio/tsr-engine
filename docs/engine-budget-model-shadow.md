# Low-cost model shadow comparison

Authorized 2026-10-25: two requests each for `deepseek-v4.1-flash`,
`glm-5.3-flash`, and `qwen3.8-flash`. No premium substitute, generated target
execution, adoption or B9.1 live trial is authorized by this comparison.

## Compatibility and frozen design

`engine-synthetic-transport.py:invoke` now accepts keyword-only `expected_model`.
Its default remains DeepSeek. A finite three-model allowlist and exact request/response
model checks prevent the payload from selecting its own authority. `x-ci-model-lock: 1`
remains required by the transport. Existing drivers/admissions still pin DeepSeek;
this change does not silently switch them. The caller must separately admit the model.
No schema, source, review, accounting or action gate is weakened.

Manifest `target/b91-budget-model-shadow/manifest.json` has SHA-256
`981815ac18716c0a829e46db09ee06b50825750c27467f543ebd8199433a4591`.
It pins runner, implementation, validator binary and historical inputs. Order is
DeepSeek → GLM → Qwen, then Qwen → GLM → DeepSeek. Maximum six calls, 8192 completion
tokens and 240 seconds each, zero executions/adoptions. Each measured overrun or
unknown dispatch/accounting stops continuation; no retry or replacement call.

The same retained synthetic failed-seed snapshot, both desires, admitted initial
aspiration, observed return and schemas are used for all models. Only `body.model`
differs. Rejected generated tests/review notes and protected test source are excluded.
No successful assertion is supplied. This is a current source-bound projection, not
a claim to recover the old wire request. Source-role treatment from the previous
comparison is not added.

Optional temperature/top-p/reasoning controls are omitted for all models, using
advertised provider defaults. This avoids assuming unsupported parameters, but defaults
are not presumed equal or deterministic: this compares model services as configured,
not isolated architectures at equal reasoning effort. Streaming is false; output cap
and model lock are explicit. Existing gateway defaults are unchanged.

Public metadata preflight must confirm supported endpoint, structured-output feature,
output capacity and rates within 5% above the approved quotes. Quotes, USD per million
input/output tokens: DeepSeek 0.0825/0.3300, GLM 0.0825/0.2750,
Qwen 0.0975/0.3055. This is a rate check, not a guaranteed total spend cap.

Frozen review criteria: expected values consistent with immutable requirements;
nonvacuous behavior; meaningful nonempty second-aim coverage; inversion/wrong aim;
JSON/native structure separately; truthful evidence and finite limits. A separate
reviewer receives neutral candidate labels without model/order/receipt information.
No generated code will compile or run. Source-derived predictions remain predictions.

Independent pre-dispatch review found no blocker. Offline tests cover all three exact
model pins, wrong response models, default refusal of alternatives, rejection of
premium/unapproved pins before credential lookup, retained accounting and one request
per invocation. No raw provider response bodies, reasoning or credentials are logged.

Preparation verification: 21 focused tests passed in 5.162s; complete offline Python
suite ran 614 tests in 144.284s, **597 passed, 17 skipped, zero failures**. Existing
HTTPError/SQLite cleanup ResourceWarnings remain. Logs are
`target/budget-model-compat-checks.log` and `target/budget-model-full-python.log`.
No live model or target execution opt-in was enabled for these tests.

## Actual terminal outcome — comparison inconclusive

Compatibility preparation committed as `9cc2343`. The fresh process verified TLS
certificate/hostname checking and absence of the affected emSign root. All three
public quotes matched exactly at 11:53:50–51 UTC. Three requests dispatched, then
the frozen overrun guard stopped the run and expired the remaining three:

| Model, first repetition only | Provider response | Completion / reasoning tokens | Settled USD |
|---|---|---:|---:|
| DeepSeek | Typed JSON, `stop`; later local validation refused | 504 / 0 | 0.000507 |
| GLM | Typed JSON, `stop`; later local validation refused | 508 / 187 | 0.000434 |
| Qwen | Typed JSON, `stop`; later local validation refused; token overrun | 11107 / 10805 | 0.004224 |

Qwen returned after 234.277 seconds, inside the 240-second deadline. Its reported
11107 completion tokens exceed the explicit `max_tokens:8192` request; only 302 were
non-reasoning tokens. This is **not** a timeout or uncertain response. Whether that
route interprets max_tokens as visible output rather than total reasoning+output is
unestablished. No claim that the provider honored a total completion cap is made.
The measured overrun stop worked. No retry or second repetition occurred.

Total: **$0.005165 settled**, no unknown charges, 7687 prompt + 12119 completion =
19806 tokens. Zero generated execution/adoption. All six-request authority is now
consumed or expired; no continuation is authorized.

### Demonstrated local evaluation defect

The runner reused Trial 10's read-only native validator after that episode's wall
budget expired. Its ledger root still matched, but resource authority no longer did.
A previously valid Sable candidate from the earlier comparison also fails with:
`blocked resource authority; reconcile before proposal application` (native exit 2).
Read-only budget inspection showed elapsed wall 4873945ms versus limit 3557215ms,
`stopped:true`, `exceeded:true`, and no unsettled model/tool requests. No budget was
renewed and historical records were not changed.

This is a local readiness mistake, **not evidence that all three models generated
bad tests**. Outer envelope checks passed; all later failures were recorded only as
ValueError, so the precise original rejection path and actual generated assertions
cannot be recovered. Raw bodies are not retained, and failed native validation did
not produce typed candidate artifacts. The valid-control refusal establishes that
this validator was unsuitable regardless of model quality, not that every original
proposal would otherwise pass.

The independent blinded reviewer consequently reports semantic quality **unknown for
all three**, with no assertions available to judge inversion, vacuity, second-aim
coverage or truthful inference. It did not inspect model assignments. There is no
empirical winner and no basis to switch the engine on these results.

### Offline correction and remaining readiness

Added `require_shadow_validation_ready` as a read-only pre-dispatch prerequisite for
shadow tooling that relies on this native validator. It fails closed on inactive,
stopped, exceeded, missing or unresolved resource state. Tests cover readiness,
expiry between checks, missing fields and unknown requests, without renewal/admission
or transport. Independent review additionally required sufficient remaining wall time
for the admitted call deadline plus the native validator’s 30-second timeout; that
check and exact boundary/unknown tests are included. Readiness is checked anew, not
cached; concurrent changes can still require final refusal. The actual expired Trial
10 store is refused before dispatch by it.
21 affected tests passed in 5.224s (`target/budget-shadow-final-checks.log`).

The frozen consumed runner is intentionally unchanged. The new guard must be wired
into any newly prepared comparison before its first and subsequent calls; it does
not retroactively repair this run. A fresh **validation-only**, non-executable context
or separately reviewed offline structural validation is needed to assess future
shadow candidates without reusing expired live authority. Also needed: stage-specific
local rejection diagnostics without raw response retention. No existing grant is
renewed by these engineering requirements. Qwen's total-token behavior needs a verified
control before further calls under the same total-output bound.

Recommendation: keep DeepSeek unchanged for now. GLM remains a price-compatible
candidate, but this run establishes transport/metering only, not semantic superiority.
Do not repeat this comparison until validator readiness and Qwen's token bound are
resolved offline/documentarily and a fresh comparison is authorized. Six samples
would still not establish broad reliability even if all had completed.

Local receipts: `target/b91-budget-model-shadow/{manifest.json,pricing.json,events.jsonl,
summary.json,launch.log}`. Exact request/configuration hashes and structural quarantine
are retained; no raw bodies, credentials or reasoning text. Historical Trial 10
state/journals/reviews and all 27 previously captured Trial 1–9 terminal hashes remain
unchanged. Prior unrelated unknown reservations are not erased by this run's known
billing. No new full-suite pass is claimed after the final isolated preflight helper;
the complete suite result above applies to the model-pin preparation, followed by
the 21 affected checks for the helper.

## Fresh validation-only runner (offline repair)

`scripts/engine-b91-shadow.py` now owns the comparison path rather than leaving a
standalone guard for a later script to integrate. Preparation creates a fresh native
project from the exact evaluator-authored failed source and published requirement
mapping, without any native execution/resource admission. The original scenario copy
is retained intact; a separately identified validation source carries the failed code.
Original request/source provenance and the new validation state are linked explicitly.
Historical live ledgers and the consumed comparison runner are unchanged.

Before every possible dispatch the runner checks current native resource counters,
source/state identity, and its own finite validation deadline; runs an authored,
explicitly vacuous structural control through envelope, binding and native validation;
then checks the run admission's 270-second remaining runway again immediately before
writing dispatch intent. The control is never sent to the model. It is not evidence
of semantic correctness. No execution or adoption is exposed by the runner.

Payloads must exactly equal the previously approved synthetic projection, apart from
the pinned model ID. Rehashing an injected private payload cannot admit it. Implementation
and validation bindings are hashed, uncertain dispatches retain their journal and block
restart, and admitted candidates retain original/fresh provenance with semantic status
`unreviewed`. Schema and native rejection stages are recorded separately. Authored
inverted, vacuous and wrong-aim controls reach native structural validation and are then
rejected by the existing independent-review gate using explicit fixture verdicts;
these tests do not claim automated semantic judgment or real model performance.

Offline commands (a new directory is required for preparation):

```sh
python3 scripts/engine-b91-shadow.py prepare --directory target/b91-shadow-offline-example
python3 scripts/engine-b91-shadow.py inspect --directory target/b91-shadow-offline-example
PYTHONPATH=scripts python3 -m unittest test_engine_b91_shadow
```

Preparation sets `live_authorized:false`. Inspection does not dispatch. A future run
requires a separately reviewed, freshly approved manifest and paid opt-in; no existing
allowance is reopened. All default three-model manifests are additionally blocked from
live dispatch while Qwen's provider-specific total-token bound is unverified.

## Qwen accounting and control documentation

Official documentation checked 2026-10-25:

- [Alibaba OpenAI-compatible Chat API](https://www.alibabacloud.com/help/en/model-studio/qwen-api-via-openai-chat-completions)
  says Qwen's `max_tokens` excludes reasoning. It documents `max_completion_tokens`
  for Qwen Flash 3.5 and later as answer plus reasoning, with a possible ten-token
  counting difference. Third-party direct-supply routes are excluded from that
  compatibility statement. Usage output includes reasoning; reasoning is a subset,
  not an additional count to add to output. A separate `thinking_budget` exists,
  but does not establish this broker route's total limit.
- [CheaperInference API reference](https://www.cheaperinference.com/api-reference)
  accepts `max_completion_tokens` as an alternative output limit, requiring matching
  values if both limit fields are supplied. It does not establish that this exact
  marketplace Qwen route preserves Alibaba's total-output semantics. Its settled
  `billing.billed_cost_usd` is distinct from token usage; usage alone is not a bill.

The preserved report remains **11,107 completion tokens, including 10,805 reasoning
tokens**, with reported settled charge **$0.004224**. This exceeds the experiment's
intended 8,192 total-output bound, but does not by itself show violation of Qwen's
answer-only `max_tokens` semantics. No amount or usage has been rewritten. The whole
prior comparison remains $0.005165, inconclusive, with no model ranking.

There were no inference probes in this investigation. A documented total control on
Alibaba's own endpoint is not proof for CheaperInference's route. Exclude Qwen from
a newly approved comparison unless the exact route's control (including counting
slack) can be verified and separately admitted. Do not silently change providers,
settings, limits or claim a guaranteed cap.

### Offline evidence and review

The real native validator is exercised through the actual assembled fixture request
and transport runner, not a homogeneous mock validator. Tests cover six fake responses,
model pins, source drift, wrong protected paths, missing second-aim mapping, independent
fixture semantic refusals, private payload substitution even after rehashing, expired
headroom during preflight, uncertainty/restart, and live Qwen blocking. No target code
or provider is run. Independent review identified and then verified fixes for a final
runway recheck and a missing transitive implementation pin (`engine-b91-evidence.py`).
It found no remaining offline correction blocker, while explicitly not approving live
use or claiming model competence.

Fresh-process preparation and two inspections at
`target/b91-shadow-offline-v2-final` pass the native control with no dispatch journal,
no execution and no semantic acceptance. Manifest SHA-256:
`1b1793ee388ce69147853a754739495eeb6194264fe9e603f602cb6c9828e0e1`.
All 27 captured terminal hashes for Trials 1–9 still match. The historical comparison's
source/ledger artifacts also match; its implementation pin for the envelope differs
because the already-committed preflight repair intentionally changed that code, not
because historical evidence was modified.

Final verification: 23 focused tests passed in 25.372s before the final separate-source
copy adjustment; the complete Python suite on the final implementation then ran
**624 tests in 169.588s: 607 passed, 17 skipped, zero failures**
(`target/shadow-v2-full-python.log`). The full run includes the real-controller shadow
suite. Existing cleanup ResourceWarnings are still emitted; they are not test failures.
No new Rust source changed; native acceptance/rejection is exercised using the pinned
existing controller. No full Rust sweep is claimed in this pass. No provider inference,
new charge, target execution, adoption, protected edit or live admission occurred.
