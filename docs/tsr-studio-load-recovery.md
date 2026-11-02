# TSR Studio load and save recovery

The observed load errors were real connection refusals: the executor-owned Python service did not survive the executor lifecycle. A current-user launchd session job now runs the loopback service independently. `ui/engine/service.sh` checks HTTP health, refuses duplicate listeners/live unhealthy jobs, and can replace its own stopped registration. This is not a login installation or a claim of automatic crash recovery.

The frontend now prevents overlapping project polls, aborts a stalled fetch after 30 seconds, rejects stale selected-project results, and clears only the read error after recovery. Action errors remain distinct. Draft text is not cleared by read recovery. The running service serves the current static files on the next page load; existing user tabs/drafts are not forcibly refreshed.

## Actual Pond state

The original Pond source has current snapshot `a4bf78a821ca52eaced2a49cfac0f9b8ad7d5864f2fa1c89f72b4d0e9e6bef89`, passing native browser receipt `1749f06121d1855b545131cba2a7dbfd76b0d476635cbfa70074ee9994904905`, and `review_required` completion. Its current relational parent projection is empty. This does not mean desires were deleted: `acceptance/guidance.rs` requires an open scope-bound admission before carrying published guidance across a changed source/decision. The UI had incorrectly thrown a save error after already appending the input.

Input now returns a truthful queued receipt when native guidance is unavailable. It neither invents contributors nor activates them. Start requires successful native admission before preparing any session, even if the financial gate later clears. The existing read model exposes active guidance, not a complete historical contributor view; a provenance-checked historical publication accessor remains needed for that view and seamless completed-project reactivation. Private requirements are not used as a substitute.

The user's message `game was fun, but tiny and ugly.  Make it really spectacularly beautiful` is preserved once in the append-only Pond UI journal under `93b1594e-7d10-42a7-b3ec-c7fa9fe7fd48`. It is queued, not model-consumed. Existing games and original run states are unchanged.

## Verification and limits

- Both actual project APIs return native state, messages, timeline and evidence independently of the financial gate.
- Safari visibly loaded TSR Studio at `127.0.0.1:8766/`, including Pond's real saved message, source/receipt, 347 campaign calls, $0.403253 known and $0.453253 committed, and one unresolved request. No Start action was clicked.
- UI regression includes temporary HTTP → native event → reload, duplicate input/start, queued inactive sources, unavailable accounting, and proof that queued inactive input invokes neither preparation nor launch.
- A browser fixture regression was added for transient API errors recovering without draft loss. Its new execution is blocked because the existing Docker daemon is unavailable; it is not claimed as passed. Earlier browser tests do not substitute for this new test.
- Independent review identified and corrected fetch-hang, service-health, queued-start and diagnostic-state issues.

Paid conversational integration remains blocked. Historical charge-risk acceptance is pending; no quarantine event or new paid request was applied. The generation-disposal repair preserves the unknown and does not clear this financial gate. No game redesign has executed.
