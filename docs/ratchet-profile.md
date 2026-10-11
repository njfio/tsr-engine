# RATCHET protection profile

> **Ownership note.** This file is shared. worker-t5 owns the profile-format sections
> (`## The profile format`, budgets, approvals, the adversarial suite). worker-t4 owns the
> section `## tsr policy check event envelope` below. Append, do not overwrite.

## tsr policy check event envelope

`tsr policy check <profile.toml> <event.json>` is the operator's one-shot gate (PRD §B5). It loads a
RATCHET protection profile and an **event envelope**, builds a `GateInput`, calls
`tsr_policy::gate::evaluate` and prints the verdict as one JSON object on stdout:

```json
{"decision":"allow","reason":"Inspect is inside the grant and the profile at state version 1"}
```

The command reads **no** scores, reflections, repository text or provider output. Everything it
reads is in the two files it is handed: the profile is authority; the envelope is the task's grant,
the log so far and the proposed candidate.

### Exit codes

| code | meaning |
| --- | --- |
| `0` | `allow` |
| `1` | `deny` |
| `2` | `ask` |
| `3` | malformed profile or event — a JSON error object is printed to **stderr** |

### The envelope

A JSON object:

```json
{
  "task": "<64 hex>",
  "candidate": "<64 hex>",
  "kind": "inspect|run_approved_test|generate|propose_patch|request_approval|stop",
  "args": { ... },
  "state_version": 1,
  "grant": {
    "task": "<64 hex>",
    "allowed_kinds": ["inspect", "run_approved_test", "generate", "propose_patch", "request_approval", "stop"],
    "read_scope_paths": ["src/", "tests/"],
    "read_scope_rts_tools": ["search", "callers"],
    "allowed_test_ids": ["cargo-test-workspace"],
    "budgets": {
      "generator_tokens": 200000,
      "tool_minutes": 30,
      "wall_clock_secs": 3600,
      "generator_calls": 40,
      "patch_bytes": 262144
    }
  },
  "log": [ /* Event, ... */ ]
}
```

| field | meaning |
| --- | --- |
| `task` | the task id (64 hex). Must equal `grant.task`; a mismatch is malformed input (exit 3). |
| `candidate` | the proposed candidate's id (64 hex). |
| `kind` | the action kind, the snake_case name of `tsr_domain::ActionKind`. |
| `args` | the typed arguments; the shape depends on `kind` (see below). |
| `state_version` | the current state version, `u64`. |
| `grant` | the task's capability grant, mirroring `tsr_policy::gate::Grant`. |
| `log` | the log so far, a JSON array of `tsr_domain::Event` (approvals, forecasts, revocations). |

### `kind` + `args` → `ActionKind` + `ActionArgs`

| `kind` | `args` | `ActionArgs` |
| --- | --- | --- |
| `inspect` | `{"path": "src/lib.rs"}` | `InspectPath` |
| `inspect` | `{"rts_tool": "search"}` | `InspectRtsTool` |
| `run_approved_test` | `{"test_id": "cargo-test-workspace"}` | `TestId` |
| `propose_patch` | `{"patch_bytes": 4096}` | `PatchBytes` |
| `request_approval` | `{"boundary": "promote_diff"\|"non_allowlisted_command"\|"network"}` | `Boundary` |
| `generate` | (ignored) | `Generate` |
| `stop` | (ignored) | `Stop` |

The candidate's `args` digest is the blake3 of the canonical JSON of the `args` object, so an
approval that binds an artifact digest binds exactly the arguments the model proposed.

### Fixtures

- `crates/tsr-cli/tests/fixtures/policy/injection-readme.json` — a README prompt-injection attempt
  (an `Inspect` of `/etc/passwd`). Repository text carries no authority, so the gate denies:
  `{"decision":"deny","reason":"inspect path /etc/passwd is absolute; the read scope is relative to the repository copy"}`.
- `crates/tsr-cli/tests/fixtures/policy/no-forecast.json` — a gated `RunApprovedTest` with no
  `Forecast` at the current state version. Denied, naming what to record:
  `{"decision":"deny","reason":"record a forecast for RunApprovedTest at state version 1 before this action"}`.

### Acceptance commands

```sh
export CARGO_HOME="$PWD/.cargo-home"; export PATH="/usr/local/cargo/bin:$PATH"
cargo build --workspace
./target/debug/tsr policy check profiles/default.toml crates/tsr-cli/tests/fixtures/policy/injection-readme.json
./target/debug/tsr policy check profiles/default.toml crates/tsr-cli/tests/fixtures/policy/no-forecast.json
```