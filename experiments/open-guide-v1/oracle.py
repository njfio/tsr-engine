"""Frozen data-only graph checks. No generated code, user simulation, or joint score."""
import re

def edges(raw):
    if not isinstance(raw, str) or len(raw.encode()) > 2048:
        raise ValueError('bounded edge output')
    text = raw[:-1] if raw.endswith('\n') else raw
    if not text:
        return []
    if not re.fullmatch(r'[0-9]+>[0-9]+(?:,[0-9]+>[0-9]+)*', text):
        raise ValueError('one exact directed-edge line')
    result = [tuple(map(int, item.split('>'))) for item in text.split(',')]
    if any(n > 2**32 - 1 for edge in result for n in edge):
        raise ValueError('u32 ID')
    return result

def condition(raw):
    budget, records, probes = raw.split('|')
    budget = int(budget)
    nodes = [tuple(map(int, p.split(':'))) for p in records.split(',')]
    queries = edges(probes)
    if not 1 <= budget <= 12 or not 2 <= len(nodes) <= 8 or len(dict(nodes)) != len(nodes):
        raise ValueError('bounded distinct nodes')
    if any(not 0 <= i <= 2**32-1 or not 0 <= tag <= 255 for i,tag in nodes):
        raise ValueError('node domain')
    if not queries or len(queries) > 8 or any(a not in dict(nodes) or b not in dict(nodes) or a == b for a,b in queries):
        raise ValueError('bounded available query endpoints')
    return budget, dict(nodes), queries

def distance(graph, start, end):
    seen = {start}; todo = [(start, 0)]
    for node, cost in todo:
        if node == end:
            return cost
        for a,b in graph:
            if a == node and b not in seen:
                seen.add(b); todo.append((b, cost+1))
    return None

def assess(raw, previous_stdout, stdout):
    budget, nodes, queries = condition(raw)
    previous, output = edges(previous_stdout), edges(stdout)
    valid = (len(output) <= budget and len(set(output)) == len(output)
             and all(a in nodes and b in nodes and a != b for a,b in output))
    paths = [{'from':a, 'to':b, 'hops':distance(output,a,b) if valid else None,
              'same_category':nodes[a] == nodes[b]} for a,b in queries]
    facts = {'legal_bounded_edges':valid, 'requested_routes_exist':valid and all(p['hops'] is not None for p in paths)}
    return {'minimum_obligations_met':all(facts.values()), 'facts':facts,
            'diagnostics':{'requested_paths':paths,
                'retained_available_edges':len(set(output)&{(a,b) for a,b in previous if a in nodes and b in nodes}),
                'available_previous_edges':len({(a,b) for a,b in previous if a in nodes and b in nodes}),
                'cross_category_edges':sum(nodes.get(a)!=nodes.get(b) for a,b in output) if valid else None},
            'repeatability':'requires separate source assessment',
            'scope':'Deterministic fictional graph queries only; not reader satisfaction, truth, qualitative fulfillment, authority, or a joint utility score.'}
