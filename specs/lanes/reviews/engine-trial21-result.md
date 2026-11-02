# Independent Trial21 rejection audit

Reviewer: `/root/b91_full_review`. Read-only evidence/source audit; no provider call, target execution, replacement review, adoption or retry. Trial21 remains terminal rejected at experiment readmission. Parent reports nine calls, three execution groups, $0.009145 and no corrections/unknowns; cumulative campaign113/33/$0.085842.

## Actual rejection

Review `c0c607f45350275329fc375a4ee23e7d29b686ef88e8c1a8afdce5854e2d9a09` is structurally valid and rejects epistemic_grounding. It says the experiment proposal reports the protected existing-behaviour case failed while the supplied current catalog reports both protected cases passed. Other criteria are met. Provider request `917431e4-ea27-434c-92c4-ba7eb753d9ba` completed with known usage/cost; this is not an enum/schema failure and is ineligible for structural correction.

## Demonstrated temporal projection defect

The rejection compares two different epochs. Independently confirmed the readmission candidate is exactly the original generated experiment from request `0364ebf40bfa519d5798d7b7513af90626dbf8b8ea7b186d8dbb64d41ef08ae2`, with original snapshot `f19b0fc4963f718456e8fe2c1e6d6641e5c129efceec71ef03385305d103fb7d`. Its original mapping says the protected preservation case reported failed, a true statement at generation time. The seed receipt and checks-before complete logs confirm that failure.

`engine-project-runtime-review.py::build` selects `subject['generated_experiment']` for experiment_readmission, then pairs it with current source/returns. It does not project `subject['previous_admission']`, an explicit historical proposal epoch, or its original return catalog. The actual request has only current passing protected outcomes and no explicit distinction between historical mapping language and claims newly asserted now. The candidate's original state/snapshot are present but unlabeled relative to the readmission binding. The reviewer consequently treats historically correct text as a current false claim.

This is concrete missing temporal/operation context in the host projection, rather than evidence that the patch or unchanged harness is semantically defective. The reviewer did correctly stop on the apparent inconsistency in its supplied view. Neither silently accepting that report nor retrying until it agrees is justified.

## Repair and coverage evidence

Current `src/lib.rs` adds only an exactly-empty branch returning `Ok(Vec::new())` and otherwise executes the original nonempty split/map/i64 parse/error/collect expression. Static source equivalence supports nonempty preservation beyond finite tests. Cargo.toml, Cargo.lock and protected tests/contract.rs remain byte-identical to the frozen source.

Patch commitment `892aa8222305063530ccad98f7533229142f4419502bd6da2da3a942188e014f` has complete edit/compile/test exits0/0/0 and protected2/2pass. Earlier generated supplemental checks tested exact-empty success, signed values, internal/leading/trailing empty fields, malformed input, overflow and whitespace. Checks-before commitment `54632cdbb7f8ccace013263567e8fb6372c49f55ea177265266f01e0a55fff44` failed at the internal-empty-field assertion after signed values completed; later assertions were not established. `checks-after` is absent. No unchanged-experiment rerun, post-verification reflection or final independent acceptance has occurred.

Relational content retains both desires but remains predominantly conjunction/contract discrimination; no new claim of non-additive emergence or causal guidance benefit follows from this audit. Separately, the recorded after-experiment reflection says supplemental checks were proposed/not executed despite recording their case outcomes: that sentence is temporally imprecise and must not be promoted as current execution truth.

## Small justified next step

Offline repair should project typed original-proposal and previous-admission provenance, original evidence epoch, unchanged harness/case identity, and separately current source/receipt evidence. Explain that this operation asks whether the exact unchanged experiment may be rerun against the new source; historical mapping text is not a newly generated current outcome claim. Preserve original text and evidence, rather than correcting it by hand. Still reject genuinely wrong original claims, changed harness/criteria, unsupported current claims or stale source; no blanket immunity for old prose. Test the old-fail/new-pass case and adverse controls deterministically before any separately admitted live trial. Trial21's valid rejection remains immutable, and this audit grants no acceptance or execution authority.


## Prospective temporal fix review (does not alter Trial21)

Reviewer: `/root/b91_full_review`. Offline source/test/evidence review only; no provider calls, executions, retry, acceptance or historical mutation.

No material source blocker found. The new projection binds the exact previously generated experiment, its stored proposal record, previous admission and original runtime review request. Request/body/schema/subject/state/source/outcome hashes are checked before historical evidence extraction. Current source remains checked separately through the ordinary projector. Changed harness, prior admission, source or original request fails closed.

The model receives original source/case-catalog/typed receipts under an explicitly historical context, plus separate current source and outcomes. Original mapping text is retained, not rewritten. The purpose distinguishes rerunning the exact unchanged experiment from newly asserting its old prose about current results. A historically false claim remains assessable against the original facts. No previous review verdict, explanation or operator review is exported. Existing finite-case/unknown-assertion limits remain explicit, and no authority is added.

This path requires the same run's original runtime experiment review. A FileReviewer-only or differently admitted linked-copy origin is not silently accepted when those provenance records are unavailable; it fails closed. This is a deliberate current compatibility boundary, not an automatic migration.

Reviewed six pure controls: old-fail/current-pass separation; unchanged candidate/no prior conclusion; modified historical text/lineage refusal; changed harness/prior admission; stale current source; tampered original request; plus the separately initialized false-claim case retaining contrary original evidence and a negative verdict. The fixture tests do not establish natural-language truth detection or live reviewer agreement.

Read actual `target/b91-trial21-temporal-readiness.json`: reconstructed request21107 bytes with11661 bytes headroom, old preservation failure/current pass, original candidate `9aec6facfd12f05fa71557f7d40e4594165d0381b3a3a67d39e20b0cd7e5a609` unchanged, files unchanged, zero model calls/executions. Trial21's rejection remains terminal and immutable.

Six pure controls and the ordinary native callback/restart path reportedly passed. The mixed shared-correction native path is still running at this checkpoint. Final test signoff awaits that terminal result; no broader suite or live success is claimed.

### Prospective fix final signoff

Run64670 terminated with eight tests passing in113.461 seconds: six pure temporal controls and two native full/restart flows, including shared corrections. An additional original-review-role/outcome tamper control and independently reset subtests then produced seven pure passes in0.056 seconds. Production source stayed unchanged during verification; diff check is clean. No remaining material blocker for this narrow prospective projection fix. These are fixture/reconstruction checks, not evidence that a live reviewer will accept; Trial21 stays rejected. A fresh admission and campaign readiness check remain required for another live trial.

### Trial22 follow-up: serialization-specific digest repair

Independent read-only inspection reproduced the later helper failure exactly. Trial22 original experiment review `388020d30f3db30bf017b6ac3cd91243876f3a136c5a0c85a82fd5520f792489` is schema-valid/accept. Its recorded validation candidate hash is `0df3d09f55e098ce73cc7752e6798464dd8190e611dcb2721e9e7f4798b95b93`, identical to the producing `cycle.digest`. The temporal helper incorrectly used envelope digest `1ffd1552f1bf4632570fd2f1c911810725eaae97062d3a654c222d0fb7486de9`. Non-ASCII report content makes their different JSON ensure_ascii conventions observable.

The one-line repair compares this field using `cycle.digest`, matching the writer and both existing runtime validation/replay paths. It neither rewrites stored identities nor accepts an arbitrary alternate hash. The new Unicode regression explicitly demonstrates differing serializer hashes, accepts the recorded convention and rejects a forged hash. Eight temporal tests reportedly passed in0.054 seconds. No material source blocker; actual read-only Trial22 reconstruction remains pending at this entry. No provider call, target execution or acceptance by reviewer.

Actual reconstruction17800 subsequently terminated successfully:21421-byte request/11347-byte headroom,150 Trial22 files and campaign ledger unchanged, zero model/target effects (`target/b91-trial22-unicode-readiness.json`). Scoped repair signoff complete. Historical failure remains recorded; this check is not a new live review or acceptance.
