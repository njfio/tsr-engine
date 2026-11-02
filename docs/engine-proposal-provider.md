# Model-shaped proposals through the persistent engine

This slice runs the complete proposal-to-return path with local JSON replay. It makes no network calls. [Captured CLI trace](engine-provider-trace.json) evaluates two supplied proposals against exactly the same state: configured `draft` is eligible; a new workflow is deferred. Applying the first creates a pending commitment, not an executed action. After process restart, an explicit failed observation changes draft's estimate from .5 to .333 and appears in the next provider request.

## Architecture

```text
SQLite events + CAS facts
  -> one-snapshot Request (admitted goals, current commitment, estimates,
       frozen forecasts, recent operator returns, structured Contrast)
  -> local JSON replay / future model JSON response
  -> strict typed decoder + state-root check + admission assessment
  -> preview (read-only), or apply:
       typed untrusted suggestion record
       -> existing ProposalProvider boundary
       -> native candidate selection + frozen forecast + pending commitment
  -> existing observe command -> Surprise + Contrast -> next Request
```

Proposal text never becomes evidence, a successful outcome, an admitted goal or an action definition. The persisted typed suggestion is a Generator record excluded from outcome counts, candidate evidence support and the preference ledger. No raw HTTP response or credentials are stored. The engine's forecast comes from its own history, not a model's rationale.

## Run it

From the repository root, use a fresh directory:

```sh
cargo +1.90.0 build -p tsr-cli --bin tsr --offline
./target/debug/tsr engine --directory target/proposal-demo configure --spec docs/engine-goal-example.json
./target/debug/tsr engine --directory target/proposal-demo request > target/proposal-request.json
python3 - <<'PY'
import json
from pathlib import Path
r = json.loads(Path('target/proposal-request.json').read_text())
base = {'protocol': r['protocol'], 'state_root': r['state_root']}
for name, proposal in [
    ('existing', {'kind':'existing', 'action':'draft', 'rationale':'A draft addresses readability.'}),
    ('new', {'kind':'new_candidate', 'description':'Invent a new workflow', 'serves':['new-goal'], 'rationale':'An idea requiring operator configuration.'})
]:
    Path('target/proposal-'+name+'.json').write_text(json.dumps(base | {'proposals':[proposal]}))
PY
# Both previews leave the same state unchanged:
./target/debug/tsr engine --directory target/proposal-demo propose --response target/proposal-existing.json
./target/debug/tsr engine --directory target/proposal-demo propose --response target/proposal-new.json
# Explicitly persist the eligible suggestion and its pending commitment:
./target/debug/tsr engine --directory target/proposal-demo propose --response target/proposal-existing.json --apply > target/proposal-applied.json
# Resume from another process, then supply the real observed result:
./target/debug/tsr engine --directory target/proposal-demo state
COMMITMENT=$(python3 -c 'import json; print(json.load(open("target/proposal-applied.json"))["state"]["pending"]["commitment"])')
./target/debug/tsr engine --directory target/proposal-demo observe --commitment "$COMMITMENT" --result docs/engine-result-failure.json
./target/debug/tsr engine --directory target/proposal-demo request > target/proposal-updated-request.json
```

Replace the example result with the user's actual observation. A response is bound to the complete state root: after a write, fetch a new request rather than reusing a stale response. `--apply` with only deferred suggestions records those possibilities but does not commit. Unknown actions and all new candidates are deferred; there is no command here to activate a new action in the current configuration.

## Contract and limits

Request protocol: `tsr-proposal@v1`, at most 32,768 serialized bytes. It includes at most eight frozen forecasts, eight returns, eight Contrast records, eight prior evidence reports and eight supporting event references per action. Only admitted preferences enter the outbound request. The goal/action limits from the configured engine still apply.

Response: at most 8,192 bytes and four proposals, each carrying bounded text (512 bytes per text field). Top-level fields are exactly `protocol`, `state_root`, `proposals`. A proposal is either:

- `existing`: `action`, `rationale`.
- `new_candidate`: `description`, `serves` (1–16 criterion names), `rationale`.

Extra fields such as `evidence`, commands, approval flags or replacement goals fail decoding. Text may contain arbitrary claims; claims have no effect on support or authority. An existing action must serve an admitted criterion and there must be no pending commitment. Among eligible offered actions, the existing persistent core selects by its derived score and freezes the expectation. Eligibility is not proof of success. New candidates remain suggestions even if their text names an existing goal.

## Existing provider transport, offline only

```sh
python3 scripts/engine-provider-envelope.py --request target/proposal-request.json > target/provider-envelope.json
```

This reads literal endpoint/model/token-limit constants from the existing `m3-cheaper-broker.py` using Python AST parsing, without importing or executing that broker. It prepares a compatible chat-completions JSON envelope with `dispatch_enabled:false`. It does not read credentials, reserve funds or send requests. This is configuration reuse, not a claimed live connection.

The existing paid adapter's scope/codec validates coding-task requests, so it cannot consume this generic goal request unchanged. The remaining concrete work for a real model-backed coding session is an explicitly authorized live transport codec with accounting, plus a coding domain adapter that maps approved candidates to the existing sealed executor and ingests independent tool outcomes. New action admission also needs an explicit operator-controlled configuration transition. Those pieces are not enabled here; no benchmark was restarted.

Validation: 34 tests pass (33 library + one CLI integration), including same-state eligible/deferred proposals, no preview mutation, stale replay refusal, restart/return feedback, rationale not entering evidence, and extra-authority/oversize response rejection. Clippy and formatting checks pass. The actual CLI replay and offline envelope preparation are captured separately from tests.
