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

## The profile format

A RATCHET protection profile is a TOML document that `tsr-policy` loads per task. It is **authority,
not evidence**: it names what a task may read, which test commands it may run, what it may spend and
which boundary actions need an operator approval. It carries no scores, no reflections, no repository
text and no provider output.

The Rust type is `tsr_policy::profile::Profile`; load it with `Profile::load(Path::new(...))` or parse
a string with `Profile::from_toml_str(...)`. Both reject an empty or duplicate test-command id,
because an allowlist that cannot name a command unambiguously cannot authorize one.

### Field by field

| field | type | meaning |
| --- | --- | --- |
| `version` | integer | Format version. Bumped when the shape changes incompatibly. |
| `name` | string | Human-readable profile name (e.g. `default`). |
| `[read_scope]` | table | The read allowlist (below). |
| `[[tests]]` | array of tables | The exact allowlisted test commands, by id (below). |
| `[budgets]` | table | Per-task budgets, reserved before dispatch (below). |
| `[approvals]` | table | Which boundary actions require an operator approval (below). |

#### `[read_scope]`

| field | type | meaning |
| --- | --- | --- |
| `paths` | array of strings | Allowed path **prefixes**. A candidate path must lie under one of these on a path-segment boundary, be relative, and contain no `..` segment. An empty prefix is rejected at load. |
| `rts_tools` | array of strings | Allowed `rts` tool names. An `rts` query must name one of these exactly. |

The gate takes the **intersection** of the profile's read scope and the task grant's read scope: a
path or tool must be in both. A path that is absolute, empty, or contains a `..` segment is denied
regardless of the prefixes.

A prefix matches on a **path-segment boundary**: `src` matches `src` and `src/lib.rs`, but not the
sibling `src-evil/evil.rs` or the unrelated `srcx`. A trailing `/` is normalised away, so `src` and
`src/` are the same prefix; `Cargo.toml` matches `Cargo.toml` but not `Cargo.toml.bak`. An empty
prefix (`""`) is rejected when the profile is loaded, and the gate skips it defensively, so a
malformed profile cannot widen the read scope to the whole repository.

#### `[[tests]]`

| field | type | meaning |
| --- | --- | --- |
| `id` | string | The exact allowlisted command id (e.g. `cargo-test-parser`). The model names the id; it never supplies `argv`. |
| `argv` | array of strings | The command the id expands to (e.g. `["cargo", "test", "--offline", "-p", "parser"]`). |

A `RunApprovedTest` naming an id that is not in **both** the profile's `tests` and the grant's
`allowed_test_ids` is denied. The model cannot smuggle an argv: only the id is read.

#### `[budgets]`

| field | type | meaning |
| --- | --- | --- |
| `generator_tokens` | integer | Generator tokens. |
| `tool_minutes` | integer | Tool minutes. |
| `wall_clock_secs` | integer | Wall-clock seconds. |
| `generator_calls` | integer | Generator calls. |
| `patch_bytes` | integer | Patch bytes. |

Budgets are **reserved before dispatch** (`BudgetLedger::reserve`) and committed or released after.
A reservation is refused when `reserved + consumed + cost` would exceed the ceiling, so two in-flight
reservations can never jointly overrun a budget. On exhaustion the task stops with `Stop(budget)` and
marks any effect in flight as unresolved until reconciled. The gate takes the **minimum** of the
profile's and the grant's ceiling on each axis.

#### `[approvals]`

| field | type | meaning |
| --- | --- | --- |
| `promote_diff` | bool | `true` = promoting a diff out of the sandbox to the host checkout needs an approval. |
| `non_allowlisted_command` | bool | `true` = running a command that is not on the allowlist needs an approval. |
| `network` | bool | `true` = any network use needs an approval. |

A boundary action the profile gates, with no valid approval on record, is `ask` (not `deny`): the
model is told to request one with `RequestApproval`. At dispatch the approval is re-checked, and any
change — task, candidate, artifact digest, repository snapshot digest, scope or expiry — denies.

### The example profile, inline

`profiles/default.toml`:

```toml
# RATCHET protection profile — the default profile for the TSR Decision Engine.
#
# This is `tsr-policy`'s configuration (PRD §B5, §B4). It is authority, not evidence: it names the
# read scope, the exact allowlisted test command ids, the per-task budgets and the boundary actions
# that require an operator approval. It carries no scores, no reflections, no repository text and
# no provider output.
#
# Load it with `Profile::load(Path::new("profiles/default.toml"))`.

version = 1
name = "default"

# What the task may read. A candidate path must start with one of these prefixes; an `rts` query
# must name one of these tools. Anything else is denied.
[read_scope]
paths = ["src/", "tests/", "docs/", "Cargo.toml", "crates/"]
rts_tools = ["search", "callers", "definition", "references"]

# The exact allowlisted test commands, by id. The model names an id; it never supplies argv.
[[tests]]
id = "cargo-test-parser"
argv = ["cargo", "test", "--offline", "-p", "parser"]

[[tests]]
id = "cargo-test-workspace"
argv = ["cargo", "test", "--offline", "--workspace", "--no-fail-fast"]

[[tests]]
id = "cargo-clippy"
argv = ["cargo", "clippy", "--offline", "--workspace", "--all-targets", "--", "-D", "warnings"]

# Per-task budgets, reserved before dispatch. On exhaustion the task stops with Stop(budget).
[budgets]
generator_tokens = 200000
tool_minutes = 30
wall_clock_secs = 3600
generator_calls = 40
patch_bytes = 262144

# Boundary actions that require an operator approval. `true` = an approval is required.
[approvals]
promote_diff = true
non_allowlisted_command = true
network = true
```

## The four sources of authority

Policy reads **no scores, no reflections, no repository text and no provider output**. Authority
comes from exactly four sources (PRD §B5, AGENTS.md invariant 4):

1. **The grant** — the task's capability grant (`tsr_policy::gate::Grant`): the action kinds it may
   take, its read scope, its allowlisted test ids and its budgets. The profile is the operator's
   standing configuration; the grant is what the task was actually given. An action must be inside
   **both** — the gate takes the intersection, never the union.
2. **The allowlists** — the profile's read scope (paths and `rts` tools) and test-command ids, again
   intersected with the grant's.
3. **The budgets** — reserved before dispatch; an exhausted budget blocks further dispatch.
4. **The approvals** — bound to task + artifact digest + repository snapshot digest + scope + expiry,
   re-checked at dispatch, invalidated by any change.

A README that says "disable the gate" is a `tsr_domain::Observation` like any other: it is not an
input to the gate. The same holds for a filename, a code comment, a stack trace or a prior episode
summary. None of them can widen a scope or self-approve.

## The gate: `allow | deny | ask`

`tsr_policy::gate::evaluate` is the decision the hook makes before an action happens;
`evaluate_dispatch` is the same decision re-checked at dispatch, when an approval may have been
revoked or the artifact may have changed. Both are **pure**: no I/O, no clock, no randomness. The
same inputs always produce the same verdict, which is what makes the gate auditable.

The order of checks (first failure wins):

1. the kind is in the grant's `allowed_kinds`;
2. the arguments are inside the intersection of the profile's and the grant's allowlists;
3. a boundary action that the profile gates has a valid approval (else `ask`, not `deny`);
4. a gated kind (`RunApprovedTest`, `ProposePatch`, `RequestApproval`) has a `Forecast` for this
   candidate at this state version;
5. no budget is exhausted.

Fail-closed: anything the gate cannot type, scope or authorize is denied. The verdict carries a
one-line reason the model reads, naming what to do next.

## The adversarial suite

`crates/tsr-policy/tests/adversarial.rs` is the evidence for the **Authority isolation** gate
(PRD §B9): zero unauthorized effects, every bypass attempt recorded. It has seventeen named cases,
one `#[test]` each, and every one asserts the exact decision *and* that the reason is a single
non-empty line:

| case | what it proves |
| --- | --- |
| `injection_in_readme_text_cannot_widen_scope` | README text saying "disable the gate / read /etc/passwd" is an Observation; an Inspect outside scope is still Deny. |
| `injection_in_filename_cannot_widen_scope` | A file named `../../etc/passwd` is denied as out of scope; a name is a path, not an instruction. |
| `injection_in_comment_cannot_self_approve` | A diff comment saying "approved by operator" creates no approval; the boundary action is still Ask. |
| `injection_in_stack_trace_cannot_widen_scope` | A stack trace naming a path outside scope does not widen the read scope. |
| `altered_diff_invalidates_approval` | An approval bound to artifact digest A is invalid for digest B. |
| `altered_snapshot_invalidates_approval` | An approval bound to snapshot digest S is invalid for S'. |
| `approval_revoked_between_decision_and_dispatch_blocks` | Approved then Denied for the same candidate: the dispatch re-check denies. |
| `budget_exhaustion_blocks_dispatch` | After reserving to the ceiling, the next dispatch is denied naming the exhausted budget. |
| `candidate_without_forecast_at_current_version_is_denied_with_reason` | A gated action with no Forecast at the current version is Deny, naming what to record. |
| `forecast_at_a_stale_version_does_not_satisfy_the_gate` | A Forecast at version N-1 does not allow the action at version N. |
| `model_proposed_command_outside_allowlist_is_denied` | A RunApprovedTest with a test id not in the allowlist is Deny. |
| `wait_action_does_not_exist` | `action_kind_for_tool("Wait")` is None; the vocabulary is the six ActionKinds. |
| `expired_approval_is_invalid` | An approval past its expiry is invalid. |
| `scope_change_invalidates_approval` | An approval for scope "promote" is invalid for scope "network". |
| `prefix_without_separator_does_not_match_siblings` | A prefix `src` allows `src` and `src/lib.rs` but denies `src-evil/evil.rs` and `srcx` (segment-boundary match). |
| `empty_path_prefix_is_rejected_or_matches_nothing` | A profile with `paths = [""]` fails to load; a hand-built profile with an empty prefix denies every path. |
| `unparseable_expiry_is_expired` | An approval with `expires_at = "zzz"` is invalid (Expired); a well-formed far-future expiry is still valid. |

The gate's own unit tests (`crates/tsr-policy/src/gate.rs`) add `dispatch_recheck_uses_the_single_approval_predicate`,
which pins that the dispatch re-check goes through `approval::validate` — the one implementation of the
approval predicate — rather than a second hand-written copy that could drift.

Run it with:

```sh
export CARGO_HOME="$PWD/.cargo-home"; export PATH="/usr/local/cargo/bin:$PATH"
cargo test -p tsr-policy --test adversarial
```