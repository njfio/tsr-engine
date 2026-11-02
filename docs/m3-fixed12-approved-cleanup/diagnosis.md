# Approved cleanup: read-only Docker diagnosis

No deletion, restart, stop, unmount or container removal performed. Cleanup remains pending; it was not denied by automatic approval review.

At2026-10-18T02:16:16Z, all11,825 open references were held by PID90000 (`/Applications/Docker.app/Contents/MacOS/com.docker.krun`, child of Docker backend PID89139). Every reference was a read-only directory descriptor, not an open regular build file. They cover six of the20 approved targets: arms15,16,17,18,19,21. Of the referenced names,3,359 still exist and8,466 do not. Exact paths and descriptors are saved in `open-reference-diagnosis.json`; per-target counts are in `open-reference-target-summary.json`.

All65 containers were inspected for status and mount metadata only. None, running or stopped, has a mount source overlapping any approved target (including ancestor/descendant overlap). There is therefore no identified experiment-owned container to stop or unmount. No unrelated contents, logs or environment values were inspected.

Four containers are running:

| Container | Workload/mount | Restart policy | Global-restart impact |
|---|---|---|---|
| laughing_turing | tsr-rust:1.90, `/Users/n/Developer/tsr-engine-reflect` bound to `/work` | no | Interrupts another TSR workspace; automatic recovery must not be assumed |
| postiz-redisinsight | RedisInsight, named volume | always | Service interruption |
| postiz-postgres | PostgreSQL17, named volume | always | Database connection interruption |
| postiz-redis | Redis7, named volume | always | Redis connection interruption |

The evidence is consistent with retained VM filesystem directory references after completed experiments. It does not prove absence of every VM-internal user or establish that ignoring the descriptors is safe. No privileged VM inspection, cache flushing, process killing or unrelated-container execution was attempted.

No safe experiment-scoped release action was identified. If release is required, the concrete broad action identified is restarting Docker Desktop and then repeating the exact active-use check; this interrupts all four running containers. Its success in releasing references must be verified, not assumed. A restart permission question was already presented before this follow-up diagnosis; it remains unanswered, not denied, and no restart was attempted. The latest instruction expressly prohibits restarting Docker or stopping unrelated containers, so this report does not authorize that action. Owner-managed release during a suitable maintenance window is also possible.

The20-path deletion scope and15,768 protected hashes remain unchanged. No partial deletion of the14 targets without observed handles was attempted. Disk-bound native integration remains paused below the200GiB guard.
