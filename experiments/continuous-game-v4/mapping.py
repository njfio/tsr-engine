"""Offline reviewed mapping/probe compiler for the harder continuous-game test.

Same contract as experiments/continuous-game-v3/mapping.py, plus one change: the
mapping carries the ``discovery`` readouts from one recorded no-input probe, and
route words are derived from them mechanically. A hard game cannot be finished by
blind words, so without a route every gameplay finding would again be Unknown. A
route that fails stays a result; there is no field for writing a route by hand.

No dispatcher, reviewer authentication or authority is created here.
"""
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


base = _load('continuous_v4_base_mapping', ROOT / 'experiments/continuous-game-v3/mapping.py')

VERSION = 'reviewed-control-map@v3'
BATCHES = base.BATCHES
ROUTES = {'crossing-first': 'crossing', 'growth-first': 'growth', 'mixed-pursuit': 'mixed'}
DIRECTIONS = ('Up', 'Right', 'Down', 'Left')
digest = base.digest
source = base.source


def _word(text):
    words = text.split() if isinstance(text, str) else []
    return words if 1 <= len(words) <= base.MAX_KEYS and all(w in DIRECTIONS for w in words) else None


def routes(mapping):
    """Route words, derived only from the recorded discovery readouts. None means unavailable."""
    found = mapping['discovery']
    crossing, growth = _word(found['route']), _word(found['shard_route'])
    mixed = growth + crossing if crossing and growth and len(growth) + len(crossing) <= base.MAX_KEYS else None
    return {'crossing': crossing, 'growth': growth, 'mixed': mixed}


def split(mapping):
    """The v3 contract fields; the discovery record must match this mapping's source."""
    if not isinstance(mapping, dict) or mapping.get('version') != VERSION or 'discovery' not in mapping:
        raise ValueError('mapping contract: ' + VERSION + ' with discovery required')
    found = mapping['discovery']
    if not isinstance(found, dict) or set(found) != {'source', 'report_sha256', 'route', 'shard_route'}:
        raise ValueError('discovery record shape')
    if found['source'] != mapping.get('source'):
        raise ValueError('stale discovery source')
    previous = {k: v for k, v in mapping.items() if k != 'discovery'}
    previous['version'] = base.VERSION
    return previous, found


def check_discovery(mapping, report):
    """The mapping's route text must be exactly what the recorded probe read."""
    found = mapping['discovery']
    if digest(report) != found['report_sha256'] or report['source'] != found['source']:
        raise ValueError('discovery report binding')
    if report['readouts'] != {'route': found['route'], 'shard_route': found['shard_route']}:
        raise ValueError('route text differs from the discovery report')


def _rebind(mapping, review):
    previous, _ = split(mapping)
    if not isinstance(review, dict) or review.get('mapping_sha256') != digest(mapping):
        raise ValueError('mapping review binding')
    return previous, {**review, 'mapping_sha256': digest(previous)}


def validate(mapping, files, review):
    previous, rebound = _rebind(mapping, review)
    result = base.validate(previous, files, rebound)
    if result.get('status') == 'mapped':
        result = {**result, 'mapping_sha256': digest(mapping), 'routes': routes(mapping)}
    return result


def search(mapping, batch):
    """Six blind words sized from coverage, then the route word when one is available."""
    previous, _ = split(mapping)
    route = routes(mapping)[ROUTES[batch]]
    words = base.search(previous, batch)
    return words + ([[mapping['controls'][base.root_at(batch, i)][d] for i, d in enumerate(route)]] if route else [])


def batch_coverage(mapping, batch, words):
    """Coverage for a batch. With one shared control set, alternating roots does not divide inputs."""
    previous, _ = split(mapping)
    if batch == 'mixed-pursuit' and mapping['controls']['crossing'] == mapping['controls']['growth']:
        return base.coverage_tool.static(previous['coverage'], {r: words for r in base.ROOTS})
    return base.batch_coverage(previous, batch, words)


def plan(mapping, files, review, batch, keys):
    valid = validate(mapping, files, review)
    if valid['status'] != 'mapped':
        return valid
    if not 1 <= len(keys) <= base.MAX_KEYS:
        raise ValueError('unsupported sequence length; no truncation')
    for i, key in enumerate(keys):
        if key not in mapping['controls'][base.root_at(batch, i)].values():
            raise ValueError('input outside reviewed batch controls')
    report = batch_coverage(mapping, batch, search(mapping, batch))
    if report['status'] != 'resolvable':
        raise ValueError('observation stage cannot resolve %s; refusing dispatch' % ', '.join(report['blocked_criteria']))
    actions = [{'kind': 'click', 'value': mapping['start']}]
    for i, key in enumerate(keys):
        if i:
            actions.append({'kind': 'wait', 'value': 100})
        actions.append({'kind': 'key', 'value': key})
    if len(actions) > base.coverage_tool.HARNESS_MAX_ACTIONS:
        raise ValueError('probe exceeds harness action bound')
    probe = {'version': 'browser-probe@v1', 'assets': sorted(files), 'entry': mapping['entry'], 'actions': actions,
             'checks': [{'selector': v, 'text': ''} for v in dict.fromkeys(mapping['readouts'].values())]}
    body = {**valid, 'batch': batch, 'probe': probe, 'coverage_report': report,
            'coverage': 'Finite ordered sequence and final DOM readouts only; no inferred success or failure. Every probe '
                        'starts a fresh page and uses advertised restart. No cross-probe state continuation. Blind word '
                        'length comes from the reviewed coverage; route words come from the recorded discovery probe.'}
    return {**body, 'plan_sha256': digest(body)}
