# Independent local M3 review

A separate read-only agent reviewed the implementation and re-reviewed the fixes.
It did not edit files, access credentials or invoke paid providers.

Resolved findings:

- Recompute CAS identities with the corpus raw blake3 digest, not the domain's
  length-prefixed event hash. Corruption, links and malformed keys are refused.
- Separate trusted provisioning from evaluated argv dispatch.
- Enforce required reads and named failing test sequencing in rules-mode broker runs.
- Preserve accrued token/tool usage when the broker becomes unavailable.
- Include failing-test identity and bounded accumulated observations in requests.
- Cover blocking stdin writes with the process broker deadline.
- Freeze dedicated test files and complete cfg(test) modules, including helper context.
- Isolate M4 verification temporary directories and test concurrent verification.
- Test actual Python reservation/settlement dispatch ordering, cap boundaries,
  unknown billing and malformed paid responses offline.

The re-review found no new authority escape. It identified a stale test expectation
following the persistent billing-halt addition: four Python tests passed, one errored.
The repaired test now verifies the halt persists and prevents a later inference
request (only catalog metadata is requested); all five pass. Outer Rules event source
means recorder, while inner payloads preserve Generator authorship; documented in status.

An earlier absolute-repository-path concern was withdrawn after checking `is_safe_repo`,
which accepts absolute paths and rejects empty/traversal paths. No workaround was added.

This review does not establish real corpus quality, provider integration, semantic
oracle soundness or complete container isolation. Live calls remain approval-blocked;
verified real-corpus tasks remain zero. See generated validation logs and acceptance
assessment for exact test scope and remaining limitations.

## Actual repository validation follow-up

The reviewer independently summed 96 full-suite summaries: 999 passed, zero failed,
15 ignored; the worker exited zero with untruncated capture. This applies to the
recorded lock and environment, not an arbitrary generic cargo-test invocation.
The reviewer found no new containment issue in --init or executable container-local
/tmp: no network, host privileges or extra mounts were introduced.

The captured cargo-mutants 24.11.2 descriptors have no names. Full-span structural
identities and required captured diffs address collision and reconstruction defects.
The reviewer identified repeated caught outcomes inflating task counts. A subsequent
regression now verifies duplicate outcome identities are rejected before a manifest
is emitted. Independent red/reversal results remain distinct from tool-reported labels.

Final read-only follow-up confirmed the duplicate-outcome rejection and actual unnamed
outcome/diff join regressions; no new material ingestion issue was found.

## Preparation and independent admission review

A subsequent read-only review required: create/export the cohort CAS, reject added
files (including target/), compare restored tracked bytes after all test runs, allow
unchanged documentation attributes, require completed full-red results rather than
Cargo exit 101 alone, and exercise a successful inert admission lifecycle. These
were implemented with bounded regression cases. Real native admission additionally
exposed standard Rust file-module identities; the checker now requires the trusted
plain module declaration and freezes its source rather than guessing a test suffix.
The stricter full-red protocol remains mandatory; the review's earlier interpretation
of PRD wording was superseded by the parent's explicit protocol-preservation direction.

Final follow-up review found no additional concrete blocker: both diff path sets are
validated against one tracked Rust file; completed test groups and infrastructure
failure checks protect the full-red gate; direct-module resolution requires frozen
trusted declarations. The reviewer did not execute tests or independently certify
worker containment; test and native-run evidence remain separately attributed.

The real core full-red run found an additional integration failure outside the
library screen. Follow-up review identified unlisted-test editing as a concrete
batch nonregression blocker. The fix captures all snapshot Rust test regions and
dedicated test resources, protects reachability declarations/attributes, and checks
before and after oracle execution. Review found the fix addresses the blocker;
inert regressions cover modification/deletion before dispatch and changes during
named/full execution, while allowing production-only edits. Native admission itself
was already protected by single-function patch scope and exact source restoration.
