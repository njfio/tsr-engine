# Independent public-desire separation review

Reviewer: separate `public_desire_review` agent, read-only. No provider/target execution.

Initial findings:
1. P1: replay seeded only from current acceptance publication; omitted parents made
   historical desire admissions fail. Fixed by retaining original public parent and
   relationship-parent origin snapshots in append-only admissions.
2. P2: withdrawal excluded only directly withdrawn parents, not transitive descendants.
   Fixed by a finite fixed-point eligibility fold; self references remain valid.

Re-review: both findings resolved by inspection; no additional material correctness
or privacy defect found within existing-parent development scope. Origins contain
previously public fields, review notes remain excluded from projections, and no
execution authority is created. Reviewer requested acceptance requirement revision
AFTER admission plus restart test; root added that explicit regression.

Scope caveat: PublicDesire is supported in ordinary propose, not the narrow advisory
reconsider branch. Independent reviewer attribution is recorded, not authenticated by
name alone. No empirical model behavior is claimed. Root owns actual test results.
