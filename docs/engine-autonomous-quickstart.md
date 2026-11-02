# Bounded autonomous engine: historical workflow

**Release hold (2026-10-27): paid launches of this legacy workflow are disabled with `OUTPUT_BOUNDARY_INCOMPATIBLE` until its output configuration, quarantine and admission are migrated and reviewed.** The commands below document the historical interface, not a currently supported live launch. Local preparation and read-only evidence remain available, subject to their existing source-pin checks. See [the release inventory](engine-output-release.md).

This was the unattended **synthetic** workflow. It runs the existing persistent Rust controller with model-generated relational guidance and alternatives, native selection, simulated returns, separate runtime semantic reviews and one bounded reconsideration. It is not permission to operate on real repositories, people or accounts. No UI or additional service is required.

Run from the repository root with the existing Python environment and `target/debug/tsr`. Existing CheaperInference credentials are consumed privately by the transport; never put them in commands or reports. No install/download is part of this workflow. The model is pinned to `gpt-6-luna`.

The latest [real Rust repair result](engine-real-repair-trial23.md) completed through the normal controller with actual sealed Cargo execution, automatic intermediate reviews and no manual stage steering or linked recovery. Final acceptance was independent. The earlier [linked repair](engine-real-repair-accepted.md) remains historical evidence. Current campaign usage is 133/140 calls, 40/40 execution groups and $0.106171/$5: the execution allowance is exhausted. The commands below describe the supported workflow, not permission to launch another run. Use the read-only inspection command in the latest result to inspect completed work.

## Desires and scope

The supported provider path accepts the frozen evaluator-authored `workshop` or `garden` scenario. Its original desires are the `spec.wants` entries in `docs/engine-synthetic-workshop-plan.json` or `docs/engine-synthetic-garden-plan.json`. Read those before choosing a scenario. The generated relational aspiration may develop, but never replaces those original meanings or grants permission. Visible fictional reports are assertions, not independently verified world truth.

Do not edit these frozen files or substitute private material into this runner. For locally supplied desires, the native interface is `target/debug/tsr engine --directory NEW_DIRECTORY direction-init --spec YOUR_SPEC.json`, where the spec contains typed `wants` with text and supported predicates. That local initialization does **not** admit external transmission or unattended real-world execution. New synthetic scenarios need explicit source/scope registration; arbitrary personal desires are not yet supported by this external workflow.

## Prepare once, then run or resume the same command

Preparation is offline. Use a new directory and an explicit future expiry; it refuses to overwrite an existing run. The reviewed-multi profile keeps both original desires active, permits two simulator operations and has no scripted withdrawal. Its maximum is ten generator plus nine reviewer requests (19 total), including one possible semantic reconsideration; unused requests expire at terminal outcome. Its supervisor deliberately restarts once after an acknowledged provider outcome to verify durable continuation; this is an engineered restart, not a spontaneous failure.

The shared campaign is already bound to a $5 total, explicitly extended 140-call and unchanged 40-operation ceiling. Do not create a second campaign to reset those limits. All generation, review and failed attempts count. Unknown requests keep reservations and stop; they are not automatically retried.

Use the checked-in entry points below with the current reviewed implementation. Set `TSR_RUN` to a new disposable directory and `TSR_EXPIRY` to an explicit UTC expiry with enough time for the bounded run:

```sh
export TSR_RUN="$PWD/target/my-reviewed-loop"
export TSR_EXPIRY="YYYY-MM-DDTHH:MM:SSZ"
export TSR_CAMPAIGN="$PWD/target/tsr-integrated-campaign-1"
export TSR_CAMPAIGN_HASH="7df4975d8cc5f99c2e847ced4969923e4fa5d40e98e9b664388fcb217642acfb"
export TSR_UPGRADE="$TSR_CAMPAIGN/schema-correction-upgrade.json"
export TSR_UPGRADE_HASH="0c296aa47b8ceb6889f2fc74bb9e524318316508ccf577a28ffb32968d7001b7"

python3 scripts/engine-workshop-prepare.py \
  --directory "$TSR_RUN" --scenario workshop --model gpt-6-luna \
  --reviewed-multi --runtime-review --review-recovery --expires "$TSR_EXPIRY"
python3 scripts/engine-campaign-next-choice.py prepare-extension \
  --campaign "$TSR_CAMPAIGN" --campaign-sha256 "$TSR_CAMPAIGN_HASH" \
  --admission "$TSR_RUN/admission.json" --extension "$TSR_RUN/scope-extension.json"
```

Inspect `admission.json`, `spec.json`, `initial-request.json` and `scope-extension.json` before the initial launch. Retain the exact extension hash printed by preparation as `TSR_EXTENSION_HASH`; do not approve a changed artifact by merely recomputing its hash. Preparation alone grants no spending authority. Within the currently approved remaining campaign, the supported **run/resume command** is:

```sh
python3 scripts/engine-campaign-next-choice.py run \
  --campaign "$TSR_CAMPAIGN" --campaign-sha256 "$TSR_CAMPAIGN_HASH" \
  --upgrade "$TSR_UPGRADE" --upgrade-sha256 "$TSR_UPGRADE_HASH" \
  --admission "$TSR_RUN/admission.json" \
  --extension "$TSR_RUN/scope-extension.json" --extension-sha256 "$TSR_EXTENSION_HASH" \
  --allow-paid-inference --allow-simulated-operation
```

Use the identical command to resume an interrupted process, never a fresh directory to retry an uncertain request. The controller verifies durable intents/outcomes and source bindings. An unresolved dispatch/application or stale admission stops for reconciliation. Expiry is not silently renewed. Do not launch two processes for the same run. A completed run is terminal; resuming does not reset slots or start another episode. The command may perform a public pricing GET before replay, but must not redispatch model calls or actions.

## Status and stop

These reads do not dispatch:

```sh
target/debug/tsr engine --directory "$TSR_RUN" direction-request
python3 scripts/engine-b91-campaign.py inspect \
  --campaign "$TSR_CAMPAIGN" --upgrade "$TSR_UPGRADE" --upgrade-sha256 "$TSR_UPGRADE_HASH"
cat "$TSR_RUN/synthetic-external/terminal.json"
```

The terminal file exists only after completion. During a run, immutable `intent-*`, `outcome-*`, `review-*`, `controller-*` and any `recovery-*` files show progress. `events.sqlite` and `cas/` hold native persistent history; never edit them. Reports summarize predicates separately from unknown rich-desire fulfillment.

To request a safe stop, create the operator stop file:

```sh
touch "$TSR_RUN/STOP"
```

This prevents subsequent dispatch/application at controller checkpoints. It does not undo an in-flight request or refund its cost. Wait for its receipt; an interrupted uncertain call must remain reserved. Do not delete STOP to bypass a terminal or authority refusal.

## Guarantees and limits

Each request is bounded to 8,192 completion tokens, 240 seconds and 32 KiB of allowlisted synthetic input. Current public rates must fit the existing $0.05 per-request reservation; actual costs are settled and unknown costs remain held. This is not a provider-enforced billing cap. Runtime review may reject, return unknown, abstain or be wrong; it supplies no external permission and does not confer formal project acceptance. Same-model reviews may share errors with generation.

An appropriate idle/stop is a valid outcome. Two useful simulator operations do not prove social fulfillment, general reliability, calibrated learning or superiority over another agent. The [integrated result](engine-supported-workflow-result.md) records the actual two-cycle proof at `4f77fe2`, including automatic review, restart and stopping. Optional next work should improve supported scope or empirical reliability, not redefine this bounded deliverable as general autonomy.

For the frozen real Cargo repair workflow, final independent acceptance now precedes optional next-proposal generation. An accepted task closes without spending another call; rejection stays explicitly unresolved and needs fresh action admission. This does not change simulator scope, make acceptance automatic, or remove the native request-size limit for genuinely unresolved work. See the [real repair result](engine-real-repair-accepted.md) for the live evidence and subsequent offline fix.
