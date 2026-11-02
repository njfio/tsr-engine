#!/usr/bin/env python3
"""Offline payload preparation only. No transport, credentials, ledger or dispatch."""
import hashlib
import json
from decimal import Decimal
from pathlib import Path

INPUT = Path(__file__).resolve().parents[1] / 'docs/engine-relational-external-inputs.json'
INPUT_SHA256 = '1ba536600596a27f10748b4e952be57294f737bca637e031b749075408a29057'
MODEL = 'deepseek-v4.1-flash'
ENDPOINT = 'https://api.cheaperinference.com/v1/chat/completions'
SYSTEM = ('This is an evaluator-authored synthetic relational benchmark, not user preferences or real observations. '
          'Consider what may become newly wanted through the relationship between the two desires. '
          'Return at most two provisional aspirations or an empty proposals list. Explain the relationship and a discriminating question. '
          'Do not merely conjoin, subdivide or replace parents. An aspiration need not be executable or satisfy prior task predicates. '
          'No truth, fulfillment, independent weight or action authority is conferred. No tools or actions. '
          'Return JSON with proposals, each containing parents (both IDs), relationship, aspiration, discriminator; '
          'each text must be nonempty and at most512 UTF-8 bytes. No extra fields.')

def payload(pair):
    raw = INPUT.read_bytes()
    if hashlib.sha256(raw).hexdigest() != INPUT_SHA256:
        raise ValueError('synthetic allowlist changed; new review required')
    cases = json.loads(raw)['cases']
    if pair not in ('A','B'):
        raise ValueError('unknown synthetic pair')
    # Reconstruct only exact frozen synthetic fields; never pass a run/request/report.
    c = cases[pair]
    user = {'synthetic_only':True,'parents':[{'id':p['id'],'text':p['text']} for p in c['parents']],
            'synthetic_premise':c['synthetic_premise']}
    body = {'model':MODEL,'stream':False,'temperature':0,'max_tokens':2048,
            'messages':[{'role':'system','content':SYSTEM},{'role':'user','content':json.dumps(user,ensure_ascii=False)}]}
    if len(json.dumps(body).encode()) > 16384:
        raise ValueError('bounded synthetic payload exceeded')
    return body

def readiness(pair, pricing=None):
    body = payload(pair)
    # Historical pricing ceilings only. Never silently treat them as a current quote.
    quote_ok = False
    bound = None
    if pricing is not None:
        if set(pricing) != {'list_input_per_million','list_output_per_million'}:
            raise ValueError('incomplete or unsupported price fields')
        rates=[]
        for key, cap in [('list_input_per_million',Decimal('.30')),('list_output_per_million',Decimal('1.20'))]:
            value=pricing[key]
            if not isinstance(value,str): raise ValueError('explicit decimal price required')
            rate=Decimal(value)
            if not rate.is_finite() or rate<0 or rate>cap: raise ValueError('price ceiling exceeded')
            rates.append(rate)
        bound=(Decimal(len(json.dumps(body).encode())+8192)*rates[0]+Decimal(2048)*rates[1])/Decimal(1000000)
        quote_ok = bound <= Decimal('.25')
    return {'payload':body,'endpoint':ENDPOINT,'input_sha256':INPUT_SHA256,
            'dispatch_enabled':False,'approval':'pending','calls_max':4,'aggregate_cap_usd':'1.00',
            'reservation_per_attempt_usd':'.25','pricing_within_assumed_bound':quote_ok,
            'conditional_estimate_usd':str(bound) if bound is not None else None,
            'cap_guaranteed':False,
            'blockers':['fresh user approval pending','no current authenticated pricing/billing-coverage evidence',
                        'byte-plus-framing token bound and reasoning/other billable units unverified',
                        'no external relational dispatcher/adoption; payload preparation only'],
            'required_future_controls':'One shared locked four-attempt ledger, reserve .25 before each dispatch, retain timeout/unknown reservations, no retries. Stop before dispatch if all billable units cannot be bounded within reservation.'}

if __name__ == '__main__':
    import argparse
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--pair',choices=['A','B'],required=True)
    print(json.dumps(readiness(p.parse_args().pair),indent=2))
