# Sustained browser harness verification

2026-10-29. Implementation: `7ec87120`, `93d1d377`. Frozen prospective plan: [harness-sustained-verification-plan.md](harness-sustained-verification-plan.md). Packet: `experiments/harness-sustained-v2/manifest.json`, SHA-256 `12b8a0759bd033497f4f058312a2f48201d04510ef860d4bb657d9dca76fc2bc`.

## Result

One live session completed two consecutive independently reviewed edit/execution checkpoints without manual intervention, correction requests, journal edits, prompt fixes, source repairs, or model-code substitution. The engine first added a help panel with mouse/keyboard controls, then consumed a second persistent, explicitly synthetic verification input and added accessible announcements. These are harness verification inputs, not claimed player feedback. The project was a separate copy of engine-authored Lantern Maze v2; delivered v1/v2 remain byte-identical.

The live session ran 10:08:43–10:11:37 UTC. It used four Luna calls (two generators, two independent semantic reviewers), two native browser execution groups, and zero corrections. One additional independently counted read-only browser verification passed **23/23 checks**, including actual maze movement, falling blocks, rotation, reachable clearing, life loss/restart, help controls, no reset/focus theft, and polite announcements. Controls were exercised through clicks/keys and virtual time; canvas/DOM inspection did not mutate game state. No browser errors were observed.

Fresh-process replay returned the completed session without dispatch. Campaign ledger, adapter journal and native JSONL hashes remained unchanged. Both checkpoints remain native `review_required`; no formal final acceptance was invented or imported. Independent functional verification is distinct from native final acceptance.

Session cost: **$0.005220**, four calls and three total execution groups. Final campaign: **292 calls / 81 groups / $0.291010**, no unknown calls/executions, no overrun. Admission allowed44calls/8groups/4sharedcorrections/$3 within existing cumulative caps; unused allowance is not a new-run grant.

## Systemic changes

- Shared strict wire-schema validation now handles emitted types/unions and explicit limits consistently. Typed request-local references replace model copying of long parent/revision identifiers. Trusted transport injects only exact request-bound metadata; mismatches remain rejected.
- Browser context/output/reservation paths use the declared provider-capacity profile, complete relevant source where it fits, explicit source windows otherwise, and prospective counted regeneration. No silent truncation or code repair.
- Exact durable stage receipts close a completed-callback/outer-journal interruption without repeating the callback. Known local capacity/technical stops are distinct from unknown external effects. Valid rejection and unknown assessment remain attributed feedback, not review-until-approval.
- Actual execution-source receipts replace a combinatorial hypothetical-source enumeration. Relationship/source lineage and stale checks remain enforced.
- A finite persistent feedback session moves through separately recorded checkpoints with one shared budget; it does not reopen a terminal grant. Feedback provenance is explicitly projected.
- Generic scope grants bind exact admission/trial/prefix/baseline. A preflight Unicode canonicalization mismatch was caught before dispatch and fixed by deriving grant identity through the enrollment implementation. The first prospective packet remains preserved.

## Local verification and failures retained

These suites overlap; counts must not be summed as unique tests.

| Evidence under target/ | Result |
|---|---|
| harness-fast-final2.log |68passed,1skip|
| harness-js-final2.log |authored JS controls passed|
| harness-native-final.log |2sealed authored full native flows passed,221.563s|
| harness-compatibility-final.log |23passed,2opt-in skips,25total|
| harness-stop-final.log |18passed|
| harness-contract-final.log |28passed,1historical opt-in skip|
| harness-identity-final.log |Unicode grant/enrollment regression passed|

The authored native flows cover relationship proposal/admission, execution, post-execution reconsideration, multi-checkpoint feedback and restart. Failure injection covers typed references, stale/cross-request data, interrupted stages, capacity/accounting and replay. These are wiring/failure-containment evidence, not live generative behavior.

Earlier broad run `harness-local-first.log` recorded68tests/14errors/6skips from changing source pins, stale binary and module path setup; the frozen reruns above resolved those issues. The first JS run used a console test double missing Playwright's `text()` method; corrected authored test passed. `cargo +1.90.0` was unavailable with the installed standalone Cargo; `cargo build -p tsr-cli --bin tsr --offline` passed. Python compilation and `git diff --check` passed. No blanket workspace test claim is made. Lefthook was absent; relevant checks ran manually.

## Limits and interpretation

This demonstrates **two unassisted feedback→proposal→independent review→edit→execution checkpoints**, not universal hands-off autonomy. The live session did not need malformed-output recovery, experience-driven relationship revision or crash recovery; those paths have local authored coverage only. It generated no new relational meaning, and no TSR causal advantage is claimed.

Some human-readable relational contracts still duplicate native rules; this is not complete generated Rust-to-wire schema unification. Reviewer near-capacity handling remains explicit source coverage/windows and may legitimately stop unknown. Recovery cannot infer the outcome of an external effect whose completion evidence is missing: it stops for reconciliation. Actual model and application budgets remain explicit (including project/response/file-count allocations); they are not advertised as immutable hardware limits or unlimited capacity.

The approved Chromium-only sandbox exception remains; outer non-root, network-none, read-only, capability/resource/mount controls were retained. No installs, public deployment, host-security changes or delivered-game edits occurred.

## Local evidence

`target/harness-sustained-live2/` contains launch/process records, original responses and source-bound journals (not committed), `run.stdout`, `replay-verification.json`, `delivered-preservation.json`, and `independent-browser/{verify.cjs,observations.json,screenshot.png,result.json}`. Failed/preflight histories are retained. Provider response bodies are intentionally not committed.

Independent result review by `/root/harness_audit` passed with no blocker. It verified the second review received complete current source, the first actual browser return and both feedback provenance records; final source equals the recorded model-generated edit sets exactly. It explicitly confirmed no relationship proposal/admission or reflection/reconsideration occurred live. Independent source/admission reviews preceded dispatch.
