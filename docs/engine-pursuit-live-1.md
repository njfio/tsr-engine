# Native pursuit live attempt 1

The approved single attempt stopped at `review_reject`. One model proposal and one
separate runtime review cost $0.002400. No corrections, execution groups, protected
test runs, relational admissions or formal acceptance occurred. The full behavioral
cycle therefore remains untested by this attempt.

The generated patch chose distinct perspectives first and then filled remaining
slots with existing distinct IDs. The reviewer rejected duplicate IDs carrying
different tags. The frozen task's global input contract specifies distinct IDs,
so that counterexample is outside its domain. Independent result review found no
demonstrated in-domain defect; source inspection is not a test pass.

A concrete integration omission helped this happen: the public projection carried
“For distinct input IDs” in parent A, but omitted the frozen global `input_contract`
that applies to both parents. The review itself acknowledged ambiguous duplicate-ID
semantics yet returned negative findings. The run's rejection is preserved. No
manual override, repair substitution, second attempt or semantic correction occurred.
A future local correction should preserve global domain assumptions in both
generator and reviewer projections, without changing the original acceptance tests.

Both responses parsed and passed structural validation without corrections. The
review gate contained the rejection. This is evidence of containment, not evidence
of semantic reviewer reliability, relational development or TSR causal advantage.

Fresh-process terminal replay returned the same rejection; campaign, adapter and
controller journal hashes were unchanged. All bills are settled. Tokens: 5,587 input
and 4,944 completion (10,531 total). Campaign now totals 220 calls, 60 groups and
$0.178524, under 228/62/$5 caps. The unused allowance for this single attempt is closed.
Older Garden unknown accounting remains separate; no historical record was reset.

The frozen packet and source remained unchanged. The pinned image was inspected
through the supported permission flow; it was not run. Normal sandbox socket access
was unavailable; authorized escalation succeeded. No security settings changed.
No new regression suite was needed for this evidence-only commit. The prior local
release tests and unexplained authored-worker timeout remain documented separately.

Machine-readable evidence index: `experiments/pursuit-live-1/result.json`. Full local
receipts/journals: `target/native-pursuit-live-1/`. No raw provider response bodies or
credentials are committed.
