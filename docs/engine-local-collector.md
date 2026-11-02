# Bounded foreground local observation collector

`scripts/engine-observe.py` connects an explicit local-source allowlist to the existing
versioned `direction-observe` path and ordinary direction generation. It does not
discover files, fetch URLs, install a daemon, create credentials, or infer permission
from source text. Rust remains responsible for observation admission, current-state
binding, desire selection, action bounds and measured returns.

## Explicit inputs

The operator supplies a fixed JSON mapping, an existing initialized notebook run,
and (only for generation) an installed local provider and frozen call allowance:

```json
{
  "root": "/absolute/canonical/path/to/admitted-sources",
  "max_bytes": 512,
  "sources": [{"id": "return", "topic": "decisions", "path": "return.txt"}],
  "continuation": {
    "model": "qwen2.5-coder:32b",
    "endpoint": "http://127.0.0.1:11435",
    "calls": 10
  }
}
```

The source is the complete UTF-8 contents of each mapped file, not an automatic
summary/extraction from a larger repository. Maximum size is 1..512 bytes per file,
with 1..32 unique files/notes. Note IDs and topics must already be admitted. Set
`continuation` to null for collection alone. No other paths are enumerated or read
as sources. The mapping itself is a bounded 16-KiB file. Configuration, including
provider/call allowance, is frozen in the run's collector journal.

```sh
# One foreground collection, no model call:
python3 scripts/engine-observe.py --directory RUN --mapping mapping.json
# Bounded foreground collection and ordinary continuation when current wants reopen:
python3 scripts/engine-observe.py --directory RUN --mapping mapping.json \
  --polls 2 --interval 30 --continue
# Cooperative stop; remove only when explicitly resuming the same authorized run:
touch RUN/collector.stop
```

There are at most ten polls per invocation and 1..60 seconds between polls. Defaults
are one poll and one second. The model budget is the existing **whole-run** journal
allowance, not an allowance per poll; the collector cannot add batches or reset it.
Model inventory is checked locally; no model is downloaded. Inference remains bounded
by 240 seconds per call. The collector neither starts nor installs the local service.

## Read, provenance and recovery boundaries

The reader opens only the canonical configured root and explicitly named children.
It rejects symlinks, traversal, hardlinked/nonregular files, oversized or empty text,
and files/directories replaced or changed during reading. Root, descriptor and named
path identities are checked before/after. This is a stable **single-file** snapshot;
multiple files are not a transactional snapshot, and a later change is observed on a
later poll. A read failure stops collection/continuation; it is not proof that the old
observation is still current and does not synthesize a new source value.

The collector records source path, content SHA-256, inode/device, size and modification
time locally. A digest binding that snapshot, mapping and expected engine root becomes
the observation reason. Paths and filesystem metadata are not added to model context.
Source text remains untrusted evidence, never commands or admission authority.

`collector.jsonl` is locked and fsynced. A submission intent is durable before Rust is
called. On restart, matching observation provenance in the engine confirms a lost
acknowledgment. An exactly unchanged engine head establishes no append occurred;
resubmitting that no-op is safe. An advanced head without the expected observation is
uncertain and stops for inspection. No journal is silently repaired or reset.

Unchanged file versions plus unchanged evidence revision suppress submission. If
another writer changes engine evidence, the collector rechecks its mapped source;
the explicit mapping remains authoritative for those notes until collection is stopped.
Stopping a collector does not undo observations already committed.

Stop checks precede source collection, observation submission and inference dispatch.
A stop received while inference is in flight cannot cancel that request, but its
response is preserved without applying an action. The reserved call remains consumed;
restart never silently applies its saved proposal. Transport uncertainty remains in
the existing inference journal and blocks duplicate calls. Interrupting the foreground
process is also supported through the same durable reconciliation rules.

## Verification

Thirteen collector tests and thirteen direction-driver tests pass. They cover unchanged
restart, revisions, frozen mappings, unknown IDs, stable-read races, root/directory/file
replacement, symlink/hardlink/FIFO/traversal/size rejection, lost acknowledgments,
uncertain advanced heads, cooperative stopping during preparation and inference, and
budget-preserving restart of a stopped response. Logs: `target/collector-checks.log`
and `target/collector-driver-checks.log`.
The final JSON result is written to stdout; generation progress goes to stderr.

Independent review identified the immediate pre-dispatch stop checkpoints, which were
added with tests. A first integration attempt opened ancestor directory descriptors
and was denied access to `Documents`, before any observation submission. The reader
was narrowed to the explicitly allowed root with pre/post identity checks; normal
sandbox collection then succeeded. No filesystem permission or security setting was
changed. Review found no blocker in the narrowed reader or stop/recovery changes.

This collector closes manual submission for configured local note sources. It does
not provide arbitrary source discovery, semantic truth verification, a global atomic
filesystem view, comprehensive preference revocation, or unlimited execution.

## Live foreground verification

The only watched input was
`target/collector-live/sources/return.txt`, mapped to existing note `return` in topic
`decisions`, with a 512-byte bound. The source was a disposable note file describing
the implemented local observation behavior. Initial state came from an isolated copy
of the completed `target/capacity-live/run`; its original receipts were preserved.

Initial collection bound the mapping without changing the engine. A second invocation
verified exact unchanged state and deduplication. Then the foreground collector ran:

```sh
python3 scripts/engine-observe.py --directory target/collector-live/run \
  --mapping target/collector-live/mapping.json --polls 2 --interval 30 --continue
```

Poll 1 recorded unchanged state. During the interval the disposable mapped file was
atomically replaced with a revised note. Poll 2 detected its new content hash and file
version, submitted one changed observation, and invoked ordinary generation without
a manual `direction-observe` command, next task, candidate, or reference answer.

The engine reopened inspection/account coverage to 2/3. Four additional installed-local
model calls followed: one invalid offer was rejected (uninspected evidence/duplicate
action), then inspection, refinement review, and recomposition. The failed offer is
retained. Final status is `satisfied` with `[1,1,1,1,1]`, five cumulative action cycles,
and unchanged standing wants/authority. Nine inference intents exist in total, including
the original five; the frozen ten-call allowance was not raised. No call was uncertain
or duplicated; each used the existing 240-second timeout.

After stopping temporary Ollama PID 33959, restarting the collector with `--continue`
returned `unchanged` and exactly identical engine state. It made no inference or new
observation. The prior inference journal remains an exact prefix, and its JSON receipts
remain byte-identical. No background collector or provider service remains running.

Local evidence: `target/collector-live/` mapping, source, baseline/unchanged/restart
results, `live-result.json` (original mixed progress/result stream), `live-final.json`
(extracted final object), `verification-summary.json`, and both journals in `run/`.
The stdout/progress separation fix was verified with its regression test and the final
clean-JSON restart; it does not change observations or generation. The source content
and raw model bodies remain ignored local evidence rather than committed responses.

Final root: `fe642cae2db0c98973a03f748be7cac5bf83cdea5ee3b82b5bd5ee15dcc971f8`.
Collector journal SHA-256:
`eb6d4c7333be8ca956d6d4e1d32612a0a7a6d816f07f73baaa2c1243cb450bcc`.
Inference journal SHA-256:
`8ed2e8c701b1b698fb06d4666c562ecbf0f5c9aac65d803308a0795705438de2`.

This closes automatic submission and continuation for explicitly mapped local note
sources. The result still measures coverage/brevity, not source truth or qualitative
understanding. It does not authorize arbitrary observation discovery or external action.
Final independent review confirmed the single mapped input, one changed observation,
four additional calls, preserved original receipts, satisfied result and unchanged
restart. No blocker remained for the intended local commit.
