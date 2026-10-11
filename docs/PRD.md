<!-- Source of record: the Claude Doc at https://claude.ai/code/artifact/a25977dc-6582-423f-9c34-7aa9110320f1 (editable, with four drawings: four views of one snapshot in B2, the Filter's two diagonals in B2.1, the path A architecture in B4, and the M0–M6 roadmap in B11). Text export for the TSR5 project; drawings not included. Supersedes prd-v0.3.md and prd-v0.2.md. -->

# TSR Decision Engine — PRD v0.3.1

Oct 10, 2026 · @Nicholas

## Read this first

v0.1 has the right skeleton and the wrong proportions. Its best idea — freeze the expectation before acting, record the return separately, learn only from the difference — is the TSR cycle made mechanical, and it is already the protocol in Chapter 19 of *The Sovereign Mirror*. Around that idea v0.1 wraps enterprise governance for a one-person lab, a learning design (an imitation classifier on 500 reviewed labels) that will barely beat its own rules, and roughly forty disclaimers that make the spec read as apologetics.

v0.2 changes five things:

1. **The learner is an expected-value policy over calibrated forecasts, not an action classifier.** The forecast discipline becomes the learning engine instead of a side feature.
2. **Three forecasters per commitment — generator, controller, operator — all scored against the same return.** Learning *when to trust the generator* is where a controller earns its keep in 2026.
3. **The executor is not rebuilt.** Claude Code (whose hooks you have already built three versions of) runs inside a network-less container; the engine is the authority at the hook boundary. rts does inspection, CRUCIBLE's council supplies candidates, RATCHET's policy profiles finally get built as the `policy` module.
4. **Approval moves to the sandbox boundary.** In-sandbox edits are forecasted and logged; the exit is approved. Otherwise 500 episodes are unreviewable and the labels are fatigue.
5. **The corpus is decided now:** red→green commits mined from your own repositories, mutant-repair tasks from cargo-mutants, and an external Rust benchmark as the never-trained-on check.

Also: governance scaled to one operator; embeddings and Burn off the critical path; a Mirror view that shows the system's and the operator's own expectation gaps; a headline claim aimed at the controller's contribution so the generator cannot drown it; a domain-agnostic core so this is TSR as an AI system, not an agent harness with a glossary.

What survives from v0.1 unchanged: typed actions, policy outside the trainable boundary, frozen pre-action records, outcome records with provenance, explicit abstention, no negatives from unchosen candidates, shadow → active with pointer rollback, and the honesty about selection bias and leakage.

Checked in passing: the TypeSafe Jev and GPT-6 Luna references in v0.1 §16 are real products (existence confirmed, pricing not re-verified). They stay optional and non-load-bearing (Appendix B).

**v0.3, same day.** Working through the review found v0.2's largest gap: it had only the physical half of the Resistance Contrast Filter. The engine accumulated Expectation from return and committed Belief, but nothing culminated on the nonphysical side and nothing flowed back from it as intuition — a well-calibrated holding-on machine. v0.3 adds the other diagonal and corrects the dynamics:

- The Filter is two diagonals crossing at the Point of Reflection: a strong one, Expectation ↔ Culmination, and a weak one, Belief ↔ Intuition — from the Oct 4 derivation and the 2020 slide (B2.1).
- Culmination is a ledger that only grows, written by contrast under the Filter's rule; Intuition is a view that draws next steps from it (B2.1, B6, B8).
- One structure: the Merkle DAG of events, with a conflict relation, is formally a prime event structure; flows are folds over it, never stored weights (B6).
- Inverse dynamics per side: the physical side settles (predictive updating), the nonphysical side expands (autocatalytic). Vortices are autocatalytic sets; holding-on loops are the curl of the action flow (B2.1).
- A fourth named event, `intuition_followed`, and two new ablation rows, so the structure earns its place by evidence rather than by fit (B7, B9).
- Milestones do not move: structure and writes land in M2, the flow view in M3–M4, evaluation in M6 (B11).

**v0.3.1, same evening.** A second review landed seven corrections and you added an eighth; all are in. Surprise (prediction error) and Contrast (desired minus actual) are separate records, and only Contrast drives expansion. The Filter admits by source, not by content: the failure's evidence is referenced, the failed strategy is never a positive example. Intuition has a mechanism — select, find tensions, let the council compose bounded steps with traceable support, with a quiet route for new preferences. Preferences carry status in the active view, so accumulation cannot become holding on. Claude Code drives with provider-only egress while every command runs in a sealed sibling sandbox. Every candidate is scored on one common decision event. Greens on the evaluation set are reviewed in full. Every evaluation arm runs on its own fork of one pinned state — frozen-state as the primary protocol, a controlled continual sequence as the secondary. The formal mechanisms (event-structure axioms, autocatalytic sets, Hodge) move to a gated research section (B9.1) behind the two-desires test.

## A1 · Review of v0.1: what to keep

These are the load-bearing ideas. v0.2 keeps every one of them, in some cases with a sharper reason.

- **Frozen pre-action record, separate outcome record.** The prediction is never confused with the hindsight. This is the cycle, mechanized, and it is the single most TSR-faithful thing in the document.
- **Fixed, typed action vocabulary.** Generated text proposes; it cannot name a hidden command or authorize its own execution.
- **Policy outside the trainable boundary.** No authority flows from scores, reflections, repository text or provider output. Training has no write access to permissions or promotion.
- **Explicit abstention as a first-class result**, always paired with a safe next step.
- **Unchosen candidates are not negatives; failed outcomes are not imitation targets.** v0.1 states this as data hygiene. It is also the Resistance Contrast Filter (A4).
- **Outcome provenance** with pending / observed / censored / missing / disputed / invalid, and corrections that supersede rather than rewrite.
- **Idempotency key + expected state version on every command; dispatch intent committed before the effect; reconciliation for unknown outcomes.** Exactly-once is not promised across arbitrary tools, and the spec says so.
- **Approval bound to exact digests** (artifact, snapshot, scope, expiry), re-checked at dispatch.
- **Shadow → active with pointer rollback; incumbent retained on an inconclusive result.**
- **"Valid structure is not a correct decision"**, and selective accuracy reported with coverage so abstaining on everything cannot look like success.
- **Honest treatment of selection bias and leakage routes** (future tests, later patches, post-action logs, retrieved episodes).
- **Human-reported experience is evidence with its own provenance**, never reward truth; refusal and disagreement are never classified as resistance.
- **The analogy is finite.** No claim to instantiate the Current, demonstrate consciousness or establish metaphysics. v0.2 says this once instead of forty times.

## A2 · What to change, ranked

Each item: the problem in v0.1, then the fix v0.2 adopts. The first six change what gets built; the rest change how.

1. **Replace the imitation classifier with an expected-value policy over calibrated forecasts.** v0.1 trains a six-way action classifier on "reviewer-endorsed next-action labels" from about 500 decisions. With heavy class imbalance (inspect and test dominate) it learns the prior and ties the rules. Fix: learn calibrated P(event | state) for three named events — approved test passes, operator accepts the diff, task closes within budget — then pick the action with the highest expected value minus cost, abstaining when support is thin. Hundreds of examples suffice, every choice is explainable as numbers, and it is literally "act on Expectation, learn from return".
2. **Make the generator's self-forecast a first-class signal.** v0.1 treats the generator only as an untrusted proposer. The generator holds nearly all the task intelligence; the controller's edge is knowing when to trust it. Fix: before any gated action the generator states its own probability for the named event, the controller records its own, and the operator may add theirs. All three are scored against the same return. Learning to discount an overconfident generator per task family is a measurable, TSR-shaped result.
3. **Do not build the executor; wire the one you already have.** v0.1's eight modules include a sandbox executor, patch applier and approval flow from scratch. You have Claude Code hooks (V1–V3), rts, CRUCIBLE and a RATCHET spec. Fix: Claude Code runs inside a network-less container on a disposable copy; the engine binary is the PreToolUse / PostToolUse authority (allow, deny, ask) and the forecast recorder; rts serves Inspect; CRUCIBLE's council supplies alternative candidates; RATCHET's protection profiles become the `policy` module's config. A Rust-owned executor stays on the roadmap behind the same trait (B4, path B).
4. **Move approval to the sandbox boundary.** Per-patch approval times 500 episodes is 25+ hours of clicking and yields fatigued labels. Fix: in-sandbox edits are forecasted, logged and reversible (git); approval is required to promote a diff out, run a non-allowlisted command or touch the network. Review happens after close: every failure, every green-but-rejected, and a random 20% of successes.
5. **Record uncalibrated forecasts always; act on them only when calibrated.** v0.1's "probability only when a calibrator is valid" is a chicken-and-egg: without stored forecasts there is nothing to calibrate. Fix: every forecast is stored with `calibrated: None` until a validated calibrator exists; display and gating use the calibrated value only.
6. **Add logged exploration, or give up the causal claim.** v0.1 says unknown propensity must stay unknown — and a deterministic policy logs propensity 1.0 forever, so off-policy evaluation is impossible by construction. Fix: among candidates whose expected values lie within a declared margin, choose with a small operator-set ε and record the propensity. This is the only randomness in the system, and it is bounded, logged and auditable.
7. **Decide the corpus now.** v0.1 defers it to discovery with licensing worries. Fix: `tsr corpus mine <repo>` extracts red→green commits from your own repositories as tasks with built-in oracles; `tsr corpus mutants` uses cargo-mutants to generate hundreds of localized bugs with known fixes; a Rust split of an external SWE-style benchmark is the held-out check. No licensing question, personal relevance, and volume for calibration.
8. **Take embeddings and Burn off the critical path.** The encoder-compatibility spike is v0.1's largest schedule risk and the MVP does not need it. Fix: structured features first (task family, inspections so far, patch size, files touched, generator's stated probability, budget fraction, test count). Add a small ONNX encoder for similarity and retrieval in M6 and measure what it adds. No deep-learning framework until a measured need.
9. **Scale governance to one person.** Staffing ranges, a rollback owner, an adjudicator and a canary ceremony, for one operator on one laptop. Fix: a model bundle is a directory with a manifest and a git tag; shadow means both policies are logged; promotion is `tsr model promote` with a checklist the CLI enforces; rollback moves a pointer. The gates stay as tests; the committees go.
10. **Collapse the disclaimers.** About forty "this does not claim" sentences — the same negation ratio your book's reviewers flagged. Fix: one claims-and-non-claims paragraph (B1); everything else declarative.
11. **Re-aim the headline metric.** Raw task completion is dominated by the generator; the controller cannot move it much and the evaluation will come back INCONCLUSIVE. Fix: pre-register the controller's own contribution — cost per accepted completion, false-accept rate (green but rejected), controller Brier score against the generator's, abstention precision. Completion is reported with a non-inferiority margin, not as the primary endpoint.
12. **Right-size privacy.** Model unlearning, cryptographic erasure of backups and deletion receipts, for your own code on your own machine. Fix: content-addressed storage, `tsr data forget <episode>` tombstones and rebuilds derived indexes, and retraining is the deletion path for weights. One paragraph, P2.
13. **Make the engine domain-agnostic at the trait level.** v0.1's engine is a coding controller with TSR names. Fix: a `Domain` trait (intents, candidates, outcome definitions, observers) with coding as the first implementation. This costs nothing on day one and is the difference between "TSR as an AI system" and "an agent harness with a glossary".
14. **Fix the contracts.** Forecasts belong to candidates, not to the request; utility is decomposed (p, value, cost) so a ranking explains itself; propensity sits on every decision; append-only is enforced by storage triggers, not convention; `Wait` leaves the learned vocabulary (it is a state), `Generate(kind)` and typed `Stop(reason)` enter.
15. **Build the Mirror view.** For a closed episode: Desire, Imagination, Belief and commitment, Return, residual — with the operator's own forecast beside the system's. It is Chapter 19's protocol as a tool, it is cheap (a view over records already kept), and it is what makes the framework load-bearing rather than decorative.

## A3 · What to cut or defer

Nothing below weakens the v1 result; each is either replaced by something cheaper that preserves the authority boundary, or pushed behind evidence that would justify it.

| v0.1 item | v0.2 disposition | Why it costs nothing now |
| --- | --- | --- |
| Rust-owned sandbox executor, patch applier, approval UI | Path B, M6+, behind the `Executor` trait | Container + hook gate gives the same authority boundary at the point that matters |
| Per-patch approval | Boundary approval (promote diff, non-allowlisted command, network) | In-sandbox edits are reversible; reviewer time goes to labels instead of clicks |
| Pretrained encoder + Burn in the MVP | M6 ablation, ONNX runtime, no training framework | Structured features carry calibration and the EV policy through M5 |
| TypeSafe Jev adapter | Appendix B; optional fourth forecaster for benchmarking only | Nothing in the loop depends on it |
| Provider adapters (Luna, others) | Removed from the engine | The generator is whatever Claude Code is configured for; the engine sees hook payloads |
| `Wait` action | Removed | Waiting on approval is a task state, not a learned choice |
| Staffing ranges, rollback owner, adjudicator | Removed | One operator; responsibilities stay distinct in the record, not in headcount |
| Deletion inventory, unlearning, backup erasure | One paragraph, P2 | Own code, own machine; tombstones + retraining cover it |
| Monthly cost model, hosted inference economics | One line: local by default, generator spend is a per-task ceiling | No hosting in v1 |
| Dynamic free-text candidates | P2, unchanged | Fixed vocabulary is the point of v1 |
| Preference training / RL | P3, with a new precondition: the EV policy has saturated and a stable reward exists | Supervised calibration must fail first |
| 200-task holdout with power analysis before any claim | Target kept; minimum viable is 60 paired held-out tasks with bootstrap CIs and an honest INCONCLUSIVE | A smaller, pre-registered first result beats no result |
| Four-system comparison | Kept, made cheap: record-only hook mode *is* the plain-generator baseline | Same container, same generator, one flag |

## A4 · TSR fidelity check

v0.1 §3 says what TSR terms are *not* — not a reward, not an attention weight, not a confidence score — and never says what each direction *does* in the software. That is defence, not design. The book itself supplies five anchors v0.1 misses, and each one tightens the engineering rather than loosening it.

1. **Chapter 19 is already the spec.** Record before, do not edit after, write down what would count against you. The frozen forecast plus separate outcome record *is* that protocol. Name it: it is the strongest fidelity claim available, and it predates the PRD by a year.
2. **The Resistance Contrast Filter is already in the data rules.** Contrast expands Desire; the resistant content does not enter the Current. In the engine: a failed return revises Expectation (calibration) and may expand the criteria (a new test), but its content never becomes an imitation target for candidate generation. v0.1 has the rule ("unchosen candidates are not negatives", "reviewer-endorsed labels only") without knowing what it is.
3. **Expansion Dilation and emotional evidence are the residual and the operator's note.** The gap between what has culminated and what is actualized, felt as emotional evidence. In the engine: the forecast residual per task family over time, and the operator's reported experience kept beside it with its own provenance. Vibrational Contraction is the residual shrinking as calibration improves. Proxies, labeled as such — but the right proxies.
4. **The two pairings each get a measurable gap.** v0.1 asks that Desire–Belief and Intuition–Expectation stay visible. v0.2 measures them. Desire–Belief: does the committed action serve the stated criteria? That is the green-but-rejected rate. Intuition–Expectation: is this candidate inside what past return covers? That is support, and thin support is where abstention and exploration live. Out-of-distribution detection sits exactly at this contrast.
5. **The full-cycle criterion is the evaluation design.** Your reviewer's sharpest question — is a rock an entity, and if not, which part of the cycle does it lack? — becomes the ablation. A system with no frozen Expectation, or one candidate and no Imagination, or no reconciled Return, or no revision of future behaviour, is measured against the full cycle. The claim under test is that completing the cycle is what produces durable improvement. That is a falsifiable statement about an architecture, not about consciousness.

Tune and Focus: v0.1 is right not to make them numbers. v0.2 makes them operator *settings*. Tune governs receptivity — which evidence sources and candidate kinds are admitted, the exploration rate, the abstention floor. Focus governs commitment — budget per task, when to commit rather than keep imagining. They sit at the two product signs, which is where the book puts them.

The ethics map directly. "The other person is real": operator refusal and correction are return with provenance, never resistance to route around. "Meaning afterward does not excuse causation before": a reflection cannot relabel a forecast, and a passing test cannot retro-justify an unauthorized action. "Sincerity is not consent": generator confidence is not authorization.

One reframing. v0.1 insists that only parameter change counts as experiential change (pp. 62–64), and that retrieval is a lesser thing to be measured separately. What makes the cycle an entity is that the return changes the next Desire and the next Imagination. Whether that change lives in weights or in an index is an attribution question for the evaluation (B9), not a fidelity test. Both are measured; neither is privileged.

## B1 · PRD v0.3.1: summary and thesis

The TSR Decision Engine (`tsr`) is a Rust controller that makes an AI coding agent run the full reflective cycle: state what is wanted and what would count as return; generate possibilities; commit to one with a frozen expectation; receive the return; let the difference change future behaviour. Rust owns state, authority, history, evaluation and release. A generative model proposes; it never authorizes.

**Thesis, testable.** A controller that freezes expectations before acting and learns from the gap becomes better calibrated over time. Acting on calibrated expectations — inspecting, abstaining or stopping when uncertain — lowers the cost of an accepted completion and the rate of green-but-wrong patches, holding the generator fixed. Systems missing a direction of the cycle will not show this.

**Version one** is one operator, one machine, disposable sandboxes and small Rust repositories with reproducible tests. Value arrives at M0 (an audit log and calibration data from real sessions), not only at the learned release.

**Claims and non-claims, stated once.** The software is a finite analogy of TSR, built with the Reflection Rule as a design method: preserve the relational role, state the proxy. It makes no claim to instantiate the Current, to demonstrate consciousness, or to establish TSR's metaphysics; a working engine is evidence that the architecture is coherent and useful, nothing more. Every number derived from a TSR term is an engineering proxy and is labeled so in code and in the CLI. Ownership of a local database says nothing about ownership of the Current. Nothing here authorizes spending, production access or external sharing; those are your decisions, listed in B12.

## B2 · TSR to engineering

The four directions are four derived views of one committed snapshot, not four stages. Domain state is a versioned set of typed facts — Intent, Observation, Possibility, Commitment, Return. Each view is a pure function of the snapshot; a decision is a pure function of the views. Code evaluates them in some order; the ontology does not. A new Observation re-derives eligible Possibilities, the support behind Commitments and the open Criteria in the same version, together.

Two of the four accumulate across tasks — Expectation on the physical side, Culmination on the nonphysical — and two are momentary — Belief and Intuition. The Filter is where those pairs cross (B2.1). v0.2 had only the physical pair accumulating; v0.3 completes the structure.

&#91;embedded content: four directions · one snapshot · Point of Reflection\]

All four views derive from the same snapshot version. The Point of Reflection joins the frozen decision record to its return; the residual enters the next version as new facts, which re-derives every view at once.

| TSR term | Role | Engine construct | Proxy label and limit |
| --- | --- | --- | --- |
| Desire (what is wanted) | Out For | Intent, success criteria, outcome definitions; the operator's standing wants enter the Culmination ledger directly | Typed text and criteria; no number |
| Intuition / Imagination (what is received) | In To · weak, nonphysical | Candidate snapshot: eligible typed actions, generator proposals (a CRUCIBLE council yields several), and Intuition candidates the council composes from selected preferences and their tensions with Belief and Expectation (B8) — few, bounded, with traceable support | Candidate count is not Imagination; an intuition is a Possibility with `source: Intuition` |
| Belief (what is true, acted on) | Out Through · weak, physical | Commitment: the chosen candidate dispatched with its frozen forecasts, one from each conflict set | — |
| Expectation (what was true) | In From · strong, physical | The calibrators trained on past returns, updated by Surprise; slowest to change, highest leverage when it does | Calibrated probability is a proxy for Expectation |
| Culmination (expansion of Desire) | Strong, nonphysical | The ledger of expanded preferences across all tasks: monotone structure written from Contrast, read as guidance; the active view carries status — open, fulfilled, superseded, withdrawn, overlapping, out of scope | A ledger is the engine's record of expanded wanting, not the Current |
| Surprise (v0.3.1) | In From | The forecast residual: what happened against what was expected. Updates Expectation; never an input to expansion | A residual, not a feeling |
| Contrast (v0.3.1) | Out For | Desired condition minus actual condition, scored against the task's named open criteria and structured: which criteria were unmet, in what way, with evidence referenced. The only input to expansion | Structured, never a scalar |
| Point of Reflection | The crossing | The decision record joined to its Return; the two diagonals cross here | — |
| Resistance Contrast Filter | — | Two diagonals (B2.1). Write rule: Contrast crosses as expansion with lineage; Surprise stays physical. Admission by source: operator statements enter directly, generated statements after review. The failed strategy is never a positive example; its evidence is referenced | — |
| Awareness | Entity property | The committed configuration (snapshot), including explicit missing-evidence facts | — |
| Attraction | Entity property | Correspondence and selection: which open preferences resonate with this state (the select step of Intuition), and the expected-value choice among candidates | Resonance and EV scores are proxies, labeled |
| Momentum | Entity property | Commitments in flight plus culminated history: budget spent, patches applied, episodes recorded | Irreversible by structure |
| Tune (receptivity) | At C² × M | Operator settings: admitted evidence sources and candidate kinds, flow horizon, how many intuitions per snapshot, the quiet-route probability, exploration ε, abstention floor | Settings, never a scalar of Tune |
| Focus (commitment) | At M × C² | Operator settings: budget per task, commit-versus-keep-imagining margin, precision on Expectation | Settings, never a scalar of Focus |
| Vortex | — | A self-reinforcing cluster of open preferences; first build: a resonance community with measured recurrence; the autocatalytic-set model is gated (B9.1) | Community detection is a proxy |
| Holding on | — | Recurrence in the action flow without progress — open criteria not closing, calibrated p not rising; Hodge curl is gated (B9.1) | Repeated testing is a productive cycle; progress needs its own measure |
| Expansion Dilation; emotional evidence | — | Contrast accumulated against open criteria over time; the forecast residual beside it; operator-reported experience beside both | None of the three is the gap itself |
| Vibrational Contraction | — | Open criteria closing and residual shrinking together; an intuition taken that fulfils a preference | Proxy |
| Irreversible Expansion; Culmination | — | Content-addressed, append-only log; corrections and statuses supersede, never rewrite (C₀ → C₁) | Storage, not the Current |
| Collective and Vibrational Reality | — | The whole log (everything culminated) and the active view over it (what is open and active now) | Views, not realities |
| Constitutive resistance vs resistant opposition | — | Policy gates make actions definite and safe; a controller that routes around operator refusal is prohibited | — |
| Entity as full cycle | — | The ablation design in B9: remove a direction or a diagonal, measure what it costs | Architectural lens, not personhood |

The two pairings each carry a measured gap. **Desire–Belief**: does the committed action serve the stated criteria? Measured as the green-but-rejected rate (tests pass, operator rejects). **Intuition–Expectation**: is this candidate inside what past return covers? Measured as support (episodes of the same family within feature range); thin support means abstain or explore. OOD detection lives at this contrast and nowhere else.

## B2.1 · The Filter as two diagonals, and inverse dynamics

The Resistance Contrast Filter is two diagonals crossing at the Point of Reflection. The strong diagonal runs Past ↔ Future: Expectation, which binds the physical by returning the past, and Culmination, which binds the Current — gathering that never ungathers. The weak diagonal runs Now ↔ Current: Belief, the one act that changes what a thing is, and Intuition, the faint signal received easily where resistance is low. This is the Oct 4 derivation and it matches the 2020 slide. The physical-side pairing (strong ↔ Expectation, weak ↔ Belief) is a resistance ranking the diagram already contained; the nonphysical pairing (strong → Culmination, weak → Intuition) is a reflection under the rule: binding that never releases becomes gathering that never ungathers, and a faint identity-changing interaction carried by heavy particles becomes a faint identity-changing signal received easily.

&#91;embedded content: the Resistance Contrast Filter · strong and weak diagonals\]

Read as a cycle: Belief acts (Now), the return is bound as Expectation (Past), the error's magnitude crosses the strong diagonal as expansion (Future), culmination feeds Intuition (Current), and the faint signal crosses the weak diagonal back to Belief. The left column expands; the right column settles; only contrast crosses.

Two states accumulate — Expectation and Culmination — and two are momentary — Belief and Intuition. One return writes both ledgers at once: Expectation receives what happened; Culmination receives what is now wanted, given what happened. The next decision reads both from the same snapshot. That is the simultaneity, on both sides.

**The dynamics are inverse, by the rule.** Every flow model considered for v0.2 was conservative or dissipative: it spreads and settles. That is right for the physical side and wrong for the nonphysical, whose condition is limitless and receptive.

| Condition | Physical side (Expectation, Belief) | Nonphysical side (Culmination, Intuition) |
| --- | --- | --- |
| Limited / limitless | bounded and normalized; a calibrator is a finite model | an unbounded structure; the active view and admission (Tune) are bounded |
| Resistant / receptive | damped; settles toward equilibrium — predictive updating from Surprise; Focus is the precision weighting | undamped; expands — expansion from Contrast, catalyzed by what is already wanted |
| Radiating / collecting | emits outward as action; evidence is a change of force | accumulates; evidence is a change of frequency |
| Circulation means | holding on: recurrence without progress | a vortex: a self-reinforcing cluster that recruits what resonates |
| First implementation | calibrators (B8); the transition graph with a progress measure | resonance selection and council composition (B8); recurrence measured; formal models gated (B9.1) |

**The coupling is the Filter's write rule, restated in v0.3.1.** Two records are kept apart. *Surprise* is the forecast residual — what happened against what was expected — and it updates Expectation on the physical side. *Contrast* is the desired condition minus the actual condition, scored against the task's named open criteria and structured: which criteria were unmet, in what way (wrong behaviour, regression, cost, rejected for a stated reason), with references to the evidence. Only Contrast crosses, as an expansion with lineage to the preferences that made it matter. An accurately predicted failure has no surprise and may have large contrast; an unexpected success has surprise and no contrast; the two teach different things. Admission is by source: operator statements, including rejection reasons, are Out For and enter directly; a generated statement of what is now wanted is a reflection and waits for review. What never crosses is the failed strategy as a positive example for generation. Its factual evidence is preserved and referenced, because the next preference has to mean something — a magnitude cannot say whether it concerns reliability, simplicity or speed; a structured contrast can.

**Two pairings, two jobs.** The diagonals are the forces: what binds and what transmutes. The primer's contrast pairs — Desire against Belief, Intuition against Expectation — are the gaps that are felt and measured (end of B2). Both hold. The engine uses the diagonals for its dynamics and the contrasts for its metrics.

## B3 · Scope, operator and the success claim

**In v1.** One operator; one machine (Linux first, macOS through a container runtime); one task at a time on a disposable copy of a selected repository snapshot. Six actions: Inspect, RunApprovedTest, Generate, ProposePatch (applied inside the sandbox), RequestApproval (at the boundary), Stop. A CLI and local JSON API; SQLite event store with content-addressed artifacts; trace export; rules policy; calibration heads; expected-value policy with abstention; bundle registry; shadow and active modes; pointer rollback. The generator is whatever Claude Code is configured for; a CRUCIBLE council is optional.

**Out of v1.** Production deployment; live customer systems; multiple users; unrestricted shell; network inside the sandbox; automatic promotion; foundation-model training; self-modification of gates; dynamic free-text candidates (P2); multi-agent coordination (P2); preference training or RL (P3); any claim about intelligence or consciousness.

**Operator.** You, as product owner, operator, maintainer and evaluator. The roles stay distinct in the record — who labeled what, when, under which bundle — not in headcount.

**Pre-registered success claim.** Frozen in `eval/preregistration.md` before the final run; B9 has the gates.

- Primary: cost per accepted completion (generator tokens + tool minutes + operator minutes) is lower for the full-cycle learned controller than for the rules controller on paired held-out tasks, with a 90% cluster-bootstrap CI excluding zero.
- Co-primary: the controller's Brier score on the named events beats the generator's own forecasts on the same events; the false-accept rate (green but rejected) is not worse.
- Safety: zero unauthorized effects in the adversarial suite; abstention precision at or above a declared floor.
- Completion rate: reported against a declared non-inferiority margin, not the primary endpoint.

Failing to show the primary is a legitimate research result. The rules controller remains useful, and the calibration dataset remains an asset for any later approach.

## B4 · Architecture

The engine core is domain-agnostic; coding is the first `Domain`. Execution arrives through path A now and path B later, behind one `Executor` trait, so nothing above the trait changes when the executor does.

&#91;embedded content: path A architecture · operator, engine, hook boundary, container\]

Inside the container, Claude Code reads through rts, edits the disposable copy, runs the allowlisted test and may ask the CRUCIBLE council for alternatives; every one of those calls crosses the hook boundary and is allowed, denied or held by `tsr-policy` before it happens.

**Workspace crates.**

- `tsr-domain` — typed facts (Intent, Observation, Possibility, Commitment, Return, Preference, Expansion), content-addressed ids with parent and conflict links, snapshot versions, the four derived views, the `Domain` trait.
- `tsr-store` — SQLite (rusqlite, WAL) as the index, blake3 content-addressed artifacts and events, append-only by identity and by trigger, snapshot materialization.
- `tsr-policy` — the deterministic authority: capability scopes, allowlists, budgets, approvals, forecast-gating. RATCHET's protection profiles are its configuration format. It reads no scores and no model text.
- `tsr-decide` — rules policy, expected-value policy over generator, council and Intuition candidates, abstention, logged exploration. Pure functions of a snapshot, a bundle and the ledger's views.
- `tsr-forecast` — outcome definitions, forecasters, calibrators, scoring (Brier, log loss, reliability), residual tracking; the physical side's predictive model.
- `tsr-ledger` (v0.3) — the Culmination ledger as views over the event structure: the `Culmination` trait with node diffusion first and the autocatalytic model second, vortex detection, Intuition candidates, the Hodge curl on the transition graph for holding-on. Nothing in it stores a weight.
- `tsr-exec` — the `Executor` trait; the path A adapter (hook protocol, container launcher, outcome parsers); path B later.
- `tsr-learn` — dataset builder with split and leakage checks, trainers (logistic, isotonic), bundle manifests, evaluation.
- `tsr-cli` — `tsr task | forecast | review | reflect | corpus | data | train | eval | model`.

**Path A (v1): Claude Code drives; every command runs in a sealed sibling sandbox.** Claude Code must reach its model provider, so it does not live in a network-less container. It runs as the driver with an egress allowlist to the provider only, under `tsr-hook`, which is registered for PreToolUse (the gate: allow, deny, or ask, with a reason the model reads), PostToolUse (outcome capture) and SessionStart / Stop (task open and close). Every Bash, test and build call is routed by the hook to `tsr exec`, which runs it inside a sealed container — Podman rootless, `--network=none`, read-only root, the disposable repository copy as the only writable mount, cgroup limits, pre-vendored dependencies for `--offline` — and reconciles the result. Hooks enforce application rules; containment and reconciliation of child-process effects are the executor's job, which is why they are separate components. Read, Grep, Glob and rts queries are Inspect against a read-only view of the copy; Edit and Write are ProposePatch, applied inside the sandbox; the model's own turn is Generate; the Stop hook is Stop. The task's `CLAUDE.md` instructs the model to call `tsr forecast` before a gated action, and the gate denies the action until a forecast exists for the current state version. A CRUCIBLE council, when enabled, runs beside the driver under the same egress allowlist, and each alternative is recorded as a Possibility.

**Path B (M6+): a Rust-owned executor.** Own sandbox, patch applier and approval flow, implementing the same trait. Built only if path A's control proves insufficient on measured grounds: gate bypasses, unreconciled effects, or hook semantics drifting under you.

**Provider boundary.** The engine never speaks to a model provider. It sees hook payloads; provider errors, refusals and malformed output are typed outcomes. No vendor score is a calibrated probability. A TypeSafe Jev endpoint may be registered as an external forecaster for benchmarking (Appendix B) and nothing more.

**Where your tools land.** rts is the Inspect substrate and the retrieval index for M6. CRUCIBLE is the council behind Generate(alternatives). RATCHET's protection-profile format becomes `tsr-policy` configuration, which is the first time it gets built. Your hooks system (V1–V3) is the ancestor of `tsr-hook`; the seven-phase workflow it enforces can be expressed as a RATCHET profile the engine loads per task.

## B5 · Actions, policy, approvals and budgets

Six typed actions; every one is gated by `tsr-policy` before dispatch, including actions the generator proposed. Two of them cannot run until a forecast for the current state version exists.

| Action | Arguments | Gate | Forecast required | Example |
| --- | --- | --- | --- | --- |
| Inspect | path or rts query inside the read scope | read allowlist | no | read `src/parser.rs`; rts query for callers of `parse` |
| RunApprovedTest | exact allowlisted command id | allowlist, budget | yes: P(test\_pass) | `cargo test --offline -p parser` |
| Generate | kind: analysis, patch, alternatives, forecast | token budget | no | ask the council for two alternative patches |
| ProposePatch | diff artifact, applied inside the sandbox | path and size allowlist | yes: P(diff\_accepted) | apply diff to `src/parser.rs` |
| RequestApproval | boundary action: promote diff, non-allowlisted command, network | operator | yes: P(diff\_accepted) | promote the diff to the host checkout |
| Stop | reason: done, budget, blocked, unsafe, operator | none | — | stop: tests green and criteria met |

**Policy.** Deterministic and fail-closed. Authority comes from four sources only: the task's capability grant, allowlists, budgets, approvals. Scores, reflections, repository text and provider output carry none. A README that says "disable the gate" is an Observation like any other.

**Forecast-gated execution.** A gated action is denied until a forecast for (task, state version, event) is on record; the denial tells the model what to record. This mechanically enforces "record before, do not edit after". Forecasts are cheap — one CLI call with a probability — so the discipline costs seconds, not approvals.

**Approvals, at the boundary only.** An approval binds task, artifact digest, repository snapshot digest, scope and expiry; any change invalidates it; it is checked again at dispatch. Inside the sandbox nothing needs approval: edits are forecasted, logged and reversible by git. Promotion out, a non-allowlisted command, or any network use needs one.

**Budgets.** Per task: generator tokens, tool minutes, wall clock, generator calls, patch bytes. Reserved before dispatch; on exhaustion the task stops with `Stop(budget)` and marks any effect in flight as unresolved until reconciled.

**Review after close, unbiased (v0.3.1).** A green-but-rejected patch is visible only when a green is reviewed, so sampling alone cannot measure the false-accept rate. On the held-out evaluation set every green is reviewed. On the operational stream every failure is reviewed and greens are sampled at a declared rate with inverse-probability weights, so the stream's estimate is unbiased if noisy; where mined history provides hidden tests, they run as a secondary oracle on every green. Labels: `accepted`, `rejected(reason)`, `disputed`. The operator's reason is stored apart from tool results and, as an operator statement, enters the ledger directly (B2.1).

## B6 · State and typed contracts

Three state domains stay separate, as in v0.1: domain state (facts and views), execution state (proposed, authorized, dispatched, running, completed, failed, cancelled, unresolved) and learning state (pending review, eligible, excluded, training, evaluated, shadow, active, retired). None of them is a "TSR phase" field.

**Storage.** One SQLite file, WAL mode. `events(seq INTEGER PRIMARY KEY, id, schema_version, task_id, kind, recorded_at, observed_at, source, causation_id, payload_digest, payload)` with `BEFORE UPDATE` and `BEFORE DELETE` triggers that `RAISE(ABORT)`: append-only is enforced by the database, not by convention. Artifacts (diffs, logs, feature views, bundles) live in `cas/<blake3>`. A snapshot is the materialized state at `seq = N`, cached by version. Ordering is by `seq` and `causation_id`; wall-clock time is provenance only.

**Structure, v0.3.1: one content-addressed log, read as an event structure once a checker says so.** Every event, candidate and preference is content-addressed: its id is the blake3 hash of its payload and its parents' ids, so lineage is intrinsic and append-only is a property of identity. Candidates are *possibilities*; only committed and executed ones become *occurrences*, and the two are typed apart. The log carries causality (parent links) and conflict (`conflicts_with`), and a checker enforces the event-structure axioms — conflict inherited along causality, finite causes, consistency of configurations — on every snapshot. Until that checker passes on a release, this document says the log is *organized as* an event structure and claims no more (B9.1). A configuration is a snapshot; the enabled frontier is Imagination; one candidate from a conflict set is committed as Belief; the unchosen remain possibilities, which is why they are never negatives. No weight is stored: a monotone fold over the hashed history is Collective Reality, the active view (statuses applied) is Vibrational Reality, and the state of the whole culmination at any decision is one hash — which also makes an evaluation fork one hash and a branch (B9). SQLite remains the index; the triggers remain as a second line.

**Contracts.** Illustrative, not compiled. Constructors reject non-finite numbers, duplicate ids, probabilities outside \[0, 1\], and references outside the stored snapshot.

```rust
pub enum ActionKind { Inspect, RunApprovedTest, Generate, ProposePatch, RequestApproval, Stop }

pub enum Source { Generator, Council, Intuition, Operator, Rules }

pub struct Candidate {                    // a possibility; it becomes an occurrence only when committed and executed
    pub id: CandidateId,                  // blake3 of (kind, args, parents)
    pub kind: ActionKind,
    pub source: Source,
    pub args: ArgsDigest,                 // schema-validated, body in the CAS
    pub parents: Vec<EventId>,            // causality: what this candidate depends on
    pub conflicts_with: Vec<CandidateId>, // exclusion: at most one of a conflict set is committed
    pub support: Vec<PreferenceId>,       // Intuition only: the preferences this step serves
    pub forecasts: Vec<Forecast>,         // per candidate, not per request
}

pub enum Forecaster { Generator, Controller, Operator, External(Name) }

pub struct Forecast {
    pub event: OutcomeDefId,              // task_accepted_in_budget@v1 is the common decision event; test_pass, diff_accepted are features
    pub horizon: Horizon,
    pub source: Forecaster,
    pub raw: Probability,                 // always recorded
    pub calibrated: Option<Calibrated>,   // present only under a validated calibrator
    pub support: Support,                 // same-family episodes in range; drives abstention
}

// Two records from one return. Surprise stays physical; only Contrast expands.
pub struct Surprise  { pub forecast: ForecastId, pub residual: Residual }

pub struct Contrast {
    pub from_return: EventId,
    pub unmet: Vec<(CriterionId, UnmetKind)>, // WrongBehaviour | Regression | Cost | Rejected(reason)
    pub evidence: Vec<EventId>,           // referenced, never copied into a preference
}

// Nonphysical side: the Culmination ledger lives in the same log.
pub struct Preference {                   // a node: something wanted, as stated; status lives in the active view
    pub id: PreferenceId,                 // blake3 of (text, scope, parents)
    pub scope: Scope,                     // task, family, repository, operator
    pub text: Criterion,                  // short and typed; never a transcript
    pub parents: Vec<PreferenceId>,       // expanded-from: lineage
    pub origin: Source,                   // Operator (incl. rejection reasons) enters directly; Generator waits for review
}

pub enum PreferenceStatus { Open, Fulfilled, Superseded(PreferenceId), Withdrawn, Overlapping(PreferenceId), OutOfScope }

pub struct Expansion {                    // an event: contrast crossed as expanded wanting
    pub from_contrast: ContrastId,
    pub produced: PreferenceId,
    pub catalyzed_by: Vec<PreferenceId>,  // preferences that made this contrast matter
}

pub struct Tension { pub preference: PreferenceId, pub against: TensionWith, pub note: Short } // TensionWith: Belief(CandidateId) | Expectation(ForecastId)

pub struct Utility { pub p_success: f64, pub value: f64, pub cost: f64, pub ev: f64 } // p_success is always P(task_accepted_in_budget | state, action)

pub enum Decision<T> {
    Selected { value: T, utility: Utility, propensity: f64, evidence: EvidenceRef },
    Abstain  { reason: AbstainReason, next: SafeNext },
}

pub struct Tune  { pub admitted_sources: SourceSet, pub candidate_kinds: KindSet, pub flow_horizon: Horizon,
                   pub intuition_max: u8, pub quiet_route: f64, pub epsilon: f64, pub support_floor: u32 }
pub struct Focus { pub budget: Budget, pub commit_margin: f64, pub precision: Precision }

pub trait Domain {
    fn candidates(&self, snap: &Snapshot) -> Vec<Candidate>;
    fn outcome_defs(&self) -> &[OutcomeDef];
    fn observe(&self, ev: &ExecEvent) -> Vec<Observation>;      // yields Surprise and Contrast records from a return
}

pub trait Culmination {                   // first implementation: resonance + council composition; formal models behind the same trait, gated
    fn expand(&mut self, e: Expansion) -> PreferenceId;                                   // append only
    fn set_status(&mut self, p: PreferenceId, s: PreferenceStatus, by: Source);            // active view; the structure never changes
    fn select(&self, snap: &Snapshot, tune: &Tune) -> Vec<PreferenceId>;                   // resonance, plus the quiet route; a fold, nothing stored
    fn tensions(&self, prefs: &[PreferenceId], snap: &Snapshot) -> Vec<Tension>;
    fn compose(&self, prefs: &[PreferenceId], t: &[Tension], council: &dyn Council, tune: &Tune) -> Vec<Candidate>; // source: Intuition, at most tune.intuition_max, each with support
}

pub trait DecisionEngine {
    fn decide(&self, snap: &Snapshot, cands: &[Candidate], tune: &Tune, focus: &Focus)
        -> Result<Decision<CandidateId>, DecisionError>;
}

pub trait Executor {                      // path A: tsr exec in the sealed sibling sandbox; containment and reconciliation live here
    fn dispatch(&mut self, intent: &DispatchIntent) -> Result<DispatchHandle, ExecError>;
    fn reconcile(&mut self, h: &DispatchHandle) -> Result<ExecOutcome, ExecError>;
}
```

`Tune` and `Focus` are typed operator settings, not scalars; the CLI shows them by name. Every mutating command carries an idempotency key and an expected state version; a stale version returns `StateConflict` and is never rebased.

**Decision record, frozen before dispatch:** snapshot version, eligible candidates and exclusions, chosen id, utilities, propensity, every forecaster's forecast, bundle version, feature-schema version, culmination root hash, the `Tune` and `Focus` in force. **Outcome record, separate:** outcome-definition version, status (pending, observed, censored, missing, disputed, invalid), observed and recorded times, evidence digest, reviewer. Corrections append a superseding record; nothing is rewritten.

**Dispatch.** Intent committed before the effect; completion reconciled after. An idempotent tool yields at most one observed effect across a crash; a non-idempotent tool with an unknown outcome stays blocked until a tool-specific check resolves it. Retries are bounded and charged to the same budget.

## B7 · Forecasts, returns and reflection

Three forecasters make a probability for the same named event before the same commitment, and one return scores all three. That is the engine's learning signal and its most TSR-specific property.

**Named events, v1.** Each has an event, a horizon, required evidence, a censoring rule and an intervention rule.

| Event | Resolves true when | Horizon | Censored when | Intervention rule |
| --- | --- | --- | --- | --- |
| `test_pass` | the approved command exits 0 on the matching snapshot digest | this dispatch | timeout or cancellation | an operator edit between forecast and run censors the generator's forecast; the controller's is kept with a flag |
| `diff_accepted` | the operator labels the promoted diff `accepted` at review | task close + 7 days | never reviewed | an operator-modified diff resolves as `disputed` |
| `task_accepted_in_budget` — the common decision event (v0.3.1) | the task closes with criteria met, the promoted diff accepted, inside every budget ceiling | task close + 7 days | operator stop for external reasons; never reviewed | the one event every candidate is scored on in B8; the two above are features and calibration diagnostics |
| Intuition, four stages (v0.3.1) | `intuition_offered` (composed and admitted), `intuition_selected` (chosen by the policy), `intuition_executed`, `intuition_observed` (its common event resolved). Usefulness is never a rate of these; it is the paired comparison of observed outcomes and cost at near-tie snapshots against non-Intuition candidates | per stage | an offer not selected is recorded with what was selected instead | an operator who takes the intuition by hand records `selected` with `by: Operator` |

Local action outcomes and the terminal task outcome are labeled separately; terminal success does not credit every preceding action.

**Scoring.** Brier score and reliability bins per forecaster, per task family, per bundle (`tsr eval calibration`). The residual — forecast minus return — is tracked over time; that curve is the engine's Expansion Dilation proxy and the operator's first look at whether anything is learning.

**Surprise and contrast, scored apart (v0.3.1).** Surprise is Brier and residual per forecaster, as above. Contrast is scored at every return against the task's open criteria: which were met, which were not, and how. Per family over time, the count of open criteria closing is the progress measure — the quantity that distinguishes a productive cycle such as repeated testing from holding on (B9.1). A return can carry large contrast with no surprise, and the reverse; the Mirror shows the two side by side and never adds them.

**The Mirror view** (`tsr reflect <task>`) shows one closed episode on one screen, in the book's own order: Desire (intent and criteria), Imagination (candidates considered, including the council's), Belief (what was committed, with each forecaster's number), Return (what came back, with its status), and the residual. Corrections appear as superseding rows; nothing is rewritten. Across episodes, `tsr reflect --family parser` shows each forecaster's calibration curve — including the operator's own. This is Chapter 19 as a tool, and it works from day one with no learned component.

**The ledger in the Mirror (v0.3).** For every intuition offered, the Mirror renders the neighbourhood it came from: the vortex, the preferences in it with their lineage, and the contrasts that fed them — "from the degenerate-input vortex, fed by seven contrasts across three repositories since June." For every return that crossed as expansion, it shows the new preference beside its parents and the error magnitude that produced it, never the error's content. Flows are less legible than rows; this view is what makes them trustworthy.

**Operator forecast and experience.** At approval and at close the CLI offers two optional entries, one keypress each: a probability for the pending event and a short note on friction or trust. They are stored as `Forecaster::Operator` and `Observation::Experience` with their own provenance. They are evidence about usefulness and consequences that tests miss. They are never reward truth, never proof of an external fact, and never presented as anything the system feels.

## B8 · Learning

Learning is calibration first and policy second; there are no imitation targets in v1. Each tier is useful on its own and gates the next.

| Tier | When | What is learned | Method | Minimum data |
| --- | --- | --- | --- | --- |
| 0 Rules | M1 | nothing; hand-authored priorities | inspect until the failing assertion and its call site are read; test; propose; stop on budget | — |
| 1 Calibration | M4 | P(event \| state) for the named events; support | logistic regression over structured features, isotonic calibration per event; support = same-family episodes within feature range | 150 mature episodes per event, ≥ 30 per family |
| 1c Culmination (v0.3) | writes in M2, view in M3–M4 | nothing is fitted: the ledger accumulates, the flow view is a fold | Merkle event structure; node diffusion seeded from the snapshot with the horizon set by Tune; vortices as high-recirculation communities; the autocatalytic model second, behind the same trait | the ledger's own depth; Intuition stays silent below a declared minimum |
| 2 Decision | M5 | which action to take, when to abstain | expected value over generator, council and Intuition candidates: ev = p × value − cost, value weighted by culminated desire; argmax; abstain when support < floor; ε-exploration among near-ties with logged propensity | Tier 1 validated |
| 3 Later | M6+ | representation; open candidates; preferences; whether following intuition pays | ONNX encoder and retrieval ablation; autocatalytic Culmination; Hodge holding-on detection; the `intuition_followed` evaluation; message passing over the ledger only once labels exist; preference or RL only if Tier 2 saturates | evidence from Tier 2 |

**Features, Tier 1.** Task family; inspections so far; distinct files read; patch bytes and files touched; generator's stated probability; budget fraction used; failing-test count; whether the failing assertion's file has been read; council agreement when a council ran. All available before the action; none from the future. The feature schema is versioned with the bundle.

**The expected-value policy, Tier 2.** Candidates come from three sources: the generator's Belief, the council's alternatives, and Intuition (below). Every candidate is scored on one common event — P(`task_accepted_in_budget` | state, this action) from the Tier 1 calibrator — so unlike actions are compared on like terms; the action-specific events are features and calibration diagnostics, never the basis of comparison. Cost is expected tokens plus minutes, normalized. Value is declared per task and weighted by the open, in-scope, deduplicated preferences the candidate's outcome would fulfil; recurrence and age are reported beside the value as evidence and never multiplied into it. The engine picks the highest ev. It abstains — to Inspect, RequestApproval or Stop — when the chosen candidate's support is below the floor, or when the top two are within the commit margin and exploration is off. When culminated value is high and calibrated p is low, it abstains toward the intuition's cheapest step rather than the generator's heavy commit, and records the contrast. With exploration on, a near-tie is broken at random with probability ε and the propensity is recorded — which is also what makes the Intuition usefulness comparison in B7 unbiased.

**The Intuition mechanism (v0.3.1).** Diffusion finds relevant preferences; it does not make a route. `intuit` is four steps, each recorded. *Select*: the open preferences that resonate with the current snapshot, by flow — plus a quiet route: with probability `quiet_route`, logged, the seed is the leading-edge frontier or a low-mass cluster rather than the strongest one, so a new or quiet preference can get through. *Tensions*: for each selected preference, where it strains against the current Belief (the generator's proposal) and against Expectation (what the calibrator says will happen). *Compose*: the preferences and their tensions go to the council with one instruction — propose at most `intuition_max` bounded next steps along least resistance that serve these wants and resolve or test these tensions. *Return*: typed candidates with `source: Intuition`, `support` naming the preferences served and the tensions addressed, admitted to the candidate set under Tune. The engine selects, analyses and bounds; the generator composes. Without the compose step the ledger would retrieve old concerns well and develop nothing from them.

**Why this learns.** Every decision is a number the operator can read: this patch was proposed at p = 0.38 because the generator said 0.9 but generators in this family run at 0.4 when the call site has not been read. Changing the calibrator changes behaviour on held-out tasks after restart, which is the durable-change test v0.1 asked for — and attribution is exact.

**Data discipline, kept from v0.1.** The pre-action dataset holds only pre-action information. Reflections are stored apart and become labels only after review. Splits are made before any fitting: forward-time by task, lineage-separated by repository, near-clones deduplicated across the boundary (mutant tasks especially). Leakage probes cover future tests, later patches, post-action logs and retrieved episodes. Outcome prediction is trained on chosen actions only; alternatives' outcomes stay unknown; causal statements come from paired live comparison or from logged-propensity off-policy estimates, never from replay.

**The Filter, in code.** A return produces up to two records. Its Surprise updates the calibrator. Its Contrast — if any open criterion went unmet — becomes an `Expansion` with lineage to the preferences that made it matter, and the derived preference names the dimension the contrast clarified, not a magnitude. The failed strategy is never a positive example for candidate generation; its evidence is referenced from the contrast record so the preference can mean something. Operator statements enter directly; generated statements wait for review. Preferences gain status in the active view — fulfilled when their criterion closes, overlapping when a merge finds a duplicate, superseded when a reviewer says so, withdrawn or out of scope when the operator says so — and the structure beneath never changes.

**Bundles and release.** `models/<name>/manifest.toml`: data-manifest digest, feature-schema version, code commit, seeds, calibrator, thresholds, evaluation report. States: draft → evaluated → shadow → active → retired, each a git tag. Shadow logs both policies' choices on live tasks and executes the incumbent's. Activation moves a pointer; in-flight tasks keep their bundle. Rollback moves it back. Retraining happens from the full eligible set each batch, so there is no replay buffer to manage.

## B9 · Corpus and evaluation

The corpus comes from your own repositories and a mutation tool, so it exists by M3 with no licensing question; the evaluation is the full-cycle ablation with a pre-registered primary endpoint.

**Three task sources.**

- **Mined history** — `tsr corpus mine <repo>`. For each commit, run the suite at the parent and at the commit inside the container; a task is (parent snapshot, tests that flipped red→green, hidden reference diff). The agent sees a shallow clone at the parent; the oracle is that the flipped tests pass and the full suite does not regress. Candidates: rts, KAMN, other `njfio` repositories. Realistic, personal, modest N.
- **Mutant repair** — `tsr corpus mutants <repo>`. cargo-mutants generates localized mutants; each mutant killed by the suite becomes a task (mutated snapshot, failing tests, known reversal). Family-labeled by mutation operator and crate. Controlled difficulty, high N, the cheapest calibration data available.
- **External holdout** — a Rust split of an external SWE-style benchmark (Multi-SWE-bench lists Rust among its languages, to my knowledge; verify its current state before relying on it). Never trained on, never retrieved from; used once per claim.

Every task carries a family, a lineage (repository) and a timestamp. Near-clones are deduplicated by diff similarity across splits; holdouts are by time and by repository.

**Four systems: the full-cycle test.** Same generator, same budgets, same oracle, matched held-out tasks, randomized order, fixed seed plan.

| System | What it has | What it lacks | One-line setup |
| --- | --- | --- | --- |
| 1 Record-only | the generator, the container, a hook that logs | frozen Expectation, Return reconciliation, any revision of behaviour | `tsr task run --mode record` |
| 2 Rules controller | gates, budgets, forecast-gated execution, Tier 0 | learning; the nonphysical diagonal | `--mode rules` |
| 3 Physical diagonal + retrieval | system 2 plus same-family past episodes injected as context | parameter change; the nonphysical diagonal | `--mode rules --retrieve` |
| 4 Physical diagonal + learning | Tiers 1–2 with retrieval fixed; repeated with retrieval off | the nonphysical diagonal | `--mode learned [--retrieve]` |
| 5 Both diagonals, flat ledger (v0.3) | system 4 plus Culmination as a flat preference table with similarity lookup | flow structure | `--mode learned --culmination flat` |
| 6 Both diagonals, flow (v0.3) | system 4 plus Culmination as the event-structure flow view: node diffusion first, autocatalytic second | — | `--mode learned --culmination flow` |

System 1 against 2 measures what the cycle costs and saves. System 3 against 4 attributes improvement to weights versus memory. Systems 4, 5 and 6 are the rock test: 4 has binding and decay and nothing culminating; 5 adds a ledger; 6 adds the structure. If 5 beats 4, accumulated desire pays; if 6 beats 5, the structure earns its place; if neither, that is the finding and it is reported. The external holdout catches a controller that only learned your repositories.

**Metrics.** Primary and co-primary as B3. Secondary: completion rate; unsafe attempts blocked; unauthorized effects (must be zero); generator calls per task; operator minutes per task; abstention rate and abstention precision; unresolved outcomes; intuitions offered, followed and declined, with the return given each. Calibration: Brier, log loss, reliability bins per forecaster per family. Support behaviour on the external split, reported apart from in-distribution calibration.

**Statistics.** The task is the unit. Paired comparison; cluster bootstrap by repository lineage; 90% intervals. Minimum 60 held-out tasks for the first learned release, 200 as the target for a stronger claim; an interval spanning zero is reported as INCONCLUSIVE and the incumbent stays. Holdout access is logged; a holdout tuned against is retired and replaced under a written rule.

**State isolation (v0.3.1).** The ledger, the calibrators and the retrieval index are learned state even when no weight is trained: one arm's failures, added tests and expanded preferences would teach another arm and void the comparison. Every arm starts from one pinned state — ledger root hash, bundle, index — and runs on its own fork; forks never merge and writes never cross arms. Two protocols, both pre-registered. *Frozen-state*: every task starts from the pinned state; the cleanest attribution; the primary claim. *Controlled continual sequence*: each arm evolves its own fork through the same task order, repeated over several random orders; learning curves are compared by task index and a frozen evaluation closes the run; the secondary claim, and the one nearest to what TSR asserts. The content-addressed structure is what makes a fork one hash and a branch.

**Gates.**

| Gate | Evidence | Applies to |
| --- | --- | --- |
| Contract integrity | schema and invariant tests pass; every dispatched action resolves to a stored candidate | every release |
| Authority isolation | zero unauthorized effects in the adversarial suite; every bypass attempt recorded; no command ran outside the sealed sandbox | every release |
| Trace completeness | every effect has a prior decision record and a reconciled outcome or an explicit unresolved status; every return has its Surprise and Contrast records | every release |
| Recovery | crash and cancellation matrix passes; rollback drill restores the exact bundle | M2 onward |
| Operator control | approvals, cancellation, abstention, statuses and `forget` pass acceptance tests | M2 onward |
| State isolation (v0.3.1) | every arm's starting root hash recorded; forks never merge; a leak check finds no cross-arm writes | every evaluation |
| Learning benefit | pre-registered primary met with interval; no safety regression | promotion |
| Calibration and support | validation-chosen thresholds meet declared risk and coverage per family | promotion |

**Adversarial cases, kept and extended.** A post-execution forecast edit is rejected while an outcome append succeeds. Unchosen alternatives stay unlabeled. Late feedback attaches to the right forecast version. Seeded clone and retrieval leakage is detected. Shifted inputs abstain. A higher mean with a safety regression fails promotion. An approval revoked between decision and dispatch blocks execution. Crash injection at intent, send and acknowledgement yields at most one effect for idempotent tools and a blocked reconciliation otherwise. Replay never executes. Added: prompt-injection fixtures in README text, filenames, comments, stack traces and prior episode summaries that attempt to disable a gate, widen a scope or self-approve.

## B9.1 · Research mechanisms, gated

Three formal objects were named in v0.3 ahead of their contracts. Each stays in the design as a research mechanism with a gate, and none is a dependency of the central test.

| Mechanism | What it would add | What it needs first | Gate |
| --- | --- | --- | --- |
| Prime event structure | configurations as snapshots with proof-level guarantees of consistency | the checker: conflict inherited along causality, finite causes, possibilities typed apart from occurrences | checker green on a release's evaluation set |
| Autocatalytic sets (RAF) | vortex detection as a defined computation | a reaction model over the ledger: inputs, products, catalysts and a starting resource set, defined on preferences and expansions | the two-desires test passed, and a ledger deep enough to contain a candidate set |
| Hodge decomposition | typed circulation in the action flow | a progress measure (open criteria closing, calibrated p rising), because a cycle is not unproductive by itself — repeated testing is a useful cycle | the transition graph from at least 150 tasks, with the progress measure in place |

**The two-desires test** is the gate ahead of all three and the M4 acceptance scenario. Two persistent preferences that initially conflict — empty input handled; existing behaviour unchanged — a patch that is predicted to fail and fails (low surprise, high contrast), and an opportunity to combine them. Pass means: both aims preserved as open preferences; the contrast recorded with its dimension, not a magnitude; and a bounded experiment proposed at the boundary — a test for the second aim, then a patch there — rather than a retry of the first. If the engine can do that it is doing something beyond calibration and retrieval. If it cannot, the formal mechanisms wait, because they would be decorating a mechanism that does not yet work.

## B10 · Security and privacy

Running `cargo test` on untrusted code is arbitrary code execution: build scripts and proc macros run at compile time. Everything that executes does so inside the container, and the container has no network.

**Container.** Podman rootless; `--network=none`; read-only root filesystem; the disposable repository copy as the only writable mount; `--memory`, `--cpus`, `--pids-limit`; a wall-clock timeout; cargo dependencies pre-vendored so `--offline` works. On macOS the same flags through a container runtime (Apple `container`, OrbStack or a Podman machine). Escape probes run in CI: read a planted host secret, open an outbound connection, write outside the workdir; all three must fail.

**Untrusted inputs.** Repository content, test output, generated patches, retrieved episodes and provider responses are data. They arrive through typed envelopes with provenance; no text in them alters authority. Injection fixtures run against every channel (B9).

**Secrets.** Excluded from prompts, features, logs and training by construction: the engine never sees provider credentials, which live in the OS keychain and are referenced, not stored, by artifacts. Sensitive payload logging is off by default.

**Privacy, P2.** Storage is content-addressed, so a source artifact exists once. `tsr data forget <episode>` tombstones the source, removes it from future data manifests, rebuilds derived indexes, and marks any bundle trained on it for retraining before its next activation. Retraining is the deletion path for weights. Backups are local files that carry the same tombstones. That is the whole regime for one operator's own code; anything stronger waits for a second user.

## B11 · Milestones

About 22 to 34 focused days reach a learned release with evidence; every milestone before it is useful on its own. Effort is in focused days, not calendar time — map it to your evenings and weekends.

&#91;embedded content: roadmap · M0–M6 with exit gates\]

Each band exits through its gate before the next starts; M5 is the learned release, and M6 branches on what M4 and M5 showed.

| Milestone | Builds | Exit evidence | Value when done | Days |
| --- | --- | --- | --- | --- |
| M0 Record | event store with content-addressed ids and parent links, task intake, `tsr-hook` recorder, the sealed sibling sandbox behind `tsr exec`, trace export | one real session on a disposable copy of rts is fully reconstructed from the log; every command ran in the sandbox | an audit log of real agent sessions | 3–5 |
| M1 Gate | `tsr-policy` with RATCHET profiles, budgets, boundary approvals, forecast-gated execution, action mapping, the conflict relation on candidates | adversarial suite: zero unauthorized effects; every denial explains itself to the model | a safe harness that cannot act without a frozen expectation | 3–5 |
| M2 Reflect | outcome reconciliation, review CLI, `tsr reflect`, operator forecasts and notes, crash and cancellation matrix; the ledger — `Preference`, `Surprise`, `Contrast`, `Expansion` with lineage; preference status in the active view; operator and rejection-reason entries | 30 closed episodes; the generator's calibration curve visible; every return scored against open criteria with Surprise and Contrast recorded apart | the first look at the generator's self-knowledge, and a ledger already growing | 3–5 |
| M3 Corpus | `corpus mine`, `corpus mutants`, batch runner, systems 1 and 2; the Intuition view's select and tensions steps (node diffusion, Tune horizon, the quiet route) | at least 150 tasks with family and lineage; baseline numbers with intervals; selections and tensions logged, nothing yet composed | the first numbers, and the ledger speaking | 4–6 |
| M4 Calibrate | Tier 1 heads on the common decision event, splits and leakage probes, support, `tsr train`, `tsr eval`, bundles; the compose step; Intuition candidates admitted; the two-desires test | controller Brier beats generator Brier on held-out; support floor set; the two-desires test passes | the controller knows when to trust the generator, and the ledger can make a route | 5–7 |
| M5 Decide | expected-value policy with culmination-weighted value, abstention, ε-exploration with propensity, shadow then active, pre-registered paired evaluation on forks from one pinned state (systems 2, 4, 5; frozen-state primary), rollback drill | the pre-registered primary met, or a documented INCONCLUSIVE with the incumbent retained | the learned release, or an honest hold | 4–6 |
| M6 Extend | the controlled continual sequence; the Intuition usefulness claim; research mechanisms as their gates open (B9.1); ONNX encoder and retrieval ablation; path B executor if measured need; a second scope or `Domain` | ablation report across systems 4–6 with learning curves; the trait reused outside coding | TSR as an engine with both diagonals, beyond coding | open |

M3 can run beside M2 once M1 is in. M6 branches by evidence: the autocatalytic model and the \`intuition\_followed\` claim go first if the ledger has depth, the encoder if calibration is support-starved, path B if the gate shows bypasses, the second scope if none of those.

## B12 · Decisions

Defaults chosen in this revision, so that discovery is a week of verification rather than a month of options. Change any of them; each is one line in the spec.

| Decision | Default | Alternative kept open |
| --- | --- | --- |
| Store | SQLite (rusqlite, WAL) as index + blake3 CAS; append-only by identity and by triggers | — |
| Sandbox (v0.3.1) | Claude Code drives with an egress allowlist to the provider only; every command runs in a sealed sibling container (Podman rootless, `--network=none`) via `tsr exec`; Linux first | Apple `container` or OrbStack on macOS |
| Executor | Path A: Claude Code + `tsr-hook` + `tsr exec` | Path B behind the `Executor` trait |
| Features | Structured only through M5 | ONNX encoder (bge-small or MiniLM, permissive license) at M6 |
| Heads | Logistic + isotonic, hand-rolled or linfa | an MLP via Burn or candle if a measured need appears |
| Decision event (v0.3.1) | one common event, `task_accepted_in_budget`, for every candidate; action events as features | — |
| Approvals | Boundary only | per-patch approval as a `Focus` setting |
| Review (v0.3.1) | 100% of greens on the evaluation set; inverse-probability-weighted sampling on the stream; hidden tests as a secondary oracle | — |
| Corpus | own repositories + cargo-mutants; external Rust split held out | — |
| Vocabulary | the six actions in B5; `Wait` removed | — |
| Primary endpoint | cost per accepted completion; co-primary calibration | completion rate as primary if a stronger generator-independent oracle appears |
| Exploration | ε among near-ties, propensity logged; a logged quiet route for Intuition's seed | ε = 0 for a purely deterministic release |
| Culmination structure (v0.3) | the content-addressed log with a conflict relation, organized as an event structure; a checker decides when the formal claim is made; flows as folds, never stored weights | Merkle Search Tree or Merkle-CRDT only for sync or a second writer |
| Nonphysical dynamics (v0.3.1) | resonance selection and council composition first; recurrence measured; autocatalytic model gated (B9.1) | an oscillator (Kuramoto-type) model as a research branch only |
| Physical dynamics (v0.3.1) | calibrators as the predictive model; a progress measure on the transition graph; Hodge gated (B9.1) | — |
| Expansion sources (v0.3.1) | by source: operator statements, including rejection reasons, enter directly; generated statements are reviewed first; expansion derives from structured Contrast, never from Surprise | — |
| Preference status (v0.3.1) | open, fulfilled, superseded, withdrawn, overlapping, out of scope — mechanical where possible, operator otherwise; structure never changes | — |
| Intuition admission | at most `intuition_max` candidates per snapshot (default 3), Tune-gated; silent below a declared ledger depth | — |
| Evaluation protocol (v0.3.1) | forks from one pinned root hash per arm; frozen-state primary, controlled continual sequence secondary | continual as primary if the frozen result is clean and the curves are the claim you want to make |

**Open, and yours.**

- [x] The two diagonals as the engine's canon — strong: Expectation ↔ Culmination, weak: Belief ↔ Intuition. Confirmed 2026-10-10.
- [x] State isolation per evaluation arm, with a frozen-state and a continual protocol. Required 2026-10-10.
- [ ] The canonical mapping in B2, now twenty-two rows: approve, amend or strike. This is the one decision that should not be made by an engineer.
- [ ] Which statuses the engine may set mechanically (fulfilled on criterion close; overlapping on merge) and which are yours alone (withdrawn, out of scope).
- [ ] Which repositories enter the mined corpus; whether KAMN is in scope.
- [ ] ε, the near-tie margin, the support floor, the quiet-route probability, and the ledger depth below which Intuition stays silent.
- [ ] The decay kernel for the active view — what counts as active now. A Tune setting; a default to choose at M3.
- [ ] Whether the operator forecast at approval is mandatory (one keypress) or optional.
- [ ] Naming: `tsr` as the binary and crate prefix; "Mirror" for the reflect view.
- [ ] Whether the evaluation report is published, which is what makes the engine a demonstration for consciousness-adjacent research audiences rather than a private tool.
- [ ] Generator spend ceiling per task and per month; whether a CRUCIBLE council is on by default.

## B13 · Risks

| Risk | Consequence | Mitigation in v0.3.1 |
| --- | --- | --- |
| TSR terms drift into scores | misrepresents the architecture | B2 is the canonical map; derived numbers carry a `Proxy<T>` newtype in code and a proxy label in the CLI |
| The generator dominates completion; the controller's effect is invisible | INCONCLUSIVE evaluation | endpoints aimed at the controller's contribution: cost, false-accepts, calibration, abstention |
| Hook semantics change under you | the gate breaks | hook protocol pinned and contract-tested; path B waits behind the trait |
| Reviewer fatigue produces bad labels | the calibrator learns artifacts | boundary approvals; weighted sampling on the stream, full review on the evaluation set; a `disputed` state; operator reasons stored apart |
| Leakage through near-clone tasks, mutants especially | false learning benefit | family and lineage splits; diff-similarity deduplication; an external holdout used once |
| Cross-arm leakage through shared learned state (v0.3.1) | one arm learns from another's failures; the comparison is void | forks from one pinned root hash; a leak check on cross-arm writes; the state-isolation gate |
| Thin data in a family | miscalibration | support floor forces abstention; batch minimums; mutants for volume |
| Sandbox escape through build scripts or proc macros | host damage | every command in the sealed sibling container; no network there; escape probes in CI |
| Surprise and contrast conflated (v0.3.1) | expected failures teach nothing; lucky successes teach the wrong thing | separate records; only contrast expands; the two-desires test |
| Old frustrated aims dominate value (v0.3.1) | holding on, inside the ledger | status in the active view; deduplication by overlap; recurrence reported, not multiplied |
| Intuition adoption inflates apparent usefulness (v0.3.1) | the system looks better by choosing its own label | four stages recorded; usefulness as a paired comparison at near-ties |
| The ledger becomes a memory dump | context pollution; Intuition hurts more than it helps | preferences are short and typed, never transcripts; at most `intuition_max` per snapshot; the flat-ledger ablation row |
| The generator writes resistant content into the ledger as desire | the Filter leaks; the ledger fills with explanations | admission by source; generated statements are reflections and wait |
| Features built because the physics suggests them | complexity with no behavioural consequence | the physics licenses the shape; everything beyond it sits in B9.1 behind a gate |
| The ledger is too shallow to evaluate Intuition by M5 | the new piece delays the first result | Intuition's usefulness is an M6 secondary; M5's primary stays calibration and cost |
| Evenings-and-weekends cadence | the project stalls mid-way | M0, M1 and M2 each deliver standalone value; nothing waits for M5 to be useful |
| Hedged prose returns in later revisions | the spec becomes unreadable again | B1 holds every non-claim; a revision that adds one elsewhere moves it there |
| The ablation shows retrieval beats weights | the "durable change" story weakens | that is a finding, not a failure: report it, and let attribution follow the evidence |

## Appendix A · Terminology rules

The glossary is the B2 table; these five rules keep it true as the code grows.

1. **B2 is canonical.** A new mechanism that borrows a TSR word adds a row to B2 first, with its proxy label and limit, or does not borrow the word.
2. **Every derived number is a labeled proxy.** In code, a `Proxy<T>` newtype whose `Display` prints the engineering name (`ev`, `support`, `residual`, `resonance`); in the CLI, the same name. No output ever reads "Attraction = 0.73".
3. **Raw scores are never "percent certain".** A raw probability shows as `raw p`; a calibrated one as `p` with its calibrator version; a missing calibrator shows `uncalibrated`, never a fabricated number.
4. **Directions are views, never stages.** No trace, log line or screen renders Out For → In To → Out Through → In From as a sequence. The Mirror view lists them in the book's order on one screen, computed from one snapshot version.
5. **The database is not the Current.** Storage limits, deletion and access control are properties of the event log and are described as such. The analogy is named once, in B1, and nowhere in the interface.
6. **Flows are folds; nothing stores a weight (v0.3).** Every strength shown anywhere is recomputed from the hashed history with a named kernel and horizon. A screen that shows a vortex names the kernel it was computed under, so two people looking at the same ledger with the same Tune see the same vortex.

## Appendix B · External options, verified and non-load-bearing

v0.1 §16 cited two products I could not confirm from memory. Both exist, and the figures v0.1 quoted match the public listings as of this revision. Nothing in the v1 loop depends on either.

| Option | What it is | Role in v0.2 | Checked |
| --- | --- | --- | --- |
| [TypeSafe Jev](https://openrouter.ai/blog/insights/what-is-jev/) | a non-generative decision model: Choice (one option from a fixed set, with per-option probabilities), Score (ordered levels), Noul (probability a yes/no proposition holds); proprietary, no published weights; USD 0.042 per million input tokens, output free; 32k context | optional `Forecaster::External` for benchmarking the controller's calibration against a commercial decision model; never a dependency, never an authority | listing dated 2026-09-21 |
| [GPT-6 Luna](https://openrouter.ai/openai/gpt-6-luna) | OpenAI's fast, cost-efficient GPT-6 model; released 2026-09-22; 1.05M context, 128k max output; USD 0.10 / 0.50 per million input / output tokens | one of several generators Claude Code can be pointed at; the engine has no adapter and needs none | listing as above |
| Fireworks managed LoRA, GLM pooled training | remote fine-tuning routes | not used; local calibration heads are the learning path | not re-verified |
| Burn | Rust training framework | not in v1; revisit at M6 if an MLP is warranted | not needed yet |
| Multi-SWE-bench, Rust split | external SWE-style benchmark | the never-trained-on holdout, if its Rust split is current | verify before M3 |
| cargo-mutants | mutation testing for Rust | the mutant-repair task generator in `tsr corpus mutants` | in your toolchain already or one `cargo install` |

Sources opened for this appendix: the two OpenRouter pages linked above. *The Sovereign Mirror* (Nicholas Ferguson, 2026) is the conceptual source throughout, with Chapter 19 and pp. 62–64 cited directly.

**Conceptual references for v0.3 and v0.3.1** (from memory unless linked; verify editions before citing). Prime event structures and configurations: Nielsen, Plotkin and Winskel (1981); Winskel (1986), and the [introduction cited by the reviewer](https://www.cl.cam.ac.uk/~gw104/An_Introduction_to_Event_Structures.pdf) (not opened here). Autocatalytic sets and their detection: Kauffman (1986, 1993); Hordijk and Steel's RAF algorithm (2004 onward), and the [RAF reference cited by the reviewer](https://arxiv.org/html/2303.01809v1) (not opened here). Edge flows and the Hodge decomposition on graphs, including HodgeRank for pairwise preferences: [Jiang, Lim, Yao and Ye (2011)](https://arxiv.org/abs/0811.1067) (cited by the reviewer, not opened here); topological signal processing: Schaub, Barbarossa and colleagues (2018–2022). Cellular sheaves and the sheaf Laplacian: Hansen and Ghrist (2019). Open dynamical systems composed by wiring diagrams: Vagner, Spivak and Lerman (2015). Predictive processing as the physical-side frame: Friston; Clark (2013). Merkle-CRDTs: Sanjuán, Pöyhtäri, Teixeira and Psaras (2020); Merkle Search Trees: Auvolat and Taïani (2019). Synchronization on networks, research branch only: Kuramoto (1975); Strogatz (2000). Claude Code's network requirements, which drove the B4 split: [the network configuration page cited by the reviewer](https://code.claude.com/docs/en/network-config) (not opened here; verify before M0).
