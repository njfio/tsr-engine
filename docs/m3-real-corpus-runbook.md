# Reproducing the bounded real-repository checks

These are controller-run diagnostics, not a completed `tsr batch` evaluation or a
150-task corpus. The original repository remains read-only. Only disposable copies
are mounted into workers; no Docker socket, credentials, event store, or host home
is mounted. Source and tests never run directly on the Mac.

The source revision and generated dependency lock are recorded in
`m3-dependency-preparation.json`. Cargo.lock is absent from that Git history; use
`m3-prepared-Cargo.lock` as explicit preparation, never claim it was committed there.
Preload public dependencies and the pinned nightly in an image before testing;
network is disabled for every source run. Dockerfiles and immutable local image IDs
are recorded in the preparation JSON. The historical writer pair has its own
`m3-writer-prepared-Cargo.lock` and image recorded in `m3-writer-pair-plan.json`.

`scripts/m3-history-probe.py` takes an immutable local image-ID file, a disposable
copy, output JSON path, a deadline of at most 600 seconds, and an argv after `--`.
It enforces nonroot execution, no network, read-only root, dropped capabilities,
no-new-privileges, 256 PIDs, 2 GB RAM, 2 CPUs, one copy mount, and 1 MiB output.
`--tmp-exec` permits Rustdoc's generated executable on container-local tmpfs.
`--init` reaps orphaned subprocesses. These grant no extra host access.

The exact commands, environment, exit codes, output, deadline and lock hashes are
in each probe JSON. The full-suite command creates private HOME/XDG directories,
sets RTS_DAEMON_BIN to the snapshot's compiled daemon, disables incremental builds,
and runs `cargo test --locked --offline --workspace --no-fail-fast -j 1 -- --test-threads=1`.
The explicit ignored tests remain ignored. No additional test was excluded.

The mutation pilot is prespecified in `m3-mutation-plan.json`: one localized
`Surface::keeps` mutation. Cargo-mutants 24.11.2 needs `--no-config --in-place`,
and listing needs `--json --diff`. Do not combine `--in-place` with `--jobs`.
Its trailing arguments are Cargo arguments, so passing harness arguments requires
`-- -- --test-threads=1` at the cargo-mutants level. Initial argument failures are
preserved and do not count as evaluated mutations.

`m3-mutant-capture/` preserves the tool's actual list, outcomes, logs and diff outside
the candidate mount. `source-manifest.json` hashes every tracked original file.
The independent controller verified every other tracked byte before running the
mutation, then restored the exact original file and verified the entire manifest.
The separate targeted red run is an ordinary Cargo invocation, not reliance on the
mutation tool's self-reported label. Full-suite baseline, incomplete full red,
targeted red and reversal records are distinct.

Operational limits: these probes have not been imported into a production task
manifest or a systems-1/2 paired baseline. The generic CLI still needs a reproducible
operator preparation/profile for this lockless repository, including vendored
image, lock insertion and environment setup. Do not extrapolate from a successful
prepared probe to generic `cargo test` in a fresh snapshot. Task IDs now include the
complete mutation span; regenerate affected manifests instead of mixing old IDs.
No paid provider call ran; the exact approval rejection is preserved separately.

## Reproducible lock preparation and independent admission

`tsr corpus prepare --request request.json --out preparation-dir` records an already
resolved lock and its provenance; it performs no dependency resolution. The request
pins the exact source object, stable repository identity, immutable image, generated
lock path and generation command. Historical parent/fix trees can use separate
`additional_snapshots` lock records. `rust_test_environment` supplies private worker
HOME/XDG directories; `build_workspace` builds the daemon prerequisite inside the
worker before tests. The only admitted full-suite argv is `cargo test --locked
--offline --workspace --no-fail-fast -j 1`; preparation cannot narrow it to a package,
library or filter. Set `TSR_M3_PREPARATION` to the resulting `preparation.json` along
with the usual immutable image/runtime/numeric-user variables for mine, mutants and
batch. The profile identity is bound into task IDs. Lock evidence stays controller-owned.

`tsr corpus mutants` accepts `--package`, `--file`, `--mutant` and optional
`--screen-target lib|bins` to bound initial screening. These produce **candidates**,
not admitted tasks. CLI and native run stdout/stderr are captured in verified CAS.

`tsr corpus qualify --tasks candidate-dir/tasks.jsonl --out new-admission-dir`
uses a fresh source copy and preparation. It requires a completed full green baseline,
unchanged test/configuration evidence, one production function body mutation,
completed exact named failures, a completed full mutated suite (exit 101), exact
reversal, named passing tests, and a completed full green suite. Every tracked byte
is checked after the final run; unexpected source file additions are rejected. One
function group is accepted once per cohort. Timeouts do not qualify. Copies and raw
run CAS remain under each task for inspection; only accepted tasks enter the output
manifest. This bounded mutation-admission path does not yet admit historical tasks.

The full-red gate preserves the operational protocol used for this work. PRD B9
(`PRD.md:414`) says “each mutant killed by the suite becomes a task”; it does not
explicitly resolve whether a completed named-test failure alone suffices. An earlier
narrower experiment was therefore superseded and is excluded from acceptance. Do
not reinterpret a timeout or a tool-reported kill as completing the full-red gate.

The current native inventory contains 5,623 descriptors and 1,393 function groups,
not 5,623 tasks. Eleven deliberately varied screens yielded nine caught candidates
in six families, from one repository lineage. A separate pilot adds one caught
candidate. `m3-native-screen/summary.json` records these distinctions. More source
lineages and independently admitted tasks are still needed for meaningful lineage
holdouts and the 150-task M3 criterion. No generator or paired-system results are
implied by known-reversal qualification.

The strict native core attempt completed and admitted one task:
`m3-native-admission/m3-qualified-core-v2/tasks.jsonl`. Full red completed with
995 passed / 4 failed / 15 ignored; exact reversal restored 478 tracked files and
full green returned 999 / 0 / 15. The other nine screened candidates are not admitted.
The batch oracle now freezes all snapshot test regions and dedicated test resources,
including tests absent from the task's named-failure list, and checks again after each
oracle command. Macro-generated test semantics and arbitrary production helpers are
not generally verified. This remains a bounded repair-test oracle, not a semantic
proof of arbitrary Rust behavior.


## Split evidence integrity (M3-SPLIT-1)

`corpus split` requires a valid, present, hash-verified, nonempty UTF-8 diff CAS object
for every task and rejects duplicate task IDs. Missing or corrupt evidence refuses
before writing a new split manifest; an existing manifest from an earlier invocation
is not evidence that the failed invocation succeeded. Grouping and snapshot probes
reuse the same captured, verified content.

Git tree IDs and descriptor-derived mutant snapshot IDs are legitimate identities,
not necessarily snapshot-content objects. Where content is absent, the snapshot
leakage probe reports `limited`; this is not a clean content-leakage check. A present
snapshot CAS object must verify. The optional task JSON field `snapshot_cas` explicitly
claims UTF-8 snapshot content and must resolve and verify; both that digest and the
original snapshot identity remain checked. Content hashing verifies the supplied
bytes, not their completeness or independent provenance as a repository snapshot.

The v5 exploratory report corrects earlier diagnostics that called identity-only
snapshot checks clean. It is not an evaluation holdout or a generator performance result.

After each admission milestone and before final evaluation, run
`python3 -B scripts/m3-audit-admissions.py --out target/m3-admitted-integrity-<new-checkpoint>`.
The output directory must be new. This preserves original exported records, refuses
duplicate task/function records, cross-checks original strict admission reports, and
executes a verified copy of the repaired split checker. The checker hash is pinned to
the v5 diagnostic; review an intentional checker update before changing that anchor.
This validates split inputs, not all qualification-run evidence. The 26-admission
script validation passed without invalidations; snapshot-content coverage remains limited.
