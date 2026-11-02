# Historical RTS external-correction plan

**Superseded by the accepted local repair (`5d709b1`). Do not dispatch this run.** The plan and
request hashes below are preserved as historical evidence; the final functional audit requires no
external inference.

**Not authorized or dispatched.** This plan matches the parent's pending ten-call / $1 request.
The previous two-call grant remains consumed. The exact prepared manifest is
[engine-rts-correction-plan.json](engine-rts-correction-plan.json).

- Provider: `https://api.cheaperinference.com/v1/chat/completions`.
- Model: `deepseek-v4.1-flash`.
- Maximum: **10 distinct corrective proposals, $1 total, 100,000 total tokens** for this new episode.
- Each call: 2,048 output tokens and a 55-second timeout. No automatic transport retries.
- Existing $100 aggregate cap and shared ledger remain authoritative. Each reservation is checked
  against both caps; the current helper's minimum reservation is $0.25.
- First request SHA-256: `f6cc1f73cda47f5671a710162c608d16a9a10f649f0174ed54f2e0b057c383fc`.

The new episode was prepared locally through `project init --carry-from`, from the failed
candidate's admitted workspace. The only editable source remains the `supported_languages`
body in `crates/rts-core/src/lib.rs`. Protected bytes and the original episode remain intact.
No journal was reset and no old receipt was rewritten. The pending episode has no paid journal.

The additions to the proposal payload are the `acceptance_cycle` fields: public guidance on
coverage and metadata, requirement-level met counts and empirical estimates, reviewed-candidate
counts, review IDs and next-step state. The exact guidance and instructions appear in the JSON
manifest. Later payloads may refresh bindings, the same admitted source window and bounded
observations/guidance within these fields; they may not add source files, private reviews,
reference correction code or hidden tests. Semantic rejection requires a fresh operator review
and explicitly published guidance before continuation.

Completion requires sealed named tests and a separate full-suite check, unchanged protected
bytes/original repository, independent verification of every original language and its grammar
version/extensions, current-source acceptance, and accepted-only export. Green tests alone leave
the goal open. Timeouts, uncertain billing, invalid proposals and exhausted budgets stop the run.

**Context limitation:** the allowed excerpt does not contain the complete metadata contract.
The plan adds no source to fill that gap. A model may therefore lack the facts needed for a
correct restoration. I recommend resolving that public-contract context gap before spending;
any additional excerpt would need separate explicit source-sharing authorization. The prepared
plan can test bounded correction under the current scope, but cannot promise completion.

The request and envelope are local files under `target/engine-rts-correction-plan`; their paths,
hashes, price reservation, exact future argv and check list are in the manifest. The argv is
review material, not authorization. No external inference should run until approval is forwarded.
