# Conserved obligations and prospective regression

The subsequent [shared-kernel extraction](engine-shared-kernel.md) connects these
mechanisms to notebook and independently reviewed project outcomes.

Kernel `context-transfer-v4-conserved-obligations` corrects three decision defects
identified in the independent review of `28b0736`.

Original wants retain independent obligation identity even when their lineage links
them. Generated note subsets inherit their original parent's obligation identity.
Initial validation prohibits subset aims and generated refinements have one earlier
parent, so that distinction is recoverable in existing notebook histories. It is
not a general semantic equivalence detector.

Only originals enter lineage diffusion and consume the attention limit. Generated
subsets of selected originals remain visible as generation hints. `selection.selected`
is the original attention set; `selected_desires` additionally includes those hints.
Ranking canonicalizes this set back to originals. An overlapping decomposition
cannot add graph mass, duplicate reward, change continuity, or manufacture successful
learning examples. Generated hints can still change what the model proposes; no new
live inference was run to measure that influence after this correction.

Ranking uses positive minimum/mean progress on attended original obligations and
subtracts losses across **all** originals, including already satisfied and unattended
ones. Known mechanical action losses cap optimistic predictions before scoring.
Each assessment exposes `standing_loss`. The loss is a tradeoff penalty, not an
absolute prohibition: sufficient predicted gain may still justify regression.
Losses are summed once per original; gains use the existing minimum plus quarter-mean
proxy. These are explicit engineering choices, not derived TSR laws. Original
obligations supplied redundantly by an operator are not automatically merged.

Belief eligibility and switching compare canonical original support sets, independent
of ordering or parent/child aliases. Actual success requires original improvement
with no original regression. Learning and belief reconstruct that success from saved
before/after vectors, including old histories, without rewriting stored labels.
Historical assessments remain historical; audit comparisons require matching kernels.

## Verification

Using installed Rust 1.90 via `/Users/n/.cargo/bin/rustup run 1.90.0`:

- `cargo test --offline -p tsr-cli --lib`: 75 passed. Log: `target/conserved-lib.log`.
- `cargo clippy --offline --workspace --all-targets -- -D warnings`: passed.
  Log: `target/conserved-clippy.log`.
- Full `cargo test --offline --workspace --no-fail-fast`: 794 passed, 2 failed,
  5 ignored. The failures are the pre-existing runtime-dependent
  `m0_forecast_default` (126 versus expected 3) and `m0_harness` (deleted fixture),
  documented in AGENTS.md. Log: `target/conserved-workspace.log`.
- Changed Rust file formatted with Rust 1.90; `git diff --check` passed.

Controls cover nested/overlapping decomposition, support permutation, unchanged
original attention, nonzero parent/child continuity equivalence, child-only progress,
legacy success-label reinterpretation, prospective coverage/brevity losses, repeated
unchanged inputs, and restart/contrary returns. The older test asserting extra subset
reward was corrected because it encoded the reviewed defect. Protected tests were
not changed. Independent review caught the label and continuity compatibility leaks;
they were corrected before commit. No live/paid inference or external calls occurred.

## What remains at the goal level

The notebook adapter still measures coverage and brevity. Its generated subsets are
**decomposition**, not new kinds of desire. Copying source notes establishes neither
truth nor understanding. This correction does not establish full TSR semantics or
unattended autonomy across domains.

The existing `tsr-domain` contracts already separate `Domain` (candidate/outcome
specification and observation), `Council` (generation from preferences/tensions),
`DecisionEngine`, `Executor`, and provenance-bearing `OutcomeDef`/`OutcomeRecord`.
They can host richer actions and evidence without replacing history, authority,
forecasting, or the reflective cycle. However, notebook `assess` currently depends on
`Spec`, `Aim`, `Move`, and `Notebook`; changing the enum alone is not a domain adapter.

The next coherent implementation is to extract its obligation-level assessment into
a pure shared kernel, with explicit immutable obligation IDs separate from lineage.
An adapter then provides bounded actions, observed satisfaction with evidence and
unknown/disputed status, supported refinement relationships, known effect constraints,
and an outcome-relevant similarity function for transferring forecasts. Reuse existing
coding acceptance evidence as one possible adapter, rather than inventing another toy.
A generated independent obligation needs separate admission rules; inferred preference
never supplies permission to act.

Required domain semantics are concrete: what outcomes are valued, what independently
observes them, whether any constraints must never be traded away, and who may admit a
new independent desire. Reconstructible repository requirements can be evidence;
subjective values and irreversible permissions cannot be inferred from a proposal.
No clarification blocks these deterministic fixes. Choosing new values or asserting
semantic truth would require that explicit domain contract, not more notebook scoring
or benchmark polishing.
