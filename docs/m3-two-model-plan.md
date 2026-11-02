# Fixed synthetic two-model comparison

The authorized schedule is six fresh independent attempts, alternating exact catalog IDs `deepseek-v4.1-flash` and `glm-5.3-flash`, three each. Every attempt uses record mode, the same known-answer task, profile, preparation, source snapshot, v3 action-specific schema and existing transport. No corpus expansion follows this experiment.

Limits: 20 inference requests and 1,800 seconds per attempt; 120 inference requests total; $5 additional within the $100 aggregate authorization. The launch ledger is $0.231086 settled with no outstanding reservation. The existing conservative $2 reservation can stop dispatch before the full $5 allowance is spent. Ordinary recorded model failures count and advance; unknown/internal failures, uncertain accounting, containment or evidence integrity halt. There are no retries or post-failure interface changes.

The catalog read on 2026-10-17 lists chat-completions support for both exact IDs. Advertised discounted input/output USD per million tokens: DeepSeek 0.085809/0.343236; GLM Flash 0.082500/0.275000. List rates are 0.15/0.60 and 0.15/0.50 respectively. The unchanged transport checks current catalog prices before each inference and records provider-settled charges.

Both use max output 16,384, the same required propose tool and schema, and omit temperature and reasoning settings. The catalog does not establish identical provider-default sampling or routing. No unsupported setting is silently substituted; unsupported request errors halt if billing is uncertain. Existing transport routing defaults remain unchanged and do not independently prove the upstream served model identity.

Native full results require independent event-root/CAS/receipt checks; visible test success alone never counts as completion. Settled schema failures and bounded known exact-edit precondition failures are retained. Native budget/blocked endings are retained as rejected outcomes. Other unexpected endings halt for review. Partial failures have no invented native result or oracle receipt.

Validation before dispatch: five comparison lifecycle/budget tests; five existing v3 adapter tests; replay of preserved schema and stale-digest native exports; read-only initial runtime/accounting/freeze checks. Independent review is required before dispatch. The experiment is synthetic, trivial, and too small for comparative performance or real-corpus acceptance claims.
