"""Independent receipt-data checker. Never imports or executes generated Rust code.

Input is exact current condition, prior stdout and current stdout extracted from
an authenticated native receipt. This function supplies no receipt authenticity,
permission or final acceptance; the caller must bind those separately.
"""
import itertools
import re


def ids(raw):
    if raw == "":
        return []
    if not isinstance(raw, str) or len(raw.encode()) > 2048:
        raise ValueError("bounded UTF-8 output required")
    # Exactly one ID line; no debug prose, empty fields, signs or trailing material.
    text = raw[:-1] if raw.endswith("\n") else raw
    if text == "":
        return []
    if re.fullmatch(r"[0-9]+(?:,[0-9]+)*", text) is None:
        raise ValueError("exact ID line required")
    result = [int(x) for x in text.split(",")]
    if any(x > 2**32 - 1 for x in result):
        raise ValueError("u32 ID required")
    return result


def assess(condition, previous_stdout, stdout):
    capacity, body = condition.split("|", 1)
    capacity = int(capacity)
    notes = [(int(i), int(t)) for i, t in (x.split(":") for x in body.split(","))]
    if not 0 <= capacity <= 2**64-1 or len(notes) > 8 or len({i for i, _ in notes}) != len(notes):
        raise ValueError("invalid frozen condition")
    previous = set(ids(previous_stdout))
    output = ids(stdout)
    available = dict(notes)
    k = min(capacity, len(notes))
    coverage = min(k, len({t for _, t in notes}))
    valid_sets = [set(v) for v in itertools.combinations(available, k)
                  if len({available[i] for i in v}) == coverage]
    best_retention = max((len(v & previous) for v in valid_sets), default=0)
    facts = {
        "cardinality": len(output) == k,
        "unique_available_ids": len(set(output)) == len(output) and set(output) <= set(available),
        "coverage": set(output) <= set(available) and len({available[i] for i in output}) == coverage,
        "maximum_feasible_retention": len(set(output) & previous) == best_retention,
    }
    return {"facts": facts, "finite_selection_valid": all(facts.values()),
            "repeatability": "not established by one returned output",
            "scope": "receipt data only; not source review, authority or qualitative fulfillment"}
