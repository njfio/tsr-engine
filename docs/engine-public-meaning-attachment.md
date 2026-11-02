# Reviewed public meaning in the existing native choice path

This local bridge connects the qualitative adapter to `adaptive/provider.rs::proposal_request_inner` and the existing `evaluate_proposal` / project-cycle preview and advance path. It adds **no selector, weights, criteria, action kinds or execution authority**.

## Binding and replay

A local operator supplies an attachment with current target root, canonical absolute source-store directory, exact source root, public-context digest and bounded independent review attribution. The target must already be configured for ordinary native choice. The source must be a distinct existing qualitative store with reviewed public meaning or relational guidance, no pending review and no withdrawn parent. The source is not copied into a fabricated coding project.

Attachment metadata is append-only in the target's existing Engine event/CAS store. Only public original/current meaning, active lineage, applicable guidance, source attribution, constraints and an explicit advisory role enter the optional `public_meaning` field of the **normal provider Request**. Source filesystem paths and private review notes remain local. Acceptance references from the source do not become target criteria. Initial context still carries no observed-return claim.

The attachment event changes the target root, so older responses are stale. Request construction, preview and selection recheck source root and content digest. A source write reservation is held during those operations so concurrent source changes cannot occur while the context is consumed. If the source changes or disappears, subsequent requests/evaluations fail closed; detach or explicitly re-review/re-attach. No source history is mutated by the read reservation. Exact attachment replay fails without another event. Fresh-process reconstruction preserves the same attachment and ordinary request.

An explicit root-and-attachment-ID-bound detach removes future guidance without deleting either history or changing original task obligations. Withdrawal in the source invalidates the old attachment rather than silently treating the withdrawn desire as active.

## Runnable local interface

Build both existing native entry points offline:

```sh
cargo +1.90.0 build -p tsr-cli --bin tsr --bin tsr-qualitative --offline
```

On a source with independently admitted meaning:

```sh
target/debug/tsr-qualitative SOURCE_DIRECTORY snapshot
```

This returns `source_root`, `context_digest` and the public context. Obtain the configured target's root through its ordinary request:

```sh
target/debug/tsr engine --directory TARGET_DIRECTORY request
```

Prepare `ATTACH.json` locally with exactly:

- `target_root`: current target request root;
- `source_directory`: canonical absolute local source directory;
- `source_root` and `context_digest`: exact snapshot output;
- `reviewer` and `reason`: bounded independent local attribution.

```sh
target/debug/tsr-qualitative TARGET_DIRECTORY attach ATTACH.json
target/debug/tsr engine --directory TARGET_DIRECTORY request
```

The returned attachment event ID and a fresh target root are required for `detach`, whose JSON fields are `target_root`, `attachment`, `reviewer`, `reason`. These commands do not invoke a provider, execute a target command or admit model-generated code. A local attachment is not an export grant: combined contexts require their own allowlist/scope review before transmission.

## What selection does—and does not—consume

The ordinary generator request now sees the reviewed meaning alongside the unchanged task context. Generated alternatives then go through existing action validation, preview and selection. The shared kernel still derives estimates/support from the task adapter's actual dimensions and history. Qualitative meaning supplies **generation guidance**, not fabricated numerical dimensions or success support. Unknown actions remain deferred; text cannot configure a new action or make it eligible.

The interface constraint remains real: when there is no independently configured action/acceptance adapter, rich desires alone do not supply the dimensional observations needed for generic action selection. Such a qualitative-only state remains preview-only. This patch connects an existing legitimate action adapter rather than inventing proxy predicates for the fictional desires.

The qualitative fold and coding public-desire fold are still separate adapters sharing storage and types. This bridge removes their isolation from the normal request/choice path; it does not claim a complete fold consolidation, continuous desire field, autonomous emergence or improved choice. No scenario-specific rules or fictional IDs occur in the bridge.

## Deterministic evidence

All new decisions are authored fixtures, not model behavior. The four attachment tests verify normal request/preview/existing selection, unchanged criteria and scores, cross-root and stale/withdrawn/unreviewed rejection, replay/restart, explicit detach and source locking. They commit only a synthetic pending candidate and never execute a command.

An additional test in the existing project-cycle suite exercises `proposal_request` → `project_cycle_preview` → `project_cycle_advance` with an attached reviewed context and authored patch. The ordinary selector commits it while configuration and acceptance revision remain unchanged; no target execution event is created.

No external calls, exports, live-generated adoption or target executions occurred. Campaign stays 191 calls / 57 groups / $0.160913, with six-call permission still pending. Prior provider preparation binary pins are historical: any eventual live admission must freeze current binaries and payloads afresh, without rewriting those prior manifests.

Final affected verification: **4 attachment tests + 3 existing provider tests + 14 project-cycle tests passed**, no failures or skips in those groups. The cycle group includes the new exact-path bridge test and completed in 75.64 seconds. Both `tsr` and `tsr-qualitative` rebuilt offline successfully. Independent final inspection found no additional blocking concern; source-guard lifetime is supported by code inspection plus the direct locking regression. No whole-workspace result is claimed.
