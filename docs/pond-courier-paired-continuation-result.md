# Pond Courier: same-model paired continuation

The paired evaluation is incomplete. It produced useful behavioral observations, not evidence that TSR outperformed an ordinary coding agent. Both arms stopped on rejected output contracts; neither completed a valid final reflection/retention decision. No failure was rewritten or resampled into a success.

## Matched setup

Implementation `9ceacb60`. Frozen protocol `experiments/pond-pair-v1/README.md`; packet `target/pond-pair-prepared-v3/prepared-manifest.json`, SHA256 `106cb876def19d335885e6faae903c6be1f710d8be4c84a9ec4ef053bd0a3f49`.

Both used CheaperInference `gpt-6-luna`, JSON-object mode, 32768 completion tokens, stream false, request timeout at most240 seconds, no temperature/seed/reasoning overrides. These match the original retained Pond Courier request records; backend weight revision is not exposed. Both received identical original source, goals, five public prior proposal/review/rejection records, source-bound 15-check findings and the exact player quote: “that was actually pretty good, I didn't think it was at first but it was pretty good after I played a little. How was TSR in that process?” No prescribed tutorial, easier difficulty or relationship was supplied.

Equal maximum 100 calls / 50 operations / $5, ten corrections and one bounded authenticated provider retry per arm; combined 200 calls / 100 operations / $10. Equal 55-minute active windows within outer 21:04:04 UTC expiry. Fixed sequential order A then B avoids dispatching while the shared ledger has an unsettled effect. Order/provider-load and stochasticity remain confounders. The ordinary controller receives the same semantic information in ordinary memory and chooses one action; TSR offers formal relationship admission and multiple alternatives. This is a whole-controller comparison, not an isolated relational ablation. Full trace schemas reveal process identity even with A/B labels.

## Actual outcomes

**A / TSR:** proposed a small visual traffic-direction-arrow edit, describing it as an untested readability hypothesis. Independent same-model review accepted it. Native execution produced a passing exact-source browser smoke receipt; the controller automatically supplied that return and current source to its next proposal. The next response proposed adding arrows again using pre-edit road-marking text that no longer existed. Native validation correctly rejected it: `exact unique changed span required`. No second edit executed. No formal relationship proposal/admission or generated new desire occurred.

The executed candidate passed the same 15 independent control-driven checks as v1, bound to its new source; screenshot inspection confirms lane arrows and intact layout. This supports functional nonregression in those checks, not improved player enjoyment. The mobile check clicks an on-screen button at a narrow viewport; it is not a real-device touch test. Those final broader 15-check results were collected after the terminal stop and were **not** subsequently reflected on by the model. The fresh native smoke evidence was returned before the failed second proposal. Thus a complete feedback cycle was not established.

**B / ordinary memory:** its only response argued for retention, reasonably noting that enjoyment after playing did not mandate a particular change. It also incorrectly described the broader assessment as an earlier-source assessment: this arm's source remained exactly v1 and matched that assessment's hashes. The response nested `abstain` inside `alternatives`, violating the explicit top-level abstention contract. Validation rejected it: `abstention cannot mix alternatives`. Retention was expressed in unadmitted text, not recorded as a valid terminal engine decision. No edit or execution occurred, and no second independent gameplay run was needed for its byte-identical v1.

Both rejections were settled and preserved. Neither was classified as an admitted format correction; both had zero corrections and zero provider retries. This exposes a recovery/classification limitation for the nested abstention, while stale exact spans must remain rejected rather than silently repaired. Changing either controller and rerunning selectively would no longer be this frozen pair. No winner is assigned.

## Evidence and accounting

- A: 3 calls, 1 native execution plus 1 independent verification operation, $0.008204; 69,018 prompt / 4,627 completion tokens.
- B: 1 call, 0 operations, $0.002173; 21,135 prompt / 512 completion tokens.
- Total: 4 calls, 2 operations, $0.010377. Campaign: 323 calls / 88 operations / $0.345373. No unknown effects or overruns.
- Zero manual game edits, semantic replacements or live stage bridges. Operator work comprised prelaunch generic implementation/review/setup, launching the prescribed arms, and final independent measurement/packaging. This is not a claim of a fully autonomous evaluation or successful unassisted full cycles: **zero completed full cycles** in this pair.
- Both terminal replays returned `replayed_without_dispatch:true`; original Pond Courier native checkpoint and delivered v1 hashes unchanged. No Safari action occurred.
- Five authored native/ordinary integration tests passed with actual isolated browser checks. Nine accounting/clock tests passed. An earlier development run rejected admission drift while source pins were changing; conclusive testing used frozen source. Restricted executions were denied Docker-socket access and produced unavailable observations; those are preserved failures, not passed tests. Deadline marker test uses mocked journal operations, not complete crash durability.

Trace roots: `target/pond-pair-prepared-v3/{A,B}`. Live logs: `target/pond-pair-{A,B}-live.log`. Retained rejected content: each arm's `adapter/output-*/failed-response-1/content.bin`. Exact read-only decode findings: `failure-diagnosis.json`. Accounting/preservation: `settled-result.json`. Independent observations and screenshots: `A/independent-browser/`. `CLOSED.json` expires unused allowance.

Playable candidate: `artifacts/pond-courier-paired-tsr-candidate/index.html`, copied byte-identically from the engine's executed source, with README/provenance/screenshots. Original `artifacts/pond-courier-v1` remains the user's delivered version.

## Limits and next engineering boundary

The player feedback reached both processes. A made a modest hypothesis-driven revision, but its later stale edit does not demonstrate reliable learning from the return. B expressed calibrated no-change intent but also misattributed source evidence and failed its envelope. Neither formed a relationship or new desire identity. The task already worked and the user enjoyed it; no-change would have been valid. These results do not disprove culmination or prove a TSR advantage.

A future prospective pair should first test exact stale-edit rejection as useful fresh-decision feedback and nested pure abstention as a known format-error regeneration, without coercing either output or relaxing source checks. Both arms must receive any repair equally and retain this failed pair. Broader independent post-action findings still need an automatically attributed, source-bound reflection entry before claiming their consumption. No additional run was launched from unused allowance.

Independent final review by `/root/handoff_audit` verified the retained failures, exact decode reasons, candidate/source hashes, 15-check result, unchanged ordinary-arm source and accounting; no blocking factual error found. It confirmed that the comparison remains incomplete and the broader final checks remain unconsumed.
