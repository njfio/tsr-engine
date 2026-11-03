"""Offline reviewed mapping/probe compiler for the harder continuous-game test.

Same contract as experiments/continuous-game-v3/mapping.py, plus one change: the
mapping carries ``routes``, one word per batch, fixed before any control trace
runs. A hard game cannot be finished by blind words, so without a route every
gameplay finding would again be Unknown. A route that fails stays a result.

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


def split(mapping):
    """The v3 contract fields, and the route declaration."""
    if not isinstance(mapping, dict) or mapping.get('version') != VERSION or 'routes' not in mapping:
        raise ValueError('mapping contract: ' + VERSION + ' with routes required')
    routes = mapping['routes']
    if not isinstance(routes, dict) or set(routes) != set(ROUTES.values()):
        raise ValueError('routes must declare exactly crossing, growth and mixed')
    for name, route in routes.items():
        if not isinstance(route, dict) or set(route) != {'directions', 'basis'} or not isinstance(route['basis'], str) or not route['basis'].strip():
            raise ValueError(name + ': route needs directions and a basis')
        word = route['directions']
        if not isinstance(word, list) or not 1 <= len(word) <= base.MAX_KEYS or any(d not in DIRECTIONS for d in word):
            raise ValueError(name + ': route directions outside the bound')
    previous = {k: v for k, v in mapping.items() if k != 'routes'}
    previous['version'] = base.VERSION
    return previous, routes


def _rebind(mapping, review):
    previous, _ = split(mapping)
    if not isinstance(review, dict) or review.get('mapping_sha256') != digest(mapping):
        raise ValueError('mapping review binding')
    return previous, {**review, 'mapping_sha256': digest(previous)}


def validate(mapping, files, review):
    previous, rebound = _rebind(mapping, review)
    result = base.validate(previous, files, rebound)
    if result.get('status') == 'mapped':
        result = {**result, 'mapping_sha256': digest(mapping), 'routes': mapping['routes']}
    return result


def search(mapping, batch):
    """Six blind words sized from coverage, then the reviewed route word."""
    previous, routes = split(mapping)
    route = routes[ROUTES[batch]]['directions']
    word = [mapping['controls'][base.root_at(batch, i)][d] for i, d in enumerate(route)]
    return base.search(previous, batch) + [word]


def plan(mapping, files, review, batch, keys):
    previous, rebound = _rebind(mapping, review)
    body = base.plan(previous, files, rebound, batch, keys)
    if body.get('status') == 'mapped':
        body = {k: v for k, v in body.items() if k != 'plan_sha256'}
        body = {**body, 'mapping_sha256': digest(mapping), 'routes': mapping['routes']}
        body['plan_sha256'] = digest(body)
    return body
