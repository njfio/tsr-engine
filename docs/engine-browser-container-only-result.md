# Browser integration under the approved container-only exception

The pinned Chromium capability probe and real protected browser tests passed.
Chromium's inner sandbox is disabled explicitly; the existing outer container
controls are retained. No host security setting or dependency changed in this
step. No provider call or engine-generated game occurred.

Implementation:

- `ProjectSpec.browser_container_only` defaults false, is operator-owned and
  source/root-bound, and adds only the protected `--container-only` harness flag.
- The existing sealed native runner performs exact multi-file edits, then runs
  Node/harness verification with read-only workspace. No application JavaScript
  executes as Node code on the host.
- `engine-browser-adapter.py` reuses the existing native cycle, JSON output mode,
  strict validation, retained diagnostics, counted corrections, owned semantic
  review, campaign accounting and uncertain-effect stop.
- Public projection verifies protected identity, source lineage, receipt and
  relationship ownership and byte limits. A fresh independently reviewed public
  checkpoint can explicitly include prior source/receipt/relationship digests;
  it does not implicitly trust arbitrary native history.
- Initial public wants are published via existing native guidance; the packet
  asserts both are active. Real later feedback uses attributed native intake.
  Authored feedback tests retain contributor identities, immutable requirements
  and ledger accounting. A fresh session never reopens a terminal allowance.
- The existing `engine-native-cycle.py --browser-v1 check/run` entry point requires
  exact new admissions. The prospective browser scope grant keeps applied caps
  at 272 calls / 74 groups / $5; it has **not** been applied.

## Verification

Completed affected checks:

- `rustup run 1.90.0 cargo test -p tsr-cli --lib browser --offline`: 4 passed.
- Explicit sealed rerun of `browser_native_multifile_feedback_restart_and_stale_controls`:
  passed with actual browser receipt, two edited files, protected checks, stale
  rejection, retained guidance and restart. Not model-driven behavior.
- `node scripts/test_engine_browser_harness.cjs`: authored controls passed.
- `TSR_BROWSER_CONTAINER_AUTHORED=1 python3 scripts/test_engine_browser_container.py -v`:
  one test/four real browser cases passed (click/key, external script denial,
  protected probe denial, application-error failure), protected hashes unchanged.
- Projection tests with the native request: 5 passed, no skip.
- Final `test_engine_browser_adapter.py` with authored transport and sealed worker:
  5 passed in 53.467s. Covers actual CLI preflight, full shared cycle, inert replay,
  one counted malformed-then-valid correction, foreign review/receipt/relationship
  and stale-source rejection, unknown-bill stop, unchanged cap scope grant, and
  explicit reviewed checkpoint after attributed authored feedback.
- Campaign tests: 13 passed, 1 skipped (unrelated explicit Rust fake-worker binary
  was not supplied). Output release tests: 7 passed.
- `git diff --check`: passed. Full workspace suite was not rerun.

Local failures were retained as diagnosis, not live results: an initial test used
an incompatible Rust digest helper; a sealed test's relative evidence path failed
under Cargo's working directory; independent review caught a missing launcher
module export; a fixture admission lacked whole-envelope deadline headroom; and
checkpoint testing exposed missing initial native guidance. These were corrected
without changing production deadlines or acceptance. Final affected tests passed.

Independent read-only reviews found no remaining material defect. The final
continuity review explicitly notes that checkpoint import depends on independent
review of exact public baseline evidence, not automatic cross-journal ancestry.

Evidence: `experiments/browser-container-only-v1/` contains the exact capability
command/output, actual browser harness reports, final adapter test log, typed
native receipt summary and unchanged campaign snapshot. Old sandbox failure
`74c03bba` and original preparation evidence remain unchanged.

## Prospective game boundary

`experiments/browser-game-v1/README.md` and `manifest.json` define the initial
10-call/two-execution session, with two shared corrections, no provider retry,
120 minutes and $0.50 reservation. New game scope approval and fresh launch
preflight remain pending. No cap increase is needed from applied 272/74 limits.
Preparation uses a blank HTML/CSS/JS skeleton, not a finished authored game.

Actual player play, design quality and TSR relationship development are not proved
by smoke checks. Candidate assets can be delivered locally after independent review;
no upload, deployment or automatic host execution is included. Subsequent genuine
user changes require a fresh reviewed checkpoint and session admission. General
new-desire identity succession and causal TSR advantage remain unsupported/unproven.

Campaign is unchanged: **259 calls, 72 groups, $0.222740**, no unknown effects.
