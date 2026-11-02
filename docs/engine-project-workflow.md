# One project entry point

For the current end-to-end CLI path, use the [ordinary operator workflow](engine-operator-workflow.md),
including local model generation, bound review drafts, correction, resume and episode carryover.

Use `tsr engine --directory RUN project COMMAND`. This groups the existing engine
operations without introducing another runner or budget ledger. Old command names
continue to work. Help is available with `tsr engine project --help`.

```sh
RUN=$(mktemp -d "$PWD/target/project-XXXXXX")
target/debug/tsr engine --directory "$RUN" project init \
  --spec examples/engine-cargo/scope.json --source examples/engine-cargo
target/debug/tsr engine --directory "$RUN" project prepare
target/debug/tsr engine --directory "$RUN" project status
target/debug/tsr engine --directory "$RUN" project request > "$RUN/request.json"
# Author a typed response against this exact request and its source handles.
target/debug/tsr engine --directory "$RUN" project run --response "$RUN/response.json"
target/debug/tsr engine --directory "$RUN" project status
target/debug/tsr engine --directory "$RUN" project export
```

A local `run --response` evaluates and commits one bounded proposal, then invokes
the existing sealed execution path. It makes no provider call. Use another fresh
`request` and response for a corrective repair. No arbitrary argv is admitted by
the response.

`project resume` executes an already committed but unattempted local action. If
there is no pending action, it reports status without execution. If a durable
execution intent exists without a completed return, it refuses to repeat the
attempt and asks for receipt reconciliation. It does not reinterpret an uncertain
attempt as a fresh one.

`project status` reads readiness, protected-file integrity, pending execution,
completed-outcome count and the latest receipt. It distinguishes dependency
preparation required, ready for a proposal, committed/unexecuted, recovery needed,
failed tests, passed and stopped execution. If admitted editable files change
after the saved execution, it reports `source_changed_since_execution` instead of
presenting the old pass as current. Status never dispatches work or calls a model.
The provider transport journal/ledger remain authoritative for transport budgets;
status does not infer charges from coding outcomes.

## Existing preparation and provider transport

When a locked crate is absent, the same entry point delegates to the existing
[separate image helper](engine-dependency-download.md):

```sh
target/debug/tsr engine --directory "$RUN" project prepare-image --out "$PWD/target/new-prepared-image"
```

The output path must be new and outside the existing engine directory. The helper
outputs a prepared spec for a **new** run; it does not alter the current admission.
Initialize that run with `project init --spec PREPARED/prepared-spec.json`, then
`project prepare`. No automatic dependency download occurs during repair.

To use the existing budgeted transport, forward its explicit arguments after
`--`; the selected engine directory cannot be overridden:

```sh
target/debug/tsr engine --directory "$RUN" project run -- \
  --local-provider http://127.0.0.1:PORT/v1/ \
  --fixture-ledger "$RUN/local-cost.jsonl" \
  --max-steps 2 --max-usd 1 --max-tokens 10000 --steps-this-invocation 1
# With the same provider and overall budget configuration:
target/debug/tsr engine --directory "$RUN" project resume -- \
  --local-provider http://127.0.0.1:PORT/v1/ \
  --fixture-ledger "$RUN/local-cost.jsonl" \
  --max-steps 2 --max-usd 1 --max-tokens 10000
```

The loopback server must already exist. These commands wrap `engine-run.py`,
retaining its ledger, journal, uncertain-charge stops and budget checks. The same
explicit live flags remain available when separately authorized; no live mode is
enabled by default. A run without a local response or explicit transport arguments
is refused. Python helpers are located in the checkout used to build the binary;
this slice is a checkout workflow, not a standalone packaged installation.

## Real admitted-task fit: blocked, not replaced by synthetic evidence

The existing admitted corpus has 151 tasks from two repository lineages. I checked
one admitted task from each against the unchanged project-scope limits, using
unchanged Git-base files and their existing controller-owned prepared locks in
disposable directories. Both imports were refused before execution:

| Admitted task | Required lock | Locked packages | Inline oracle/source file |
| --- | ---: | ---: | ---: |
| agentid `29b4ceb35e05…` | 99,810 bytes | 404 | `validation.rs`: 20,086 bytes |
| RTS `80d827e3a4c1…` | 84,080 bytes | 348 | `lib.rs`: 13,454 bytes |

At that checkpoint the adapter allowed 4096 bytes per file, 8192 source bytes total and 32 lock
packages. [Subsequent bounded-window support](engine-protected-windows.md) removes
the large protected-asset and inline-byte-protection blockers; workspace import
and execution support remain unfinished. Both repositories also use workspace inheritance; agentid additionally
uses dependency features. At that checkpoint, whole-file edit permissions could not keep
inline tests protected while editing implementation in that same file. These
requirements apply to the repositories' preserved workspace manifests and locks,
so selecting another task does not remove the lockfile/manifest blockers.

[Concrete fit evidence](engine-admitted-task-fit.json) records exact task IDs,
Git snapshots, preparation lock identities, sizes and CLI errors. The import
checks use necessary files to demonstrate the size blocker, not a claim that the
partial file lists form runnable projects. They do not apply mutations, execute
project tests or count as real repair outcomes. Neither source repository changed.

The narrow gap addressed here is support for namespaced Rust test IDs such as
`validation::tests::valid_e164`; unsafe/empty namespace segments remain refused.
Completing a genuine corpus integration still requires workspace dependency
inheritance/features, a larger explicit source graph and compatible bounded
build-cache/named-test execution. Larger protected assets and inline-byte windows
are now implemented as documented in the linked follow-up. Merely raising byte limits or extracting a
function into a new synthetic harness would not preserve the requested real-task
scope. No permission denial occurred: this is an implementation capability block.

## Checks performed

- `cargo test -p tsr-cli --lib --test engine_cli --offline`: 51 library tests and
  two CLI integration tests passed.
- `cargo clippy -p tsr-cli --all-targets --offline -- -D warnings`: passed.
- Unified local CLI sealed Cargo regression: passed. It resumed an unattempted
  commitment, recorded failed tests, restarted with feedback, executed a corrective
  `project run --response`, passed four tests, reported current status, and exported
  the patch. Protected files stayed unchanged and there were zero provider calls.
- Forwarded transport budget regression: passed, stopping before network access.
- Both real admitted-task imports: refused with `project file type/size changed`.
  **Zero real-task tests executed.**

[CLI regression evidence](engine-project-entry-evidence.json) is explicitly labeled
synthetic. It verifies wiring only and does not substitute for the blocked real
integration. No paid calls, external source transmission, Docker restart, cleanup,
push, PR, merge or deployment occurred.

## Independent goal acceptance

Passing tests leave a project `tests_passed_unaccepted` until an operator independently
accepts the current source and evidence. See the [acceptance workflow](engine-project-acceptance.md)
for requirements, review, stale-evidence handling and export status.

Continue beyond a rejection with [persistent feedback and episode carryover](engine-continuous-cycle.md).
This connects independent requirements and review to preferences, frozen semantic expectations,
fresh generated repairs and later episodes while preserving execution and spending bounds.
