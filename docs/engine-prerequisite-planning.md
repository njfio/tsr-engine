# Generated prerequisites for standing desires

The next core gap after local observation collection was immediate-only valuation.
An account desire with no separate inspection desire could not motivate reading its
required evidence: inspection leaves account coverage unchanged, so every proposal
was rejected. Asking the operator to add a desire merely to reward this action would
hide the gap.

The notebook adapter now accepts an optional `continuation` on a generated direction.
It contains one future `action` and a terminal `anticipated` vector. The existing
`anticipated` remains the forecast immediately after the first action. An absent
continuation preserves the old format and serialized legacy records.

Both steps pass the existing admitted-notebook transition validator. The first must
acquire new knowledge; its intermediate effects may change standing predicates. The second must be
unavailable before that acquisition and valid afterward. This deliberately supports
inspection prerequisites followed by composition, not arbitrary multistep
planning. The generator chooses note sets, proposed synthesis and forecasts; the
controller does not supply a reference answer or fixed candidate menu.

The same domain-independent obligation kernel assesses the hypothetical terminal
transition against the original observed state. Its shared `assess_path` contract
accepts intermediate bounds and charges each original obligation’s worst potential
loss across intermediate and terminal lower bounds exactly once, at full weight.
Unknown current satisfaction remains unknown; one is used only as a conservative
risk baseline. Refinements cannot multiply loss or benefit. Both action costs count, and a named 10% delay charge applies to positive
expected gain. This is an explicit engineering choice, not a numerical TSR law.
For notebook plans, independently simulated mechanical terminal bounds are exact;
calibration cannot change these exact effects, but learned effectiveness still affects
value. Other adapters can supply uncertain intervals. Ordinary immediate actions
retain their calibrated capacity bounds. Public projections match those actually scored. Remaining cycle budget must accommodate
both steps. Duplicate first actions do not count as distinct alternatives merely
because their proposed continuations or prose differ. With one admitted note and no
inspected evidence, one alternative is permitted: inspection is the sole executable
first action. The controller and provider contract derive this same minimum; other
states retain two distinct alternatives.

Only the first action is committed and executed. Its dispatch forecast concerns
immediate predicate progress, so a neutral prerequisite predicts zero and receives
zero controller residual when the predicates remain unchanged. Inaccurate bounded
model forecasts remain frozen and are reported separately as `generator_residual`;
they cannot change this independently bounded immediate effect or leak into terminal
calibration. Rejecting such forecasts would be inconsistent with ordinary actions,
whose optimistic predictions are already bounded by capacity. The measured new knowledge is
visible separately. The frozen continuation is an unresolved possibility; it is never
a queued action, an observed success, or new permission. The next call receives the
actual return and prior plan and must propose fresh alternatives against current state.
Restart, stop, observation revision and all existing budgets remain effective.

`Assessment.shared` scores the hypothetical terminal action. The outer score and cost
also include the first action and delay; `plan.assessment_scope` makes this distinction
explicit. Terminal estimates borrow related *actual direct-action* returns. An inspection return trains inspection expectations, including nonneutral first-step
progress; action-family matching prevents it from labeling a future composition. If a later freshly chosen composition occurs, its ordinary return
updates expectations. There is no invented label for whether an abandoned plan would
have succeeded. `plan.terminal_status` remains unresolved rather than retroactively
claiming that a subsequently successful action proves the whole plan was followed.

The PRD B2 defines Belief as commitment, not a general propositional truth model.
This change strengthens prospective commitment without inventing such a model. It
also does not expand the notebook's meaning of coverage/brevity into truth or
understanding. Source text remains untrusted evidence. Semantic project acceptance
still needs independent review; project prerequisite planning is not yet connected.

## Run

Use the already installed provider and toolchain described in
[the local quickstart](engine-local-quickstart.md). Initialize a fresh directory with
a spec containing account and concise wants, admitted notes, an attention limit no
larger than the number of wants, and the action budget:

```sh
target/debug/tsr engine --directory target/my-planning-run direction-init --spec SPEC.json
python3 scripts/engine-directions.py --directory target/my-planning-run \
  --model qwen2.5-coder:32b --endpoint http://127.0.0.1:11435 --calls 10
target/debug/tsr engine --directory target/my-planning-run direction-request
```

The provider must already be running locally; this installs nothing. The driver keeps
its immutable call allowance, 240-second per-call timeout and uncertain-call refusal.
It proposes no arbitrary commands or source access. With no known evidence and only
account/brevity/inclusion wants, its schema requires a continuation because no directly
productive action is available. This constrains contract shape, not note choices or
future forecasts. The schema also derives the eligible action set from current evidence:
inspect references only unread admitted IDs; immediate compose references known IDs;
no inspection or continuation is offered once all notes are known. Planned composition
can reference admitted IDs, but Rust verifies that the actual first step enables it.
Explicit HTTP errors are journaled as terminal failures and consume
their reserved call; connection loss/timeouts remain uncertain and forbid retry.

## Verification

Deterministic controls verify that account-plus-brevity can motivate a prerequisite
without inventing an inspection want, immediate satisfaction stays unmet, restart
preserves the exact state, revised evidence cannot execute the saved continuation,
invalid dependencies and insufficient action budget are rejected, and a real later
composition return changes the expectation and score of a subsequent planned option.
Removing only that history restores the prior. Existing desire, commitment,
expectation, decomposition and standing-loss controls remain in the affected suite.


## Live verification record

Evidence is under `target/prerequisite-live/`. The source spec derives from the existing
standing-wants example, retaining account and brevity and removing the inspection want;
there is no candidate solution in that spec. The installed `qwen2.5-coder:32b` runs only
through the temporary loopback provider, with a frozen ten-attempt allowance and
240-second request timeout. All errors and generated offers remain preserved.

Attempts 1–2 proposed ordinary inspection with impossible immediate account progress.
Both were rejected. Attempt 3 received a terminal HTTP 500 after an unsupported schema
composition; the response was reconciled with the temporary server's terminal log and
recorded without refunding its slot. Attempts 4–5 generated two-step plans but were
rejected by an overly strict first-forecast equality check. Independent review agreed
that the controller should bound these forecasts consistently with ordinary actions;
the corrected implementation preserves raw model error rather than treating inaccurate
forecasts as authorization or fabricated success.

Attempt 6 offered four genuinely generated plans. The chosen plan inspected all three
notes, with hypothetical later composition, scoring 0.04125 versus 0.01375 for each
single-note plan. The actual account remained empty and satisfaction stayed `[0,0]`.
Controller dispatch residual was zero; model immediate forecast error `[0.75,0]` stayed
visible. No terminal composition executed. `planning-ablation.json` removes only the
continuations from these same generated first actions under identical initial state:
all become nonproductive and no state changes. This is a planning ablation, not evidence
that one generated composition is semantically better than another.

Attempt 7 generated two coverage refinements with the same note set; one was admitted
and the duplicate rejected. It inherits its parent's influence rather than receiving
extra reward. This is decomposition of an existing coverage obligation, not a new
fundamental value or demonstrated semantic distinction.

Attempts 8–9 repeated already completed inspections and were rejected. The provider
contract was corrected to expose the state-derived eligible frontier. Attempt 10 then
proposed compositions, but three alternatives attached invalid prerequisite
continuations to their composition actions; only one alternative was valid, so the
offer was rejected. The final contract additionally removes continuations when all
notes are known. This last contract correction has deterministic coverage but **has not
been verified with another fresh local generation**: the ten-attempt allowance is
exhausted, not silently extended.

Final live result: **one committed inspection, one admitted coverage refinement, seven
rejected offers, one terminal provider error, ten attempted calls, and zero paid calls**.
The current satisfaction vector is `[0,0,0]` (account, brevity, derived coverage), and the
account remains empty. The model's valid final composition was not silently extracted
or substituted as an authored answer. `verification-summary.json` and `restart.json`
record identical restart state. The temporary provider was stopped after terminal
completion. This is live evidence of generated prerequisite commitment and honest
return, **not a completed live account cycle**.

Final deterministic validation: 85 CLI tests passed (one existing ignored), 12 domain
tests passed, 17 driver tests passed, 13 collector tests passed. Workspace/all-target
Clippy with warnings denied passed. Logs: `target/planning-checks.log`,
`target/planning-driver-checks.log`, `target/planning-collector-checks.log`, and
`target/planning-clippy.log`. Independent review covered horizon separation, raw forecast
preservation, contract shape, authority, legacy receipts, budget and uncertain restart.

## Historical next steps at 3d22af5

The immediate next authorized work is a fresh, explicitly bounded local batch to
verify the final eligible-frontier contract through composition and subsequent learned
selection. The present allowance is exhausted; no semantic user choice blocks that
verification, and no external provider is required.

The remaining core generalization is an explicit intermediate-outcome contract: a
prerequisite may affect some standing wants while enabling progress on others. The
current path intentionally accepts only predicate-neutral inspection. Extend the shared
assessment with independently supplied intermediate effect bounds and standing-loss
constraints, then connect those contracts to existing adapter transitions. Do not infer
a new permission, value, or risk tolerance from a generated plan. Project-adapter
prerequisites need independent evidence of their enabling relation; a model's assertion
alone is insufficient. This is a concrete next implementation direction, not a claim
that broader engine development must await generic user direction.


## Shared intermediate effects follow-up

`tsr_domain::decision::assess_path` now provides domain-independent path assessment.
`PlanAssessment.path` exposes intermediate bounds, risk, additional risk charge,
first effort, delay and the separately scored terminal assessment. No path assessment
executes or authorizes either action. Notebook validation still admits only a new
inspection enabling a previously unavailable composition, and only inspection runs.

Independent review caught and corrected terminal optimism hiding uncertain harm,
and a trace mismatch after exact notebook bounds. Regression tests cover terminal
uncertainty, overlapping losses, reversed stage losses, unknown observations,
refinement conservation, unchanged controls, actual first-step learning, absence of
fabricated composition learning, and restart. Numerical loss and delay choices are
engineering policy, not claimed qualitative TSR laws. Transfer remains coarse action
family/topic/context similarity.

The follow-up live batch uses the original journal and a separately authorized ten-call
allowance named `composition-contract`. Attempt 12 generated and executed fresh
composition after attempt 6’s inspection: observed account coverage became 1/3 and
brevity became 1. Attempt 13 admitted an additional coverage decomposition. Attempts
14–15 were rejected for insufficient conserved expected progress. This establishes
prerequisite → actual return → fresh composition, not complete account satisfaction.
Earlier failures and the historical verification summary remain preserved.

Reproduce state inspection without inference:

```sh
target/debug/tsr engine --directory target/prerequisite-live/run direction-request
```

The operator supplies admitted notes, standing predicates, local provider and budgets.
The model supplies directions and forecasts; the adapter measures ID coverage/brevity.
These measurements establish neither source truth nor qualitative understanding.


Follow-up checks: 86 CLI unit tests passed (one existing ignored), 15 domain tests
passed, 17 Python driver tests passed, and workspace/all-target Clippy with denied
warnings passed. Logs are `target/path-final-tests.log`, `target/path-python.log`,
and `target/path-final-clippy.log`. Independent review confirmed the corrected
risk calculation, trace alignment and first-action learning separation. Git hooks
could not run because `lefthook` is absent; these checks were run explicitly.


Attempt 16 repeated the same nonproductive original forecast and stopped. The next
contract revision explicitly defines topic-account and inspection fractions and that
composition replaces, rather than appends to, the claim set. It supplies measurement
semantics, not a candidate or desired answer. The named allowance remains capped at
20 cumulative attempts (10 historical plus 10 follow-up).


Final follow-up result: seven of ten additional calls used (17 cumulative), all
terminal. Attempt 17 still forecast derived coverage improvement without original
account progress despite explicit predicate semantics, and the repeated-offer stop
fired. Three slots remain unused. Final observation is `[1/3,1,1/3,0]`; restart root
is `f30ec183d8e0dc223536da73744004610fdc8c209c76bbfed64f68412c2fd00f`.
`target/prerequisite-live/followup-summary.json` and `restart-followup.json` preserve
the follow-up outcome separately from historical evidence. Temporary Ollama PID
12965 was terminated after all calls became terminal. No paid calls occurred.

The concrete remaining integration limitation is consistent forecasting across
original/derived obligation identities. A next useful local change is an explicitly
identified per-obligation forecast interface with field-level consistency feedback,
rather than repeated positional arrays or authored candidate substitution. Preserve
raw forecasts and conservative scoring; changing the interface must not silently
raise predictions to force acceptance. This is a local-provider reliability issue,
not permission to weaken wants or a need for new fundamental-value semantics.
