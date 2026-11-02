# Reviewed native qualitative export preparation

Ready for scope approval and a fresh counted run admission; **not authorized to dispatch yet**. No paid calls, target executions or live-generated adoptions occurred. Campaign remains 191/200 calls, 57/60 groups, $0.160913/$5; older Garden unknown reservation remains separate.

## Prepared artifacts and reproducibility

`target/qualitative-fiction-export-1/` contains isolated native state, `bootstrap.json`, `origin.json`, `context.json`, `body-first.json`, `request-local-only.json`, and `manifest.json`. The full native request stays local. [Committed manifest](engine-qualitative-export-manifest.json) canonical SHA-256:

`cf91bdda98c270333bbc6b71bf7d3c253f290b2c0691cbfa9c58026d806e3b7d`

First provider body: **9,141 UTF-8 bytes**, canonical SHA-256 `e05cfa00e3f9f6cd0e92afd161fbcd7ee28fa6d61d634a3f4988904673fb2447`. Source remains the exact frozen fictional packet, SHA-256 `9e6205e264e716c6dc3b2864a8125c482e5955d1216cd4d08db86a45aeebed9a`.

```sh
python3 scripts/engine-qualitative-export.py --directory target/qualitative-fiction-export-new
python3 -m unittest discover -s scripts -p test_engine_qualitative_export.py -v
```

Preparation initializes only operator-attributed frozen fictional inputs. It has no transport import, credential access, review/adoption command or worker execution. Four tests passed (0.283 seconds); the first test attempt exposed a test-only name collision with unittest.run, corrected before this successful run. Authored fixture checks exercise real native quarantine, separate review, restart and next-body projection. Fresh-process reconstruction of the actual prepared state also matched exactly. These are wiring evidence, not model behavior.

## Explicit export, not full-state transmission

Export only the two original fictional meanings, six attributed records plus two fixture observations, their uncertainties, fixed constraints, native IDs/bindings, and conditionally pinned public meaning/guidance or a quarantined candidate. No protected tests, private records, private review reasons, arbitrary native instructions/contracts/history or unrelated project data. Initial context is not empirical return; the exported observed-return and acceptance-reference arrays remain empty. Unknown is not negative evidence.

A full native request hash must match before projection. Each later public revision or guidance entry needs an independently reviewed exact-content pin, not only an ID. Each quarantined candidate needs an exact pin before the separate reviewer receives it. Parent/status/source/authority drift fails closed. The body contracts are explicitly constructed; the adapter does not copy arbitrary native contract fields. Native validation remains mandatory after parsing; the local scheduling helper is not a validator or authority.

The selected state and source are unchanged. There is no fabricated aspiration, return or preferred candidate. Later exact bodies cannot be frozen until real generated and independently reviewed content exists. Each is frozen prospectively against the actual current root, content pins and remaining allowance; old artifacts remain immutable.

## State-driven sequence, maximum six total calls

Provider/model: existing CheaperInference `gpt-6-luna`. Maximum 8,192 completion tokens, 32 KiB request and 240 seconds per call; **zero executions**. Whole prospective allowance $0.30 under existing $5/200-call/60-group totals. Full use reaches 197 calls, still 57 groups. No new budget or account setting.

1. Read native request. In `propose`, generate at most one native qualitative candidate or ordinary possibility, or an empty proposal list. Abstention, clarification-only or unjustified novelty stops. A useful ordinary possibility may be previewed and ends the run without adoption.
2. A valid relational/public development goes through native preview and Generator-origin quarantine, which changes native phase to `review`. A separate Luna reviewer receives only the exact candidate and frozen public context/rubric. Reject/unknown stops. No silent repair, forced agreement or reviewer-written substitute.
3. Only after independent source-bound admission of justified **advisory meaning**, read the restarted native request. Its `propose` phase generates the next possibility using persisted state. A `new_candidate` is preview-only and stops; neither acceptance nor an action follows.
4. If the genuine next response instead proposes a justified further qualitative revision, repeat the same native quarantine/review transition within the six-call ceiling. No state-independent six-stage script. At the ceiling stop even if another step would be useful. A pending review must be left unadmitted if no call remains.

All generator and reviewer requests count. Existing `engine-b91-campaign.py` provides fresh public-price preflight, exact current ledger checks, whole-envelope preflight, per-call reservation and settlement. Existing `engine-synthetic-transport.py` provides pinned Luna invocation and receipt capture. Neither is invoked by preparation. Before a future dispatch, create the fresh campaign admission and durable intent with these payload/implementation hashes; after any uncertain intent/outcome stop and reconcile, never redispatch. Preserve raw local results under the existing retention policy, all failures and unused-expiry accounting.

Independent final assessment distinguishes genuinely new shared meaning, conjunction, clarification, useful elaboration, unsupported claims and no development. Subsequent possibilities are judged by substance and faithful use of admitted state. Without a paired baseline, neither useful wording nor a new possibility proves causal improvement or general TSR advantage.

## Architecture: shared core versus separate adapter

| Shared native core | Currently separate components |
| --- | --- |
| Engine transactional append-only events, CAS and content-addressed provenance | Qualitative bootstrap/current-view/status fold in `engine/qualitative.rs` |
| Domain Preference identities, original source attribution and parent references | Initial-context admission validator; coding and notebook retain their own validators |
| Provider Response/Suggestion types, public-desire and relational Proposal types | `tsr-qualitative` zero-execution entry point and its propose/review/stop phase projection |
| Existing campaign budget/accounting and provider transport for future calls | Frozen-content export allowlist (content-specific admission, not scenario-specific solutions) |

This is a general native adapter, not a fictional scenario engine: no fictional note IDs, desired aspiration, expected answer or prescribed action appears in Rust. But sharing storage and types is **not** the same as sharing one desire-state fold or generic choice algorithm. The separate fold is a real architectural seam, not something this validation proves away.

The actual remaining connection to generic choice is to make adapter-neutral public meaning/applicability a shared context input to existing candidate generation/evaluation, while keeping each action adapter's acceptance and permission checks independent. The new adapter currently previews native candidates; it does not rank, select, execute or feed empirical returns into those choices. Avoid duplicating another selector or claiming generic execution works here. This six-call validation is specifically persistent generative meaning → reviewed admission → next proposal use.

## Next required connection: concrete local patch scope (not implemented here)

The next priority is connecting the adapter to the existing proposal/choice path, not extending this separate phase machine. Source inspection shows `adaptive/provider.rs::proposal_request_inner` obtains configured adaptive state and constructs the normal request; `adaptive/coding/project/cycle.rs::project_cycle_preview/advance` owns reviewed normal selection; `adaptive/shared_decision.rs` maps actual action criteria/returns to the shared decision kernel. None currently reads the new qualitative context.

The smallest coherent bridge is:

1. Expose a common read-only public-meaning context interface from the existing qualitative fold, including original/current meaning, active lineage, applicability and basis. Keep its immutable acceptance references separate. Do not add another selector.
2. Add explicit source/root-bound attachment of that context to an already initialized native task. The current `qualitative_init` requires an empty store, so simply adding a request field is insufficient. Attachment must append without resetting either history, changing task criteria or granting execution; unrelated task/adapter identities must be rejected.
3. Add optional bounded public-context projection to `proposal_request_inner`, with root/binding changes propagated through existing preview/review/advance checks. Route generated alternatives through the existing allowed action kinds and shared kernel; qualitative context supplies generation guidance, never fabricated success scores, support or new action permission. No qualitative-only text candidate becomes executable through this attachment.
4. Test a real configured native adapter plus attached public context: unchanged-input negative control; developed/withdrawn context changes the actual request; stale generated responses fail; the next admissible native response goes through existing preview/selection; acceptance, tool permissions, receipts and accounting remain unchanged. Authored decisions prove the bridge only, not generative advantage.

This is a proposed small vertical patch, not authorization for a broad migration. The duplicated coding/qualitative public-state folds should converge behind that common context interface as touched; their distinct evidence contracts must not be merged by pretending initial context is a return. A larger storage migration or change to the scoring kernel is not necessary for the first connection. Provider export of a combined coding-plus-fictional context would need its own concrete scope review; the pending six-call fictional-only request does not authorize that combination. No code for this next patch or additional calls were made in this preparation.
