# Remaining supplementary campaign readiness

Planning only: no qualification, cleanup, paid calls or historical retries. Current free space 212.74 GiB. The exact 35-directory cleanup manifest remains unchanged and pending approval. No future launch-freeze files exist.

| Batch | Candidates | Allocation | Initial free space | Predecessor |
| --- | ---: | ---: | ---: | --- |
| Daemon | 6 | 56 GiB | 256 GiB | completed context |
| Wire/protocol and roots | 4 | 40 GiB | 240 GiB | daemon |
| Predicate validation/parsing | 4 | 40 GiB | 240 GiB | wire |
| Structural filtering/serialization/language | 3 | 32 GiB | 232 GiB | predicates |

All batches use separate namespaces, at most two workers and the reviewed module-v2 binary, SHA-256 7ee73bc691b8d90d2de8fc9c7615ebaf4a5a6afee5b368303ba387603efaadc5. The four wire/root candidates are compatible with the older resolver too; using the verified module-v2 explicitly for the new campaigns keeps provenance consistent without relabeling previous runs. Every existing frozen engine remains unchanged.

The budgets use 8 GiB per retained qualification copy plus 8 GiB for screening/evidence overhead. They are conservative planning allowances based on the measured 5.104 GiB daemon full-suite build, not measured supplementary peaks or hard quotas. All four allocations total 168 GiB. Estimated cleanup would give about 391.45 GiB free, leaving about 223.45 GiB after all allocations. APFS shared extents and concurrent usage may reduce this margin: every cohort must pass fresh preflight, and the 200 GiB live guard remains unchanged.

All 17 descriptors occur exactly once in the stored source inventory and are distinct by file/function. Deleted original-line positions from 114 exported prior RTS task records do not overlap their spans. Source compatibility checks resolve all 17 with module-v2. These checks neither execute the candidates nor establish qualification. Exact source revision and selectors remain frozen; refresh dedup against new outcomes before each activation. Shared oracles and related functions (especially predicate scanner/validator) remain correlated, not independent statistical evidence.

The three new preparation freezes each bind 75 file hashes. Their controllers require a separate, independently reviewed launch freeze: it must bind the preparation freeze hash, preserve immutable input hashes, bind the completed predecessor's complete/final/audit and current corpus summary, and provide the audited admission count. Those predecessor outcomes do not exist yet. Missing or unreviewed activation refuses before state creation. Observers have separate paths and never launch successors.

## Path to the task threshold

Start with the already prepared six daemon candidates, then the four wire/root candidates if authorized activation and storage checks pass. These ten opportunities are the clearest bounded path to eight additional admissions; even perfect yield from the first six reaches only 148. If fewer than eight of the first ten qualify, use the seven remaining predicate/structural candidates in their frozen batches. Preserve every rejection/unviable outcome; no retries or counted compatibility checks. Stop launching unneeded batches once the corpus objective is satisfied unless further work is authorized. There is no guarantee of eight successes; all 17 succeeding gives a ceiling of 159.

## Regression and review evidence

Independent review found M3-SUPP-01: a broad budget substitution accidentally changed SHA-256 helper/key names to sha240/sha232 in the new preparation. Added nonmocked digest and engine-plan tests reproduced three failures before the fix. Restored SHA-256, refreshed the preparation freezes and verified each actual plan against the real module-v2 binary. All 41 small controller tests now pass. Before/after logs are docs/m3-validation/m3-supp-01-before.log and m3-supp-01-after.log. No candidate code ran.

## Decisions and acceptance gaps

The pending exact cleanup approval is the current user decision; without it, more storage is needed (at least 43.26 GiB additional free space for the first batch). No new paid-model or private-data access is needed for these local qualifications. Future activation requires terminal reconciliation and independent review, not relaxed guards.

At 142 admissions, eight more legitimate admissions are needed for 150. Reaching 150 does not supply M3 systems 1/2 baseline comparisons with intervals or selections/tensions evidence. The existing two observed pairs in separate cohorts cannot be pooled into a performance claim. Paid paired evaluation is outside this task; evaluation design, authorized execution and its evidence remain a separate work item. M4/M5 held-out and learned-controller criteria also remain separate.
