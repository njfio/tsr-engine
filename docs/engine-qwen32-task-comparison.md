# Installed Qwen32B comparison on the named-list task

New prospective episode at `target/collection-qwen32-episode/`, initialized from
6747b3c. Episode identity:
`1391ae0ac2e8bbaadf105567c9eed615181a7b6b654ff08f61353d388c37f0fc`.
The two prior rejected episodes are preserved and remain closed. This is another
exposed development episode of the same task/clone, not independent corpus support.

Installed Ollama model `qwen2.5-coder:32b` (32.8B Q4_K_M), digest
`4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`,
was verified against the local tags endpoint. No download or external inference.
Generation is temperature zero, context 16,384 and output ceiling 2,048 tokens.

Scope is identical to the supporting-context episode: 323 frozen files from the
original admitted mutant, only bytes 5862..6562 of Surface::parse editable, and
617 downstream validator bytes read-only. Independent rejected history is carried;
initial guidance identifies that current source is the original mutant and explains
previous observed failures without replacement code. All original requirements,
named/full-suite oracles and optional-hook exclusion remain unchanged. No reference
repair or protected test answer is exposed to generation.

Frozen limits: at most 3 generation calls, each 240 seconds; at most 2 sealed executions;
55,296 tokens, 32,768 reconstructed patch bytes, 2,400,000 tool-ms, 3,600,000 wall-ms;
zero external charge. The existing pinned no-network image and sealed executor are
used. Admission, source hashes and provider metadata are saved before execution.
Active reviewer effort is unknown; no complete-coverage attestation is fabricated.

Result: promotion rejected because full-suite verification is incomplete. No
accepted repair, mature episode or calibrated-performance claim is made.

First call generated a scoped 579-byte replacement using 1,711 input and 261 output tokens in 60,483,116,333 ns. Independent static review found parsing semantics met; the rationale incorrectly says to filter out nonempty names while the code does the opposite. Static review and its 0.8 forecast do not replace observed tests.

## Terminal observations

- Named protected test passed after 114,072ms, exit 0.
- Full workspace oracle timed out at 240,426ms, exit 124, while daemon unit tests
  were still running. Thirteen component summaries completed; partial success is
  not a full-suite pass. Full output/digests are retained in coding receipts.
- Independent static review found named-list, unknown-name and profile semantics
  met. Its final decision rejects promotion solely because required full-suite
  verification is not established. This is not a finding that the repair is wrong.
- One operator-directed warm verification retry was attempted using the existing
  action and current root. The controller refused it BEFORE execution with
  `stale project inspection handle`. No bypass, patch rewrite or retry followed.
  There is only ONE sealed execution, not two. An interim progress claim that the
  rerun was accepted was corrected immediately when its terminal refusal arrived.

One local generation call consumed 1,711 input+261 output tokens and 60,483,116,333ns
model duration. Total charges: 1,972 tokens, 13,357 patch bytes, 359,224 tool-ms, zero external
charge. Two generation slots and one actual execution were unused; none were
silently transferred to another episode. Final accounting has zero unknown usages,
unsettled requests, unresolved tool requests or exceeded limits. Provider PID 2204
was stopped and its tool session exited0. Integrity verified 323 frozen source files,
322 protected working files and all outside-window bytes. No original target edits.

Closure: **2026-10-22T07:47:31.813Z**, reviewed rejection of promotion.
Earliest calendar maturity: **2026-10-29T07:47:31.813Z**. Eligibility remains
`missing_independent_coverage_review`, label null; active reviewer minutes remain
unknown. No positive coverage attestation or accepted-only export was created.
The original forecast 0.8 remains prediction data, not proof or calibrated evidence.

## Concrete integration gap

`project_validate_patch` correctly binds an edit to its original snapshot. Reusing
that edit after it changes source is stale even when the operator merely intends
to rerun verification. The existing project path has no dedicated verification-only
action for current source. More model calls would not fix this orchestration gap.

The next local implementation is a fresh source-bound verification action that
runs only already-admitted sealed named/full commands, freezes its own commitment
and controller forecast, reserves normal tool/wall budgets, and checks source/test
immutability before and after. It must not apply a patch or charge patch bytes,
recycle a generator forecast, relax stale-edit validation, reopen this closed
episode or reset historical budgets. Tests should cover changed-source rejection,
timeout/reservation persistence, restart, accepted-only export and receipt linkage.
This is a new legitimate action contract, not a way around the refused edit. It was
identified but not implemented in this comparison checkpoint.

Qwen32B generated a statically sound candidate on its first attempt where the smaller
model failed. This single scoped result does not establish reliability or superiority.
Operational calibrated evidence, independent review effort and the protected-test
cutoff decisions remain separate dependencies. No hosted provider is needed to
address the newly observed verification integration gap.

Evidence: admission/preparation/provider-pin; original proposal and local receipt
journal; `execution-1.json`, `oracle-summary.json`, `verification-rerun.json`,
`execution-2.stderr`, independent static/final review, bound `review-1.json`,
`closure.json`, `final-budget.json`, `final-eligibility.json` and `integrity.json` in
the evidence directory. Current CLI build passed; no source implementation changed
in this pass. The prior 321-test/clippy result applies to the unchanged implementation;
no new full engine suite pass is claimed. Protected formatting/cutoff tests unchanged.
