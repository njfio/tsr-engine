# Preserved envelope / admissibility audit (c137ee2)

No inference, generated candidate substitution, live execution, or live-state edits
were performed. This audit reads requests 19–28 and current controller code;
requests 19–24 used qwen2.5-coder:32b, 25–27 used the installed Qwen tools model,
and 28 used mistral:latest. All ten terminal results are inability, not assessed
candidate outcomes. Request 13 remains unknown/abandoned, not refunded.

## Findings

| Constraint | Preserved requests and actual controller | Conclusion |
| --- | --- | --- |
| Joint satisfaction | Requests 20–28 explicitly permit partial progress; 19 did not have that clarification. `constraint_status` certificate is absent from all ten generation contexts. Typed aims still name their full desired target. | No requirement that every want become satisfied; keep targets unchanged. |
| Alternative count | Every preserved schema requires two (max two); Rust requires two valid actions, except the single-note empty frontier. Rust selects only among fallback-eligible alternatives. | Two selectable improvements are **not** required. The prompt's “useful distinct possibilities” was stronger than validation; corrected to valid alternatives that may reveal tradeoffs and fail later scoring. |
| Distinctness | Python full-plan identity includes continuation. Rust deduplicates dispatched first actions. | Concrete mismatch: two future plans for the same inspection passed stage one and failed Rust. Stage one now requires distinct first-action effects as well, retaining full-plan identity for evidence. |
| Unchanged accounts | Same claim set and title word count are the same notebook measured effect; summaries bind the state and preserve receipt provenance. Only an entire offer of previously established unchanged effects is excluded. | Changed effects remain admissible. Different wording alone is not a new direction. This domain equivalence is not a semantic-language claim. |
| Losses | Shared kernel subtracts standing loss; bounded fallback permits positive-net reversible direct tradeoffs, not arbitrary harmful exploration. Assessment instruction said protect fulfilled wants from regression. | Overstrong wording corrected: forecast losses explicitly; independent accounting determines admissibility. No desire is weakened. |
| Prerequisite | No selected original measures knowledge alone here; inspection needs a dependent composition. First-step effects exclude continuation; two action slots required; later action is selected anew. | Consistent with controller support and authority. Six action slots remained; budget did not cause inability. |
| Claims and lengths | Compose uses known IDs; planned compose may use IDs acquired by its inspection. Word count includes full source text and title; unknown lengths stay unknown. | Schema does not fix title length or demand full coverage for every action. Hidden feasibility witnesses were never inserted into requests. |

The independent feasibility analysis remains valid: a selectable partial
composition and a valid, potentially unselectable inspection plan are structurally
possible. The two-alternative requirement does not prove this state infeasible.
No specific witness action is supplied to the live generator by this correction.
The general minimum-two design may deserve separate future review for other
frontiers; this audit does not prove all possible states have two alternatives.

## Correction and evidence

`scripts/engine-directions.py` now uses the explicit valid-alternative contract,
clarifies reversible-loss assessment, and rejects duplicate first-action effects
before freezing possibilities. Full-plan identities and old receipts remain
unchanged. New request metadata uses `valid-first-action-alternatives@v2`.
`scripts/test_engine_directions.py` tests different continuations sharing a first
step, unchanged controls, frozen identity, restart and stale-stage behavior.

Offline checks passed: 63 Python tests, one exact-effect regression, three
prerequisite tests and one reversible-tradeoff test. Logs are recorded in `target/envelope-audit-python.log` and
`target/envelope-audit-rust.log`. They are implementation evidence, not model
recovery. No current response has exercised the corrected wording. Consequently
prior reports establish a capability limit **under the recorded contracts**;
they do not prove an intrinsic inability of the installed models under every
correct representation. Another identical audit or unchanged sampling would add
no evidence. A later authorized live check must explicitly name this changed
contract; none is dispatched in this pass.

## Next original PRD mechanism

PRD §B8 explicitly specifies commit-margin abstention and optional epsilon
near-tie exploration with logged propensity. `deliberation::choose_supported`
currently chooses deterministic highest score (input order breaks exact ties),
and committed propensity is always 1.0. Quiet-route attention and model sampling
do not implement action-selection exploration. The domain already defines
`AbstainReason::NearTie`, `Tune.epsilon`, and `Focus.commit_margin`.

The next implementable core step is a shared, evidence-labeled bounded selection
policy over already assessed/authorized candidates: margin abstention by default,
explicit operator exploration settings, reproducible draws and exact propensity,
with restart, permutation and no-authority-expansion controls. It must use and
report proxy scores until validated Tier 1 common-event estimates exist, and
must not call that calibrated Tier 2. This requires no invented value theory or
new external authority. Mature-episode data/calibration remain a separate exact
dependency: 151 qualified tasks are not 150 eligible mature episodes per event
and 30 per family; current independently eligible count remains unknown.

Independent read-only review confirmed all three corrections, including the
single-stage loss wording, with no remaining scoped blocker. Diff checks passed.
