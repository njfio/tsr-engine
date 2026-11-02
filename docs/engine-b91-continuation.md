# B9.1 final-stage continuation

This path completes only the two pending generative stages after independently
verified patch and unchanged experiment receipts. It does not repeat executions,
reopen Trial 11, or claim that a retained source prediction is an observation.

`scripts/engine-b91-continuation.py` uses a new directory, admission and driver
ledger. Its admission pins the parent admission, STOP, driver/provider journals,
entire native project file inventory, source snapshot, criteria, commitment and
native state root. Preparation checks the exact settled four-call prefix and both
successful execution receipts. Source, unknown accounting or history drift fails
before dispatch. The parent is checked again at each mutation boundary.

The native project is copied exactly. Its old shadow/resource namespace is retained
byte-for-byte as `parent-shadow` in the continuation directory; it is not erased,
reclassified or counted as a new execution. A new prospective native resource
episode is opened in the clone. All original main events and CAS remain a prefix.
The copied provider journal retains its exact original rows. An explicit relocation
view changes only the local controller directory, never historical event hashes or
provider input authority. No new origin is presented as an original live episode.

Native admission is two calls/81,920 total reserved model tokens, zero patch bytes.
The native lifecycle currently requires a positive tool ceiling; its minimum 1 ms
is used solely to make guidance carryover representable. The runner has **no worker
or commit path**, rejects execution/commit methods, and has zero execution-group
admission. This does not authorize even a 1 ms command. The original fixed synthetic
scope and immutable tests remain unchanged. Rebinding guidance is computed by the
native source/intent/scope checks; the runner never republishes goals.

Each late-stage request uses the explicit 32,768-byte UTF-8 wire profile, at most
32,768 input tokens reserved, 8,192 completion tokens and a 240-second deadline.
The default full-trial driver remains unchanged. Durable nested intents distinguish
completed stage replay from uncertain provider/application effects. Unknown or
invalid results never auto-retry or reclaim reservations. Missing independent
review is a pause, not a new request or automatic approval.

The final proposal is never committed or executed. Independent semantic review
remains operator-assisted. Closing the new episode is advisory-only: no automatic
acceptance/rejection/training label, maturity or causal-benefit claim. Historical
Trial 11 remains incomplete; any future continuation result must be linked to it
explicitly when assessing the combined sequence.

## Commands (live permission still required)

Prepare a fresh draft only immediately before a separately approved run:

```sh
python3 scripts/engine-b91-continuation.py prepare \
  --parent-admission target/b91-live-trial11.admission.json \
  --trial target/b91-trial11-continuation \
  --admission target/b91-trial11-continuation.admission.json
```

Review that exact draft and record its printed SHA-256. Once explicitly authorized:

```sh
python3 scripts/engine-b91-continuation.py run \
  --admission target/b91-trial11-continuation.admission.json \
  --admission-sha256 REVIEWED_HASH \
  --reviews target/b91-trial11-continuation-reviews \
  --allow-paid-inference
```

Review files are locally requested through the existing source/subject-bound
`FileReviewer` contract. On a normal review pause, supply the independent review
and rerun the identical command before the admission expires. Do not edit generated
output, change the admission, remove STOP or retry an uncertain operation.
Read-only journal inspection performs no model or target action:

```sh
python3 scripts/engine-b91-continuation.py inspect \
  --admission target/b91-trial11-continuation.admission.json
```

No live continuation is authorized by these instructions. Fixture results below
are authored mechanics evidence, not actual model quality or a live B9.1 pass.

## Offline verification evidence (2026-10-25)

- Complete Python suite: **645 tests, 622 passed and 23 skipped**, no failures,
  192.932 seconds (`target/b91-continuation-full-python.log`). Live calls and real
  target execution opt-ins were disabled. Native fixture tests were run separately.
- Separate Python process terminal replay: **1 passed**, 66.461 seconds
  (`target/b91-continuation-process-replay.log`). Provider and reviewer callbacks
  deliberately fail if invoked; the entire continuation tree remains unchanged.
  This test was added after full-suite discovery and verified separately.
- Preserved actual Trial 11 preflight: zero model calls, zero tool milliseconds,
  two original parents visible after native carryover, pending payloads 13,910 and
  16,290 bytes. The **entire original trial tree** remained byte-identical
  (`target/b91-continuation-preserved-final.log`). This was an offline clone only;
  it has a STOP marker and is not a live admission.
- Initial regression failure correctly found native read-time derived CAS cache
  creation in the parent. Native prefix inspection now occurs only in disposable
  copies, with original inventories checked before and after. No historical event,
  source, STOP or billing record was rewritten to fix that test.
- Review identified two restart defects: normal review-pending was wrongly treated
  as terminal, and expiry prevented recovery after terminal-result persistence but
  before summary-file creation. Both have focused before/after coverage. Only a
  validated admission-bound terminal journal with no pending effects exempts
  read-only recovery from expiry; a stray terminal file cannot do so.
- Independent final source review (`readiness_review`) found no remaining blocking
  implementation issue. It specifically reviewed parent preservation, accounting
  archival, review pauses, expiry and advisory-versus-acceptance boundaries.

The old accounting archive remains inspectable and is never counted again in the
new episode. This continuation's final independent judgment remains advisory: it
can support a report on the linked decision cycle, but **native accepted-task
completion is not adopted automatically**. An explicit future admission would be
needed to adopt an independently reviewed acceptance/rejection in the clone. It
would still confer no permission to execute the final proposed change or relabel
the parent's stopped outcome. No data maturity or calibrated-accuracy claim is made.

Final targeted command:

```sh
PYTHONPATH=scripts \
TSR_B91_TEST_WORKER_BIN="$PWD/target/debug/deps/tsr_cli-62904bb7a7cdafee" \
python3 -m unittest test_engine_b91_continuation -v
```

The six tests present at that invocation passed in 278.884 seconds
(`target/b91-continuation-native-verified.log`): completed-effect restart boundaries,
real file-review pause/resume, parent source/limit rejection, interruptions both
before provider invocation and after a fake provider result, CLI pause-versus-stop
classification, and expired terminal-summary recovery. The subsequent separately
invoked fresh-process replay test passed as recorded above. Unknown transport
outcomes retained native reservations and did not dispatch again. All worker
receipts in these tests are explicitly cfg(test) fixtures, with zero target commands.

Launch readiness: the implementation and offline preparation are verified. The
remaining gate is **fresh live authorization and review of its exact new admission**,
not another code or capacity blocker. The previously prepared offline clone is
stopped and must not be promoted into a live grant. A new prepare command pins the
current implementation and unchanged parent again. Final semantic quality and the
model's actual revision/proposal remain unproven until that bounded live run.

## Subsequent approved live result

The separately approved two-call continuation completed on 2026-10-25, with zero
executions and $0.002005 settled cost. See
[the linked result report](engine-b91-trial11-continuation-result.md) for actual
semantic findings, replay integrity and advisory-only limits. That grant is now
consumed; the commands above do not authorize further calls.
