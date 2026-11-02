# Relational culmination: interpretation before implementation

2026-10-22. User correction: “why don't desires culminate with each other becoming
more than they were alone?” This review supersedes the priority of further provider
comparisons. Request34 was already terminal; its receipt remains, the remaining
slot expired, and the runtime stopped. No new calls or execution occurred.

## Diagnosis

We implemented preservation, decomposition and balancing of existing wants more
fully than the process through which their relationship changes what becomes wanted.
That is a real representational gap, not merely a weak-model explanation.

The repository's PRD B2.1 says a return adds both what happened to Expectation and
“what is now wanted, given what happened” to Culmination, with expansion catalyzed
by existing wants. B8 specifies joint composition of bounded next steps. These are
related but not identical: finding an action that serves A and B is not yet forming
an aspiration C that becomes conceivable because A and B are considered together.
B9.1's two-desires test preserves conflicting aims and proposes a boundary experiment;
passing it alone would not establish that additional relational capacity.

Source basis is the checked-in PRD text, including its account of the TSR diagrams
and book. The external original drawings/book were not independently inspected here;
this is an engineering interpretation of the available material and the user's
correction, not a formalization of all TSR claims.

| Actual mechanism | What it supports | What it lacks |
| --- | --- | --- |
| `tsr-domain/src/lib.rs::Preference` / `Expansion` | Multiple parents, catalysts, source, content-addressed lineage | A typed operational meaning for an emergent relationship |
| `tsr-ledger/src/write.rs::Ledger::expand` | Contrast-backed append-only expansion | Generation/interpretation of the relational aspiration itself |
| `tsr-ledger/src/selection.rs::select` | Parent/catalyst neighbourhood affects attention | Diffusion creates no new wanted condition |
| `tsr-domain/src/lib.rs::Council::compose` | Joint preference/tension input to action composition | Returns candidates, not a durable evolving relational aspiration |
| `engine/deliberation.rs::Refinement`, `refine_directions` | One parent; unmet note subsets admitted as InspectNotes/IncludeNotes | Multiple-parent aspiration, a new relation, or qualitatively different predicate |
| `engine/deliberation.rs::original` and shared `decision.rs::roots` | Conserves obligation influence across decomposition | Treats refinements as one original's descendants; cannot represent relational novelty |
| `decision.rs::assess` | Progress, uncertainty, standing loss, costs across originals | Adding scores does not generate meaning or a new desired possibility |

The generic ledger can hold multi-parent text, and other adaptive paths can retain
proposed new criteria. Therefore this is not a claim that all storage is immutable
checklist code. The missing connected path is **relation → proposed aspiration →
changed possibility generation → observed feedback → further relational evolution**.
The narrow notebook adapter further limits actions to inspect or concatenate original
note text, so even a capable generator cannot actualize richer forms through it.

## Minimal model to explore

Keep three distinctions, using existing ledger and Council contracts:

1. **Standing intent and authority.** Original wants and explicit acceptance criteria
   retain identity and history. Permissions and budgets remain independently admitted.
   Evidence never says an original criterion was met merely because a new aspiration
   is attractive. A desire may evolve without permissions expanding.
2. **Relational aspiration proposal.** A bounded proposal references two or more
   distinct active preferences, their current versions, a particular relation/tension,
   supporting contrast/return IDs, and a new desired condition in typed text. It states
   what becomes possible through their relationship and what is still unknown. The
   structure records that this is a generated hypothesis, not established meaning,
   permission, truth or independent decision weight. Canonical parent sets prevent
   permutation duplicates; relations are not encoded as single-parent subset refinement.
3. **Operational interpretation and feedback.** The Council may use that hypothesis
   to generate a bounded discriminating possibility within existing authority, with
   support anchored in the original wants. A Domain supplies what can actually be
   observed. A separate reviewed interpretation can admit the aspiration as a preference
   and define any new criterion; until then semantic fulfillment remains unknown.
   No structural check can certify arbitrary prose as emergent or desired by the user.

Record proposal, review/admission, observation links, revision and withdrawal as
append-only facts. A proposal can be contradicted, remain unresolved, or lead to a
revised relation. An admitted relational preference can later participate with other
preferences, including its parents, in further composition. Parent evidence/status
changes invalidate stale applicability without deleting the proposal or old return.
Failed strategies remain evidence, not endorsed templates. No scalar “emergence”
bonus or proliferation of duplicate wants should increase selection weight.

This would reuse multi-parent Preference/Expansion provenance and the existing
source-review boundary. It needs a small explicit proposal/view contract and a
Council output extension, rather than a second ledger or a replacement decision engine.
Keep the shared obligation kernel for checking explicit effects/standing losses;
relational imagination must not be reduced to that kernel's numeric output.

## Example of the distinction, not a supplied runtime answer

“Preserve complete detail” together with “avoid burdening attention” could suggest
an aspiration such as making depth available while keeping orientation easy. That
can motivate a layered representation, which is more than maximizing two existing
scores independently. It is an illustrative candidate interpretation, not an inferred
user endorsement or a solution fed to the model.

In the preserved notebook task, literal full original-text coverage plus a four-word
account remains impossible. A layered representation would require a different
adapter capability and possibly a reviewed change in criterion interpretation.
The old criteria cannot be silently relabeled satisfied. Yet the engine should be
able to retain and investigate the aspiration instead of rejecting it solely because
it is not a subset of one existing note-coverage predicate.

## PRD boundary and smallest next milestone

The existing Filter admits expansion only from structured Contrast. Preserve that
rule: unreviewed relational possibilities may be proposed during joint consideration;
committed return-derived expansion still cites real contrast. Do not invent contrast
from surprise or treat success as an unmet criterion. Whether successful combination
alone should create new admitted desire is a separate semantic extension, not needed
for a first contrast-grounded milestone.

The minimal coherent next milestone is a **relational proposal loop**, initially
without new action authority: persist a two-parent aspiration proposal from a real
contrast, show it changes the Council's input and generated possibility structure,
link an independently observed return, then revise or qualify the relation on the
next pass. Reuse an admitted adapter with observations capable of distinguishing
the proposed relation. A deterministic fixture can verify mechanics; it cannot
establish genuine model emergence. No provider runs are implied by this design.

Acceptance controls:

- A+B yields a representable relational proposal, not simply a subset, concatenated
  labels, an AND expression, or summed scores. A reviewer assesses the semantic claim;
  structure alone cannot prove it.
- A-only, B-only and A+B contrasts preserve the same observation/authority. Verify
  which relation/possibility changes and keep unchanged-state controls.
- New desire proposals cannot mark parents fulfilled, change acceptance predicates,
  acquire authority, or inflate weight through duplicate parent permutations.
- Contradictory/unknown returns remain explicit; revisions survive restart and can
  participate in a later relation rather than vanish after one action.
- Withdrawn/out-of-scope parents and stale provenance prevent current operational use.
- Actual discovery remains separate from scripted fixture behavior and finite tests.

One consequential later choice is independent preference value: should an admitted
relational aspiration acquire its own decision weight, or guide possibilities serving
its parents? Do not choose a numerical answer silently. The first milestone can
preserve it as an explicit unresolved dimension and demonstrate relational generation
without assigning a bonus. Review/endorsement of a genuinely new desired condition
is distinct from authorization to act on it.

## Updated completion assessment

Preservation/authority/evidence machinery is useful and stays. Multi-desire tradeoff
and decomposition are implemented. **Relational culmination as a connected evolving
mechanism is missing.** General semantic acceptance and calibration data remain limits,
but neither should displace this newly clarified core priority. Another successful
patch, arithmetic probe or transport fix would not close this gap.

Independent code/PRD review agreed with the diagnosis and identified the independent-
weight choice above. No speculative redesign or runtime behavior was implemented.
