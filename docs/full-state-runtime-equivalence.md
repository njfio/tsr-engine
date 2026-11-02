# Full-state runtime equivalence: verified for the completed checkpoint

This offline addendum closes the specific verification gap recorded in
`continuous-game-live-v1-continuation.md` and
`continuous-development-effective-prompt-audit.md`. Those historical reports and
their evidence remain unchanged. This is not another model trial.

Both preserved binaries constructed the complete native `project cycle request`
from identical immutable copies of the completed eighteen-bundle checkpoint.
Each copy occupied the **same canonical directory**, avoiding any difference in
directory-derived inspection handles. No output fields were removed, rewritten or
normalized. The offline debug process was allowed to finish; production's
120-second subprocess deadline was not changed.

| Runtime | Elapsed | Exit | Complete output SHA-256 |
|---|---:|---:|---|
| Preserved debug | 136.714690s | 0 | `2636b3b17cf387369ce97f813f071d50150f13d804b5b5594bcfe5bd1be382db` |
| Optimized | 19.227039s | 0 | `2636b3b17cf387369ce97f813f071d50150f13d804b5b5594bcfe5bd1be382db` |

**The entire outputs are byte-identical and parsed JSON is equal.** Neither copy
changed any file, including derived shadow state. Original native files, all
tracked adapter/cycle journals, both campaign ledgers and all 39 files in the
completed outcome manifest remained unchanged. No provider calls, new admissions,
artifact executions, accounting changes or runtime installation occurred.

The debug binary is pinned to `2ff8ffa7e19a769eecbfdcd5fff6fad5e3e992150aa22d6c756020a09c1ca72d`;
optimized is `822c5b8fcbb2f0965296e9c1170008856d8618eb36dd8198904eafb74911ef3b`.
They are the previously preserved builds. The optimized build provenance records
Rust 1.90.0, the same Rust source/lockfile, and explicitly enabled debug assertions
and overflow checks. No rebuild with different flags was substituted.

Raw outputs, input fingerprints, per-profile file maps and terminal result are
under `target/full-state-runtime-equivalence/`. The reproducible offline driver is
`experiments/continuous-game-resume-v1/full-state-equivalence.py`; it creates a new
output directory exclusively and never reuses or rewrites completed evidence.

Coverage is **one completed large state and one request-construction operation**.
This is not universal equivalence of all operations, numerical behavior or future
histories. Sequential debug-first execution permits OS-cache effects; timings are
observations, not a controlled cold-cache speedup benchmark. No redundant
recomputation was isolated by this comparison, so no speculative caching or
validation refactor was made. Independent method and outcome review are recorded
with the retained evidence. The requested checkpoint equivalence is verified;
broader equivalence would require separate coverage.
