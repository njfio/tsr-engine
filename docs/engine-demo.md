# Local bounded TSR demonstration

From the repository root, replay the actual five-stage workshop run without inference:

```sh
python3 scripts/engine-demo.py
```

The default is **read-only historical replay**. It prints source roots, recorded code
revision, each stage's usage/cost, the generated aspiration, actual simulator event,
retention and number of unexecuted next proposals. Current dispatch/action counts are
zero. It verifies every indexed evidence-file SHA256 and receipt links. It never imports
transport, reads credentials, executes an action, repairs evidence or regenerates missing
files. The checked-in `engine-workshop-replay-index.json` is the historical trust anchor;
this is not a fresh SQLite/CAS audit or proof of current engine compatibility.

Evidence is local under `target/workshop-ready/run`. If it is unavailable on another
checkout, replay fails explicitly. Copy the complete indexed evidence from this approved
synthetic run into a local directory and use `replay --evidence /path/to/copy`; do not
regenerate it and call it replay. No private sessions or raw provider bodies are needed.

The recorded run used five calls, one internal simulated action, zero manual interventions,
and $0.005951. Two reviewers found the minimum bounded demonstration met, with imperfect
necessity/consent language and no causal superiority or general autonomy established.
Read [the full result](engine-workshop-live-result.md) for qualifications. Retaining the
aspiration after a compatible return was valid; it was not claimed as new insight.

The earlier deterministic queue walkthrough is preserved unchanged in
[engine-queue-demo.md](engine-queue-demo.md). It remains useful for local mechanism
inspection but is not the generated relational workshop demonstration.

## Preparing a new run is separate

Offline preparation initializes only the pinned evaluator-authored synthetic setup:

```sh
mkdir -p target/my-workshop
python3 scripts/engine-demo.py prepare \
  --directory target/my-workshop/run \
  --expires '<explicit future ISO-8601 timestamp with timezone>'
```

A new paid run requires separate authorization, the reviewed bounded admission and an
explicit opt-in. The following command is documented, **not executed in this turn**:

```sh
python3 scripts/engine-demo.py run \
  --directory target/my-workshop/run \
  --admission target/my-workshop/run/admission.json \
  --allow-paid-calls
```

It invokes the existing normal driver with the configured CheaperInference model; no
account/key changes occur. The opt-in flag does not replace authorization or alter the
five-call/one-action bounds. Missing opt-in stops before the driver is invoked. The last
stage generates proposals only. Failure, inability, uncertainty, expiry or withdrawal
stops as documented in [integration instructions](engine-workshop-integration.md).
Never reset a consumed admission to rerun an uncertain request.

## Remaining priorities

1. **Meaning and evidence:** generated hypotheses still sometimes become categorical
   necessities or blur narrow permission with broader consent. Typed roles preserve
   authority boundaries but do not establish sound inference. The next semantic test
   should change the desire pair and return, not add another schema to obtain a pass.
2. **Repeatability:** historical replay is now deterministic. The stochastic semantic
   result needs a separately preregistered run, reporting every outcome with the same
   acceptance rubric. One success is not reliable autonomy.
3. **Influence:** proposal compatibility with an aspiration is observed; causal benefit
   has not been established. Prior small guidance ablations were mixed. No causal claim
   is made from copying the aspiration into context.
4. **Operational calibration:** preserve the inventory of 151 qualified corpus tasks;
   this does not establish 150 mature independent reviewed episodes per common event,
   family support or calibrated Tier2 eligibility. Default fallback remains appropriate.
5. **Cross-domain semantics:** shared mechanics exist, but independent project review and
   domain-specific correctness still matter. This synthetic demo does not remove them.

Existing commitment, conserved obligation scoring, prospective state, authority gates,
restart, receipt accounting and supplemental evidence are not the current semantic
bottleneck. More executor bookkeeping is not the next success criterion.

## Proposed next validation — not admitted

Freeze one new **evaluator-authored community-garden simulation** before any call. Pair
wanting dependable shared harvest planning with wanting collective curiosity about
unfamiliar growing practices. Neither is the user's actual preference. Use the same
bounded internal-recording adapter, not real garden operations.

Independently freeze two trial reports: one promising result limited to a particular
plot/condition, and one negative result under a different condition with an explicit
measurement limitation. Show both as available records; recording one adds only its
actual event. Do not make a hypothesis or a preferred trial report for the model. This
can challenge universal claims *if* the model makes them, but compatible hypotheses may
be retained. It tests honest interpretation of evidence, not hidden discovery or agronomic
truth. Predicate gains are bookkeeping, not harvest reliability or curiosity fulfillment.

Use the existing five-stage shape, at most five calls and one simulated operation,
8192/8192/32768/8192/8192 completion tokens and 240 seconds each. Final proposals never
execute. Stop on the first ordinary failure; no replacement scenario, favorable-output
selection or added calls. Before admission, freeze exact records, source hashes and
independent semantic criteria: both parents contribute to the relationship; uncertainty
and condition-specific limitations survive; a negative/limited result does not invent
universal prohibition or permission; revision or retention is justified; next proposals
remain within scope. No required winning phrase, action, contradiction or novelty.

That new scenario needs explicit registry review and independent design review; its exact offline plan is now in `engine-synthetic-garden-plan.json`.
Admission and outcomes are recorded separately in `engine-garden-validation.md`. This is the next test of
transfer beyond the workshop, not an open-ended benchmark campaign.

Current offline test results and sandbox limitations: [engine-demo-checks.md](engine-demo-checks.md).

## Replaying the stopped garden attempt

```sh
python3 scripts/engine-demo.py replay --scenario garden
```

This separately pinned historical replay displays `recorded_total_usd: null`, unknown
usage/cost, the retained reservation and zero actions. A zero settled subtotal is not
zero total cost. It never invokes the normal driver, polls billing, repairs evidence
or retries generation. Missing or changed local evidence fails closed. The workshop
remains the default; an unsuccessful garden attempt is not replaced with its result.

## Replaying completed Garden Trial 2 with its historical uncertainty

```sh
python3 scripts/engine-demo.py replay --scenario garden-trial2
```

Read-only replay verifies both pinned local evidence sets. It displays Trial 2's
settled **$0.006318**, Trial 1's retained unknown charge/reservation, and a **null
combined total**. Missing either evidence set fails; it does not fetch, regenerate or
reconcile it. Both historical trials retain their original contracts and semantic
caveats. The newer evidence-ontology fix is not retroactively applied to replay.
