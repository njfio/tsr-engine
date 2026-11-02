# A goal, a next step, and an observed return

The configured adapter runs on user-supplied goals, criteria, action descriptions and results. No Rust rebuild is needed to change those inputs. It suggests a step; the user performs it and reports the result. It makes no model calls and executes no suggested action.

From the repository root:

```sh
cargo +1.90.0 build -p tsr-cli --bin tsr --offline
./target/debug/tsr engine --directory target/report-success configure --spec docs/engine-goal-example.json
./target/debug/tsr engine --directory target/report-success next > target/report-next.json
python3 -c 'import json; print(json.load(open("target/report-next.json"))["pending"]["commitment"])'
# Copy that commitment ID into COMMITMENT below. A separate process can resume it:
./target/debug/tsr engine --directory target/report-success state
./target/debug/tsr engine --directory target/report-success observe --commitment COMMITMENT --result docs/engine-result-success.json
./target/debug/tsr engine --directory target/report-success next
```

For the failure path, use a fresh `target/report-failure` directory, repeat `configure` and `next`, and supply its commitment ID with `docs/engine-result-failure.json`. Never reuse an ID from another directory. `next` refuses while a commitment is pending; stale or duplicate observations are refused. `configure` refuses to overwrite a store.

Captured independent-process runs are in [engine-outcome-paths.json](engine-outcome-paths.json):

| Same initial configuration | First step | Observed result | Updated draft estimate | Next step |
|---|---|---|---|---|
| Success path | draft | readable criterion met | .667 | draft |
| Failure path | draft | readable criterion unmet | .333 | outline |

Both paths begin with draft=.5, outline=.5; costs are .05 and .15. The outcome difference, not a cycle index, changes the next choice. Prior operator-supplied evidence can also change the first choice. An initial commitment breaks equal-score ties only; it cannot override a better-supported option. Under unmet pressure, an exact tie favors an alternative to the previous commitment.

## What drives the mechanism

- **Proposal generation:** `ProposalProvider` receives `State`, including current/pending commitment, active criteria, per-action estimates, support and unmet pressure. The local implementation offers configured action IDs ordered by derived score. No proposal can add an action, write an observation, or admit a preference.
- **Belief commitment:** `next` records possibilities, validates IDs and admitted criterion support, chooses the highest score among the offered eligible options, freezes the forecast and complete active criterion set, then records a pending commitment. The configuration is operator-declared initial support; it is not independent evidence of success.
- **Expectation:** counts are folded from explicit prior evidence and observed returns using `(1+successes)/(2+successes+failures)`. Only returns with the same frozen criterion set enter the current estimate. Changing active criteria does not relabel old outcomes or inherit their forecasts as evidence for the new objective.
- **Choice:** score is `prediction × number of distinct active criteria served − declared cost`. Candidate support names only admitted preferences mapped to that action. All alternatives use the same active-criteria success event. These are empirical engineering estimates, not validated calibrated probabilities.
- **Return:** `CriteriaAdapter` compares the supplied `met` names with the frozen criteria. An outcome is an operator report with evidence text, not independently verified truth. It records Surprise separately from structured Contrast. Only unmet Contrast can support a proposed refinement.
- **Accumulated desires:** active criteria affect action value; pending refinements do not. `unmet_pressure=failures/(1+failures)` is bounded, derived, and used only for tie-breaking away from the prior commitment. No stored desire weight or automatic authority escalation exists. The active view is capped at 16 criteria; the append-only history remains durable.

A failed result can optionally propose a refinement:

```json
{
  "met": [],
  "evidence": "The report was difficult to follow.",
  "proposed_refinement": {
    "criterion": "concise",
    "text": "Keep the report concise",
    "actions": ["draft"]
  }
}
```

The returned state's new preference has `status: null`. To admit it deliberately:

```sh
./target/debug/tsr engine --directory target/report-failure accept PREFERENCE_ID
```

Admission affects future choices only; any already-pending commitment retains its frozen criteria. Without a proposed refinement, failure adds evidence and contrast but creates no new preference. A focused test verifies that the above refinement does not change ranking while pending, then changes ranking after admission. Generated preferences cannot remap an existing criterion or reuse an admitted preference identity.

## Review of f0d67e1 and limits

The original `init|step|show` queue demo remains available. Its `LocalProposer` reads estimates, costs, last route and pressure; there is no cycle-number branch. But routes, one-item probes and delivery outcomes are built into Rust. Every route serves the same standing goal, so accumulating/admitting refinements did not change its ranking. The configured adapter removes those restrictions on input and result entry.

The adapter currently supports named Boolean criteria and a fixed configured action menu. Outcomes and initial support are operator supplied; it does not inspect the real-world result. Goals are standing criteria, not an automatically closed task. Richer observation validation, action execution, new action discovery and calibrated production policies remain future adapters. `DomainAdapter` exposes outcome semantics; `ProposalProvider` exposes proposal generation. The default CLI uses local implementations only and has no provider credentials or egress.

Validation: focused tests change outcome paths, initial commitment at equal value, prior evidence, and preference admission; they also check restart with a pending commitment, duplicate/stale returns, invalid observations and invented provider actions. Existing queue and CLI tests remain passing. Benchmarks remain stopped; this work spent no provider funds.
