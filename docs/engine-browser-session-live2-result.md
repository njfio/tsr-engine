# Lantern Maze continuation: incomplete output

The engine received the attributed user feedback, “Keep going game needs to be more of a mashup between Pac-Man and Tetris”, in a separate persistent derivative of v1. Prior contributors, source, observations and episode history were retained. No playable v2 was produced.

Implementation: `d3927a71`, `c81f30a5`, `a17b386c`. Packet: `experiments/browser-game-continuation-v1/manifest.json`, SHA-256 `42a8d010b61573ef91fc8879be6034fb55ffe7c23c04fe32b016c4427d29042a`.

## Actual outcome

Run evidence: `target/browser-game-session-live2-launch`. Started 2026-10-29 05:29:36 UTC. The first proposal request returned HTTP 200, provider request `e7e11654-af18-4e75-9871-acb81db258f7`, with `finish_reason: length` and `incomplete_output`. Completion usage reached the configured 8,192-token ceiling, including 6,732 reasoning tokens. The retained content ends mid-edit; its 4,205 bytes were retained completely (SHA-256 `de2faa0af5fc58685c74d3e2a310a633d384fa2c2aa130a9d683fb717bf9c806`). Retention truncation did not cause the failure.

The admitted correction policy excludes incomplete wire output. No correction, semantic review, source edit, browser execution, adoption or acceptance followed. The run closed its unused allowances. The outer cycle retains pending proposal_0 and refuses redispatch; this is fail-closed restart behavior, not clean terminal replay. No retry was attempted.

## Accounting and preservation

This attempt: 1 call, 0 operations, 0 corrections, $0.003610 settled. Campaign: 265 calls, 74 operations, $0.232177 settled; no unknown calls or operations. Independently checked against the campaign ledger with its exact upgrade binding.

Original v1 native history, delivered game and protected files were verified unchanged. Safari was not refreshed or opened. v1 remains the available playable artifact at `artifacts/lantern-maze-v1/index.html`.

## Interpretation and next dependency

This is a transport/output-capacity stop, not evidence of a failed game design or TSR relationship development. The incomplete proposed source was never run. The configured completion ceiling includes reasoning and left too little capacity for a complete edit response. A larger financial/call allowance alone does not change that request ceiling.

Before another dispatch, the generic browser boundary needs a reviewed capacity-aware policy for known-settled incomplete output, preserving fresh counted requests, fixed source and semantic task, strict validation and no unknown-effect replay. Do not silently reclassify the historical failure, concatenate partial code, reopen this closed admission or claim a v2 exists. Actual provider-supported token/reasoning controls must be verified before choosing the correction. Existing safe incremental-edit support can also be used without prescribing game content.

Independent result review confirmed the receipt classification, accounting, preservation and fail-closed refusal. Local preparation tests covered history extension, changed-feedback routing, source projection, source/stale/unknown guards and replay; those tests do not establish live v2 behavior.
