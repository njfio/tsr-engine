# Independent review of fixed12 offline analysis

Reviewed 2026-10-18, read-only except this review document. No candidate execution, inference, harness changes, cleanup, or live-run preparation.

**No numerical or causal-interpretation correction required.** The eleven-pair table and terminal-cause table agree with the saved results, diagnostic metadata, and immutable SQLite evidence. The 22 classified arms comprise two successes, one oracle rejection, two driver prerequisite blocks, five edit-precondition failures, nine truncations, one tool-shape failure, and two token-bound stops. The interrupted and unlaunched arms remain separate. Three final-oracle completions are 13.6% of classified arms and 12.5% of assigned arms. Seventeen classified arms stopped on edit/interface/token issues.

Formal denial check: all **17 available native result records** have empty `denials`; the five edit-precondition process failures have no native result record. Across **all 22 exported SQLite stores**, there are **zero denial-tag events**. Pairs 1 and 11 instead have standalone `stop:blocked` driver-prerequisite closures. The draft correctly distinguishes these from formal commit denials and does not infer the blocked responses' counterfactual success. Pair 3's Rules arm passed the prerequisite sequence and later failed an edit precondition.

The nine truncation diagnostics each record `finish_reason=length` and missing tool calls. The single tool-shape failure records multiple tool calls. The draft appropriately retains uncertainty about upstream causes and the composite stale/oversized edit error.

Pair 8's five settled reservation IDs per arm were joined to ledger usage: Rules used 104,447 tokens, leaving 95,553; its 99,099 bound exceeds that by 3,546. Record used 104,011, leaving 95,989; its 99,119 bound exceeds that by 3,130. These match native generator-token counters. The subsequent encode-stage diagnostics have no reservation ID: token availability, not financial capacity, stopped dispatch.

Independently reran bounded `du -sk` for every listed workspace/build directory. Every row matches the inventory: 20 build directories total **70,793,314,304 bytes / 65.93141174316406 GiB**; 22 whole workspaces total **70,888,665,088 bytes / 66.02021408081055 GiB**. The per-pair rounded table matches. These overlapping allocation estimates must not be added or treated as guaranteed recovered APFS space. No deletion was performed or authorized by this review.

The PRD distinguishes M3's at-least-150-task, baseline-interval, selection/tension criteria from the minimum 60 held-out tasks for the first learned release. The draft preserves that distinction, the two incomplete-run admission provenance caveat, and the lack of an M3 acceptance or gate-superiority conclusion.
