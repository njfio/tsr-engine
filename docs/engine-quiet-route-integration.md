# Quiet-route integration and cross-family result

## Cross-family verification

Current local metadata confirmed `mistral:latest` has a chat template, completion
capability and 32,768-token context, with no remote host/model. Digest:
`61e88e884507ba5e06c49b40e6226884b2a16e872382c2b44a42f2d119d804a5`.
Metadata is `target/mistral-current-metadata.json`. No model was downloaded.

The explicit `mistral-v1` comparison used the identical current state and rules,
possibility temperature 0.7, seed 7304, and a one-sample search. Request 28 returned
inability; no candidate was frozen, assessed or executed. This is a result, not a
transport/format problem, so no retry was dispatched. One of three remaining slots
was consumed; two remain unused. Provider PID 28704 was terminated and its session
exited zero. No paid or external call occurred.

Evidence under `target/support-live/run`:

- Request envelope: `request-28-ed36e3454561.json`.
- Inability: `inability-28.json`, SHA-256
  `b076cea57f679826a33717baacbdc7679a0ea6370cd08c905625065a90495c3e`.
- Validation metadata: `provider-response-28.json`, SHA-256
  `f7f7277f9eba2672795089571b4be1258ab537c7ec949260a03e738de8f320c8`.
- Terminal output: `target/support-live/mistral-comparison-live.log`.

The preserved root remains
`3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`.
Desires, observation [0.5, 0.8, 0.0] and joint conflict remain unchanged. Unknown13,
inability24 and all subsequent receipts remain preserved.

The tested Qwen and Mistral paths are capability-limited in this concrete context:
none generated a useful candidate despite an independent mechanical partial-
progress witness. This does not prove universal provider incapacity or impossibility
of partial progress. No further redundant trial is warranted; live recovery remains
unverified. The following implementation is a separate core improvement, not a
claim to solve generation.

## PRD gap and implementation

PRD §B8 specifies a logged quiet route so a leading-edge or low-mass desire can
receive attention. `tsr-ledger::selection::select` already implements a deterministic
snapshot/seed draw and leading-edge seeding. However, the notebook decision entry
point always supplied probability zero and seed zero. Ordinary operators could
not enable that existing mechanism durably.

`tsr engine --directory RUN direction-tune --update tune.json` now accepts:

```json
{
  "state_root": "<root from direction-request>",
  "quiet_route": 0.1,
  "seed": 19,
  "reason": "Allow some leading-edge attention among admitted standing wants"
}
```

Use the current root; stale updates fail. Probability must be finite in [0,1] and
the reason must be nonempty and at most 512 bytes. Zero restores the normal route;
one forces the quiet route for a controlled comparison. The update appends an
Operator-sourced, content-addressed `SelectionTuned` record. Reads reject generated
or invalid tuning records. Existing histories without one retain `(0,0)` behavior.
The latest operator setting is folded on every request and survives restart.

The command reuses the shared selection kernel; it adds no scoring formula,
weights, desire, candidate or authority. Attention remains bounded by the existing
specification. Fulfilled/withdrawn/out-of-scope desires remain filtered by existing
admission rules. All standing original losses remain protected during later action
assessment even when attention focuses elsewhere. Generator schemas cannot set
this tuning. Reasons and settings remain inspectable in the hashed record, and
`direction-request.selection` shows probability, seed and `quiet_selected`.

A fixed seed does not mean identical draws across different state roots. At the
same root, repeated reads and restart are identical. A changed state changes the
logged draw input, as the existing kernel specifies. Settings are episode-local
operator configuration; they are not autonomous preference learning or proof that
a generated direction will improve. No tuning was applied to support-live.

## Runnable operator verification

With a disposable initialized notebook run:

```sh
target/debug/tsr engine --directory RUN direction-request > request.json
python3 - <<'PY'
import json
from pathlib import Path
state = json.loads(Path('request.json').read_text())
Path('tune.json').write_text(json.dumps({
    'state_root': state['state_root'], 'quiet_route': 0.1, 'seed': 19,
    'reason': 'Operator enables bounded leading-edge attention'
}))
PY
target/debug/tsr engine --directory RUN direction-tune --update tune.json
target/debug/tsr engine --directory RUN direction-request
```

The actual disposable CLI verification is `target/quiet-route-operator/`:
`spec.json`, `tune.json`, `run/`, and `verification.json`. With probability one as a
controlled intervention and attention limit one, selection changed from original
want 0 to want 2 under identical observations, wants, returns and authority. A new
CLI process reproduced the full response. This is synthetic causal selection
verification, not model generation or satisfaction evidence. Internal state was
not hand-edited.

## Checks

- CLI library: 99 passed, one existing diagnostic ignored.
- Ledger: 25 tests passed across its library/integration suites.
- Clippy with warnings denied and CLI build passed.
- Diff whitespace check passed; independent review found no blocker.

Logs: `target/quiet-route-cli-tests.log`, `target/quiet-route-ledger-tests.log`,
`target/quiet-route-clippy.log`, `target/quiet-route-build.log` and
`target/quiet-route-focused.log`. Tests include causal selection, unchanged reads,
restart, restoration, withdrawal, invalid/stale updates and rejection of generated
tuning provenance. Original protected tests and unrelated work are unchanged.
Corpus remains 151 qualified tasks; eligible independent mature episode count
remains unknown. No calibrated superiority or unattended completion is claimed.
