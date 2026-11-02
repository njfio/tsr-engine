# Live6: engine-generated mashup prototype, incomplete continuation

Implementation 677caae8; packet `experiments/browser-game-continuation-v5/manifest.json`, SHA256 `9ed087ef19bbe2ca6cce0ae26a2dddf7c694f49bfb1d2b810ce938d2ea97a8db`. Trial `b39076e30918e6cd69a2d1d180f890c0d414bdddcd2b0f9b75edcc366c5970d3`. Admission `f6195d5ae4d20f00ff0b436fd4b807cfe726f560759018e17f90b552fd72c720`.

The corrected context path supplied all three complete source files. First generation had duplicate JSON keys and was retained unadmitted; one counted correction produced a complete edit proposal. Separate runtime review accepted it without the previous missing-source unknown. The engine edited app.js and index.html itself. One native isolated browser execution passed. No operator game-code edits occurred.

The design implements the previously admitted shared-board hypothesis: falling pieces occupy the maze and lock into obstacles while the original collector and pursuer remain. This is evidence of implementation consistent with preserved relationship guidance, not causal proof that the relational mechanism improved choice. There was no newly admitted relationship revision in this run.

## Actual independent gameplay evidence

`target/browser-game-session-live6/independent-browser/observations.json` records unmodified-source Playwright observations under the pinned network-disabled container. Ten of eleven checks passed: start, movement/collection, timed falling, rotation, hard drop/locking/new piece, clearing locked cells on restart, visible jam, restarting after jam, controls and absence of script errors. The restart collectible-count check failed: 193 initially, 189 after one restart, 161 after another.

Independent static review confirmed a second defect: row clearing is impossible through controls. Pieces spawn at x 1..9, span at most four columns, and have no horizontal movement. They cannot occupy x 13..18, while every interior row includes open x 17/18 cells that clearing requires. Restart regenerates dots from modified walls before a later listener restores base walls. These are real game defects, not test failures to suppress.

Unchanged engine output is available at `artifacts/lantern-maze-v2-prototype/index.html`, with README, screenshot and source hashes. It is explicitly an unfinished prototype, not a completed or formally accepted mashup. Controls: arrows/WASD, Q/E rotate, Space drop; reload the page for a genuinely fresh board. V1 remains unchanged. No browser was opened or refreshed for the user.

## Post-return controller blocker

After the completed execution, native reconsideration was offered. Projection rejected the carried relationship as unowned because its native-derived `current` flag changed true→false with the new source snapshot. Every other history field was identical. Ownership incorrectly hashed mutable applicability with immutable provenance.

The process stopped before another model request. The outer journal retains pending `reconsider_1_proposal`; fresh-process replay safely refused redispatch with `uncertain recorded effect; no retry`. This is not clean terminal completion. All actual provider calls and executions are settled; the pending controller stage is conservatively unresolved. No journal was rewritten, reopened or bypassed.

A prospective local fix now allows only a boolean applicability flip against an exact admitted full-record hash. It preserves the actual false value; changed meaning, parents, review or root are rejected. Independent review passed. Eleven capacity/provenance tests and two affected native tests passed. The actual preserved before/after history also passes the new check while all other field mutations fail. No live calls were made with this later fix. The frozen v5 packet remains tied to 677caae8; it has not been repinned retroactively.

## Accounting and preservation

Launch 07:07:45UTC, process stopped 07:11:31UTC on 2026-10-29. Three model calls, one shared correction, two operation groups (native execution plus independent functional check), settled cost $0.007243. Campaign 276 calls / 76 operations / $0.256900; no unknown external charges or executions. Remaining cumulative capacity 189 calls / 98 groups, but no second attempt was launched.

Original live5 native history, V1, protected browser harness and probe are byte-identical. Native execution receipts: `d5a5c78db729af2bffe9ccbf2bac576b69d96445eb1974c5f74d22792d956884`, `a8c326cfdaf2f8c9588ec295e7d19ca6f98d6669a12ba3d2ca380cc2a219bc09`, `ca6c7c57f57d1297d6354fda61685dca5e797a85636c5ba4c4ede10592fc84e3`.

Evidence: `target/browser-game-session-live6/{adapter/adapter.jsonl,cycle/cycle.jsonl,run.stderr,final-accounting.json,preservation-check.json,replay.stderr}`, `target/browser-session-preflight/{actual-review-coverage.json,context-final-unit-tests.log,context-native-tests.log,context-sealed-approved-tests.log,live-history-regression.json,relationship-native-tests.log}`. Earlier default-sandbox browser tests failed because Docker socket access was denied; both passed through the normal approved execution path. A misspelled test selector and an erroneous direct-double-review assertion were corrected locally; no hidden live retry.

Next substantive work is a verified separate source-bound recovery from the known completed execution, preserving the pending original and admitted relationship, then letting the engine receive genuine gameplay defects and repair its own design. It must not rerun the completed edit, fabricate a return, manually repair game code or bypass the pending-stage guard. The requested usable mashup remains unfinished.
