# Receipt-fact parity and prospective guide opportunity evaluation

Local implementation/preparation only. No model API, target service, corpus read,
real campaign operation, or historical acceptance mutation occurred. Campaign stays
259 calls / 72 groups / $0.222740; original open-guide and continuation results remain
unchanged. Ledger SHA-256:
`b0b86fdc3f9cae48832212c21c800d0e2f8850186c3806f124f0d9e78a765d02`.

## Implemented behavior

`engine-objective-adapter.py` computes the pinned receipt checker facts in common
`project()` after exact owned receipt/source/root/schedule checks, before the
28,000-byte context limit. Ordinary proposals, initial relational proposals,
reconsideration, proposal reviews and round/final reviews receive identical facts.
This removes a projection asymmetry; it does not produce or endorse meaning.

The new `guide-opportunity@v1` checker reports orientation within a disclosed two-link
window separately from unfamiliar-category access opportunities and continuity.
Invalid graphs report unavailable access facts as null, not false/empty observations.
No combined reward or changed acceptance threshold. Source/criteria/permissions
remain independent of model interpretation; initial context is not a return.

An exact prospective campaign grant validator supports 272/74 -> 283/77 only with
existing manifest/prefix/authority/suite checks and unchanged $5 cap. No grant was
written to the real ledger. Unknown reservations survive extension and block dispatch.

## Verification performed

- New packet: 9 tests passed, including multiple valid organizations, unchanged input,
  category relabeling, invalid/unassessable output, source consistency, checker
  tampering/path controls and receipt digest binding.
- Historical open-guide packet: 7 tests passed, unchanged.
- Generic objective cycle: 9 tests passed, including unknown/restart/review/abstention.
- Campaign extension fixtures: 22 tests passed, including new exact prospective cap,
  wrong prefix/scope/caps, no replay, append-only counters and unknown reservation.
- Continuation controls: 14 passed, 1 skipped (separate cfg(test) continuation full
  flow not enabled; this change tests fresh native objective flow below).
- Real native CLI/common Cycle with authored transport and cfg(test) receipts: 1 full
  flow passed, 20 authored requests, 4 stub worker groups (patch + three rounds).
  Covers initial relation, no-new-meaning reconsideration, ordinary service choices,
  two format corrections, interruption/restart and inert terminal replay. Every
  request checks shared facts and exclusions. Altered receipts are rejected before
  derivation; stale roots/sources and oversized derived context reject.
  These are authored wiring observations, not target execution or model behavior.
- Native flow took 894.939 seconds. Maximum measured serialized request: 31,234 bytes,
  leaving 1,534 bytes under 32 KiB. This is not a worst-case valid-history guarantee.
  More verbose live output may stop incomplete; no truncation or limit increase.
- After that flow, the independently identified invalid-output null branch and
  prospective accounting validator were finalized and tested with targeted checks.
  Valid-output diagnostic behavior was unchanged; the expensive native test was not
  repeated for those isolated branches. The public version label was also made
  prospective. Fresh final pins and native serialization were checked afterward.
- Independent code/concept review by `/root/parity_review`: no blocking findings;
  reviewer independently reran all 9 packet tests. `git diff --check` passed.
- No Rust production changes; no blanket Rust suite rerun.

Commands:

```sh
PYTHONDONTWRITEBYTECODE=1 TSR_OBJECTIVE_TEST_PACKET=experiments/guide-opportunity-v1 TSR_OBJECTIVE_TEST_CHECKER=guide-opportunity@v1 TSR_OBJECTIVE_TEST_WORKER_BIN="$PWD/target/debug/deps/tsr_cli-95f29d427efcd336" python3 -m unittest discover -s scripts -p test_engine_objective_adapter.py -v
python3 -m unittest discover -s experiments/guide-opportunity-v1 -p test_packet.py -v
python3 -m unittest discover -s experiments/open-guide-v1 -p test_packet.py -v
python3 -m unittest discover -s scripts -p test_engine_objective_cycle.py -v
python3 -m unittest discover -s scripts -p test_engine_b91_campaign_extension.py -v
python3 -m unittest discover -s scripts -p test_engine_objective_continuation.py -v
```

The draft invalid-output test first failed (false rather than null), then passed after
the correction. No failure was hidden or reclassified. Full authored stage log is
`experiments/guide-opportunity-v1-native-fixture.log` (no provider response bodies).

## Frozen prospective packet and boundary

Manifest: `experiments/guide-opportunity-v1/manifest.json`
SHA-256: `4b605009f79b08c1105d2f0e607d1e611d14ac571b0170e812d78b1b85f4853a`.
Preparation: `target/guide-opportunity-prepared-1`; local preflight:
`target/guide-opportunity-prepared-1/LOCAL_PREFLIGHT.json`.
Final-pin initial request sizes: proposal 13,892 bytes, review 16,344 bytes.
Fresh-process native binding unchanged; no dispatch/adoption. All manifest pins
match and ledger digest remains unchanged after preparation.

See packet README for full export scope, rubric, information-equal comparator design
and no-live-authority runbook. Exact future request: one native-arm batch, at most
24 Luna calls including all reviewers and two shared format corrections, 5 groups
(3 service + 2 optional patch), 120 minutes, $1.20 reservation within $5. Proposed
total caps 283 calls / 77 groups, no provider-failure retry or second attempt. Zero
corpus reads, only disposable lib.rs edits, protected entry/checker/criteria,
existing pinned non-root networkless runtime. Evidence-supported final acceptance
only in separate review copies. Fresh approval, rates, source/ledger upgrade binding
and admission remain required; this preparation does not activate them.

This reuses the three prior conditions with a new prospective observation protocol;
it is not held-out generalization. Access opportunity is not observed human discovery.
A positive relational claim requires grounded admission and demonstrated later use;
stored prose is insufficient. Retention/abstention are valid. No baseline is run;
the role-only ordinary-memory comparator still needs separate implementation/review.
General new-desire identity succession remains unsupported. Contributors/history
persist; permission revocation is distinct from culmination.
