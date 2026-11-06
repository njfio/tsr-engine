#!/usr/bin/env python3
"""Observation-stage coverage: can a planned stage resolve each criterion at all?

The corrected continuous-development test (docs/continuous-game-live-v2-result.md)
ran 36 control traces whose fixed six-key sequences reached neither the first fly
nor the far bank. Every gameplay finding was therefore Unknown, and Unknown is not
unmet: no Contrast and no observed success could form, so the evidence routes to
development were closed before the model chose anything.

This tool makes that visible before dispatch and after observation. It grants no
authority, calls no model and runs nothing.

* ``static``: each criterion declares ``min_inputs``, the operator's reviewed lower
  bound on inputs needed to reach a state where that criterion could be judged
  (with a short ``basis``). A stage is ``blocked`` when no planned sequence for a
  criterion is that long, or when a sequence exceeds the harness action bound.
* ``observed``: after observation, a criterion whose readout text is identical in
  every trace is ``invariant``: the stage did not discriminate anything for it.

Exit status from the CLI: 0 resolvable/varied, 2 blocked/invariant, 1 bad input.
"""
import argparse
import json
import sys

PROFILE = 'observation-coverage@v1'
# Ordered actions the current protected harness accepts (scripts/engine-browser-harness.cjs).
HARNESS_MAX_ACTIONS = 64


def actions_for(keys):
    """Probe actions for one trace: start click, keys, and a short wait between keys."""
    return 1 + len(keys) + max(0, len(keys) - 1)


def _criteria(coverage):
    if not isinstance(coverage, dict) or not coverage:
        raise ValueError('coverage must map each criterion to {min_inputs, basis}')
    for name, item in coverage.items():
        if not isinstance(name, str) or not name:
            raise ValueError('criterion name required')
        if not isinstance(item, dict) or set(item) != {'min_inputs', 'basis'}:
            raise ValueError(name + ': {min_inputs, basis} required')
        if type(item['min_inputs']) is not int or item['min_inputs'] < 1:
            raise ValueError(name + ': min_inputs must be a positive integer')
        if not isinstance(item['basis'], str) or not item['basis'].strip():
            raise ValueError(name + ': basis must say why that many inputs are needed')
    return coverage


def static(coverage, sequences, max_actions=HARNESS_MAX_ACTIONS):
    """Check planned sequences ({criterion: [[key, ...], ...]}) against declared needs."""
    _criteria(coverage)
    if not isinstance(sequences, dict):
        raise ValueError('sequences must map criteria to lists of key sequences')
    unknown = sorted(set(sequences) - set(coverage))
    if unknown:
        raise ValueError('sequences name undeclared criteria: ' + ', '.join(unknown))
    report = {}
    for name, need in sorted(coverage.items()):
        planned = sequences.get(name, [])
        if not isinstance(planned, list) or any(
                not isinstance(s, list) or not s or any(not isinstance(k, str) or not k for k in s) for s in planned):
            raise ValueError(name + ': each sequence is a nonempty list of key names')
        lengths = [len(s) for s in planned]
        over = [i for i, s in enumerate(planned) if actions_for(s) > max_actions]
        longest = max(lengths, default=0)
        if over:
            status, reason = 'blocked', 'sequences %s exceed the %d-action harness bound' % (over, max_actions)
        elif longest < need['min_inputs']:
            status = 'blocked'
            reason = ('no planned sequence reaches %d inputs (longest %d); every finding would be Unknown, '
                      'so no Contrast or observed success can form' % (need['min_inputs'], longest))
        else:
            status, reason = 'resolvable', '%d of %d sequences reach %d inputs' % (
                sum(n >= need['min_inputs'] for n in lengths), len(lengths), need['min_inputs'])
        report[name] = {'status': status, 'reason': reason, 'min_inputs': need['min_inputs'],
                        'longest_sequence': longest, 'sequences': len(lengths), 'basis': need['basis']}
    blocked = sorted(k for k, v in report.items() if v['status'] == 'blocked')
    return {'version': PROFILE, 'mode': 'static', 'status': 'blocked' if blocked else 'resolvable',
            'blocked_criteria': blocked, 'criteria': report, 'max_actions': max_actions,
            'interpretation': 'Planning check only: resolvable means a trace is long enough to reach a judgeable '
                              'state, not that it will; it is not evidence of success, failure or fulfillment.'}


def readouts(observation):
    """Final readout text per selector from one browser-observation@v1 report."""
    if not isinstance(observation, dict) or observation.get('version') != 'browser-observation@v1':
        raise ValueError('browser-observation@v1 report required')
    found = {}
    for item in observation.get('observations', []):
        if isinstance(item, dict) and item.get('kind') == 'text':
            found[item['selector']] = item['text']
    return found


def observed(selectors, observations):
    """Per criterion ({criterion: selector}), did any trace's readout differ from the others?"""
    if not isinstance(selectors, dict) or not selectors:
        raise ValueError('selectors must map each criterion to its readout selector')
    if not isinstance(observations, list) or len(observations) < 2:
        raise ValueError('at least two observation reports are needed to compare traces')
    texts = [readouts(o) for o in observations]
    report = {}
    for name, selector in sorted(selectors.items()):
        values = [t.get(selector) for t in texts]
        if any(v is None for v in values):
            report[name] = {'status': 'missing', 'selector': selector,
                            'reason': 'readout absent from %d of %d traces' % (values.count(None), len(values))}
            continue
        distinct = sorted(set(values))
        report[name] = {'status': 'varied' if len(distinct) > 1 else 'invariant', 'selector': selector,
                        'distinct_readouts': len(distinct), 'traces': len(values),
                        'reason': ('readouts differ across traces' if len(distinct) > 1 else
                                   'every trace ended with the same readout; this stage discriminated nothing '
                                   'for this criterion, so its findings can only be Unknown')}
    flagged = sorted(k for k, v in report.items() if v['status'] != 'varied')
    return {'version': PROFILE, 'mode': 'observed', 'status': 'invariant' if flagged else 'varied',
            'flagged_criteria': flagged, 'criteria': report,
            'interpretation': 'Variation shows the stage reached different states, not which state is wanted. '
                              'Invariance does not prove the criterion is unmet.'}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.split('\n\n')[0])
    sub = parser.add_subparsers(dest='mode', required=True)
    s = sub.add_parser('static', help='check planned sequences against declared min_inputs')
    s.add_argument('--coverage', required=True, help='JSON {criterion: {min_inputs, basis}}')
    s.add_argument('--sequences', required=True, help='JSON {criterion: [[key, ...], ...]}')
    s.add_argument('--max-actions', type=int, default=HARNESS_MAX_ACTIONS)
    o = sub.add_parser('observed', help='check observation reports for readout variation')
    o.add_argument('--selectors', required=True, help='JSON {criterion: readout selector}')
    o.add_argument('reports', nargs='+', help='browser-observation@v1 JSON files')
    args = parser.parse_args(argv)
    try:
        def load(path):
            with open(path, encoding='utf-8') as f:
                return json.load(f)
        if args.mode == 'static':
            result = static(load(args.coverage), load(args.sequences), args.max_actions)
        else:
            result = observed(load(args.selectors), [load(p) for p in args.reports])
    except (OSError, ValueError, KeyError) as error:
        print('observation coverage: ' + str(error), file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result['status'] in ('resolvable', 'varied') else 2


if __name__ == '__main__':
    sys.exit(main())
