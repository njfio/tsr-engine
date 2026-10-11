# Lanes — how the swarms run

Three lanes from one spec. One contract (`crates/tsr-domain`) lands first; each lane then runs in
its own workspace so no two runs ever share a checkout (ds-swarm's operator rule: never write to a
checkout with a live run).

| Lane | Brief | Workspace | Branch | Milestone |
| --- | --- | --- | --- | --- |
| 01 core | `01-core.md` | `~/Developer/tsr-engine` | `lane/core` | M0 |
| 02 policy | `02-policy.md` | `~/Developer/tsr-engine-policy` (second clone) | `lane/policy` | M1 |
| 03 rts | `03-rts.md` | `~/Developer/rs-agent-code-utility` | `lane/tsr-inspect` | small PR |

## One-time setup (on the Mac, in a terminal with the keys)

```bash
# 0. keys: ~/.config/ds_swarm/env exports CHEAPINFERENCE_API_KEY (and others). Never commit it.
source ~/.config/ds_swarm/env

# 1. build ds-swarm once
cd ~/Downloads/ds-swarm-main && cargo build --release
export PATH="$HOME/Downloads/ds-swarm-main/target/release:$PATH"
ds-swarm doctor --provider cheaperinference
ds-swarm models --provider cheaperinference | grep -i deepseek     # pick the 4.1 flash id

# 2. the tsr repo: init, first commit, GitHub remote (lands PRs)
cd ~/Developer/tsr-engine && git init -b main && git add -A && git commit -m "chore: M0 skeleton — contract crate, append-only store, red M0 test, lane briefs"
gh repo create njfio/tsr-engine --private --source=. --remote=origin --push

# 3. workspaces
git clone ~/Developer/tsr-engine ~/Developer/tsr-engine-policy && git -C ~/Developer/tsr-engine-policy remote set-url origin git@github.com:njfio/tsr-engine.git
git -C ~/Developer/tsr-engine checkout -b lane/core
git -C ~/Developer/tsr-engine-policy checkout -b lane/policy
git clone git@github.com:njfio/rs-agent-code-utility.git ~/Developer/rs-agent-code-utility && git -C ~/Developer/rs-agent-code-utility checkout -b lane/tsr-inspect
```

## Launch (one terminal per lane, or three `ds-swarm serve` processes)

Each run keeps its run directory *inside its workspace* (`--run-dir`), so the orchestrator can read
`events.jsonl`, `state.json` and `report.md` through the connected folder while the run is live.
Sandboxing the swarm's own commands with `--sandbox docker` is strongly preferred (ADR-009); drop it
only if Docker is not installed, and say so in the run's notes.

```bash
source ~/.config/ds_swarm/env
MODEL="<the deepseek 4.1 flash id from ds-swarm models>"
COMMON="--provider cheaperinference --model $MODEL --sandbox docker --sandbox-image rust:1.90-bookworm"

# lane 01 — core (M0)
cd ~/Developer/tsr-engine && ds-swarm run "$(sed -n '/^## Goal/,/^## Definition/p' specs/lanes/01-core.md | sed '1d;$d')" \
  --dod "$(sed -n '/^## Definition of done/,/^## Out of scope/p' specs/lanes/01-core.md | sed '1d;$d')" \
  --workspace . --run-dir .run/lane-01 $COMMON \
  --max-tokens 900000 --deadline-secs 10800 --max-agents 6 --reviews security,architect,correctness

# lane 02 — policy (M1)
cd ~/Developer/tsr-engine-policy && ds-swarm run "$(sed -n '/^## Goal/,/^## Definition/p' specs/lanes/02-policy.md | sed '1d;$d')" \
  --dod "$(sed -n '/^## Definition of done/,/^## Out of scope/p' specs/lanes/02-policy.md | sed '1d;$d')" \
  --workspace . --run-dir .run/lane-02 $COMMON \
  --max-tokens 700000 --deadline-secs 7200 --max-agents 5 --reviews security,architect,correctness

# lane 03 — rts (the brief lives in this repo; copy it into the rts workspace first)
cp ~/Developer/tsr-engine/specs/lanes/03-rts.md ~/Developer/rs-agent-code-utility/specs-tsr-lane.md
cd ~/Developer/rs-agent-code-utility && ds-swarm run "$(sed -n '/^## Goal/,/^## Definition/p' specs-tsr-lane.md | sed '1d;$d')" \
  --dod "$(sed -n '/^## Definition of done/,/^## Acceptance/p' specs-tsr-lane.md | sed '1d;$d')" \
  --workspace . --run-dir .run/lane-03 $COMMON \
  --max-tokens 400000 --deadline-secs 5400 --max-agents 4 --reviews security,correctness
```

To watch and steer from a browser: `ds-swarm serve --run-dir ~/Developer/tsr-engine/.run/lane-01`
(ports 7373, 7374, 7375 for the three lanes). The dashboard delivers operator messages to one agent
or the team; the orchestrator uses the same loopback API.

## The orchestrator's loop

1. Every 20–30 minutes: read each lane's `report.md` and the tail of `events.jsonl`; note stalls,
   rejected verdicts, budget pressure.
2. A reviewer rejection with a repair task on the board is normal; a director that stopped with work
   outstanding, or a run that ended `failed`, gets `ds-swarm resume <run-dir> --max-tokens …` after
   the reason is read.
3. A lane that touches `crates/tsr-domain` outside a `contract:` commit gets an operator message
   naming the rule.
4. Landed PRs are reviewed against the lane's definition of done and the PRD section, not against
   the director's summary. Merge order: 01 first (it owns the store the others build on), then 02,
   then 03.
5. After the three land: record one real Claude Code session through `tsr hook` on a disposable copy
   of rts and commit it as `fixtures/m0/session.events.jsonl` — M0's real exit evidence.
