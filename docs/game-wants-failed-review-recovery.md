# Exact ordinary-review recovery

This repair continues the already approved pair; it does not rerun TSR or start another experiment. The original HTTP400, generated proposal, pending review, lookup, reports and all spending remain preserved.

`scripts/engine-browser-failed-review.py` separates accounting reconciliation from one-shot retry authority. It authenticates the original request and retained response header, exact provider singleton failed-zero record, generated proposal association, native source, and pending ordinary review. Reconciliation appends native and campaign proofs idempotently, with exclusive launch/cycle locks and campaign CAS. Only the exact unknown bill may be removed from blocking accounting; other unknowns, overrun and Stop remain blockers.

A separately reviewed immutable ticket pins this implementation and permits one identical review request. The original ordinary Adapter and Cycle continue with the retained proposal and callback: generation is not repeated, counters are not reset, and global HTTP503 recovery policy is unchanged. A second provider failure or malformed fresh response stops. A valid rejection remains criticism for the ordinary next-choice path. A retained completed review closes an interrupted outer callback without another request. Once the original review completes, the spent ticket no longer intercepts ordinary future recovery.

The HTTP400 cause is still unknown because the existing transport withheld its diagnostic body. Exact zero use/charge establishes safe accounting and absence of a review judgment; it does not imply another request will succeed.

## Offline verification

Final focused run: **15 passed in 155.880 seconds**, no paid calls. Authored tests exercise provider-ID/charge/usage/chronology negatives, stale source and unknown effects, Stop and duplicate lock controls, native-proof-before-campaign interruption, idempotent reconciliation, interruption after claim/reservation, exact wire reuse, reviewed-result restart, second503, malformed response, valid rejection → fresh retention, and inert replay. These are authored fixtures, not live success or TSR development.

An initial invocation accidentally included the imported base class: 32 checks passed, 3 opt-ins skipped, and 2 new draft fixture cases failed on an incorrect profile constant. Subsequent focused runs exposed and fixed the native reconcile-branch guard and fixture/accounted receipt labeling. All logs remain under `/tmp/failed-review-recovery-*.log` and are copied into the local run evidence before live continuation. Full workspace/Clippy cleanliness and new browser execution through this recovery are not claimed.

Independent `/root/binding_review` reviewed the patch and exact live proof. Approved proof: `4cea5e78d7082cfdcafa0986ae509271a7c558c27038a9b69ac4b208a02f3f3e`; approved one-shot review: `2f4f47ab61ef289b869bec0a85c837a440293aa34051a809a564e39d60f7fe27`. The final 15-test run passed. Actual reconciliation and live continuation outcomes must be recorded separately; this document alone does not claim they happened.

## Live preflight correction

The first reconciliation stopped before either accounting mutation or provider dispatch: it compared the campaign binding using the native JSON canonicalization. The actual retained Unicode request matches `campaign.digest`; native identities still correctly use `e.sha`. The comparison is corrected only at this boundary, and every authored recovery fixture now includes Unicode in its retained proposal. The failed check and original ticket remain preserved; the revised ticket is `489e3d2b7257bd22e5a958ef517b5ca58fabe41dc96da5d223344ec7b886af97`. Independent review approved this narrow correction conditional on the repeated 15-test suite. This is an additional operator repair, not model behavior.
