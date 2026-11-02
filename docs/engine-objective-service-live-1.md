# Finite objective live batch 1: incomplete

The single approved batch ran at commit `ca4e7089`. It generated a source repair,
passed separate runtime review, and passed the immutable ABI smoke check. It then
incorrectly stopped as `idle / no_eligible_open_attention`, before service round 1.
No service obligation, final objective review, acceptance, relationship revision or
adaptive pursuit was demonstrated. The successful smoke result is not objective completion.

The native attention projection in `project/attention.rs` sets pending objective
contributors to `status=None`. The generic ledger selector requires `Some(Open)`.
Once smoke succeeds, its mechanical preference is excluded and the two still-active
objective contributors are also filtered out. Native objective status still reports
two active contributors, round 0, next=propose and no service receipts. This is an
implementation defect, not model abstention or a desire culminating into another form.

Independent result/source review: `/root/objective_live_review`. That review also
found why the authored full-flow test missed this: it enacted a relationship then
service directly; its patch was decoded/previewed but never enacted. A regression
must exercise successful patch/smoke followed by ordinary service choice. No fix or
restart was applied to this closed run.

Accounting: two Luna calls, zero corrections, one patch/smoke group, zero service
groups or reads. Reported tokens: 7,376 prompt and 6,426 completion. Settled cost
$0.003131. Campaign totals: 228 calls, 63 groups, $0.187888; no campaign unknowns.
Unused batch allowances expired at terminal outcome; remaining campaign capacity
is not a new attempt grant. The historical older Garden unknown is unaffected.

Fresh-process launcher replay returned `replayed_without_dispatch: true` with
unchanged counts. Frozen packet, implementation and protected service hashes were
checked; the protected service also matches in the actual coding workspace.
No acceptance copy was made because evidence cannot support objective acceptance.

Evidence: `experiments/objective-service-live-1/result.json`; isolated full local
journals under `target/objective-service-live-1`. Raw provider bodies are not committed.
Launch: 2026-10-28 11:54:05 UTC; admission deadline: 13:52:43 UTC (stricter than
120 minutes from launch). Pinned runtime and fresh official rates were verified.
The initial sandbox DNS/socket restriction was resolved through normal approved
escalation before dispatch; no runtime/network control was weakened.
