# Independent campaign preparation review

2026-10-26, reviewer `/root/core_gap_review`, implementation by separate driver
and accounting agents. Source review found no remaining blocker after two repairs:
UTF8 campaign bounds now match actual transport encoding; recognized resource-stop
enums survive Driver.provider so final-call overruns cannot proceed as clean results.

Legacy16KiB admissions remain unchanged. ExplicitLuna v2 binds final2 stages32KiB,
perstage reservations/native180224token ceiling, sixcalls/fourgroups/240secondcall
and240000toolms limits. Shared campaign ledger binds trial admissions and one path,
locks/appends durable intent before effects, counts all trials against100calls40groups,
reserves50000microUSD perrequest under5Mtotal, retainsunknowns andblocksreplay/overrun.
This is operational conservative accounting, not a provider-guaranteed billing cap.
Officialmetadata preflight anddurable priceevidence precede Driver initialization.
No new credentials/settings or sourceclass expansion. Native review gates remain.

Reported completed verification:4 driver tests including fullnative v2 sixstage/fourgroup
Fake sequence;25continuation/envelope tests with5environment skips; focused native
resource-stop test passes;12campaign unit tests11pass1native skip; separate fullnative
campaignguard test passed51.416seconds with6fakecalls4fakegroups and no replay count.
No livecalls or targetexecutions were made by reviewers/implementation agents.
