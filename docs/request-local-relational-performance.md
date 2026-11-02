# Request-local relational projection reuse

Offline authored verification, 2026-10-31. Parent authorization is restricted to
this optimization; paid calls and live packet rebinding remain prohibited.

## Change and integrity boundary

`project_cycle_request_inner` computes one relational projection and passes it by
immutable reference to reconsideration and continuation-anchor calculation. It
lives on the stack for this request only; standalone continuation/anchor methods
still compute fresh context. No general cache, persistent snapshot, new dependency,
permission change, semantic rule, model limit or timeout increase was added.

All existing native, continuation-CAS, source and final resource checks remain.
An explicit final equality also binds the shared relational input snapshot to the
final current source snapshot. The private deterministic test hook is a no-op in
normal calls and has no external control interface. There is no reuse across a
provider call, write, execution, restart or later request.

The new regression mutates source and native CAS after projection and confirms
rejection; restoration/restart preserves the original binding without new events.
It also closes lifecycle authority in the separate shadow store, confirms the
native head remains unchanged, and proves request rejection and stopped restart.
Existing tests retain stale responses, replay, unknown effects, resource changes,
retention, advisory review and closure coverage.

## Verification and preserved failures

Final cycle suite: **22 passed, 0 failed**, 104.66s. Independent source review
accepted the final patch and checks, and separately inspected fixed-state results.

Initial new-fixture failures are retained: it first requested verification closure
without verification, then incorrectly expected advisory closure to change the
native head. Diagnostic output established `cycle inputs changed; read again`;
lifecycle authority is stored separately (`lifecycle.rs:107,129`). The assertion
was corrected to check actual lifecycle closure, unchanged native head and restart,
without changing production closure validation. All 21 pre-existing tests passed
in those earlier runs. These are local fixture corrections, not live behavior.

Two Cargo test launches were interrupted after prolonged pre-test silence (exit
130); their compiled binaries were then invoked directly, with separate logs. The
final direct invocation completed successfully. These launch delays remain
unexplained and are not counted as successful runs. The measured binary build
completed in 4m15s, outside benchmark timing.

## Matched fixed-state comparison

Baseline includes the previous decode-only cache (`1a736f6a`). Five alternating
pairs per command, separate processes, no OS-cache flush. All **30 outputs** matched
after excluding only `elapsed_wall_ms`; event payloads, CAS and editable files kept
the same fingerprint. The fixture was already in its expired/blocked budget state;
no authority was renewed or rewritten for the comparison.

| Command | Baseline median | Request-local median |
| --- | ---: | ---: |
| Cycle request | 2.663889s | 2.186431s |
| Full project request (control) | 1.710683s | 1.636293s |
| Criterion context (control) | 0.320120s | 0.318232s |

The targeted cycle request is approximately **18% faster**. The smaller control
changes are not attributed to this patch. These warm measurements are not a
cold-start guarantee or scaling study.

Baseline binary SHA-256:
`babed00f1ff248d6e8d171033b5855b536115e53725bd409bc7f705916cc1a14`.
New binary SHA-256:
`3ead8b3e75a36c963d432bfa12bede084bef002b966fce6d09786d42418ce035`.
Fixed-state fingerprint:
`26aff17a443fd5d194753c11c64bae29ec6b12f9802e7c167dc848fd5c787f56`.

## Complete authored flow

The first profiling invocation omitted the inherited native-fixture opt-in and
was **skipped**, not passed. Its log and profile remain preserved. The correctly
enabled fixture uses both `TSR_BROWSER_ADAPTER_AUTHORED=1` and
`TSR_BROWSER_SEALED_AUTHORED=1`; it runs authored model responses, native admission
and the existing isolated real browser. No paid model request is made.

The complete active authored flow **passed in 513.755s**, with the same **556
native calls**. Its cycle body took **512.233s**, versus **558.240s** in the preceding
identical decode-cache fixture: **8.2% lower body time**. Raw total fell from
632.198s to 513.755s (18.7%), but the prior initialization was 73.041s and the new
initialization 0.382s. Disappearance of that unexplained outlier is **not** credited
to the optimization.

Native calls consumed 488.410s. Cycle-request total fell from 326.340s to 289.193s
(122 calls); previews from 87.686s to 81.817s (12 calls); full-project requests
changed from 51.789s to 52.059s (47 calls). The targeted command accounts for most
of the measured body reduction. These are separate complete-fixture runs, not a
statistical study or causal performance guarantee under arbitrary system load.
No other tests or benchmark ran concurrently with this measured full flow.

The complete fixture retained its ordinary returned observation, independently
reviewed named-criterion Contrast, real native Preference/Council eligibility,
subsequent composition, retention, fresh-session/restart and integrity assertions.
Semantic responses are authored; the isolated browser execution is real. No live
model emergence or new user preference was inferred from this test.

Scoped rustfmt and `git diff --check` passed. The previous incomplete broad
workspace run and baseline lint issues were not relabeled or rerun here; this
patch's verification is the 22 cycle tests, fixed-state comparison and complete
active authored integration. Ledger and frozen manifest hashes remain unchanged.

All raw logs, fixed-state stdout/stderr, comparison script, profile reproducer and
CLI timings are under `target/relational-local-performance/`; the active full flow
is in `active-flow/`. Prior full-flow evidence remains under
`target/criterion-performance/optimized/`.

## Scope remaining

This changes runtime cost, not TSR semantics or demonstrated model behavior.
The ordinary-return → named-criterion Contrast → reviewed native Preference →
Council eligibility → later composition chain remains authored-test evidence,
not proof of spontaneous live culmination or comparative advantage. The next
behavioral verification requires the outstanding live authorization and a new
prospective binding to verified code; the old frozen packet is not modified.
Successful-combination identity formation without Contrast remains a separate
semantic boundary, not something introduced by a performance patch.
