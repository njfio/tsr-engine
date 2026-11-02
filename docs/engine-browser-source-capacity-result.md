# Browser source capacity and scoped editing — local result

The browser no longer inherits the small Rust edit-size contract. It can inspect
explicit source windows and make incremental exact changes to larger files. This
is local engineering evidence; no live game or paid call was made. Frozen
`495295d0` and previous results remain unchanged.

## Implemented behavior

- Removed browser application-level 2 KiB span/4 KiB aggregate/8 KiB file gates.
  Default Rust/native callers retain their previous limits.
- Added explicit source indexes and UTF-8 byte windows. Generator-only
  `inspect:{path,offset}` selects another admitted editable window using a counted
  request. It cannot read protected/outside files, change permissions, or invent
  omitted contents. Source identities, exact matching and stale checks remain.
- Shared strict validation now reports precise full-envelope byte paths. Eligible
  deterministic size regeneration is opt-in in the new admission and shares the
  finite correction budget. Unknown effects, mixed alternatives, wrong paths,
  stale sources and semantic rejection do not qualify. No output is repaired.
- Native parsing and advancement use browser-specific local resource envelopes.
  The 64 KiB shared executor setting was software, not a hardware limit: browser
  exact validation can use the existing 1 MiB complete-snapshot memory budget,
  while the default executor retains 64 KiB. This is still finite software resource
  accounting, not unlimited capacity.
- Actual writes send only exact patch spans through UTF-8-safe argument chunks,
  rather than resending complete files. A fixed controller shell/Node program
  applies them in the preexisting writable edit phase. Browser JavaScript remains
  in the read-only execution phase. No outer isolation or path authority changed.
- Independent review receives explicit omission markers and must return unknown
  where unseen code prevents a criterion assessment. Model transport remains
  32 KiB/request, 8192 output tokens and 240 seconds. Complete oversized requests
  stop without clipping. Native response128 KiB/advance256 KiB and local8 MiB
  envelopes are resource budgets, not promises every maximum can be combined.

## Verification and failures retained

24 affected checks passed across the final tested paths:

- Eight unique adapter tests: seven passed in the frozen suite; the affected
  full browser test passed separately after fixing write-phase routing (36.456s).
  It exercises authored multi-file code above the old edit limit, native preview,
  independent authored review, actual isolated click/key execution (movement,
  collection and pause), feedback persistence and inert terminal replay.
- Seven projection tests, including actual native serialization, UTF-8 byte
  diagnostics, explicit large-source windows, protected/stale rejection and
  attributed feedback.
- Six shared correction tests, including unknown/semantic/identity rejection.
- Native browser-profile/default-Rust isolation test; explicit larger-text/default
  executor isolation test; existing exact-edit preconditions test (including
  overlapping matches, path escape and invalid source).

Build succeeded with Rust1.90 offline. `git diff --check` passed. Independent source
review passed after resolving downstream parser, producer-envelope and process
argument issues. A review claim about overlap semantics was checked against actual
Rust `char_indices` logic and withdrawn: native and Node both reject overlaps.

Earlier local failures were real and remain in target logs: a missing shape hook;
a diagnostic parser rejecting the over-limit count it needed to report; a stale
assertion expecting the removed64 KiB cap; and Node edits routed to the read-only
phase. None were hidden by changing deadlines, criteria or model outputs. The
final phase correction retains the already-authorized writable edit phase. No
full workspace suite or live model validation is claimed.

Authored fixture `scripts/testdata/browser-project/control-fixture.js` is harness
infrastructure, not the requested engine-built game. The tested browser fixture is
not a measure of player satisfaction or TSR culmination. Safe refusal of an
interrupted/terminal error remains distinct from clean completed replay.

Logs: `target/browser-source-{frozen-tests,write-phase-test,projection-final,
correction-tests,native-final,broker-tests,default-edit-tests}.log`.
Local preparation: `target/browser-game-v3-prepared-local2/`.

## Packet and remaining live boundary

Packet: `experiments/browser-game-v3/README.md` and `manifest.json`.
SHA-256: `21703bd06d7814bb3dd5b8c3196a358ddf26148bf2cb44d8413ce424ae093ca4`.
Local preparation and production request serialization passed without dispatch.
The packet prepares a blank project and prospective admissions only.

Proposed cohesive session:26 total Luna calls (all inspections/generation/review,
including2 shared format/size corrections),8 isolated execution groups,120 minutes,
$1.30 reservation below the user's $10/run ceiling. No provider-failure retry or
automatic second session. Stop at a reviewed functional checkpoint or a legitimate
terminal condition, then take genuine user feedback. No game code is supplied by
the operator. Same public data categories, protected tests and pinned runtime.

Campaign is unchanged at262 calls/72 operations/$0.225523, no unknowns. Applied
caps remain272/74/$5; available capacity is10 calls/2 operations. The proposed run
needs total caps288/80. Automatic approval review rejected adding that new grant
profile before the command executed because no operational-cap increase was
approved. The campaign code and ledger were not modified. This is the exact live
blocker: explicit authority and reviewed implementation for the new cap grant,
then fresh pins/pricing/admission. The packet is locally prepared, not live admitted.
No additional run, installation, deployment, public upload or security change occurred.
