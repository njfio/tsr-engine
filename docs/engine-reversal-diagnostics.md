# Mutation reversal diagnostics and hook boundary

This continues the unresolved aggregate failures recorded in
[the acceptance audit](engine-prd-acceptance-audit.md). Earlier results and protected
assertions remain unchanged. No inference, provider access or target-repository
execution is part of this investigation.

## Diagnostic contract

`mutants::reversal_diff` retains its public `Option<String>` interface. The corpus
construction path uses a checked internal result that identifies base materialization,
mutated materialization, patch directory/spawn/write/wait/exit, diff spawn/wait/exit
and empty diff. Errors identify base tree, mutation digest and scratch identity;
the mutation itself already resides in the corpus CAS. No source or patch body is
added to the diagnostic.

Patch stderr is drained concurrently, retaining only its first 4 KiB; formatted
failure details are bounded to 4,096 characters. A failed stdin write still reaps the
child. Existing patch acceptance and scratch cleanup rules are unchanged during the
initial reproduction. Subprocesses have no new internal timeout: the diagnostic
aggregate run has an explicit 1,200-second process-group bound. No timeout-extension
or successful-rerun substitution is permitted.

The initial four focused tests passed: valid nested-copy application, directory/patch failure
separation, bounded stderr drainage, and materialization failure classification.
Independent code review found no validation regression and noted the difference
between the byte bound on captured stderr and character bound on formatted details.

## Original hook enforcement boundary

`crates/tsr-cli/src/hook.rs::run` sends `PreToolUse` to `record_tool`, not to the
policy gate. `main.rs::run_hook` returns success on unreadable input, store errors
and recording failures. These are intentional recorder semantics, not proof of
pre-tool enforcement. `tsr policy check` is a separate entry point.

**Concrete risk:** installing this recorder does not prevent a direct host tool
call that violates a TSR policy, lacks a forecast, or bypasses `tsr exec`. Recording
such a call afterwards does not undo its effects. `tsr exec` containment applies
when that wrapper is actually invoked; the hook does not require it. Newer guarded
engine adapters have their own authorization path, which does not retroactively
guard arbitrary Claude tool calls.

A future enforcement change needs an explicitly authorized pre-tool gate path,
correct deny/ask propagation, forecast/identity binding, fail-closed handling and
compatibility coverage. It must preserve the recorder as such or explicitly migrate
its contract. Protected hook/exec files and their tests are not edited here.

## Reproduction and established defect

The diagnostic-only aggregate completed in 822.47 seconds: **965 passed, 2 failed,
11 ignored**. Only the baseline M0 failures recurred. This was not called a fix.
A bounded whole-mutant-target repetition stopped at its third run: after two passes,
`caught_mutants_become_tasks` failed with `ambiguous duplicate outcome identity`.
Its supplied ordinary outcome fixture does not contain that duplicate; another
concurrent test deliberately supplies the duplicate fixture. Retained run receipts
and surviving scratch contents are captured in
`target/reversal-diagnostics/contamination-evidence.json`. This is evidence consistent
with interference, not by itself proof of which shared path was used.

An atomic fail-closed allocation probe replaced the original remove-and-recreate
behavior without changing timestamp naming. Eight whole-target repetitions passed.
A direct 16-thread, 64-allocation-per-thread test then reproduced multiple **EEXIST**
errors with identical timestamp-derived paths. See
`target/reversal-diagnostics/concurrent-allocation.log`. The old implementation would
have removed and recreated each existing directory. Wall-clock nanoseconds were not
a uniqueness guarantee on this host. This establishes a real scratch ownership defect
that can delete, replace or mix another simultaneous reversal/kill-run workspace.
It does not recover the exact discarded stage of every historical reversal failure.

## Repair

Scratch allocation now combines a process-local atomic sequence with exclusive
`create_dir`. It never deletes or reuses a pre-existing candidate. Existing-path
collisions retry with a fresh sequence; other errors propagate; 128 unsuccessful
allocation attempts fail closed. The kill-run caller no longer removes its freshly
allocated directory before materialization. No new dependency, authority, runtime
setting, protected test change or acceptance relaxation is involved.

The 1,024-allocation regression failed before this correction and passed afterwards,
checking uniqueness and preservation of every ownership marker. All five focused
diagnostic/allocation tests passed. Two independent reviewers confirmed the repair
and the narrow causal qualification. Early-error cleanup of retained kill-run copies
is a separate pre-existing behavior; this change does not silently discard them.

## Final verification

- Five focused tests passed, including the previously failing 1,024-allocation test.
- Three whole-mutant-target runs: **29 passed, 0 failed, 1 ignored each**, 38.15
  seconds total including the first build. All outcomes were retained.
- One complete final workspace run: **966 passed, 2 failed, 11 ignored**, exit 101,
  1,003.96 seconds within its 1,200-second bound. Both previously failing mutant
  tests and the contamination test passed. Only the two documented, unchanged M0
  runtime-dependent failures remained. No clean-workspace claim is made.
- `cargo +1.90.0 clippy -p tsr-corpus --all-targets --offline -- -D warnings`
  passed (12.27 seconds). Rustfmt check on the changed source and `git diff --check` passed. Protected files
  and assertions are unchanged. No model/provider calls or target edits occurred.

Both bounded subprocess wrappers reached terminal status; the owned session closed.
No model runtime was started and no global process-list bypass was attempted.
All earlier failures remain in their original logs; passing repetitions do not erase
or retrospectively explain them.

Evidence under `target/reversal-diagnostics/` (local, not bundled in the commit):
`workspace.log` and `result.json` (diagnostic-only aggregate), `mutants-3.log`
(contamination), `collision-*.log` (eight fail-closed probe runs),
`concurrent-allocation.log` (actual EEXIST reproduction), `fixed-focused.log`,
`fixed/mutants-{1,2,3}.log`, and `fixed/workspace.log` plus `fixed/result.json`.

SHA-256 of the failing allocation log:
`183fea6546c9c49b7effc203912561d3a5e06c87d7e9b1cc61c75b04f86f6a90`.
SHA-256 of the final full workspace log:
`26781cf1cf3a00ab03c310f8f17cd59f30073bfaf93a176ca6436191a81ea31c`.

The demonstrated allocator race is repaired. The old logs still cannot establish
that it caused every historical reversal error; stage-specific diagnostics remain
available should another failure recur. This is an implementation/reliability repair,
not new evidence of semantic culmination or learned-controller superiority.
