# Conflicting-wants continuation (2026-10-22)

The local admission starts at `a07134b`, on preserved notebook state
`3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`.
It permits at most three new local calls, including assessment, each with a
240-second timeout, 32768 context setting and 2048 output-token cap.
Installed `qwen2.5-coder:32b` is pinned to
`4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`.
No external inference or source transmission is admitted.

The original wants remain full note coverage and at most four account words.
The existing controller proves joint conflict from admitted note constraints;
that does not prohibit useful partial progress. Its measurements are ID coverage
and word count, not truth or understanding. No feasibility witness or authored
replacement candidate was supplied to the generator.

## General feedback correction

Request 30 ended in inability without an action. Independent review found that
historical forecast diagnostics retained positional direction numbers after
removing their referenced proposal, and incorrectly told the generator to change
an action when its immediate forecast disagreed with a preserved component.
A planned inspection can have unchanged immediate predicates and a separately
assessed terminal composition; a forecast mismatch alone does not disprove it.

The driver now binds each diagnostic to its historical request, state, file hash,
receipt-verification status and original action/continuation. It explicitly
separates forecast correction from a mechanically unchanged account. Missing
legacy receipt hashes are not represented as verified; malformed present hashes
fail closed. All original evidence remains untouched.

Request 31 uses identical system instructions, schema, model, temperature 0.7
and seed 7304. Only `unchanged_effect_feedback` differs. This is a controlled
input comparison, not proof of why a stochastic model responds as it does.

Evidence is under `target/support-partial-qwen32/`: frozen admission and before
state, correction admission, envelope comparison, driver outputs and tests.
Requests and terminal records remain in `target/support-live/run/`.

## Checks

`python3 -m unittest discover -s scripts -p test_engine_directions.py`: 65 passed.
The new checks cover historical provenance, unchanged-state repeatability,
stale-root exclusion, preservation of the input context and empty-digest rejection.
Python compilation, `git diff --check`, and the current Rust CLI build passed.
No Rust/controller behavior or protected tests changed. Independent review
identified the two defects and reviewed the correction.

## Terminal outcome and next gap

Two of the three admitted calls dispatched: requests 30 and 31 both terminated
with inability. Request 31 cited unavailable detailed-note content and constraints.
There was no assessment, selected action, observed return or new outcome learning.
The final state root, standing wants and conflict certificate equal their frozen
values. The feedback repair therefore did **not** demonstrate provider recovery.
One slot is unused; it is not silently expanded into another two-stage attempt.
The temporary Ollama server was stopped after both driver calls were terminal.
Historical unknown request 13 remains consumed and unreclassified.

This is a capability limit of this recorded model/contract combination, not proof
that useful partial action is impossible. The schema still admits composition of
known evidence and inspection with a dependent plan; unavailable note text is
intentional, not permission to reveal it before inspection. No actionable result
was substituted by hand. Actual token usage is unknown in the existing inability
metadata, despite bounded configured context/output and zero external charge.

The next core gap remains generation of useful possibilities under partial
observability and conflicting wants. A useful next investigation is a separately
reviewed information-acquisition contract: establish whether requiring a detailed
terminal plan before inspecting unknown evidence unnecessarily couples intuition
to forecasting. Any change must retain standing-loss accounting and bounded
information value, rather than automatically selecting inspections or exposing
unobserved source content. This pass does not change that policy or establish
that it caused the inability. Another identical provider probe is not justified.
Operational calibration remains inactive; no mature evidence was created.
