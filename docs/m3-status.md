## Qualification checkpoint: 62 admissions (2026-10-15)

[Structure progress, verified exports and approaching resource boundary](m3-qualification-followup-62.md). Two workers active; guard still monitoring near200GiB. No cleanup or paid calls.

## Qualification checkpoint: 59 admissions (2026-10-15)

[Structure admissions, preserved rejections and guarded continuation](m3-qualification-followup-59.md). Two workers active; no storage pause, cleanup or paid calls.

## Qualification checkpoint: 57 admissions (2026-10-15)

[Storage completion, structure transition and reserve checks](m3-qualification-followup-57.md). Structure has two supervised workers; no storage halt, paid calls or cleanup.

## Qualification checkpoint: 56 admissions (2026-10-15)

[Verified exports, completed filtering transition and active storage queue](m3-qualification-followup-56.md). One storage case remains active; no resource halt, paid calls or cleanup.

## Qualification checkpoint: 48 admissions (2026-10-15)

[Core reconciliation, guarded continuation and storage projection](m3-qualification-followup-48.md). Four more strict admissions; filtering extension running. No paid calls or cleanup.

## Offline readiness follow-up (2026-10-15)

See [offline repair, bounded extension, resource guard and cleanup dry run](m3-offline-readiness-followup.md). Admissions remain 44; two further candidates are scheduled, not admitted. New evaluator frozen separately; no additional paid calls.

## Qualification checkpoint and next evaluation readiness

At 2026-10-15T04:06Z, 44 distinct admissions are exported and two local RTS qualification
workers remain active under the original finite controller. All frozen hashes match. Free disk
is 278.59 GiB (178.59 GiB above the reserve). No further paid calls or cleanup occurred.

The current candidate ceiling is only 148 total admissions at perfect yield, and the remaining
104 candidates project roughly 494 GiB of retained directories. Capacity is therefore likely
to halt the queue before completion. Do not lower the reserve or delete unapproved artifacts.

[Per-arm diagnosis and readiness recommendation](m3-next-evaluation-readiness.md) separates
valid test-tampering/workflow rejections from observation/token limits and the transport timeout.
It preserves the original outcomes and recommends offline, versioned context/diagnostic fixes
before any new paid approval. [Measured capacity checkpoint](m3-qualification-capacity-checkpoint.json).

## Previous terminal comparison report

## Repaired comparison terminal; billing reconciled; local qualification resumed

Updated2026-10-15T04:00Z. [Final cohort report](m3-paired-cohort-2-final.md).
The repaired evaluator passed strict outline revalidation: baseline999/0/15, all three named
red/green checks, mutated996/3/15, full reversal999/0/15 (passed/failed/ignored). Independent
review verified the task, engine identity and evidence relationships. This existing-task
revalidation does not inflate the corpus: **44 distinct admissions**,18AgentID and26RTS.
[44-task split-input diagnostic](m3-split-audit-expansion-44/report.json) passed; this is not an
untouched confirmatory holdout.

The four-task paid retry is terminal: record1rejected/3unavailable, rules3rejected/1unstarted.
Zero accepted; one fully observed pair, both rejected. Two record arms hit the conservative
token reservation bound. The final record arm timed out after dispatch, preventing its rules
arm from starting. Provider metadata later confirmed its $0.003789 charge; the completion
was never delivered or replayed. All original outcomes and initial accounting snapshots
remain preserved. [Final accounting](m3-paired-cohort-2-final-accounting.json) supersedes only
the initial export's unsettled dollars: **$0.119525 cohort, $0.165094 cumulative, $0 reserved**,
78 settled calls including prior attempts/probe. No new paid attempt was launched.

The original local-only coordinators29252/29246 resumed at03:58:52Z after verifying all seven
paid process exits, no paid workers or M3 containers, and287.8GiB free. Workers66598/66599 now
qualify the next two RTS tasks. The100GiB disk floor and two-worker limit remain unchanged.
[Reconciliation receipt](m3-paired-cohort-2-reconciliation.json) records that billing was pending
at resume; the later final-accounting receipt resolves it. No known handoff issue remains.

M3 acceptance remains unmet:44 of at least150 qualified tasks, incomplete exploratory paired
coverage, no accepted repairs in this cohort. No push, PR, merge, deployment, additional32-task
paid launch, cleanup deletion, prompt tuning, policy change, or oracle relaxation occurred.

## Historical scheduled state (superseded above)

## Repaired retry frozen and scheduled —2026-10-15

Commit38196f9 repairs the exact crate-level `deny(unsafe_code)` resolver false negative and
updates approved provider ceilings to $0.30/$1.20 per million input/output tokens with a $2
reservation. The aggregate cap remains $100 including prior spending. See
[repair diagnosis](m3-module-2-repair.md), [independent review](m3-module-2-review.md), and
[verification](m3-validation/m3-module-2-summary.json):709 Rust tests passed,5 ignored,2 known
M0 tests filtered; fmt/clippy/build passed;15 broker tests and6 scheduling tests passed.

The separately [frozen four-task retry](m3-paired-cohort-2-plan.json) is scheduled by
handoff PID95937. Its verified state is `waiting-for-active-pair`: native workers34506 and34507
continue, while coordinators29252 and29246 are paused. No third sealed worker was launched.
After both workers finish, the repaired engine must revalidate all strict admission phases
for the affected outline task before any paid call. The retry remains pending, not validated
or complete. Original results are retained. The queue resumes automatically after clean
completion; abnormal paid completion requires reconciliation. The original2700-second worker
timeout guard and100GiB disk floor remain in force. [Launch receipt](m3-paired-cohort-2-launch.json).

At launch, cumulative spending remains $0.045569 across36 settled calls with no outstanding
reservation.42 admissions are exported. M3 acceptance is still unmet: at least150 tasks and
adequate observed baseline comparisons are required. No extra32-task paid launch, push, PR,
merge, deployment, or build-artifact deletion was performed.

## Historical attempt1 result

## Four-task cohort terminal; local qualification resumed

The frozen cohort ended at **2026-10-14 22:33:50 America/New_York**. Six arms were
unavailable before inference because current provider list rates exceeded the frozen
price guard. The fourth task's record arm exited 2 during trusted-test identity preflight;
its rules arm was skipped by the existing halt rule. There are **zero observed task
pairs**, and these infrastructure-censored outcomes support no repair-performance claim.
All 8 planned arms remain represented in [results](m3-paired-cohort-1-result.json) and
[raw execution evidence](m3-paired-cohort-1-execution/).

No new inference call, reservation or charge occurred: cumulative settled spending is
**$0.045569**, 36 settled calls, 0 outstanding. Catalog list input/output rates were $0.30/$1.20
per million versus frozen ceilings $0.15/$0.60. Discounted rates do not satisfy the
list-price check. [Catalog evidence](m3-paired-cohort-1-price-blocker.json).

The frozen module evaluator rejects non-doc ancestor-file attributes; RTS daemon
`main.rs` has `#![deny(unsafe_code)]`, so the `outline::tests::glob_ext_match` identity
cannot resolve through this stricter evaluator. Its earlier qualification used the older
engine. No oracle rule or financial bound was changed, and no paid retry was launched.

The pair-boundary scheduling repair succeeded. After terminal reconciliation confirmed
no paid workers/containers and a reconciled ledger, both original native coordinators
were resumed at 22:35:02 Eastern. Both completed children were reaped with exit 0; the next
two qualification jobs started. See [reconciliation](m3-paired-cohort-1-reconciliation.json)
and [resume verification](m3-paired-cohort-1-resumed.json). Reap timestamps include the
intentional pause and are not qualification compute durations.

The corpus now has **42 strict admissions**, with the [42-task integrity audit](m3-split-audit-expansion-42/report.json)
passing. M3 remains incomplete. The planned additional 32 paid tasks remain unlaunched.
Any price-compatible or evaluator-compatible retry requires a separately frozen matched
attempt preserving these results; it must not silently revise this completed cohort.

## Pair-boundary scheduling repair

The original whole-cohort waiter was retired before any paid work began. The reviewed
replacement `scripts/m3-pair-boundary-handoff.py` pauses the original qualification
coordinator while its two active workers finish, then runs the unchanged approved
four-task cohort. It checks worker identities, strict phase evidence, container/process
exit, and shared billing reconciliation. It resumes the original queues after clean
paid completion. A guard returns timeout control to the original coordinator before
an active qualification reaches its original 2700-second deadline.

At launch (2026-10-14 22:06:13 America/New_York), replacement PID 3607 was waiting for
the active pair. Two earlier RTS tasks were admitted, bringing verified admissions
to40; aggregate export remains38 until a quiescent qualification boundary. Five offline
scheduling tests passed and independent review found no remaining blocker. No additional
32-task paid run is authorized or launched. See [repair evidence](m3-pair-boundary-repair.json)
and [40-task checkpoint](m3-progress-40.json). Live status: `target/m3-pair-boundary-handoff-state/status.json`.

## Approved four-task launch

Following explicit payload/destination approval, normal execution review accepted the
frozen four-task paired launch at **2026-10-14 21:28:55 America/New_York**.
Controller PID 47051 waits for the RTS core cohort to finish; it has not yet made new
paid calls. Native coordinator PID 29246 is paused while its current child cohort continues.
The reviewed handoff resumes native qualification only after clean paid worker completion.
The same cumulative $100 cap includes **$0.045569** already spent.
[Authorization and launch receipt](m3-expanded-paid-launch-authorization.json).
Earlier denial records below remain historical evidence.

## Current expansion checkpoint

Qualification resumed under renewed owner instruction. Six AgentID candidates completed
strict admission, bringing the corpus to **38**. The [38-task integrity audit](m3-split-audit-expansion-38/report.json)
passed. A terminal interruption lost the final controller receipt; two process exits
remain unknown, explicitly preserved in [reconciliation](m3-expansion-qualification-reconciliation.json).
All six admission artifacts include completed strict phases; no exit was invented or task rerun.

The independent local-only continuation started at **2026-10-15 01:19:49 UTC**
(controller PID 29246), beginning the ten-candidate RTS core cohort. Its frozen plan
allows at most 17 sequential cohorts / 110 candidates, at most two sealed workers, with the
100 GiB reserve and existing command/time/output limits. No cleanup is authorized.
Live state is under `target/m3-following-native-expansion-state` and each cohort's state directory.

The frozen four-task paid cohort **did not start**. Automatic approval review rejected
its expanded source/test-data disclosure because it recognized only the earlier two-task
pilot authorization. [Exact denial](m3-expanded-paid-denial.json) is preserved; no retry or
indirect paid execution occurred. The owner subsequently directly approved the expanded
four-task disclosure under the same aggregate cap. The normal retry was again rejected before
process creation because review required explicit payload/destination authorization.
See [retry denial](m3-expanded-paid-retry-denial.json). No paid execution occurred.
Independent native qualification is being resumed; explicit authorization is requested.
Cumulative spend remains **$0.045569**, with no outstanding reservations.

Verification: **707 passed, 0 failed, 5 ignored**, two documented M0 filters; fmt/clippy
pass; 14 broker/launcher regressions plus one reporting integration regression pass.
M3 acceptance is still unmet. No push, PR, merge, deployment or cleanup was performed.

## Latest: bounded pilot retry completed

The approved three-arm retry completed at 2026-10-15 00:14:42 UTC. See
[machine-readable results](m3-pilot-retry-v2-result.json), [repair rationale](m3-pilot-repair.md),
and [prospective plan](m3-pilot-retry-v2-plan.json). This supersedes earlier pilot-status
statements below; all original failures and authorization history remain preserved.

| Source | Record | Rules | Paired eligibility |
| --- | --- | --- | --- |
| AgentID | Retry unavailable: terminal invalid tool-call shape | Original accepted | Excluded |
| RTS | Retry rejected: named tests pass, full-suite check fails | Retry rejected: frozen test-evidence guard | One usable task pair |

RTS rules changed protected evidence in `crates/rts-mcp/src/surface.rs`; the guard
rejected it before a full oracle verdict. Its zero `wall_clock_ms` is a frozen-engine
reporting limitation, not zero runtime. Controller elapsed time was about 391 seconds.
The valid paired acceptance result is 0/1 for both modes, with no performance-improvement
claim or meaningful generalization from one task. No diagnosed terminal model error was retried.

The SQLite startup collision was fixed by per-arm controller stores. Live traffic
successfully transported a 73,314-byte envelope around a 67,522-byte protocol input,
validating the distinct wire limit. Original RTS failure subtypes remain unknown.
Ten offline broker/SQLite regressions passed. Independent review approved the corrected
input-freeze and reporting checks. All pilot processes and M3 containers are stopped.

Aggregate provider spend is **$0.045569**: probe $0.000112, original pilot $0.016441,
retry $0.029016. All 36 calls settled; outstanding reservations $0; no billing halt.
No corpus qualification, push, PR, merge, deployment, or cleanup deletion was performed.
The corpus remains 32 admitted tasks; **M3 acceptance remains unmet**.

# M3 implementation and acceptance assessment

Local branch `codex/m3-evidence-pipeline` integrates corpus lane `57a1222` with
main `8b74019`, then adds defensive oracle, worker, selection, reviewed-tension and
broker work. Checkpoints `72669cc` and `8253d8a` preserve the earlier stages.
The original operator checkout and Claude's M0/store lane were not edited. No push,
PR, merge or deployment occurred. The harmless provider compatibility probe executed for $0.000112; no paid paired task run executed.

## Authorized pilot result (2026-10-14 19:50 Eastern)

Normal execution review accepted the owner’s explicit direct approval. The two-task pilot completed once: AgentID rules mode received oracle acceptance; AgentID record mode failed at CLI store opening (SQLite lock); both RTS modes returned unavailable after broker errors. There are zero usable paired comparisons. Eleven new settled model calls cost$0.016441; including the earlier probe, total settled cost is$0.016553 with no outstanding reservation. No retries occurred, and no pilot workers or M3 containers remain. Corpus qualification remains cancelled at32 admissions. See [pilot results](m3-live-pilot-result.json) and [authorization](m3-live-pilot-authorization.json). Historical zero-paid-call statements below describe earlier checkpoints.

## Cancellation checkpoint (2026-10-14 22:17 UTC)

Owner cancelled this job. Controller50304 and workers4189/8312 stopped; their two containers were removed, with no remaining M3 containers. Completed32 admissions and all evidence are preserved. Method-name validation and line/column conversion are incomplete, not admissions. The116 prepared candidates have not launched. No cleanup or paid calls occurred. See [cancellation evidence](m3-cancellation-checkpoint.json). The normal approval reissue was automatically denied without a pending approval ID; history remains preserved. A new task must not bypass that denial.

The earlier checkpoint below describes progress before cancellation.

## Current continuation checkpoint (2026-10-14)

The [generated admission index](m3-native-admission/README.md) and
[raw summary](m3-native-admission/summary.json) are the current count authority.
At this checkpoint 32 distinct production-function tasks are admitted: 20 from
`rs-agent-code-utility` and twelve from `agentid-rs`, across two lineages and nine families.
The broad cohort has admitted path relativization (61ff42944df2) literal-prefix parsing (ad6371ac1e52), surface-subset checking (e87fb59cfd92), retry backoff (7c1f762eb728), deadline-environment parsing (d12256fc06e3), path-prefix stripping (369295f610ed), latency bucket indexing (e47c64beb4e3), and latency bucket lower-edge conversion (51a5865dadd1); glob matching (0d00a69acdfe) and byte-offset lookup (2d6c105f3698) and grammar-string parsing (f3b2377f03d7) and test-path classification (758d7b339a20) also completed; remaining jobs continue.
Both later AgentID cohorts completed: seven of seven and three of three independently
qualified. Final completion: 2026-10-14 19:40:23 UTC. The next reviewed code-utility plan
contains fourteen distinct functions in four bounded screens, with two qualification
workers and a 100 GiB free-space floor; controller PID50304 completed all four screens (14 selected, 14 caught) and is qualifying with two workers. The first pair completed full baselines green (999 passed, 0 failed, 15 ignored) before mutation checks.
The launch headroom check found 456 GiB free and no remaining task containers. A later check found 447 GiB free; two task containers were observed during qualification. Cancellation-token remains censored because
its full-red output exceeded the 1 MiB cap; no retry or gate relaxation occurred.
Incomplete, failed and censored attempts are excluded.
The finite detached queue writes ten-second heartbeats and applies a 45-minute
outer bound per job, with at most two sealed workers. Existing 600-second per-command
bounds and evidence-integrity gates remain unchanged.

The independently selected second lineage, `njfio/agentid-rs`, has a pinned,
sealed full baseline of **456 passed, 0 failed, 0 ignored** in 236.486 seconds.
Its corrected two-function mutation screen and both independent qualifications completed successfully. Each task has a completed full-green baseline, named/full-red evidence, exact reversal, and named/full-green evidence. See [selection provenance](m3-second-source-selection.json),
[baseline](m3-agentid-baseline-v2.json), and [mutation plan](m3-agentid-mutation-plan.json).

The latest [implementation verification](m3-module-validation.json) passed
707 workspace tests (0 failed, 5 ignored, 2 documented M0 filters), including 31
focused split tests. Offline all-target Clippy with warnings denied, CLI build, and
formatting checks passed using Rust 1.90. These inert regression fixtures establish
implementation checks, not real paired-system performance. There are still zero real
paired system runs. The harmless provider probe succeeded for $0.000112, but the
prepared paired pilot was separately denied by automatic approval review before
launch. M3 acceptance is not met. Historical checkpoints below retain their original
scope and should not be read as the current task count. Local commits reported missing
lefthook, so commit hooks are not claimed as passing.

The split-integrity review found and repaired a fail-open CAS path: absent or corrupted
hidden diffs could previously become empty strings. The splitter now requires verified,
nonempty UTF-8 diff evidence, rejects duplicate task IDs, and reuses captured content.
The [v5 exploratory diagnostic](m3-combined-dedup-diagnostic-v5.json) supersedes earlier
snapshot-probe claims: all 34 current candidates provide source identities without snapshot
CAS content, so that probe is **limited**, not clean. Zero clone groups and the two other
clean probes do not establish complete cross-snapshot leakage coverage or a reserved holdout.
The frozen qualification engine and admission criteria are unchanged.
All 32 currently admitted original records also passed the repaired splitter in the [admission integrity audit](m3-admitted-integrity-32/report.json): no duplicate IDs, invalid required diff CAS, or invalidated cases. Snapshot-content coverage remains limited for all 32. Original records were preserved; repeat this check as admissions grow and before final evaluation.

The reviewed follow-on pool now has 116 unlaunched candidates, plus two still pending in the broad cohort. If all pass, the reviewed pool reaches exactly150 tasks; there is no margin for rejected or censored cases, and qualification remains outstanding. The behavior (five), language (nine), signature (five), metrics (three), and AgentID policy (two) candidate plans passed independent source review and have not launched. A further cache-key candidate passed independent review; a proposed byte-range mutation was excluded because the same function is already in the active broad cohort.

The separate [module-v1 engine](m3-module-engine.json), built from commit `345308474d4596fb7761829ddeca76a8c7849712`, resolves plain nested external-module test identities and freezes declaring ancestors and absent alternative paths. Its four inert regressions cover layouts, ambiguous/unsupported declarations, prefix spoofing, and before/after tampering. The active cohort and earlier prepared cohorts keep frozen engine238ac39. Only the new fuzzy (two), syntax (eight), typed-language (twelve), reference (nine), and context (seven) plans use module-v1; none has launched. Non-doc production attributes such as `must_use`, macro/conditional ancestors, custom module paths and mixed inline/external chains remain unsupported. Inventory counts remain ceilings, not demonstrated usable capacity.
See [follow-on review](m3-following-cohort-review.json).

## Implemented

- Corpus mining, mutation ingestion, lineage-safe split/leakage checks, record/rules
  batch execution and paired lineage-bootstrap evaluation from the imported lane.
- Trusted snapshot test/config bytes captured before the driver; missing evidence,
  ambiguous test identities, changed test modules/dedicated test files, changed
  protected configuration, links and nonregular files fail closed. CAS reads verify
  the raw content digest, including mutation diffs. Candidate code never restores tests.
- Trusted provisioning is a separate Runner method; evaluated argv cannot invoke a
  host provisioning shortcut. Sealed workers use a preloaded digest-pinned image,
  nonroot uid:gid, no network, read-only root, dropped capabilities, resource limits,
  bounded output/time, and only a disposable copy mounted. No secret/store/socket mount.
- `tsr select` records reproducible scope/lineage diffusion and abstention as hashed
  evidence. No stored weights, admission changes or policy authority are introduced.
- `tsr tensions --store ... --task ... --reviewed-links ... --operator-review
  --family ... --repository ...` validates explicit operator-reviewed criterion and
  candidate/forecast impact links. It records the reviewed input and derived trace as
  separate Operator/Rules evidence. Unknown targets, stale state and unselected/closed
  preferences are refused; unsupported conflicts abstain. The operator must explicitly
  assert open criteria: free text and missing evidence are never inferred conflicts.
- Provider-neutral `tsr-broker@v1`: inspect, allowlisted-test-id, bounded replacement,
  or stop proposals only. Controller checks fresh IDs/state, profile scope, budgets,
  forecasts and rules-mode required-read/test sequencing. Test argv comes only from
  the operator profile and executes through the worker. Generator authorship is kept
  in candidate/forecast payloads; outer `Source::Rules` identifies the event recorder.
  Observations/history are bounded untrusted data. Broker failure preserves accrued usage.
- CLI `--driver broker-replay` exercises the protocol without credentials/network.
  `broker-contexts.json` maps task IDs to trusted `required_reads` and
  `failing_test_id`; `broker-replay.json` contains `[proposal, forecast] pairs`.
- Optional `--driver cheaper-inference` uses a fixed trusted Python adapter, never
  model-supplied argv. It requires explicit live authorization and absolute controller
  Python/script/shared cost-journal paths. The adapter uses one fixed inexpensive model,
  caps request/output, refuses redirects and implicit retries, reserves $1 before each
  dispatch against a shared $100 ceiling, retains uncertain reservations, and reconciles
  exact settled billing. Out-of-bound billing permanently halts the journal pending
  operator reconciliation. These are local safeguards, not provider-side account limits.
  No secret or request/response body is logged. A harmless compatibility probe succeeded;
  paired task execution is not live-validated.
- Explicit preparation profiles pin generated lock bytes, source trees, stable repository
  identity and immutable image. Per-tree history locks are supported. Native mine,
  mutants and batch use the same controller-owned overlay and optional sealed prerequisite
  build; full-suite argv cannot be narrowed by a profile. Raw command evidence is retained.
- Independent `corpus qualify` separates caught candidates from admitted mutants. It
  requires full baseline green, completed named and full red, exact reversal, named and
  full green, unchanged source/test/configuration evidence and single-function deduplication.
  Patch scope excludes additional files even under build directories. Ordinary direct Rust
  file-module test names are checked against frozen trusted module declarations.
- M4 bundle verification now uses independent RAII temporary directories, fixing the
  concurrent PID/content-name collision. A concurrent regression covers that defect.

## Validation

Generated `m3-evidence.md` and raw logs in `m3-validation/` distinguish successful
runs from earlier failed repair iterations. CLI fixture migration uses real bounded
Git snapshots and inert workers that emit synthetic test evidence; no candidate Rust
or arbitrary candidate argv runs on the host. Replay/tensions CLI wiring, oracle
matrix, CAS corruption, sequencing, usage preservation, process timeout and billing
cap/refusal/halt tests are covered. Independent review is recorded in `m3-review.md`.

Earlier workspace verification checkpoint: **696 passed, 0 failed, 5 ignored, 2 filtered**.
The focused admission/full-suite-integrity library run passed **20 tests** (overlapping count).
The actual-outcome parser regression is included in the separately passing
mutation test target (**29 passed, 0 failed, 1 ignored**; overlapping counts). Serial library verification: **271
passed, 0 failed, 1 ignored** (overlaps workspace counts). Five Python offline
checks pass; workspace build, Rust formatting and all-target Clippy with warnings
denied pass on Rust 1.90.

Two documented M0 tests are explicitly filtered from the workspace run because their
runtime-detection behavior is unrelated to M3 and would invoke the old executor:
`exec_resolves_the_session_task_and_refuses_without_one` and
`selftest_is_reproducible_structurally_excluding_duration_ms_by_name`.
Frozen M0 test/source files were not modified. Existing opt-in tests remain ignored;
the prior explicitly approved real Docker bounded smoke passed separately.
Do not describe this as an unqualified full-suite or real-corpus pass.

## Operational evidence and blockers

The approved local rts history at `308a5e74298254e4282c5c2952259be3bbfe62b2`
has 199 first-parent commits, 198 nonroot comparisons, 144 touching Rust and 106
adding test-attribute lines. These are inventory counts, not red-to-green tasks.
`m3-history-inventory.json` records the candidates. The initial independently admitted task is described below; see the current admission index for all subsequent results.

The first nonroot, offline, sealed HEAD probe failed on missing `anyhow`. Public
registry dependencies were subsequently vendored in a disposable clone, then copied
into a local image without running repository code during the build. A second sealed
probe with the original source snapshot and `cargo test --locked --offline -p rts-mcp --lib`
failed because no lock file exists in that Git snapshot. The earlier claim that a
committed lock needed updating was incorrect: Cargo.lock is ignored, and no lock
file appears in the inspected history. Evidence is in
`m3-history-probe.json` and `m3-history-dependency-probe.json`. The original repository
was not changed. A generated lock is now explicitly preserved as `m3-prepared-Cargo.lock`, with its
SHA-256 and preparation provenance in `m3-dependency-preparation.json`. This is an
evaluation dependency preparation, not an original historical artifact.
The prepared HEAD snapshot first passed 54 rts-mcp library tests. The subsequent
full-suite environment failures are preserved in the probe JSON files. They were
resolved without changing source assertions or excluding tests: preload the pinned
nightly toolchain; set private HOME/XDG directories and RTS_DAEMON_BIN; serialize
Cargo/tests; permit Rustdoc's generated harnesses on container-local /tmp; and use
Docker --init to reap orphaned test descendants within the unchanged 256-PID limit.
`m3-init-full.json` records **999 passed, 0 failed, 15 ignored**, exit 0,
untruncated, with the same prepared lock hash. This is actual repository validation,
not 999 corpus tasks. Production evaluation must reproduce the recorded environment.

The prespecified historical writer pair was tested three times per revision with
unchanged source and a separately recorded common generated lock. All six targeted
runs passed; it supplies no failing-parent evidence and is not admitted. The fix also
changes test synchronization, independently preventing a casual red/green inference.
See `m3-writer-pair-plan.json` and its six result records.

The bounded one-mutation pilot uses cargo-mutants 24.11.2 on real rts-mcp source.
Its 22 listed descriptors all omit names; this exposed and repaired name-key collisions,
missing `--diff`, and unsafe whole-line diff synthesis in ingestion. Structural keys
now include full spans, and duplicate list/outcome identities fail closed. Default
mutation runs ignore repository tool configuration and use the already-disposable
copy in place, avoiding a second build tree on the bounded temporary filesystem. The captured
real-tool fixture exercises these shapes. The initial CLI argument failures are kept
separate from the successful single-mutant run. Independent full-suite mutation and
reversal verification are recorded separately; tool-reported caught status alone is
not admission evidence. The independent full red attempt reached its 360-second
deadline (613 passed, 10 failed, 12 ignored in completed groups), so it is incomplete.
A separate independent library run reproduced the three named failures (51 passed,
3 failed, exit 101). Exact reversal restored every tracked byte, and the full suite
then passed again: **999 passed, 0 failed, 15 ignored**, exit 0 in 294.6 seconds.
This establishes one bounded mutation pilot, not a production corpus admission.
`m3-corpus-progress.json` distinguishes this unadmitted surface pilot from the later
core admission and records zero paired provider runs.
The reproducible preparation profile is now `m3-preparations/rts-head-v2/preparation.json`.
Its predecessor exposed a fresh-copy missing-daemon prerequisite; v2 explicitly builds
that prerequisite in the sealed worker. A fresh native pilot now emits one candidate.
Eleven separate varied screens emitted nine caught candidates across six families;
the selection covered seven families, but one family supplied no caught candidate.
All ten candidates (pilot plus cohort) share **one repository lineage**. Screening
is not admission; the first core admission is described below and subsequent admissions are indexed separately. Native descriptors (5,623) and distinct function
groups (1,393) are inventory only; no 150-task claim follows from these counts.
The earlier 600-second full-red retry also timed out and remains censored.

The operational admission protocol retains completed full-red evidence. PRD B9 at
line 414 says “each mutant killed by the suite becomes a task”; it does not explicitly
specify full-red completion. A narrower interpretation was considered but superseded
before any acceptance claim, preserving the stronger existing protocol. Both initial
native admission attempts were refused before testing because external file-module
identities were unsupported. That checker limitation has been repaired and regression
covered. The strict retry admitted task `223a50b42faf1dfecba3f358329b0fbc44e232c8b2641249077cc7eb48f6ef38`:
full baseline **999/0/15** (passed/failed/ignored), three completed named failures,
full mutated suite **995/4/15**, completed exit 101 in 268.9 seconds, exact restoration
of **478 tracked files**, three named passing tests, then full green **999/0/15** in
287.5 seconds. Its family is `(BinaryOperator, rust_tree_sitter)` and it shares the
same single repository lineage. These are real-source test outcomes, not generator
performance or paired systems-1/2 outcomes. Raw CAS, phase records, accepted manifest
and admission protocol are under `m3-native-admission/m3-qualified-core-v2/`.

The fourth full-red failure (`read_handlers_round_trip`) also exposed a batch-oracle
gap: unlisted tests could be edited by a later repair session. The checker now freezes
all snapshot Rust test regions, test-module reachability and dedicated tests/ files
and resources, with checks before and after every oracle command. An inert regression
verifies unlisted-test changes/deletions and tampering during named/full execution;
production bodies outside protected regions remain editable. See the runbook and
generated native evidence for exact preparation, source, image and task identities.

A harmless single Stop request to
`https://api.cheaperinference.com/v1/chat/completions` using `deepseek-v4.1-flash`
was rejected by automatic approval review before execution. The review cited the
trusted prohibition on paid model calls and did not accept quoted later approval as
trusted authorization. A single retry after newly supplied delegated authorization evidence was also
rejected before process creation: the reviewer required a trusted user message or
authenticated approval. Exact request and rejection are in `m3-approval-retry.json`.
Those earlier denied attempts created no call or charge. A later fresh approval passed review for the harmless Stop-only probe, which settled at $0.000112 (667 tokens). The subsequent two-task paired pilot was separately denied before process creation because the reviewer did not recognize authorization for source/test-data disclosure. See `m3-approval-resolution.json` and `m3-live-pilot-blocker.json`. No paired calls executed. At the owner’s explicit request, the normal approval mechanism was reissued once and automatically denied again without an actionable approval prompt or ID; see [reissue result](m3-approval-reissue.json). No further retry or alternate route is being attempted. Owner approval was renewed and is acknowledged; the remaining block is the execution-review decision, not a request for another verbal approval. The [mechanism inspection](m3-approval-mechanism-check.json) found no exposed pending approval ID or verified HTTPS approval continuation for iOS. Read-only catalog metadata is preserved
in `m3-provider-catalog.json`; catalog availability does not prove tool-call support.
Provider behavior and accounting claims follow the official
[CheaperInference documentation](https://www.cheaperinference.com/docs), but require
live verification once the execution-review block is resolved.

## Acceptance

**M3 acceptance is not met.** PRD B11 requires at least 150 qualifying tasks with
family/lineage, real systems 1/2 baseline results and intervals, and logged selection
and tension steps. The local implementation now provides those pipeline/recording
interfaces, but synthetic tests cannot supply the missing corpus and outcome evidence.
M4 additionally needs 150 resolved fit/calibrate tasks and at least 60 held-out tasks
for its first learned release; a 150-task total is insufficient for both requirements.

The oracle is not a semantic verifier of arbitrary hostile Rust: external helpers,
macros and fabricated test output can affect meaning. Freezing test files/modules
reduces tampering but does not replace independent hidden checks and operator review.
The prior bounded worker smoke is not a full isolation/cancellation certification.
No escape payloads, unrelated private sessions or private logs were used.
