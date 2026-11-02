# Desire development and succession boundary audit

2026-10-31. Offline audit after the staged-experience launch was denied. No paid dispatch, state migration, game edit, or change to the frozen evaluation packet was performed.

## Sources and findings

The source of record is `docs/PRD.md` (B2.1, B6 and B8), read with `docs/engine-relational-culmination-review.md`. The external original book and drawings were not available in the scoped checkout. This audit does not claim to have checked them.

Three implemented mechanisms have different semantics:

1. `crates/tsr-ledger/src/write.rs::Ledger::expand` creates a content-addressed preference with contributing parents from a recorded, nonempty Contrast. Generated preferences require reviewed opening before activation. The engine also has a separate `Engine::expand` helper (`crates/tsr-cli/src/engine.rs`) that appends an Expansion; its callers must be audited rather than assuming it has the ledger method's validation. The narrow refinement path in `engine/deliberation.rs::refine_directions` derives note-related refinements of an unmet parent. These existing new identities are not a general browser relational-succession implementation.
2. `crates/tsr-cli/src/engine/adaptive/coding/project/desire.rs` supports independently reviewed clarification/development of an existing active identity, retaining ancestry and revision history. It does not create a new preference identity or change acceptance criteria. `scripts/engine-browser-project.py:106` rejects `inputs.public_desires`, so this native capability is not available through the present browser adapter.
3. `project/relational.rs` admits relational aspirations and permits reviewed prior aspirations as catalysts while preserving their contributing parents. These are `CodingRecord::RelationalAdmission` records, not ledger preference identities. They can influence possibility generation without creating new independent obligations, fulfillment claims, selection bonuses or permissions.

The acceptance feedback path initializes aims only when `acceptance_current_aims()` is empty (`project/acceptance/cycle.rs`). Its initialization/status logic is not evidence that ordinary feedback withdraws and replaces desires on every round. Administrative status, permission cancellation, meaning development and culmination must remain distinct.

## Smallest supported changes, and the unresolved boundary

The smallest concrete wiring repair would expose the existing same-identity public-development proposal, review and admission path through the browser adapter, with exact current-source bindings and unchanged criteria/permissions. That should be called meaning development, not new-desire formation. It requires coordinated response-schema, review, projection, admission and restart tests; removing the projection rejection alone would be unsafe.

A further contrast-backed, multi-parent generated preference can reuse ledger persistence and independent review. However, general browser succession still lacks a specified contract for when that preference becomes an eligible contributor, what standing obligations/criteria it has, and how its fulfillment affects subsequent pursuit while preserving every contributing parent. The design review explicitly leaves independent preference value unresolved. Successful combination cannot be relabeled as unmet Contrast merely to pass expansion validation.

No speculative runtime implementation was made in this audit. In particular, we did not give a derived desire independent value, replace parents, waive an original obligation, prescribe a third goal, or count stored aspiration prose as behavioral culmination. The current frozen evaluation remains usable for testing reviewed reusable composition, not general identity succession. Absence of novelty on a satisfied task remains a legitimate outcome.

Independent read-only review by `browser_objective_review` agreed with this mechanism separation, the public-development wiring gap and the unresolved eligibility/obligation contract. No new automated runtime tests were required for this documentation-only audit; the prior staged-experience test results remain separately recorded in `docs/staged-experience-repair.md`.

## Live launch remains denied

Both exact launch attempts were denied before execution. `target/relational-experience-live-v1/permission-block.json` preserves both reviewer reasons. Neither `launch.json` nor `run.log` exists. No new call, execution group or uncertain effect resulted.

The reviewer asserted a "$100 two-task pilot" restriction. Its only identified source here is the reviewer's rejection text; no matching direct user instruction or repository policy was found in the inspected material. The first attempt had the delegated unlimited-engine authorization and original user history available. The second had no new user confirmation: it added the existing append-only policy binding and verified non-execution, referring to the original October 30 unlimited instruction. Review still denied. Neither attempt placed authorization evidence inside the action payload.

No further retry, alternate endpoint, cap change or indirect dispatch is permitted by this audit. The parent must resolve the specific approval-review authority mismatch. Last verified accounting is 395 calls, 129 groups, $0.644668 known and $0.694668 committed, including the historical $0.05 uncertain estimate (not a verified maximum); this audit adds no spend.
