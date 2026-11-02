# Engine-owned browser game: proposed longer-horizon evaluation

Status: local source assessment and plan, not a launch packet or working web adapter.
The user redirected work before the graph batch dispatched. Graph calls=0,
operations=0, grant/upgrade attachments=0; no uncertain external effect. Its separate
prepared state is held in `target/guide-opportunity-live-1/HELD.json`. The graph
allowance is not transferable. Campaign remains 259 calls, 72 operations, $0.222740;
actual attached caps remain 272/74/$5. All earlier results remain frozen.

## A bounded initial brief for the engine

> Build an original, playable single-player browser arcade game inspired by
> maze-chase, falling-block play, or a combination you judge coherent. Choose and
> explain the rules, visual language and difficulty progression; do not merely
> reproduce a supplied design. Make it easy to understand and control while giving
> repeated play meaningful challenge. Keep it responsive and revisable as I play
> and request changes. Produce a local playable artifact, then use actual test and
> player feedback to decide what to retain or change. You may ask a focused question
> or stop when evidence or permission is insufficient.

Minimum obligations: loads locally without external assets/network; visible control
instructions; working start, pause/resume and restart; player input changes actual
play; an observable score/progress and end/retry loop; no uncaught exceptions during
protected smoke play. Keyboard support first; touch/responsive behavior assessed
separately rather than silently promising every device. Original simple graphics;
no downloaded sprites, brand assets or external accounts. Fixed safety/compatibility
checks are separate from aesthetic and gameplay judgment. We do not provide a
finished game, winning mechanics, implementation patches, target relationship or
mandatory dissatisfaction. The engine must originate and execute design choices.

## Verified architecture limits and minimum generic work

1. **File/action profile.** Native ProjectSpec already supports up to eight editable
   files and edit sets up to four files, snapshots, exact source handles, protected
   files and sealed execution. But `project.rs:313` only admits `.rs`; execution at
   `project.rs:692` is Rust/Cargo-specific. Add a versioned static-web profile to
   these same interfaces, not a new game controller. Allow bounded HTML/CSS/JS
   files in one disposable app directory, with fixed validation/test commands.
   No model-supplied shell commands, dependency installs or package scripts.
   An initial empty document/style/script shell is scaffolding, not a game solution.
   Additional module paths need bounded generic path admission (or pre-admitted
   empty slots), not scenario-specific rules. Keep old Rust profiles unchanged.
2. **Transport/project context.** The current objective adapter explicitly restricts
   projection to one editable file and a source whitelist. Reuse its strict output,
   transport-owned metadata, raw diagnostics, correction budget and independent
   reviews while extending the public project projection to bounded multi-file
   snapshots/inspection windows. Do not just remove the whitelist. The 2 KiB edit
   and 4 KiB total-patch limits are unsuitable for a full game. Prospective profile
   candidate: at most eight files/32 KiB total source, 8 KiB per edit, 16 KiB per
   edit set and 96 KiB cumulative changed bytes. Validate these locally before asking
   launch approval. Prefer existing bounded inspection windows and keep 32 KiB
   model requests; complete relevant evidence must fit or stop, never silently
   truncate. If that cannot fit, return the exact finite payload change needed.
3. **Actual browser return.** There is no reviewed browser execution/receipt adapter
   in this native path. Add a protected browser harness over the same sealed runner:
   load only the scoped app; bounded key/pointer/wait actions; capture console errors,
   screenshots, observed UI transitions and test results bound to source/test/action
   hashes. A fixed browser script is an independent probe, not engine reasoning.
   Engine-selected exploratory interactions must be bounded typed actions, not
   arbitrary eval. Browser isolation must block outside navigation/network and
   preserve browser sandboxing. Do not use host shell/browser tools as an unrecorded
   substitute for engine execution. Evidence must enter existing observation,
   forecast, review, relationship and next-choice paths.
4. **Runtime availability is unverified.** The existing approved pinned image is a
   Rust runtime; no reviewed Node/Chromium/browser capability is established. Inspect
   existing runtime inventory read-only first. If a suitable already installed image
   is absent, a specific pinned runtime acquisition needs approval; no download,
   installation, sandbox disabling or credential change is authorized now.
5. **Real user changes.** Use generic persistent project/desire state, not the frozen
   three-condition service schedule. `acceptance.rs:142` supports operator requirement
   revisions but explicitly blocks them on configured finite objectives; that graph
   objective cannot honestly represent spontaneous future user requests. Existing
   public desire/lineage and attributed guidance are reusable. Wire a waiting
   checkpoint and append-only user-change intake at settled action boundaries,
   distinguishing changed wanting from new acceptance obligations and permissions.
   Retain prior contributing meanings, observations and decisions; do not reinterpret
   culmination as withdrawal. Rebind source/requirements/reviews before acting and
   never retroactively change acceptance of a completed version. Unknown effects
   must be reconciled before intake/resume. Cross-identity desire succession remains
   outside this slice.

These are generic frontend execution/intake gaps, not missing game-specific rules.
No game implementation or runtime capability is claimed by this plan.

## One coherent workflow and deliverables

**Local prerequisite work (no provider/target run):** implement the web profile,
bounded browser receipt path and user-checkpoint projection with authored fixtures.
Test forbidden paths/network/commands, multi-file tamper, source-bound review,
protected checks, unknown action, accounting and restart. Verify actual runtime
availability without pulling/installing. Freeze complete initial brief, public
export projection, tool limits and independent test protocol. Independent review
must inspect the runnable path before live approval. A browser capable runtime is
an explicit release blocker, not something to handwave until after dispatch.

**Checkpoint 1 — playable version:** engine generates options/design and files,
receives independent review, executes build/browser tests and chooses repairs or
retention. Deliver its local artifact with controls and test/evidence report. Stop
and wait for the user to play; a black-screen scaffold is not a playable milestone.
If the bounded session cannot produce a playable result, preserve/report failure.

**Checkpoint 2 — actual user request:** intake the user's real words with provenance,
not prewritten fake feedback. The engine reconciles this with prior wants and actual
play returns, proposes its own design response, implements and rechecks. Deliver
before/after artifacts and explain observed tradeoffs without claiming enjoyment
from automated checks. A robust unchanged choice or clarification can be valid.

**Checkpoint 3 — another real change and regression:** retain the same persistent
project/history, use the next genuine request, and test both requested change and
prior minimum behavior. Demonstrate fresh-process resume and safe waiting/stop.
Checkpoints are evaluator structure, not emergent TSR sequencing.

Independent assessment separates technical validity, response to attributed feedback,
and receipt-grounded relationship development affecting later pursuit. Record all
attempts/corrections/assistance. Screenshots and assertions cannot establish fun;
player reactions are attributed observations, not unquestionable reward truth.
No required novelty, failure, relationship revision or numeric joint reward. No
TSR causal-advantage claim without a separately approved information-equal baseline.

## Prospective bounded budget, subject to local verification and explicit approval

Same CheaperInference gpt-6-luna. Propose a single campaign segment across up to three
interactive sessions: **60 TOTAL model calls** (all generation/reviews plus at most
six shared format corrections), **18 isolated action groups**, **six active hours**
(maximum two hours per session). Each session conservatively fits 20 calls at
240 seconds plus six groups at at most 180 seconds within two hours; give each
session a fresh source-bound admission under the one overall ledger envelope.
Waiting for human feedback consumes no model/action effects and does not silently
extend a session deadline. Define an overall authorization expiry in the eventual
approval; further user changes outside that window need renewed admission.

An action group comprises one admitted edit/check/browser bundle or one bounded
browser investigation, with exact sub-action/time limits frozen before launch.
Count failures and browser-only investigations, not just successful builds. Model
requests keep 8192 completion tokens; no automatic provider-failure retry initially,
no unknown replay and no uncounted review. Semantic rejection is not a format error.
No guarantee all trajectories fit; preserve incomplete results at any cap.

At the existing conservative $0.05/call allowance, reserve **$3.00** within the
existing $5 total (baseline $0.222740). Prospective campaign caps would be **319 calls
and 90 operations** from actual use259/72, not a fresh budget. This is an estimate,
not authorization or a fresh price quote. Verify request-size/rate reservation after
local integration; if it fails, report precise revised bounds before launch.

Export only the public game brief, model-created source, current public requirements,
explicitly approved user feedback, bounded screenshots/console/test facts and reviewed
advisory memory. Arbitrary personal content is not covered. No private repository,
protected test source, credentials or full native state. Local playable delivery only;
no hosted deployment/public upload. Final evidence-supported acceptance may be bundled
for separate review copies; the user judges subjective satisfaction independently.

## Evidence and review

Source inspected: `project.rs` file scope/validation/command creation; `project/cycle.rs`
ordinary choices/reconsideration; `project/acceptance.rs` operator requirements and
pending-work gate; `project/acceptance/guidance.rs` provenance/carry; `project/desire.rs`
original meanings/lineage; `project/objective.rs` frozen finite schedule;
`scripts/engine-objective-adapter.py` current single-file export limitation.
Independent conceptual critique by `/root/parity_review` emphasized genuine engine
ownership, real user returns, separate safety/fulfillment judgments, no manufactured
changes and no claim that scheduled checkpoints themselves demonstrate culmination.
No code was changed for this pivot and no new functional tests were run. Existing
local graph preflight completed with zero effects; its result is held, not launched.
