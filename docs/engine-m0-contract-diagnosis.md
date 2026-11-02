# M0 contracts and revision continuity after a05a22d

2026-10-22. Offline diagnosis; no inference, downloads, protected edits or hook changes.
The prior normal-environment Rust sweep remains **966 passed, 2 failed, 11 ignored**.
The controlled comparisons below do not replace that result.

## Exact failures

`m0_forecast_default::exec_resolves_the_session_task_and_refuses_without_one`
expects exit 3 and no execution events after opening a session, implicitly assuming
no container runtime. It does not isolate runtime discovery. `exec.rs::Runtime::detect`
accepts an installed binary answering `--version`; this Mac has Docker 27.3.1. The
retained test store contains task_opened, proposed, committed, dispatched and executed
(exit 127, 1230 ms). Session resolution worked. The test captured stderr as empty because
`RealRunner` combines container stderr into the returned execution log/stdout.

With supported per-command approval, a single diagnostic used the already-installed
default Debian image by immutable ID, no mounts/network/pulls, read-only root, dropped
capabilities, 64 MiB, one CPU, 32 PIDs and a 30-second process bound. `cargo test`
failed at OCI startup: `exec: "cargo": executable file not found in $PATH`, exit 127
in 0.867303 seconds. No project code ran. Image ID:
`sha256:eb593cf2c358cacef45ca0a424bbc7d30cfa3466265fc2662b9466a0ca6ba1c5`.
This explains the current prerequisite failure consistently with the retained receipt;
it does not recover the old raw log from its digest.

There is also a production classification gap: `RealRunner::run` treats successful
launch of the Docker CLI as a `RunOutput`, even when the container command never starts.
`run_with` therefore records Executed instead of the documented Unresolved startup
failure. A blanket exit-127 conversion would be wrong: a started command can itself
return 127. Correct repair needs reliable runtime-start evidence and tests distinguishing
those cases. `exec.rs` is protected; it was not edited.

`m0_harness::selftest_is_reproducible_structurally_excluding_duration_ms_by_name`
passes its two FakeRunner comparisons, then enters the real-runtime branch. Its second
`scratch("run1")` call removes the directory just populated by `run_selftest`, before
`structural_trace_of_fixture` reads it. The aggregate NotFound at line 195 follows that
deletion; the same mistake exists for run2. A compatible runtime cannot fix this.

## Controlled comparison and exact required exceptions

The two existing compiled tests were each run once with a child-only PATH containing
an empty directory. Both passed. The harness explicitly skipped its real-runner half;
this demonstrates the no-runtime contract and fake reproducibility, not real execution.

Minimal protected changes requiring explicit approval:

1. Isolate runtime discovery in the refusal test's child command; retain exit-3 and
   event assertions. Do not make production pretend an installed runtime is absent.
2. Save each harness scratch path once and reuse it for execution and fixture reading;
   retain every structural assertion and fixture.
3. Separately repair protected `exec.rs` startup classification using trustworthy
   lifecycle evidence, preserving genuine command exits and containment. This does not
   authorize changing the protected hook or its enforcement behavior.

Actual Cargo execution needs an explicitly admitted Rust-capable image/toolchain;
the default Debian image does not supply it. No image installation is proposed as a
way to make the refusal test green. These actions are **not attempted, awaiting the
specific repository-rule exception**, not denied by a tool. Initial sandbox socket
access failed; supported approval permitted only the diagnostic described above.

## Independent offline continuity coverage

PRD B6/B9 durable history and recovery require safe continuation after return review.
New `scripts/test_engine_revision_continuity.py` exercises the real controller with a
fake provider in fresh processes. Interrupted revision application before commit and
after possible commit remains uncertain on repeated restart, with no fifth call or
extra action. A durable revision receipt allows the final proposal-only request to
consume the exact current relational guidance and state root. Parents, authority and
single-action limits remain unchanged. No production defect was found in this path.

The fixture retains meaning with null proposed development. These checks establish
persistence/propagation, not model emergence, meaningful development or the exact B9.1
two-desires acceptance gate. Independent code review found no blocker and confirmed
that limit. An initial new-test failure omitted the explicit workshop context argument;
the new test was corrected, without changing production or existing tests.

Validation:

```sh
PYTHONPATH=scripts python3 -m unittest test_engine_revision_continuity \
  test_engine_relational_development test_engine_synthetic_cycle test_engine_workshop_cycle
```

**22 passed in 65.194 seconds.** Two focused new tests also passed separately.
Evidence: `target/m0-contract-diagnosis/{default-image-probe.json,
controlled-no-runtime.json,continuity-regressions.log,revision-tests-fixed.log}`;
prior aggregate: `target/reversal-diagnostics/fixed/workspace.log`.
No broad Rust rerun was needed for new Python tests/documentation; the two normal-
environment failures remain open. No claim of full-engine reliability follows.
