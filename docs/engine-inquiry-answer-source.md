# Answer-source suitability: local routing correction

The inquiry/selection abstention recorded in `engine-inquiry-live-result.md` stands.
It is not retrospectively adopted as a corpus investigation or relabeled successful.

The existing `Inquiry` now optionally declares a typed `answer_source`, reviewed
with its complete immutable proposal through the existing native review path:

| Declared source | Current route |
|---|---|
| `corpus_assertions`, exact configured corpus digest and bounded rationale | `corpus_candidate` after independent admission and freshness checks |
| `user_preference` | `waiting: awaiting_user_preference` |
| `user_authority` | `waiting: awaiting_user_authority` |
| `external_evidence` | `waiting: unsupported_evidence_source` |
| `mixed` | `waiting: mixed_sources_unresolved` |
| `unknown` or omitted legacy field | `waiting: answer_source_unknown` |

`question-request` exposes `information_routes` for each persistent inquiry.
Unreviewed, rejected or stale questions wait rather than reach the reader. Waiting
is a reconstruction of persistent metadata/status, not a fabricated answer or
fulfilled desire. It does not erase the question, contact the user, choose a topic,
create a new inquiry or activate an exploration policy.

The existing native reader checks source suitability again **inside its write
lock**, before any file read. It requires a current admitted inquiry and exact
matching corpus. Existing root, allowlist, content, lineage and replay checks remain.
The declared source is not execution authority: explicit local operation admission
is still separate. A corpus question concerns what the records assert; it is not
verification that those statements are true in the world.

This is deliberately not keyword classification. The model or operator proposes
the source; independent review must assess whether that attribution is justified.
A falsely labeled question could still fool a fallible reviewer. The deterministic
controls prove routing of the reviewed metadata, not semantic infallibility.
Unsupported factual questions are represented as requiring external evidence, not
quietly converted into questions about fictional assertions. Mixed questions wait
rather than resolving just the convenient part.

Existing native candidate memory, review, current-parent checks and retrieval path
are reused. No separate selector or scenario policy was introduced. Old serialized
inquiries remain readable; missing source is conservatively unresolved. Historical
raw outputs and expired grant manifests are unchanged. The old paid runner/schema
has not been adapted or reauthorized by this local patch; its pinned binary is now
stale and fails closed. Authored fake-provider tests explicitly supply source
metadata to test the native path; this is not evidence of a new live run.

Local controls cover user preference, user authority, unsupported world-factual
questions, mixed and unknown sources, omitted legacy source, wrong corpus,
unreviewed/rejected inquiry, valid corpus read, parent development, withdrawal and
reactivation, no-mutation rejection, and reopening persistent state.

Remaining design decision: may the engine choose a small reversible exploration
among equally fitting directions without asking the user to prioritize? That
policy remains pending and is **not** enabled here. Even if later approved, it
must not be misrepresented as a user preference or permission for other effects.

No provider calls, campaign inspections, exports or live adoptions in this patch.
Campaign remains **196 calls / 57 groups / $0.162874**. No fresh run is authorized.

Independent source review found no blocking defect. Reviewers must inspect the
complete inquiry in `question-request`, including `answer_source`, rather than
only the generic candidate text in `pending_reviews.response`. Existing frozen
provider schemas do not yet request the new source field; future export changes
require prospective preparation, not reuse of an expired grant.

Verification: five inquiry Rust tests passed, ten local Python integration/export
checks passed, and the native CLI built offline. The broader qualitative Rust
filter passed all 22 tests.
No full-workspace pass is claimed.
