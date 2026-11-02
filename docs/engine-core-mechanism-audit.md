# Core mechanism audit (from a78d9df)

This maps operational mechanisms, not terminology or passing tests to overall
completion. PRD §B2 explicitly makes Belief a momentary action commitment with
frozen forecasts. It does not specify a truth-maintaining semantic hypothesis
system. Adding one would require a separate domain contract; it is not silently
substituted here.

| PRD mechanism | Status | Operational evidence / limit |
| --- | --- | --- |
| Persistent desires, status and lineage (§B6/B8) | Implemented, bounded | Ledger fold plus notebook/project admission and status; generated notebook refinements are verified coverage decompositions, not new fundamental values. |
| Joint desire assessment | Implemented | `tsr-domain/src/decision.rs::assess/assess_path`: conserved original weight, standing loss, uncertainty and intermediate costs; notebook and project adapters share it. |
| Intuition: select/tensions/compose (§B8) | Partial | `deliberation_request`, ledger `select`, two-stage local driver; composition failed on preserved live state across tested Qwen/Mistral configurations. No useful alternative was manufactured. |
| Quiet attention (§B8) | Implemented operator setting | `update_selection_tune` exposes existing shared ledger route, durable and replayable. Not autonomous learned attention. |
| Belief: dispatched commitment and frozen forecasts (§B2/B6) | Implemented | Notebook `deliberate` records chosen action/forecasts then return. Project execution separately binds receipts and acceptance. This is not source-truth verification. |
| Revisability of commitment under revised evidence | Implemented, bounded | Notebook `current_commitment` releases continuity after evidence revision or unproductive return; project `project_rank_options` requires matching current source/return and releases on rejected, unresolved or stale review. This is action-support revision, not semantic hypothesis inference. |
| Expectation from observed returns (§B8) | Implemented proxy / calibration incomplete | Related-return probability/residual folds and applicability/support reporting; mature validated common-event calibration is not established. 151 qualified tasks do not prove mature episodes. |
| Surprise versus Contrast (§B6/B8) | Implemented, bounded | Return stores residuals separately from unmet-dimension contrast; only contrast admits bounded refinements. |
| Observe/action/return/restart loop | Implemented within adapters | Versioned notebook observations and admitted foreground collector; real scoped coding receipts and manual semantic review. Generation remains an operational bottleneck. |
| Independent semantic acceptance | Partial | Project operator review is independent of generator and required for accepted export. Notebook coverage/brevity is not truth or understanding. |
| Policy-aware learning attribution | Implemented, bounded | [Saved-draw interventions](engine-policy-learning-interventions.md) preserve factual support and expose changed eligibility separately. Stored comparisons are model-state controls, not unobserved outcomes or empirical causal identification. |
| Near-tie margin and action exploration (§B8) | Implemented, explicit activation | Shared versioned selection over eligible notebook and adaptive/project candidates, margin abstention and opt-in epsilon with recorded propensity. Existing unconfigured runs retain legacy behavior; proxy scores, not calibrated Tier 2. See [selection evidence](engine-near-tie-selection.md). |
| Calibrated Tier2, propensity-based superiority (§B8/B9) | Not established | Requires qualifying mature common-event data and validation, not another notebook demonstration. |
| Semantic hypothesis revision, new-value invention | Not specified by current operational contract | No generic fact-truth claims or arbitrary values are added. |
| Event-structure/Hodge/autocatalytic research (§B9.1) | Deferred/gated | Not chosen as a substitute for the functional decision cycle. |

## Selected correction

Release prior commitment continuity when its independently observed evidence
revision no longer matches current notebook evidence. Preserve the historical
commitment and return, and keep related experience available to the existing
expectation mechanism. Expose a read-only commitment review explaining whether
there is no commitment, its evidence was superseded, its return did not support
progress, or it is supported by the current return. Changed evidence is not itself
proof that the old action was wrong; the old support is simply no longer current.
No desired condition is weakened and no new action is authorized.

Project support now additionally requires a matching productive receipt for the
current source and a `review_required` or `accepted` status. Before review this is
explicitly provisional support, not semantic acceptance. Missing requirements,
unresolved/rejected review or stale review conservatively release continuity.
`OptionView.commitment_supported` exposes eligibility; the kernel may still impose
zero continuity cost when all supported wants are fulfilled. Historical returns
remain available to expectation learning. Notebook adapter version v10 prevents
old v9 traces being used as same-semantics learning counterfactuals.

## Verification and remaining work

- CLI library suite: 101 passed, one existing diagnostic ignored.
- Clippy with warnings denied, CLI build and diff whitespace checks passed.
  Independent read-only review found no remaining blocker after the v10 bump.
- Notebook restart/history control preserves original wants and receipts after
  revision. A controlled comparison fixes observations, offers and expectation
  inputs: revised support changes selection; restoration restores selection.
  This is a deterministic causal test, not another live generation claim.
- Project control changes only disposable source bytes, releases support, verifies
  restart, restores bytes and recovers identical options. Independent rejection
  then releases support durably; existing semantic expectation transfer checks pass.
- Logs: `target/commitment-cli-tests.log`, `target/commitment-project-tests.log`,
  `target/commitment-clippy.log`, `target/commitment-build.log` (local artifacts).

The principal operational gap remains useful possibility generation in the
preserved conflicting-desire state, not missing truth-system terminology. Tested
installed providers reported inability; no live slots were used in this change.
The completed [envelope audit](engine-envelope-admissibility-audit.md) found
wording and first-action distinctness mismatches and corrected them without new
inference. Prior inability remains evidence under the old contract, not proof of
universal model incapability. Shared near-tie margin and propensity-aware selection are now implemented
with explicit operator activation; see [selection evidence](engine-near-tie-selection.md).
The older argmax-only learning audit is suppressed under that policy and replaced
by stored [saved-draw learning interventions](engine-policy-learning-interventions.md).
Pre-action common-event shadow receipts and support/provenance checks are now
implemented without factual selection influence; see [shadow integration](engine-common-event-shadow.md).
The [project lifecycle evidence bridge](engine-project-lifecycle.md) now preserves
prospective admission, tool usage and review-bound closure. Numerical compatibility now uses shared schema dispatch; operational support
remains gated on eligible evidence. [Receipt telemetry](engine-project-telemetry.md) now provides a
versioned observation schema and prospective instrumented local budgets. A matching
dataset/model contract, validated artifact and independently qualifying evidence
remain concrete dependencies; legacy/uninstrumented coverage remains unknown.
Notebook coverage remains a separate event; no new native calibration is claimed. Validated
data/bundles and useful generator output remain distinct dependencies. Neither
this correction nor configurable quiet attention establishes autonomous learned
attention. Mature common-event calibration additionally needs independently
reviewed, deduplicated, close-plus-seven-day eligible episodes; the eligible count
is unknown, while the 151 qualified corpus tasks remain recorded.

Receipt evidence has a [reconstructible dataset and numerical bridge](engine-project-receipt-datasets.md)
using the shared runtime extractor and frozen causal prefixes. Builder, maturity
reconstruction, leakage probes and pre-action application dispatch by schema.
Prospective eligibility reconstructs original admission, independent review and
complete instrumented resource evidence; synthetic/replay sources remain excluded.
Inventories stay unlabeled and only separately validated eligible evidence can
supply a dataset label. Operational model fitting now depends on actual mature,
independent data and support, not an export-only implementation gap. No qualifying
production count or fitted operational model is claimed. Provider-generation limits
and manual independent project review remain separate practical dependencies.


## Remaining work after receipt integration

- **Compatibility decision resolved (2026-10-22):** the user approved the two
  command-only cutoff migrations. Commit `a706950` adds simulated
  `--as-of 2028-01-01T00:00:00Z`; fixtures/assertions and maturity policy are unchanged.
  All eight corpus tests and eleven maturity tests pass. Original migration rationale is
  documented in [maturity compatibility](engine-maturity.md#legacy-cli-compatibility-repair).
- **Operational evidence:** mature independent common-event support and validated
  calibration are absent, not replaced by 151 qualified tasks or synthetic labels.
  The implemented prospective lifecycle, receipt schema and review path can collect
  this evidence in already admitted isolated projects. An independent operator must
  attest semantic review and complete measured coverage; actual time must elapse.
- **Tier 2 integration:** [explicit gated activation plumbing](engine-tier2-activation.md)
  now reruns operational provenance, maturity/support, applicability and held-out
  validation before using common-event probabilities. Default remains shadow/fallback;
  no operational activation is established. Optional pre-action generator forecast
  collection now supports the required paired baseline. Real calibration/activation
  still depends on eligible operational evidence; tests exercise mechanics only.
- **Provider capability:** preserved local composition/search attempts failed to
  produce useful feasible alternatives in the conflicting-wants state after the
  recorded interface repairs. This remains an intuition-generation limitation;
  receipt integration and additional calibration claims do not solve it. No new
  inference is authorized in this pass.
- **Domain semantics:** project semantic acceptance remains independent manual
  review; notebook coverage is not understanding. Broader semantic verification
  requires an explicit domain contract, not an invented truth checker. Quiet
  attention remains configured, not autonomously learned. Research/superiority
  claims and new fundamental value invention are not functional-completion tests.

Safe next steps are the narrow synthetic invocation migration if authorized,
verification with eligible evidence once available, and prospective collection of independently reviewed, instrumented operational episodes
through the existing local adapters. No hosted calls, additional corpus access or
unattended external authority follow from this plan. The tested activation plumbing cannot substitute for missing operational evidence;
no usable calibrated model or whole-system completion is claimed.

## Prospective collection follow-up

[Five-stratum collection preparation](engine-prospective-collection.md) separates mutation task provenance from live versus synthetic/replay execution. Explicit prospective training admission now records pre-action evidence without requiring an existing model bundle. No operational episode or mature label was created. The next concrete independent implementation gap is receipt-derived normalized expected tokens/minutes for selection; current fixed action-cost proxies and budget gates do not implement that PRD B8 mechanism. Out-of-band full-workspace oracle accounting also requires integration before claiming complete collection coverage.

## Resource cost and verification follow-up

[Budget-normalized execution costs](engine-resource-cost-selection.md) now feed the shared project kernel from prior compatible complete receipts, with explicit bound fallback, sunk resources and provenance. Preparation and optional full-workspace oracle execution are included in cumulative accounting and reconstructed eligibility. This closes the concrete cost/verification plumbing gaps above within the project adapter; it does not establish calibrated completion costs or remove prospective scope/reviewer/data/provider dependencies.

## One-task readiness audit

Independent review at447b247 found no additional defensible core mechanism blocker independent of provider usefulness, mature evidence and domain semantics. [Concrete RTS admission](engine-one-task-readiness.md) closes the vague scope dependency. Remaining measurement gap: prospectively record active operator/reviewer minutes; current receipt cost covers tool/token costs, not human effort. Do not infer that metric from timestamps. The prepared development task is not a fresh holdout and has no episode/closure/maturity yet.

## First prospective task outcome

[The admitted RTS episode](engine-one-task-outcome.md) closed rejected after 3 local calls and 2 sealed executions; no test or semantic acceptance was obtained. Accounting, independent rejection and closure remained enforced. Supporting implementation context was unavailable outside the edit window; optional bounded read-only spans now address that adapter limitation for future admissions, without changing edit authority or the closed episode. Provider recovery is unverified. Active reviewer effort and complete-coverage attestation remain unavailable, with eligibility blocked.

## Final readiness reconciliation

[The audit after 381b6d8](engine-readiness-final-audit.md) distinguishes the two rejected real named-list episodes from earlier notebook and supported_languages trials. No further local core correctness defect was demonstrated. A bounded comparison with the already-installed 32B model remains unrun on this specific task; no hosted provider is yet required. Mature operational evidence, complete reviewer-effort instrumentation and independent domain review remain explicit dependencies. The protected cutoff invocation decisions and one protected formatting line are separately identified. No full-system reliability, calibration or acceptance claim follows from implementation coverage.

## Qwen32B comparison and newly observed integration gap

[The exact-task Qwen32B comparison](engine-qwen32-task-comparison.md) produced a statically correct model-authored candidate and a named-test pass in one call. Full-suite verification timed out; promotion was rejected without claiming incorrect code. A warm rerun of the same edit was correctly refused as stale. A source-bound verification-only action is now a concrete local integration gap; the former audit finding of no demonstrated gap is superseded by this live evidence. No extra inference is needed to implement that action contract, and stale-edit checks must remain intact.

## Post-acceptance audit (after a372bdc)

The exact Qwen32B repair now has passing default-suite evidence and independent
semantic acceptance; [termination/environment evidence](engine-executor-termination.md)
records the prior failures and corrections. [Verified origin export](engine-origin-export.md)
closes the observed empty-export provenance gap without turning verification into
another edit or rewriting rejected episodes. This is executor/evidence integration,
not proof of the broader decision goal.

Ranked remaining gaps and dependencies:

1. **Core correctness, repaired in this pass:** requirement aliases could duplicate
   one desire identity and break shared ranking. `acceptance::validate_requirements`
   now rejects new duplicates; `acceptance_kernel_inputs` conserves legacy aliases
   and aggregates contradictory evidence conservatively. Restart and causal evidence
   controls verify useful behavior, not merely prompt inclusion.
2. **Intuition usefulness, still partial:** the existing notebook loop generates
   directions from standing wants, observes return, refines and reconsiders through
   shared assessment. Preserved conflicting-wants attempts still contain provider
   inability; one successful coding repair does not erase them. [The subsequent bounded continuation](engine-partial-continuation.md) used two
   pinned Qwen32B calls on that exact preserved state, before/after a reviewed
   feedback-provenance correction. Both ended in inability; no action or learning
   was fabricated. The remaining question is useful generation under partial
   observability, including whether the prerequisite planning contract is too
   demanding before inspection. No policy relaxation is established by inability.
3. **Cross-project continuous choice, not established:** project acceptance return
   writes surprise/contrast and desire status; stale-source fulfillment reopens in
   the adaptive view. `acceptance_cycle` exposes corrective readiness and explicit
   carryover retains reviewed experience. The coding driver still requires published
   feedback and stops at acceptance. It does not autonomously select a later project
   goal. That distinction is real. The generic bounded notebook cycle is broader
   than a repair ticket, but its coverage predicates are not general understanding.
   Do not silently bypass explicit context publication or invent new project access
   in order to call this unattended autonomy.
4. **Domain review dependency:** independently reviewed semantic judgments remain
   operator-assisted. Mechanizing a new semantic domain requires an admitted
   observable contract; generator self-approval is not a substitute. A useful local
   implementation next step is explicit prospective reviewer-effort instrumentation
   (start/pause/stop with unknown idle intervals), not inferring effort from elapsed
   task time. This is evidence measurement, not a new TSR decision theory.
5. **Operational calibration and superiority:** eligible mature independent episodes,
   measured resource coverage, common-event support and held-out validation remain
   unestablished. Preserve the151 qualified tasks fact; neither those tasks nor this
   accepted replay export establish mature support. Tier2 remains gated/off by default.
6. **Protected test decisions:** the two cutoff invocation migrations were approved
   and completed in `a706950`. The unrelated protected formatting difference remains
   unchanged. Neither grants operational maturity.

Desire, Belief and Expectation are causally wired through conserved assessment,
standing loss, revisable commitment support and observed-return learning. Intuition
has genuine local-generation evidence but bounded usefulness and domain scope.
No additional original-PRD core defect independent of those dependencies was
established in this audit; optional research mechanisms/new-value invention must
not be substituted as arbitrary completion gates. The repaired alias defect is the
highest-priority deterministic core improvement identified here. The next concrete
core validation is an unchanged-state continuation on the preserved conflicting-wants
case when local inference is admitted; further export/test milestones cannot resolve
that generator limitation by themselves. Whole-system or unattended completion is
not claimed.

## Partial-observability correction

[Inspect-first fallback](engine-inspect-first.md) closes the demonstrated adapter
restriction requiring concrete terminal content before relevant unknown evidence
could be acquired. It uses original-bound dependencies, exact first effects and
cost-only fallback ordering after ordinary productive options. Independent review
and deterministic full adapter/restart tests pass. Genuine provider recovery remains
unverified; the preserved live run and its unused call slot are unchanged.

[The genuine inspect-first continuation](engine-inspect-first-live.md) subsequently
stopped at possibility-generation inability on its first of four admitted calls.
The state and desires were unchanged; no assessment or action occurred. The
mechanism is deterministic-test verified, but provider recovery is still not
established. The old unused slot was expired and no allowance was pooled.


## Subsequent attention (2026-10-22)

`project attention` projects the next admitted residual desire using the existing
lineage selector, current source-bound test/review observations, and ledger status.
See [contract and checks](engine-next-attention.md). It exposes commitment and
resource blockers without inference or new authority. The normal proposal path now consumes a separately allowlisted, frozen attention
binding and records consumption. Unsupported registered routes remain ineligible
without crashing the residual-desire fold. This does **not** establish provider
usefulness or automatically launch a next episode. The next verification is the
two-call fresh invoice plan in that contract, requiring a new bounded live admission.
No new goal, execution or model call is implicit in attention selection.


### Fresh attention continuation verified

The bounded invoice validation on `13c0ce6` consumed two genuinely generated local
proposals and two isolated executions. Discount repair left subtotal unmet; observed
return and separate review led to subtotal attention, its repair passed, and final
review left no eligible residual. [Evidence and limits](engine-attention-live.md).
This removes the specific unverified integration gap. Independent semantic review
remains manual. Unchanged published guidance now rebinds through an explicit scope/source provenance fold (fresh local continuation verified without republication; see engine-guidance-live.md); operational mature data and
general provider reliability are not established by this synthetic success.

### Verifier contract integration

The [quarantined verifier boundary](engine-verifier-contract.md) now joins explicit
operator admission, source/criteria/evidence-bound requests, durable invocation
receipts, typed findings and separate adoption to the existing acceptance/return
loop. Unknown/disagreement do not create failure pressure; known observations retain
per-desire distinctions with conservative alias conjunction. This removes a missing
contract/integration layer, not the actual evaluator capability dependency.
The [bounded local transport](engine-local-verifier.md) now enforces configuration,
source isolation, deadlines, reservation reconciliation and quarantine. Stub regressions
verify transport mechanics; no live verifier competence has been established. Review
remains operator-adopted, automated acceptance disabled. The next dependency is the
explicit two-request shadow evaluation and domain-sufficient semantic coverage.

### Live advisory verifier boundary

[Two pinned local Mistral shadow calls](engine-verifier-shadow.md) completed with
valid schemas, but only 3/6 correct criterion labels and fabricated passing receipts
in both cases. No adoption occurred. Transport mechanics are now live-observed;
semantic verifier competence remains a demonstrated dependency. Next concrete
correction: typed observed-evidence prerequisites, without silently rewriting
findings or claiming that it fixes arithmetic interpretation. Advisory-only native
lifecycle closure also lacks a non-acceptance terminal path; review was not forged.

### Typed evidence and advisory lifecycle follow-through

[Typed prerequisites](engine-verifier-evidence.md) now distinguish static inference,
observed tests and mixed findings. Current immutable receipt/source/suite/outcome
binding blocks unsupported observed-success claims; prose is not a proof channel.
The prior two shadow episodes now have advisory-only closure, unchanged reports,
charges and unresolved obligations, and explicit training ineligibility. Arithmetic
semantic competence remains unestablished; no additional model calls were made.

## Supplemental semantic evidence (2026-10-22)

The [finite-check evidence path](engine-supplemental-evidence.md) now binds a
separately authored/reviewed check mapping, unchanged candidate, protected harness,
prospective check episode and immutable receipt. It can refute a mapped arithmetic
requirement; finite success remains unknown for universal semantics. Current
counterexamples invalidate verifier requests, veto contradictory `met` claims and
reopen the current acceptance projection without rewriting historical review.
Explicit operator withdrawal of a mistaken mapping is append-only. This is a
concrete return/acceptance integration correction, not new generation competence.

No live model or project execution occurred for this change. The next operational
validation is a separately admitted, zero-model isolated finite-check episode on
a preserved synthetic candidate. That can establish bounded counterexample
transport, not universal correctness, M4's two-desires gate, mature calibration,
or superiority. Preserved conflicting-wants inability and Mistral shadow failures
remain contrary evidence. No further generic supervisor or semantic truth engine
is justified by the PRD audit; useful composition and independently eligible data
remain the principal dependencies.

The [subsequent single supplemental episode](engine-supplemental-live.md) imported
actual bounded arithmetic counterexamples without inference. It also found and
corrected the future check template's newline mismatch. This is evidence-path
validation, not resolution of the conflicting-wants generation failure. Inspection
of preserved request 32 supports testing a possibility-stage state-first context:
current authority/obligations/effects move ahead of historical forecast diagnostics,
with all actual outcomes retained and full context archived. Deterministic controls
verify representation preservation; no model recovery is claimed or tested yet.

The [state-first comparison](engine-context-comparison.md) is now terminal: one
pinned Qwen32B possibility call returned inability, the unused slot expired, and
the exact entire direction state remained unchanged. No assessment or execution
ran. This representation hypothesis did not resolve the generation limit; no
unchanged retry or success claim is justified. Verification-only inconclusive
closure is a distinct lifecycle correction, not a substitute for useful Intuition.

The [independent root-cause review](engine-root-cause-review.md) found the proposal
call already conversation-stateless, no schema defaults/examples or previous
inability text, and explicit permission for partial progress. The model's typed
request-33 reason nevertheless demands all wants without conflict. No reproducible
contract defect was established. The updated functional checklist therefore keeps
useful conflicting-wants generation and general independent semantic acceptance
open; it does not equate more lifecycle plumbing with TSR autonomy. The known
2-second stub failure did not recur in a bounded full 8-thread rerun; exact delay
attribution remains unresolved and no deadline/assertion was weakened.

The opt-in [historical narrative ablation](engine-narrative-ablation.md) used one
local Qwen32B call and returned the same inability. The unused slot expired, state
remained unchanged and runtime stopped. Default projection and full assessment stay
unchanged. No useful alternative was established; probing this pinned configuration
is stopped. Installed alternative inventory is documented without inference or a
claim that availability implies competence.

## Corrected priority: relational culmination

The user's question about desires becoming more together exposes a core gap beyond
provider failure. [Relational culmination review](engine-relational-culmination-review.md)
distinguishes joint action scoring and single-parent decomposition from a new,
evidence-linked multi-parent aspiration that changes future possibilities. The ledger
has multi-parent provenance, but the operational notebook path lacks that connected
mechanism. Authority/explicit criteria must remain protected while relational desire
can evolve. This diagnosis supersedes treating further context probes or verifier
plumbing as the next core step; no speculative implementation has begun.

## Current checkpoint: bounded relational cycle, f6b2b37

This supersedes the earlier “no speculative implementation has begun” status above,
without erasing that historical diagnosis. Multi-parent qualitative lineage, return
review, hypothesis/constraint context roles and normal-driver continuation are now
implemented. The live workshop completed aspiration → possibilities → assessment and
one simulator return → justified retention → next possibilities, with no manual stage
steering. Independent reviewers found its bounded minimum met, with semantic caveats.
See [recorded result](engine-workshop-live-result.md) and [replay/remaining priorities](engine-demo.md).

Functioning bounded demonstration is distinct from general autonomy. Control of categorical
hypothesis drift, precise evidence/permission interpretation, stochastic repeatability
and causal contribution of guidance remain unestablished. Mature independent common-event
calibration data are a separate operational dependency; 151 qualified tasks must not be
erased or conflated with that denominator. Existing mechanical infrastructure is not a
reason to keep adding generic plumbing. The next proposed semantic validation is a
preregistered different desire pair with limited/contrary simulated return, not a
post-hoc search for another favorable workshop output. No new live calls are admitted
by this audit.

## Garden transfer and evidence ontology checkpoint

Garden Trial 2 completed the generated relational cycle on another synthetic pair,
with one internal recording and no manual stage guidance. Both reviewers found the
bounded minimum demonstrated, but evidence-status interpretation only partially met:
V1 conflated already-visible report assertions with unrealized recording. V2 now
separates visibility, assertion trust, event receipts and unknown world truth offline.
See [contract/tests/next validation](engine-evidence-ontology.md). Historical records
and the initial unknown attempt remain unchanged; known and unknown costs replay
separately. This is implemented contract correctness, not demonstrated live semantic
improvement. Scoped functioning is established within the simulated adapter; reliability,
precise epistemic interpretation, cross-domain semantics and causal guidance benefit
remain open. Mature independently reviewed common-event evidence/Tier2 qualification
remain separate data dependencies. Next validation targets this concrete ambiguity,
not more executor machinery or a claim that synthetic bookkeeping satisfies rich wants.

## Post-Trial11 acceptance boundary

The [source-backed gap assessment](engine-post-trial11-gap-assessment.md) separates
the literal PRD B9.1 gate (demonstrated with an evaluator-authored seed forecast)
from native accepted-task status (independent decision deliberately not adopted),
resource-episode eligibility, and empirical reliability. Existing acceptance code
already updates scoped desires, reviewed belief/expectation evidence and subsequent
ranking. The extra generated boundary experiment is optional, not a new acceptance
criterion. No code defect or new provider/target execution requirement was established;
the smallest next proof point is separately authorized review/adoption of the already
tested candidate, without altering historical closures or claiming mature calibration.

## Independent acceptance and semantic-return continuity

[Linked Trial 11 acceptance](engine-trial11-independent-acceptance.md) is now
recorded after independent source/receipt review, with zero provider calls or target
executions. Both original semantic desires are fulfilled in that copy; one reviewed
return resolves the frozen forecast and persists across restart. Historical resource
closures and the optional unexecuted experiment remain unchanged. This is scoped
acceptance, not general autonomy or calibrated learning.

The subsequent core audit reproduced a concrete omission: a semantic rejection
could leave relational guidance current when tool receipts/source did not change.
[The narrow binding correction](engine-semantic-return-relational-binding.md) now
allows that return to trigger qualitative reconsideration while keeping private
prose and authority separate. Fixture checks establish the mechanism, not useful
model-generated revision. The next empirical question is whether generation uses
that newly visible independent judgment honestly; no live allowance is implied.

## Active desires and integrated choice after `9babd3c`

[Guidance-to-choice trace and controls](engine-semantic-guidance-choice.md) separate
qualitative possibility guidance from shared-kernel selection. Current relational
revisions reach ordinary proposal requests, but do not silently create weights or
permissions. Fixed-option tests demonstrate independent rejected versus unresolved
return changing learned projection/ranking, and the normal selected commitment;
prose-only revision is correctly invariant when options and evidence stay fixed.

A reproduced selection defect allowed withdrawn/out-of-scope semantic desires to
remain in ranking and recorded candidate support. Both now use folded active aims;
restoration recovers prior evidence. Fulfilled standing-loss protection, original
acceptance criteria and relational lineage remain intact. This is a real decision
boundary repair, not evidence that generated culmination improves choices. Useful
context-sensitive possibility generation and reliable outcomes remain empirical;
no additional numeric value theory or prompt-format benchmark was introduced.
