# Lantern Maze v2 — verified playable mashup checkpoint

Engine implementation: `24284262`. Prospective packet: `experiments/browser-game-recovery-v4/manifest.json`, SHA-256 `6462e1a8563b85cf2f6c2e06ff7b9b5403f88c9a2fbf2c931cba0e808fa03310`. Run: `target/browser-game-session-live10`.

## Delivered result

`artifacts/lantern-maze-v2/index.html` and `target/lantern-maze-v2.zip` contain byte-identical engine-generated source. The operator added only README/provenance and copied the independent screenshot. V1, unfinished v2 prototype and all failed runs remain unchanged. No Safari action, upload or deployment occurred.

The engine chose horizontal falling-piece steering and completed open-lane-segment clearing, integrated with maze collection and pursuit. It restored base maze walls before calculating collectibles. Controls: arrows/WASD maze movement; J/L piece steering; Q/E rotation; Space drop; Start/Restart reset. From a freshly opened page: Start, E, Space, Space, Space demonstrably clears a lane segment.

Independent protected browser execution passed, followed by 16/16 separate gameplay checks. These included actual keyboard-reachable clearing (no injected game state), horizontal movement, rotation, drop, clock-driven falling, maze collection, repeated restart with 193 collectibles and cleared placed blocks, actual pursuit-induced life loss, and subsequent restoration to three lives. Playwright virtual clock is disclosed. Canvas/DOM reads observe state without altering game code. Source inspection independently confirmed the reachable sequence. Protected harness/configuration files were unchanged.

Source SHA-256:
- app.js: `5a8aadfa34a921e111e15c381993f5d182f6e42981093ea4350a370dd4d38317`
- index.html: `5938a9f7632fa4a27597bc5e1acceeccc2420ae40025e94da1d95b9ebdfaf499`
- style.css: `6481b0239be5ae45a322d5c097305c93da674ffbed205fc4578db9cc0447c363`

## Actual trace and limits

Live8 independently reviewed and admitted a refinement of the original shared-board relationship, retaining both contributor identities and original meaning. Live10's ordinary proposal then implemented reachable segment clearing and reset changes; independent runtime review approved before execution. Post-return reconsideration was independently reviewed and admitted. It conservatively treated the bounded smoke result as insufficient to verify clearing/reset effectiveness and still carried the earlier independent defect assessment. The later 16-check gameplay assessment was not consumed by that reflection. Do not claim it learned the independently verified successful outcome, general desire identity succession, causal TSR advantage, or full autonomous evaluation reliability.

Native terminal status is `review_required`, intentionally awaiting independent acceptance or further feedback. No formal native result acceptance was imported. Fresh-process replay returned that status without model requests, executions, ledger changes or native-state changes.

Remaining limitations: clearing applies to open maze-lane segments, not full Tetris rows/rules. Piece sequence continues across Restart (page reload resets it). Balance, winning the entire maze and player satisfaction are unproven. The 16 tests establish the listed functional behaviors only.

## Preserved failures and costs

- Live7: one request, $0.001434, wrong case-name citation plus scalar unknown; unadmitted, no execution. Original pending outer journal preserved. Prospective source-bound schema fix `01a13862`.
- Live8: five requests, $0.009412; admitted relationship development, then reset-only semantic rejection and duplicate-file edit layout failure; no execution. Prospective validated-layout regeneration `9c87b33d`, no merging or changed source validation.
- Live9: two requests, $0.007753; broader valid proposal, then reviewer mistyped a candidate hash. No execution or review adoption. Prospective exact request-local citation aliases `24284262`; full request and raw response bound before deterministic expansion, unchanged native validation afterward. Old wrong citation is never repaired.
- Live10: four requests, $0.010291, zero format corrections, one native browser execution plus one independent gameplay execution. No provider-failure retries.

Since the requested recovery baseline276/76/$0.256900:12 requests,2 operations,$0.028890. Final campaign:288 calls,78 operations,$0.285790, no unknown calls/executions and no overrun. Cumulative caps remain465/174/$10.232177; this run remained below$10. No further run is implied by leftover capacity.

## Supporting evidence

- `target/browser-game-session-live10/{run.stdout,cycle/cycle.jsonl,adapter/adapter.jsonl,final-accounting.json}`
- `target/browser-game-session-live10/independent-browser/{verify.cjs,observations.json,result.json,screenshot.png}`
- `target/browser-game-session-live10/{source-lineage.json,replay-verification.json}`
- Local correction tests:6 return-schema/native tests,3 edit-layout controls (15 unrelated opt-in tests skipped),19 alias/association/native tests. Prior full authored recovery flow passed with an actual isolated fixture browser execution; it is wiring evidence, not live game behavior.
- Independent source, exact-binding and final-result review: `/root/binding_review`.

An earlier schema native test encountered pin drift while source edits were still landing; the frozen rerun passed. An alias integration test initially expected legacy long evidence keys; it was updated to inspect the explicitly published alias map and passed. These failures remain in local logs. Lefthook was absent; listed manual tests supplied verification.
