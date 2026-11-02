# M3 qualification throughput

Measurements are generated from completed, independently admitted cases in
[m3-throughput.json](m3-throughput.json). They are summed sealed-command durations,
not wall-clock delivery promises. The sample is small and purposively selected.

At the seven-admission checkpoint, AgentID's two cases cost 7.61–7.63 command minutes
each. The five code-utility cases cost 20.77–26.46 minutes (median 22.50).
AgentID's two-case qualification job took 15m51s wall clock, compared with 15m14s of
command time: about 37 seconds of combined copying, hashing and controller overhead.
The separate two-mutation screen took 2m10s. The later Python-rendering code-utility case took 26m22s wall time, with 25m01s in sealed commands and 81 seconds of other overhead. Interrupted earlier attempts are preserved
but excluded from accepted-case timings; they still consumed real machine time.

The principal code-utility cost is the repeated whole workspace suite. Baselines cost
8.2–9.1 minutes including remaining compilation, prerequisites another 2.1–3.0 minutes,
and each warm full-red/full-green phase about 4.3–5.3 minutes. Rust test-harness
summaries alone account for roughly 254–337 seconds per full suite. AgentID's warm
full suites cost about 39 seconds, with 35–37 seconds in test harnesses; cold baseline
compilation dominates its 4.1–4.3 minute baseline. Named checks additionally rebuild
changed crates once for the mutation and once for reversal.

With two continuously occupied workers and 8–28 minutes per attempt, 143 additional
admissions would require approximately **10–34 hours of qualification alone if every
attempt qualified**, or **19–67 hours if half qualified at similar per-attempt cost**.
These are conditional planning scenarios, not estimated probabilities or a promise.
They exclude candidate discovery, screening, setup, interruptions, rejected candidates
with different costs, and real paired model/test runs. The existing candidate inventory
is not evidence that 143 additional independent admissible tasks exist.

## Throughput changes considered

- Vendored dependencies and toolchains are already baked into digest-pinned read-only
  images. This avoids downloads and registry mutation, but does not avoid compilation.
- The most direct next experiment is a trusted, immutable dependency-only compiled
  cache keyed by source/dependency locks, toolchain, target, features, flags, environment
  and preparation digest. Seed a separate writable copy for each case; never allow a
  candidate to modify a shared cache or promote its artifacts into the trusted seed.
  Keep all fresh baseline/red/reversal/green executions and integrity checks. Exclude
  workspace executables, procedural macros and build-script outputs unless their
  provenance and invalidation are independently established. This is a proposal, not
  an implemented or measured speedup. Path-sensitive Cargo fingerprints may reduce it.
- Reusing a completed baseline verdict is **not supported by the current protocol**:
  `admission.rs` explicitly verifies original bytes, executes baseline, then verifies
  them again for each disposable copy. No baseline was reused. Any future certificate
  design needs a separately reviewed protocol version binding the exact source tree,
  preparation/image/lock, suite, environment and oracle version, with clean per-case
  integrity checks. It must retain completed mutant-full and reversed-full runs.
- Do not remove slow integration tests, skip named checks, shorten the suite, or turn
  timeout/interruption into admission. Do not raise worker concurrency without measuring
  Docker/host CPU, memory, disk and test-port headroom. This cohort remains at two.
- The detached finite queue eliminated the observed terminal-session interruption
  mechanism in the current run; this improves completion reliability without changing
  qualification criteria. Heartbeats and bounded cleanup make recovery inspectable.

Disposable copies also need an explicit retention policy: measured completed code-utility copies are about 5 GiB each. At 150 such cases this could exceed the approximately 500 GiB currently free. Preserve manifests, immutable preparation, raw CAS, phase records, admission decisions and source revisions before removing only completed task-owned disposable source/build copies. Never remove active copies or original checkouts. No cleanup is needed for this small cohort yet.

A larger stratified screen across eligible functions is needed before estimating yield.
The next corpus expansion should choose independent functions across families and
lineages, record all rejected/censored attempts, and use the resulting yield and cost
distribution to revise these planning ranges. Real system comparisons remain a separate
cost and evidence requirement.
