# RTS replay full-suite closeout

**Result: 999 passed, 0 failed, 15 ignored; no filtered tests.** The corrected
full-suite run exited 0 in 318,632 ms, after a 2,814 ms cached workspace build.
All 96 test groups and the complete test-name inventory match the prior successful
full-suite receipt. All 479 source-file hashes, the original repository status and
the exported patch hash remained unchanged.

This validates the exported patch from `9fa4705` for task
`80d827e3a4c15b244e211567a7bfe6679035d07736e6ea3db7c08c1cdc64f4ad`.
It is one local reference-correction replay, not model performance or a new cohort.

The base revision is `02cca9df15e72f6506275157b0198526950d6fcb`.
The project's AGENTS.md prescribes `cargo build --workspace` followed by
`cargo test --workspace`, because integration tests launch sibling binaries.
The admitted preparation adds locked/offline resolution, one build job,
`--no-fail-fast` and serial tests. A prior receipt recorded 144,644 ms for the
workspace build and 588,575 ms for the full suite. This closeout allows 1,200,000 ms
per stage; timeout or incomplete execution is inconclusive.

The complete Git base tree was copied into `target/engine-rts-full-suite-1/source`,
then the admitted mutation and the **same exported patch** were applied. The
existing prepared lock was added. The repaired implementation and all 56 previously
protected files matched the named-test replay. The resulting 479-file inventory
was hashed before execution. Original source and test files were not edited.

The pinned image is
`sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6`.
`rts_full_suite` uses the existing SealedRunner: no network, nonroot UID/GID
501:20, read-only root and source, capability drop, no new privileges, 2 CPUs,
2 GiB RAM and 256 PIDs. Private build output is mounted at `/tsr-build`;
it starts as a **copy** of the passing replay's artifacts, never a shared writable
cache. The existing 8 GiB/100,000-entry monitor remains active. Each stream has a
1 MiB capture limit. Runtime state and HOME use private `/tmp` directories;
`RTS_DAEMON_BIN=/tsr-build/debug/rts-daemon` points to the built sibling binary.
`CARGO_TARGET_DIR=/tsr-build` also directs nested rustdoc builds there, and
`CARGO_NET_OFFLINE=true` applies offline resolution to those child commands.

Commands inside the sealed worker:

```sh
cargo build --locked --offline --workspace -j 1 --target-dir /tsr-build
cargo test --locked --offline --workspace -j 1 --target-dir /tsr-build --no-fail-fast -- --test-threads=1
```

Controller commands:

```sh
cargo build -p tsr-corpus --example rts_full_suite --offline
target/debug/examples/rts_full_suite target/engine-rts-full-suite-1
target/debug/examples/rts_full_suite target/engine-rts-full-suite-1 retry-2
python3 scripts/verify-rts-full-suite.py target/engine-rts-full-suite-1 retry-2
```

The verifier checks the complete post-run source inventory, original repository
status and unchanged patch hash, then records counts, all test-group summaries,
exact argv, limits, timings and output hashes in
[the result](engine-rts-full-suite-result.json). Full stdout/stderr and the source
inventory remain in the local run directory.

The controller example passed clippy with `-D warnings` and rustfmt. No engine
policy, source correction, test assertion, image or Docker configuration was
changed. No paid calls, external source transmission, push or publication occurred.

The [first attempt](engine-rts-full-suite-first-attempt.json) built successfully
in 97,124 ms, then reported 997 passed, 2 failed and 15 ignored in 458,995 ms.
Both failures were `public_api` tests: nested rustdoc builds tried to write under
the read-only source tree. The nested command does not inherit the parent's
`--target-dir` argument. The scoped worker wrapper now exports `CARGO_TARGET_DIR`
instead of making source writable. The complete suite was rerun successfully with the same
source, patch and private compilation artifacts; first-attempt evidence is retained.
