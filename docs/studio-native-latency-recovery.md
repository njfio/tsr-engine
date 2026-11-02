# Studio native reconstruction and interrupted-admission repair

The retained Pond cycle stopped during local `advance_1`; provider requests and
prior game executions were settled. The delivered visual-v4 remains unchanged.
This repair does not classify an unknown external effect as safe to replay.

## Concrete cause and narrow change

Sampling the retained 191-event history identified nested reconstruction in
`continuation_anchor`, including duplicate relational projection. The change
reuses that one projection and caches decoded adaptive records by content hash
within a single Engine instance. Every access still rereads CAS bytes, hashes
those bytes, reloads the event sequence, checks ordered parents/schema/task, and
recomputes event identity. No state/authority cache, persisted checkpoint, lost
history, changed timeout, or permission expansion is introduced.

On the same retained history, read-only cycle request output was exactly equal:
26.9744 seconds originally, 21.4066 seconds after (about 21% reduction).
Successful isolated native admission measured 72.5442 → 62.1426 seconds (about
14% reduction). Both selected `resume`, with `executed=false`; all non-provenance
output fields matched. Event IDs, next state root and binding differ because the
separate admissions have different wall-clock provenance. These are individual
measurements, not a guaranteed deadline. The previous live120-second timeout also
reflects runtime variability; the timeout remains120 seconds.

The first cloned admission rejected a directory-bound inspection handle. That
failed benchmark is preserved. The successful benchmark used fresh native
clone-local handles and an explicitly authored local review, retaining the model
edit text. It is test infrastructure, not new live approval or game execution.

## Recovery boundary

`native-advance-not-committed@v1` requires exact authenticated journal/admission
hashes, a dead worker and exclusive old journal locks, no unsettled external or
native effects, and the exact native head retained by the completed continuation.
The native CLI independently verifies the history. The head is checked again
after source/scope/requirements/resource verification. Any committed transition,
changed prefix or unknown effect refuses recovery. The old pending journal stays
pending; a separate append-only Studio disposition permits fresh prospective
preparation, never a fabricated advance receipt or blind replay.

Stop is rechecked under the same control lock that reserves the new request and
switches its active identity. A Stop arriving later binds to that new request and
is checked before dispatch. Desires/history are not deleted.

## Verification

- Three Rust decoded-cache tests passed: changed/missing CAS, external and local
  appends/rollback/reopen equivalence, broken event chain.
- Twenty native cycle regression tests passed in106.04 seconds, including
  source drift, unknown effects, restart and independently reviewed progression.
- Twelve recovery tests passed, including full retained-prefix verification with
  explicitly authored native faults, wrong resource root, head change during
  verification, live worker, unknown effects, tampering and Stop race.
- Thirty-three local API/lifecycle/recovery tests passed. First socket-bound run
  failed under managed sandbox networking; the same suite passed with authorized
  loopback test access. No provider calls occurred in fixtures.
- Independent review found and resolved the Stop race and direct test-discovery
  placement. A proposed additional event equality cache was not implemented:
  floating-point equality would not be a safe content-identity witness.

Detailed logs, samples and isolated benchmark copies are under
`target/native-latency-repair/`. Historical fixtures and delivered game files are
not edited. Live recovery outcome and final accounting will be appended after
verification and actual continuation.


## Receipt source projection follow-up

The source-bound recovery resumed five model calls without a game execution.
Two independent reviews rejected unsupported current-source smoke-test claims.
Their supplied packet labeled the native receipt `source_unknown`, even though
its authenticated `source_after` matched the current snapshot. The native
projection had filtered for this match but omitted the field the public adapter
needed. The reviews remain valid judgments of their supplied evidence; they are
not overwritten or promoted to approval.

The prospective correction exports `snapshot: source_after` after the existing
native receipt and Rules provenance checks. Missing, stale and unrelated returns
remain excluded. A red/green native regression verifies exact binding, restart,
and invalidation after source change. The relational suite passed 15 tests with
one optional semantic-shadow fixture ignored. Cross-language verification also
exposed an obsolete authored runner that expected whole-file pairs instead of
the current chunked exact-span payload; only that test double is updated.

The third retained proposal removed the disputed claim, but is unreviewed and
unexecuted. A cooperative operator pause allowed its request and local preview to
settle before repair. This is an intervention, not engine-selected completion.
The preserved run is `4940a823-d477-4047-8883-41a0affeb3f6`.
At that pause: 384 cumulative calls, 120 execution groups, $0.585387 known charges
and $0.635387 committed accounting. The historical $0.05 quarantined generation
reservation remains an estimate, not a reconciled charge or verified maximum.
This run added five calls, zero executions and $0.036617 known charges.

The native test startup delay resolved without security changes. Its build took
4m23s, followed by an OS loader wait, then the suite ran in 23.86s. These are local
verification delays, not provider latency. No duplicate test process was started.


Final cross-language verification passed: the authored native multi-file flow
passed (7.43s), and all 24 Python evidence-epoch/projection checks passed without
skips using its real serialized request. An initial test-only type-name error
was corrected (`ExactPatch`); its failed build log is retained. No provider or
container execution was used by these fixtures.

The second old-directory compile remained in `SearchPath::new`/`readdir` after
six minutes. It was interrupted, its orphan child explicitly stopped and exit
verified. A separate `CARGO_TARGET_DIR` under the task ran entirely offline,
preserving the old directory and every native/artifact record. The final affected
fixture compiled in 16.76s. This avoids the measured build-cache bottleneck; it is
not a claim of a further native replay speedup.
