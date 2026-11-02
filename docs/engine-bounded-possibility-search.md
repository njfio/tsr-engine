# Bounded diverse possibility search

The prior temperature-zero request 20 returned inability despite independently
established feasible partial progress. This is not proof that sampling variation
will recover it. The independent witness is not supplied to generation.

`engine-directions.py --two-stage` accepts an explicit named search:

```sh
python3 scripts/engine-directions.py --directory target/support-live/run \
  --model qwen2.5-coder:32b --endpoint http://127.0.0.1:11435 --calls 4 \
  --batch exact-effects-v1 --batch-calls 6 --two-stage \
  --possibility-search diverse-possibilities-v1 \
  --possibility-temperature 0.7 --possibility-seed 7301 --possibility-samples 3 \
  --stop-file target/support-live/recovery.stop
```

This reuses the existing six-call grant, of which request 20 already consumed one;
it grants no additional calls. At most three possibility samples can be dispatched
across restarts of this named search. Every possibility, assessment and refinement
request also consumes the global call allowance. Assessment can proceed after the
possibility sample limit if a proposal is pending and global budget remains.

The search configuration is append-only and frozen by name. Possibility sample N
uses seed 7301+N-1 and temperature 0.7. Every started receipt records stage and full
sampling options; the hashed request envelope also contains those options. Seeded
configuration aids reproduction but does not guarantee identical GPU/provider
output across runtime versions. Assessment and refinement retain temperature zero
and their existing contracts. There is no selection among hidden samples: each
returned proposal goes through the existing validation and controller in order.

An inability remains preserved and can lead to the next separately budgeted seed.
The search stops at its sample limit, global budget, operator stop, unresolved
transport outcome, or existing repeated-validation-error guard. Pending immutable
possibilities still require assessment against their original root; stale state
retires them. Unknown dispatches consume samples and calls, never refundable.
Existing action-effect identity checks still reject wording-only novelty and
mechanically unchanged account offers. Sampling grants no authority and makes no
predicted action safe by itself; controller and standing-loss checks are unchanged.

Tests: `python3 -m unittest discover -s scripts -p test_engine_directions.py`
(53 passed; `target/diverse-sampling-tests.log`). Added controls cover frozen search
configuration, finite bounded options, stage separation, seed restart identity,
unknown-slot consumption and end-to-end inability preservation without application
or budget reset. Existing tests cover action identity, stale state, interruption,
immutable assessment and no-progress refusal. No protected tests were changed.

Full coverage plus four-word brevity remains an actual conflict. Neither a useful
partial action nor exhaustion of this search establishes full satisfaction or
objective impossibility. Source truth and qualitative understanding remain outside
these notebook predicates.

## Live result

Requests 21, 22 and 23 used seeds 7301, 7302 and 7303 at temperature 0.7.
All returned explicit possibility-stage inability. No frozen possibilities,
assessment calls, controller actions or changes to standing wants resulted.
The driver exited `possibility_search_exhausted` with root
`3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`,
observation [0.5, 0.8, 0.0] and two prior action returns. The inability files,
request envelopes and `local-directions.jsonl` remain under
`target/support-live/run`; terminal output is
`target/support-live/diverse-search-live.log`. Their response hashes are:

- 21: `4602318b7ed46f4a2ad0d85379f730eb01d865b1ef94941bb006759355a02d86`
- 22: `93463527e79d248ba055924882e286ede752abeb085d193245ed2828f850fff4`
- 23: `68d8c5bdc071bd4e1c3391904f24f37b7a1bc5bf2c46b9d79c27b0670a033ffc`

Three of the five remaining slots were consumed; two remain unused. The named
search's three-sample limit is exhausted and is not reset or renamed. Provider
PID 95759 was terminated; its session exited zero and listener was absent.
No paid/external calls, target edits or publication occurred. Unknown request 13
and previous inability receipts remain preserved.

Diverse sampling did not recover useful alternatives in this state. The third
response generalized prior failed accounts into absence of alternatives; that
model assertion is not a controller proof. The independent partial-progress
witness still stands. Further unchanged requests are not justified. The remaining
core limitation is useful possibility generation from residual desires and prior
rejections, not the already corrected exact-effect selection gate. An evidence-
grounded next investigation is whether repeated rejection context causes this
incorrect generalization; any correction must retain action-effect distinctions,
all raw evidence, and avoid supplying the feasibility answer.

## Read-only conflict status

After the live experiment, `tsr engine --directory RUN direction-request` gained
`constraint_status`: a scoped coverage-versus-brevity certificate over active
original desires. It unions required account IDs, counts each source once, and
compares the minimum account length with active brevity budgets. Only inspected
source lengths are used; an unread validated nonempty source contributes one
word, and the required title contributes one. No hidden source length or content
is exposed. `proven_joint_conflict` means joint satisfaction is impossible under
this representation; `joint_conflict_not_established` does not assert feasibility.
The result binds the observation revision and names original desire indices in
the returned standing_desires. Statuses, selection, learning and authority remain
unchanged. Useful partial progress remains possible.

The driver excludes this operator certificate from model context. It was not
present during the sampling experiment and is not used as an authored next action.
Tests cover known/unread boundaries, changed observations, withdrawn constraints,
unchanged controls, overlapping originals and refinement conservation. Independent
review cleared the certificate after correcting wording to preserve existing
withdrawn statuses. A test initially constructed an invalid parentless refinement;
it was corrected to use a valid original account plus a parent-linked refinement.

Final checks: CLI library 97 passed, one existing diagnostic ignored; driver 54
passed; Clippy with warnings denied, build, Python compilation and diff checks
passed. Logs: `target/diverse-search-cli-tests.log`,
`target/diverse-sampling-tests.log`, `target/diverse-search-clippy.log`,
`target/diverse-search-build.log`. The real read-only request in
`target/support-live/post-diverse-search-state.json` reports minimum five words
versus maximum four with the original root unchanged. Corpus accounting remains
151 qualified tasks; eligible independent mature episode count remains unknown.
