#!/usr/bin/env python3
"""One shared bounded Luna campaign ledger; no implicit dispatch or account changes.

The legacy 50,000 microUSD reservation is an operational floor. Explicit v6
browser requests reserve their conservative serialized-token quote when larger.
Unknown outcomes retain the individual reservation and block the campaign.
"""
import argparse
import contextlib
import decimal
import datetime
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import time
import urllib.request

_capacity_spec=importlib.util.spec_from_file_location('request_capacity',Path(__file__).with_name('engine-request-capacity.py'))
capacity=importlib.util.module_from_spec(_capacity_spec);_capacity_spec.loader.exec_module(capacity)

MODEL = 'gpt-6-luna'
ENDPOINT = 'https://api.cheaperinference.com/v1/chat/completions'
CATALOG = 'https://api.cheaperinference.com/public/models/gpt-6-luna'
HARNESS_GRANT = 'native-browser-harness@v1'
UNLIMITED_GRANT = 'native-browser-harness@v3'
HARNESS_GRANTS = (HARNESS_GRANT, 'native-browser-harness@v2', UNLIMITED_GRANT)
UNLIMITED_AUTHORITY = {'message_id':'Sentinel_2cfe102b0c2081919168e000777daa4b','at':'2026-10-30T11:10:38Z','text':'Literally stop asking appending decision in the engine. We haven’t been close and we are using a super cheap model. I want that to have a flag to unlimited please and'}
def exceeds(value, ceiling): return ceiling is not None and value > ceiling
def remaining(ceiling, used): return None if ceiling is None else ceiling-used
def validate_unlimited_policy(value, manifest_hash, head):
    fields={'version','manifest_sha256','ledger_prefix_head','provider','model','source_class','authority','quarantined_unknown_keys'}
    if not isinstance(value,dict) or set(value)!=fields or value['version']!='browser-unlimited-policy@v1':raise ValueError('unlimited policy shape')
    if value['manifest_sha256']!=manifest_hash or value['ledger_prefix_head']!=head:raise ValueError('unlimited policy prefix')
    if value['provider']!='CheaperInference' or value['model']!=MODEL or value['source_class']!='newly-authored-nonprivate-browser-project' or value['authority']!=UNLIMITED_AUTHORITY:raise ValueError('unlimited policy authority/scope')
    keys=value['quarantined_unknown_keys']
    if not isinstance(keys,list) or len(keys)!=len(set(keys)):raise ValueError('unlimited quarantine keys')
    for key in keys:identity(key)
CAP = 5_000_000
RESERVE = 50_000
RATES = {'input_per_million':'0.065000', 'output_per_million':'0.325000'}
# Each category may consume at most the existing reservation on its own; the
# additive four-category estimate below must ALSO fit that same reservation.
PRICE_CEILINGS = {key:str(decimal.Decimal(RESERVE)/tokens) for key,tokens in (
    ('input_per_million',32768),('output_per_million',8192),
    ('cache_read_per_million',32768),('cache_write_per_million',32768))}

def canonical(v): return json.dumps(v,sort_keys=True,separators=(',',':'),allow_nan=False).encode()
def digest(v): return hashlib.sha256(canonical(v)).hexdigest()
def load_module(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

def strict(raw):
    def pairs(items):
        out={}
        for k,v in items:
            if k in out:raise ValueError('duplicate key')
            out[k]=v
        return out
    return json.loads(raw,object_pairs_hook=pairs,parse_constant=lambda _:(_ for _ in ()).throw(ValueError('nonfinite')))

def identity(v):
    if not isinstance(v,str) or len(v)!=64 or any(c not in '0123456789abcdef' for c in v):raise ValueError('opaque SHA256 identity required')
    return v


DEFAULT_PRICE_SCOPE={'zdr':False,'model':MODEL,'region':None,'protocol':'chat_completions','serviceTier':'standard','pricingVariant':'default'}
def scope_rates(value,threshold):
    if type(threshold) is not int or threshold<=0:raise ValueError('unsupported tiered threshold')
    if not isinstance(value,dict) or set(value)!={'id','version','scope','tiers','effectiveAt','expiresAt'} or value['scope']!=DEFAULT_PRICE_SCOPE:raise ValueError('unsupported tiered scope')
    identity(value['version'])
    if not isinstance(value['id'],str) or not value['id']:raise ValueError('missing tier scope identity')
    if any(type(value[k]) is not int for k in ('effectiveAt','expiresAt')) or not 0<=value['effectiveAt']<value['expiresAt']:raise ValueError('tier validity interval')
    tiers=value['tiers']
    if not isinstance(tiers,list) or len(tiers)!=2:raise ValueError('unsupported tier coverage')
    mapping={'input_per_million':'input_tokens','output_per_million':'output_tokens','cache_read_per_million':'cache_read_tokens','cache_write_per_million':'cache_write_5m_tokens'}
    bounds=[];rates=[]
    for tier in tiers:
        if not isinstance(tier,dict) or set(tier)!={'id','rates','minContext','maxContext'} or not isinstance(tier['id'],str) or not tier['id']:raise ValueError('tier shape')
        if type(tier['minContext']) is not int or (tier['maxContext'] is not None and type(tier['maxContext']) is not int):raise ValueError('tier bounds')
        bounds.append((tier['minContext'],tier['maxContext']))
        if not isinstance(tier['rates'],dict) or set(tier['rates'])!=set(mapping.values()):raise ValueError('tier categories')
        row={}
        for key,name in mapping.items():
            raw=tier['rates'][name]
            if not isinstance(raw,str):raise ValueError('tier rate type')
            try:amount=decimal.Decimal(raw)
            except decimal.InvalidOperation:raise ValueError('tier rate invalid') from None
            if not amount.is_finite() or amount<0:raise ValueError('tier rate invalid')
            row[key]=amount*1000000
        rates.append(row)
    if set(bounds)!={(0,threshold+1),(threshold+1,None)} or len({x['id'] for x in tiers})!=2:raise ValueError('tier coverage mismatch')
    return {key:str(max(row[key] for row in rates)) for key in mapping}

def validate_quote(q):
    legacy='version' not in q
    tiered=q.get('version')=='b91-price-quote@v3'
    keys={'model','rates','retrieved_at','catalog_sha256'} | (set() if legacy else {'version','ceilings','input_token_price_threshold'})
    if tiered:keys|={'pricing_scope'}
    if set(q)!=keys or q.get('model')!=MODEL:raise ValueError('price quote shape')
    identity(q['catalog_sha256'])
    stamp=q['retrieved_at']
    if type(stamp) not in (int,float) or not decimal.Decimal(str(stamp)).is_finite() or stamp<0:raise ValueError('price quote timestamp')
    if legacy:
        if q['rates']!=RATES:raise ValueError('legacy price provenance drift')
        return q['rates']
    if tiered:
        rates=scope_rates(q['pricing_scope'],q['input_token_price_threshold'])
        if q['rates']!=rates:raise ValueError('tier envelope mismatch')
        if not q['pricing_scope']['effectiveAt']<=stamp*1000<q['pricing_scope']['expiresAt']:raise ValueError('tier quote outside validity')
    elif type(q.get('input_token_price_threshold')) is not int or q['input_token_price_threshold']!=-1:raise ValueError('unsupported tiered pricing')
    if q['version'] not in ('b91-price-quote@v2','b91-price-quote@v3') or set(q['ceilings'])!=set(PRICE_CEILINGS) or set(q['rates'])!=set(PRICE_CEILINGS):raise ValueError('price quote version/rates')
    for key,maximum in PRICE_CEILINGS.items():
        if not isinstance(q['rates'][key],str) or not isinstance(q['ceilings'][key],str):raise ValueError('price rate type')
        try:rate=decimal.Decimal(q['rates'][key]);ceiling=decimal.Decimal(q['ceilings'][key])
        except decimal.InvalidOperation:raise ValueError('price rate invalid') from None
        if not rate.is_finite() or not ceiling.is_finite() or not 0<=rate<=ceiling<=decimal.Decimal(maximum):raise ValueError('official pricing drift')
    if rate_estimate(q['rates'])>RESERVE:raise ValueError('official quote exceeds reservation')
    return q['rates']

def quote_estimate(q,completion_tokens=8192,input_tokens=32768):
    return rate_estimate(validate_quote(q),completion_tokens,input_tokens)

def rate_estimate(rates,completion_tokens=8192,input_tokens=32768):
    input_rates=sum((decimal.Decimal(rates[k]) for k in ('input_per_million','cache_read_per_million','cache_write_per_million') if k in rates),decimal.Decimal(0))
    # USD/million times tokens is microUSD. Charge all input categories together,
    # deliberately overestimating mutually exclusive cache/read classifications.
    return decimal.Decimal(input_tokens)*input_rates+decimal.Decimal(completion_tokens)*decimal.Decimal(rates['output_per_million'])

def validate_call_grant(grant,manifest_hash,previous_head,current_limit):
    fields={'version','manifest_sha256','ledger_prefix_head','previous_max_calls','max_calls',
            'max_execution_groups','cap_microusd','reservation_microusd','authority_sha256','reviewer'}
    if not isinstance(grant,dict) or set(grant)!=fields:raise ValueError('call grant shape')
    if grant['version'] not in ('b91-call-extension@v1','b91-inquiry-call-extension@v1','b91-inquiry-call-extension@v2'):raise ValueError('call grant version')
    for key in ('manifest_sha256','ledger_prefix_head','authority_sha256'):identity(grant[key])
    if grant['manifest_sha256']!=manifest_hash or grant['ledger_prefix_head']!=previous_head:raise ValueError('call grant source mismatch')
    expected={'previous_max_calls':100,'max_calls':140,'max_execution_groups':40,'cap_microusd':CAP,'reservation_microusd':RESERVE}
    if grant['version']=='b91-inquiry-call-extension@v1':expected.update(previous_max_calls=200,max_calls=202,max_execution_groups=60)
    if grant['version']=='b91-inquiry-call-extension@v2':expected.update(previous_max_calls=202,max_calls=220,max_execution_groups=60)
    if current_limit!=expected['previous_max_calls'] or any(type(grant[k]) is not int or grant[k]!=v for k,v in expected.items()):raise ValueError('only exact approved call extension permitted')
    if not isinstance(grant['reviewer'],str) or not 1<=len(grant['reviewer'].encode())<=256:raise ValueError('call grant reviewer')

def validate_harness_grant(grant,manifest_hash,previous_head,current_calls,current_groups,current_cap):
    """Scope-only grant: exact fresh session within already effective authority."""
    fields={'version','manifest_sha256','ledger_prefix_head','previous_max_calls','max_calls',
            'previous_max_execution_groups','max_execution_groups','cap_microusd','reservation_microusd',
            'authority_sha256','reviewer','suite_manifest_sha256','source_class',
            'session_trial_sha256','session_admission_sha256','baseline',
            'session_max_calls','session_max_execution_groups','session_cap_microusd'}
    unlimited=grant.get('version')==UNLIMITED_GRANT
    if unlimited:fields|={'unlimited_policy_sha256','expires_at'}
    if set(grant)!=fields:raise ValueError('harness scope grant shape')
    for key in ('manifest_sha256','ledger_prefix_head','authority_sha256','suite_manifest_sha256','session_trial_sha256','session_admission_sha256'):identity(grant[key])
    if grant['manifest_sha256']!=manifest_hash or grant['ledger_prefix_head']!=previous_head:raise ValueError('harness scope prefix mismatch')
    baseline=grant['baseline']
    if not isinstance(baseline,dict) or set(baseline)!={'calls','execution_groups','settled_microusd'} or any(type(x) is not int or x<0 for x in baseline.values()):raise ValueError('harness settled baseline shape')
    if unlimited:
        identity(grant['unlimited_policy_sha256'])
        nulls=('previous_max_calls','max_calls','previous_max_execution_groups','max_execution_groups','cap_microusd','session_max_calls','session_max_execution_groups','session_cap_microusd','expires_at')
        if any(grant[k] is not None for k in nulls) or any(v is not None for v in (current_calls,current_groups,current_cap)):raise ValueError('unlimited profile requires explicit null ceilings')
        if grant['reservation_microusd']!=RESERVE or grant['source_class']!='newly-authored-nonprivate-browser-project' or not isinstance(grant['reviewer'],str) or not 1<=len(grant['reviewer'].encode())<=256:raise ValueError('unlimited scope/reviewer')
        return
    for key,maximum in (('session_max_calls',200),('session_max_execution_groups',100),('session_cap_microusd',10000000)):
        if type(grant[key]) is not int or not (1 if key=='session_max_calls' else 0)<=grant[key]<=maximum:raise ValueError('finite authorized session ceiling exceeded')
    expected={'previous_max_calls':current_calls,'max_calls':current_calls,
              'previous_max_execution_groups':current_groups,'max_execution_groups':current_groups,
              'cap_microusd':current_cap,'reservation_microusd':RESERVE}
    if grant['version']=='native-browser-harness@v2':
        expected.update(max_calls=max(current_calls,baseline['calls']+grant['session_max_calls']),max_execution_groups=max(current_groups,baseline['execution_groups']+grant['session_max_execution_groups']),cap_microusd=max(current_cap or 0,baseline['settled_microusd']+grant['session_cap_microusd']))
    if current_cap is None or any(type(grant[k]) is not int or grant[k]!=v for k,v in expected.items()):raise ValueError('harness grant ceilings do not match exact prospective authority')
    current_calls=expected['max_calls'];current_groups=expected['max_execution_groups'];current_cap=expected['cap_microusd']
    if grant['source_class']!='newly-authored-nonprivate-browser-project':raise ValueError('harness public source scope')
    if not isinstance(grant['reviewer'],str) or not 1<=len(grant['reviewer'].encode())<=256:raise ValueError('harness scope independent reviewer')
    baseline=grant['baseline']
    if not isinstance(baseline,dict) or set(baseline)!={'calls','execution_groups','settled_microusd'} or any(type(x) is not int or x<0 for x in baseline.values()):raise ValueError('harness settled baseline shape')
    for key,maximum in (('session_max_calls',min(200,current_calls-baseline['calls'])),('session_max_execution_groups',min(100,current_groups-baseline['execution_groups'])),('session_cap_microusd',min(10000000,current_cap-baseline['settled_microusd']))):
        if type(grant[key]) is not int or not (1 if key=='session_max_calls' else 0)<=grant[key]<=maximum:raise ValueError('harness session exceeds remaining campaign allowance')
    if grant['session_max_calls']*RESERVE>grant['session_cap_microusd']:raise ValueError('harness whole-session reservation floor does not fit')


PAIR_GRANT='native-browser-pair@v1'
def validate_pair_grant(grant,manifest_hash,previous_head,current_calls,current_groups,current_cap):
    parent={k:v for k,v in grant.items() if k!='arms'};parent['version']='native-browser-harness@v2'
    validate_harness_grant(parent,manifest_hash,previous_head,current_calls,current_groups,current_cap)
    arms=grant.get('arms')
    if not isinstance(arms,list) or len(arms)!=2:raise ValueError('exactly two paired arms')
    if any(parent[k]%2 for k in ('session_max_calls','session_max_execution_groups','session_cap_microusd')):raise ValueError('equal integer arm ceilings')
    seen=set()
    for arm in arms:
        if not isinstance(arm,dict) or set(arm)!={'trial','admission','suite','max_calls','max_operations','cap_microusd'}:raise ValueError('paired arm binding shape')
        for key in ('trial','admission','suite'):identity(arm[key])
        if arm['trial'] in seen or arm['trial']==parent['session_trial_sha256']:raise ValueError('duplicate/parent paired trial')
        seen.add(arm['trial'])
        for key,total in (('max_calls','session_max_calls'),('max_operations','session_max_execution_groups'),('cap_microusd','session_cap_microusd')):
            if type(arm[key]) is not int or arm[key]!=parent[total]//2:raise ValueError('unequal paired arm authority')

def validate_evaluation_grant(grant,manifest_hash,previous_head,current_calls,current_groups,current_cap=None):
    if isinstance(grant,dict) and grant.get('version')==PAIR_GRANT:return validate_pair_grant(grant,manifest_hash,previous_head,current_calls,current_groups,current_cap)
    if isinstance(grant,dict) and grant.get('version') in HARNESS_GRANTS:
        return validate_harness_grant(grant,manifest_hash,previous_head,current_calls,current_groups,current_cap)
    fields={'version','manifest_sha256','ledger_prefix_head','previous_max_calls','max_calls',
            'previous_max_execution_groups','max_execution_groups','cap_microusd','reservation_microusd',
            'authority_sha256','reviewer','suite_manifest_sha256','source_class'}
    if isinstance(grant,dict) and grant.get('version') in ('native-browser-session@v1','native-browser-session@v2','native-browser-session@v3','native-browser-session@v4','native-browser-session@v5','native-browser-session@v6','native-browser-session@v7','native-browser-session@v8','native-browser-session@v9','native-browser-session@v10'):fields|={'session_trial_sha256','session_admission_sha256'}
    if not isinstance(grant,dict) or set(grant)!=fields:raise ValueError('evaluation grant shape')
    if grant['version'] not in ('b91-evaluation-extension@v1','native-pursuit-extension@v1','native-pursuit-extension@v2','native-information-extension@v1','native-information-extension@v2','native-objective-extension@v1','native-objective-extension@v2','native-objective-extension@v3','native-open-guide-extension@v1','native-open-guide-extension@v2','native-browser-initial@v1','native-browser-session@v1','native-browser-session@v2','native-browser-session@v3','native-browser-session@v4','native-browser-session@v5','native-browser-session@v6','native-browser-session@v7','native-browser-session@v8','native-browser-session@v9','native-browser-session@v10'):raise ValueError('evaluation grant version')
    for key in ('manifest_sha256','ledger_prefix_head','authority_sha256','suite_manifest_sha256'):identity(grant[key])
    if grant['manifest_sha256']!=manifest_hash or grant['ledger_prefix_head']!=previous_head:raise ValueError('evaluation grant source mismatch')
    expected={'previous_max_calls':140,'max_calls':200,'previous_max_execution_groups':40,
              'max_execution_groups':60,'cap_microusd':CAP,'reservation_microusd':RESERVE}
    source_class='newly-authored-nonprivate-rust-repair-suite'
    if grant['version']=='native-pursuit-extension@v1':
        expected.update(previous_max_calls=220,max_calls=228,previous_max_execution_groups=60,max_execution_groups=62)
        source_class='newly-authored-nonprivate-native-pursuit'
    if grant['version']=='native-pursuit-extension@v2':
        expected.update(previous_max_calls=228,max_calls=230,previous_max_execution_groups=62,max_execution_groups=62)
        source_class='newly-authored-nonprivate-native-pursuit'
    if grant['version']=='native-information-extension@v1':
        expected.update(previous_max_calls=230,max_calls=238,previous_max_execution_groups=62,max_execution_groups=64)
        source_class='newly-authored-nonprivate-native-information-pursuit'
    if grant['version']=='native-information-extension@v2':
        expected.update(previous_max_calls=238,max_calls=239,previous_max_execution_groups=64,max_execution_groups=64)
        source_class='newly-authored-nonprivate-native-information-pursuit'
    if grant['version']=='native-objective-extension@v1':
        expected.update(previous_max_calls=239,max_calls=250,previous_max_execution_groups=64,max_execution_groups=67)
        source_class='newly-authored-nonprivate-finite-service-objective'
    if grant['version']=='native-objective-extension@v2':
        expected.update(previous_max_calls=250,max_calls=252,previous_max_execution_groups=67,max_execution_groups=68)
        source_class='newly-authored-nonprivate-finite-service-objective'
    if grant['version']=='native-objective-extension@v3':
        expected.update(previous_max_calls=252,max_calls=258,previous_max_execution_groups=68,max_execution_groups=70)
        source_class='newly-authored-nonprivate-finite-service-objective'
    if grant['version']=='native-open-guide-extension@v1':
        expected.update(previous_max_calls=258,max_calls=272,previous_max_execution_groups=70,max_execution_groups=74)
        source_class='newly-authored-nonprivate-open-guide-objective'
    if grant['version']=='native-open-guide-extension@v2':
        expected.update(previous_max_calls=272,max_calls=283,previous_max_execution_groups=74,max_execution_groups=77)
        source_class='newly-authored-nonprivate-open-guide-objective'
    if grant['version']=='native-browser-initial@v1':
        expected.update(previous_max_calls=272,max_calls=272,previous_max_execution_groups=74,max_execution_groups=74)
        source_class='newly-authored-nonprivate-browser-project'
    if grant['version']=='native-browser-session@v1':
        identity(grant['session_trial_sha256']);identity(grant['session_admission_sha256'])
        expected.update(previous_max_calls=272,max_calls=462,previous_max_execution_groups=74,max_execution_groups=172,cap_microusd=10225523)
        source_class='newly-authored-nonprivate-browser-project'
    if grant['version']=='native-browser-session@v2':
        identity(grant['session_trial_sha256']);identity(grant['session_admission_sha256'])
        expected.update(previous_max_calls=462,max_calls=464,previous_max_execution_groups=172,max_execution_groups=174,cap_microusd=10228567)
        source_class='newly-authored-nonprivate-browser-project'
    if grant['version']=='native-browser-session@v3':
        identity(grant['session_trial_sha256']);identity(grant['session_admission_sha256'])
        expected.update(previous_max_calls=464,max_calls=465,previous_max_execution_groups=174,max_execution_groups=174,cap_microusd=10232177)
        source_class='newly-authored-nonprivate-browser-project'
    if grant['version']=='native-browser-session@v4':
        identity(grant['session_trial_sha256']);identity(grant['session_admission_sha256'])
        expected.update(previous_max_calls=465,max_calls=465,previous_max_execution_groups=174,max_execution_groups=174,cap_microusd=10232177)
        source_class='newly-authored-nonprivate-browser-project'
    if grant['version']=='native-browser-session@v5':
        identity(grant['session_trial_sha256']);identity(grant['session_admission_sha256'])
        expected.update(previous_max_calls=465,max_calls=465,previous_max_execution_groups=174,max_execution_groups=174,cap_microusd=10232177)
        source_class='newly-authored-nonprivate-browser-project'
    if grant['version'] in ('native-browser-session@v6','native-browser-session@v7','native-browser-session@v8','native-browser-session@v9','native-browser-session@v10'):
        identity(grant['session_trial_sha256']);identity(grant['session_admission_sha256'])
        expected.update(previous_max_calls=465,max_calls=465,previous_max_execution_groups=174,max_execution_groups=174,cap_microusd=10232177)
        source_class='newly-authored-nonprivate-browser-project'
    if current_calls!=expected['previous_max_calls'] or current_groups!=expected['previous_max_execution_groups'] or any(type(grant[k]) is not int or grant[k]!=v for k,v in expected.items()):
        raise ValueError('only exact prospectively reviewed evaluation extension permitted')
    if grant['source_class']!=source_class:raise ValueError('evaluation source scope')
    if not isinstance(grant['reviewer'],str) or not 1<=len(grant['reviewer'].encode())<=256:raise ValueError('evaluation grant reviewer')

def fresh(path,value):
    with os.fdopen(os.open(path,os.O_CREAT|os.O_EXCL|os.O_WRONLY|os.O_NOFOLLOW,0o600),'wb') as f:
        f.write(canonical(value)+b'\n');f.flush();os.fsync(f.fileno())

def manifest(campaign_id,ledger_path,*,fixture=False):
    return {'version':'b91-luna-campaign@v1','campaign_id':identity(campaign_id),'fixture_only':fixture,
            'ledger_path':str(Path(ledger_path).resolve()),'implementation_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            'provider':ENDPOINT,'model':MODEL,'source_class':'frozen-synthetic-b91-only',
            'max_calls':100,'max_execution_groups':40,'cap_microusd':CAP,'reservation_microusd':RESERVE,
            'max_completion_tokens':8192,'max_payload_bytes':32768,'call_seconds':240,
            'rates':RATES,'price_policy':'conservative_allowance_unknown_retained_no_provider_cap',
            'created_at':time.time()}

def validate(m,upgrade=None,upgrade_hash=None):
    expected=manifest(m.get('campaign_id'),m.get('ledger_path'),fixture=m.get('fixture_only'))
    if type(m.get('fixture_only')) is not bool or type(m.get('created_at')) not in (int,float):raise ValueError('campaign manifest shape')
    expected['created_at']=m['created_at']
    if upgrade is not None:
        fields={'version','old_manifest_sha256','old_implementation_sha256','new_implementation_sha256','ledger_prefix_head','reviewer','reason'}
        if (set(upgrade)!=fields or upgrade['version']!='b91-campaign-upgrade@v1' or digest(upgrade)!=upgrade_hash
                or upgrade['old_manifest_sha256']!=digest(m) or upgrade['old_implementation_sha256']!=m['implementation_sha256']
                or upgrade['new_implementation_sha256']!=expected['implementation_sha256']):raise ValueError('unreviewed or mismatched implementation upgrade')
        for key in ('old_manifest_sha256','old_implementation_sha256','new_implementation_sha256','ledger_prefix_head'):identity(upgrade[key])
        if any(not isinstance(upgrade[k],str) or not 1<=len(upgrade[k])<=512 for k in ('reviewer','reason')):raise ValueError('bounded independent upgrade review required')
        expected['implementation_sha256']=m['implementation_sha256']
    if m!=expected:raise ValueError('campaign bounds/provider/scope changed')

class Ledger:
    def __init__(self,path,m,expected_hash,*,create=False,upgrade=None,upgrade_hash=None):
        validate(m,upgrade,upgrade_hash)
        self.upgrade=upgrade;self.upgrade_hash=upgrade_hash
        if digest(m)!=expected_hash:raise ValueError('campaign approval hash mismatch')
        self.path=Path(path);self.manifest=m;self.hash=expected_hash
        if str(self.path.resolve())!=m['ledger_path']:raise ValueError('campaign ledger cannot be forked to another path')
        if self.path.is_symlink():raise ValueError('ledger symlink')
        if create:fresh(self.path,self.row([],{'kind':'manifest','value':m}))
        elif not self.path.is_file():raise ValueError('missing campaign ledger; never recreate/reset on run')
        self.status()
    @staticmethod
    def row(rows,event):
        v={'sequence':len(rows),'previous':digest(rows[-1]) if rows else None,'event':event}
        return {**v,'hash':digest(v)}
    @contextlib.contextmanager
    def locked(self):
        with os.fdopen(os.open(self.path,os.O_RDWR|os.O_NOFOLLOW),'r+',encoding='utf8') as f:
            fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
            data=f.read(4*1024*1024+1)
            if len(data.encode())>4*1024*1024:raise ValueError('campaign ledger bound')
            rows=[strict(line) for line in data.splitlines()]
            for i,r in enumerate(rows):
                if r!=self.row(rows[:i],r['event']):raise ValueError('campaign chain changed')
            if not rows or rows[0]['event']!={'kind':'manifest','value':self.manifest}:raise ValueError('wrong campaign history')
            if self.upgrade and not any(r['hash']==self.upgrade['ledger_prefix_head'] for r in rows):raise ValueError('upgrade ledger prefix missing')
            self.fold(rows)
            yield f,rows
    def fold(self,rows):
        trials={};calls={};executions={};settled={};returned={};reconciled={};null_rows={};max_calls=100;max_groups=40;suite_hash=None;cost_cap=CAP;session_suite=None;session_trial=None;session_admission=None;session_version=None;session_limits=None;granted_sessions=set();prices={};paired_sessions={};pair_limits=None;quarantined={};risk_accepted={};unlimited_policy=None;unlimited_keys=set();execution_not_started={};execution_partial={};return_rows={}
        for index,row in enumerate(rows[1:],1):
            e=row['event'];kind=e['kind'];key=e['key'];identity(key)
            if kind=='upgrade':
                if digest(e['value'])!=key or e['value']['old_manifest_sha256']!=self.hash:raise ValueError('upgrade provenance mismatch')
            elif kind=='unlimited_policy':
                if unlimited_policy is not None or set(e)!={'kind','key','value'} or digest(e['value'])!=key:raise ValueError('duplicate/invalid unlimited policy')
                validate_unlimited_policy(e['value'],self.hash,rows[index-1]['hash'])
                effective_before={**settled,**reconciled}
                unresolved={k for k in calls if k not in effective_before or effective_before[k]['microusd'] is None}
                allowed=set(e['value']['quarantined_unknown_keys'])
                if allowed!=unresolved or not allowed<=set(quarantined) or any((k not in returned or returned[k]['outcome']!='returned') and k not in execution_not_started and k not in execution_partial for k in executions):raise ValueError('unlimited policy cannot bypass unknown effects')
                unlimited_policy=e['value'];unlimited_keys=allowed;max_calls=None;max_groups=None;cost_cap=None
            elif kind=='call_grant':
                if set(e)!={'kind','key','value'} or digest(e['value'])!=key:raise ValueError('call grant event mismatch')
                validate_call_grant(e['value'],self.hash,rows[index-1]['hash'],max_calls)
                max_calls=e['value']['max_calls']
            elif kind=='evaluation_grant':
                if set(e)!={'kind','key','value'} or digest(e['value'])!=key:raise ValueError('evaluation grant event mismatch')
                if unlimited_policy and e['value'].get('version')!=UNLIMITED_GRANT:raise ValueError('unlimited policy requires scoped browser grant')
                if e['value'].get('version')==UNLIMITED_GRANT and (not unlimited_policy or e['value']['unlimited_policy_sha256']!=digest(unlimited_policy)):raise ValueError('unlimited grant policy mismatch')
                validate_evaluation_grant(e['value'],self.hash,rows[index-1]['hash'],max_calls,max_groups,cost_cap)
                if e['value']['version'] in ('native-browser-session@v1','native-browser-session@v2','native-browser-session@v3','native-browser-session@v4','native-browser-session@v5','native-browser-session@v6','native-browser-session@v7','native-browser-session@v8','native-browser-session@v9','native-browser-session@v10'):
                    session_version=e['value']['version']
                    continuation=e['value']['version']!='native-browser-session@v1'
                    baseline={'native-browser-session@v1':(262,72,225523),'native-browser-session@v2':(264,74,228567),'native-browser-session@v3':(265,74,232177),'native-browser-session@v4':(266,74,234255),'native-browser-session@v5':(270,74,241944),'native-browser-session@v6':(273,74,249657),'native-browser-session@v7':(276,76,256900),'native-browser-session@v8':(277,76,258334),'native-browser-session@v9':(282,76,267746),'native-browser-session@v10':(284,76,275499)}[e['value']['version']]
                    effective_before={**settled,**reconciled}
                    if (session_suite is None if continuation else session_suite is not None) or len(calls)!=baseline[0] or len(executions)!=baseline[1] or any(k not in effective_before or effective_before[k]['microusd'] is None for k in calls) or any((k not in returned or returned[k]['outcome']!='returned') and k not in execution_not_started and k not in execution_partial for k in executions) or sum(v['microusd'] for v in effective_before.values())!=baseline[2]:raise ValueError('session grant needs exact settled historical baseline')
                    session_trial=e['value']['session_trial_sha256'];session_admission=e['value']['session_admission_sha256']
                    if session_trial in trials:raise ValueError('session cannot reopen a historical trial')
                    cost_cap=e['value']['cap_microusd'];session_suite=e['value']['suite_manifest_sha256']
                if e['value']['version'] in (*HARNESS_GRANTS,PAIR_GRANT):
                    grant=e['value'];effective_before={**settled,**reconciled}
                    if any((k not in effective_before or effective_before[k]['microusd'] is None) and k not in unlimited_keys for k in calls) or any((k not in returned or returned[k]['outcome']!='returned') and k not in execution_not_started and k not in execution_partial for k in executions):raise ValueError('harness scope requires settled effects; unknowns cannot be retried')
                    baseline={'calls':len(calls),'execution_groups':len(executions),'settled_microusd':sum(v['microusd'] for v in effective_before.values() if v['microusd'] is not None)}
                    if baseline!=grant['baseline'] or exceeds(baseline['settled_microusd'],cost_cap) or any(v['microusd'] is not None and v['microusd']>calls[k]['reserved_microusd'] for k,v in effective_before.items()):raise ValueError('harness baseline changed or overrun')
                    if grant['session_trial_sha256'] in trials or grant['session_trial_sha256'] in granted_sessions:raise ValueError('harness scope requires a fresh trial, never reopened authority')
                    cost_cap=grant['cap_microusd'];session_version=grant['version'];session_trial=grant['session_trial_sha256'];session_admission=grant['session_admission_sha256'];session_suite=grant['suite_manifest_sha256']
                    session_limits={k:grant[k] for k in ('session_max_calls','session_max_execution_groups','session_cap_microusd')}
                    paired_sessions={};pair_limits=None
                    if grant['version']==PAIR_GRANT:
                        if any(x['trial'] in trials or x['trial'] in granted_sessions for x in grant['arms']):raise ValueError('paired arm cannot reopen old authority')
                        paired_sessions={x['trial']:x for x in grant['arms']};granted_sessions.update(paired_sessions);pair_limits=session_limits
                        session_trial=None;session_admission=None;session_suite=None;session_limits=None
                if 'session_trial_sha256' in e['value']:granted_sessions.add(e['value']['session_trial_sha256'])
                if not unlimited_policy and risk_accepted and any(e['value']['max_calls']>v['max_calls'] or e['value']['max_execution_groups']>v['max_execution_groups'] or e['value']['cap_microusd']>v['max_committed_microusd'] for k,v in risk_accepted.items() if k not in reconciled):raise ValueError('unresolved accepted liability cannot expand cumulative authority')
                max_calls=e['value']['max_calls'];max_groups=e['value']['max_execution_groups'];suite_hash=e['value']['suite_manifest_sha256']
            elif kind=='generation_quarantined':
                v=e['value']
                fields={'version','trial','request_id','admission_sha256','journal_sha256','request_body_sha256','campaign_binding','source','review_sha256','financial_status','output_policy','directory'}
                if set(e)!={'kind','key','value'} or set(v)!=fields or key in quarantined or key not in calls:raise ValueError('quarantine shape/duplicate/call')
                if key in settled and settled[key]['microusd'] is not None or key in reconciled:raise ValueError('only unknown generation may be quarantined')
                if v['version']!='unknown-generation-quarantine@v1' or v['financial_status']!='unverified_ceiling' or v['output_policy']!='permanently_discard':raise ValueError('quarantine cannot settle or unlock financial uncertainty')
                for field in ('trial','admission_sha256','journal_sha256','request_body_sha256','campaign_binding','review_sha256'):identity(v[field])
                if set(v['source'])!={'state_root','source_snapshot','scope'}:raise ValueError('quarantine source shape')
                for source_id in v['source'].values():identity(source_id)
                if calls[key]['trial']!=v['trial'] or calls[key]['binding']!=v['campaign_binding'] or trials[v['trial']]['admission']!=v['admission_sha256'] or key!=digest({'trial':v['trial'],'request':v['request_id'],'kind':'call'}):raise ValueError('quarantine request substitution')
                if any(x['trial']==v['trial'] for x in executions.values()):raise ValueError('quarantine requires no executions in the affected trial')
                if any((k not in returned or returned[k]['outcome']=='unknown') and k not in execution_not_started and k not in execution_partial for k in executions):raise ValueError('unknown execution cannot be quarantined')
                quarantined[key]=v
            elif kind=='generation_risk_accepted':
                v=e['value'];fields={'version','decision_sha256','disclosure_sha256','quarantine_sha256','manifest_sha256','ledger_prefix_head','source','approved_at','expires_at','baseline_committed_microusd','max_committed_microusd','max_calls','max_execution_groups'}
                if set(e)!={'kind','key','value'} or set(v)!=fields or key not in quarantined or key in risk_accepted:raise ValueError('risk acceptance requires exact single quarantine')
                if v['version']!='unknown-generation-risk-acceptance@v1' or v['quarantine_sha256']!=digest(quarantined[key]) or v['manifest_sha256']!=self.hash or v['ledger_prefix_head']!=rows[index-1]['hash'] or v['source']!=quarantined[key]['source']:raise ValueError('risk acceptance binding mismatch')
                for field in ('decision_sha256','disclosure_sha256'):identity(v[field])
                from datetime import datetime
                start=datetime.fromisoformat(v['approved_at']).timestamp();end=datetime.fromisoformat(v['expires_at']).timestamp()
                if not 0<end-start<=7200:raise ValueError('risk acceptance deadline')
                effective_now={**settled,**reconciled}
                current=sum(effective_now[k]['microusd'] if k in effective_now and effective_now[k]['microusd'] is not None else calls[k]['reserved_microusd'] for k in calls)
                if v['baseline_committed_microusd']!=current or v['max_committed_microusd']!=cost_cap or v['max_calls']!=max_calls or v['max_execution_groups']!=max_groups:raise ValueError('risk acceptance cannot reset or expand allowances')
                if any((k not in returned or returned[k]['outcome']=='unknown') and k not in execution_not_started and k not in execution_partial for k in executions):raise ValueError('unknown execution remains blocked')
                risk_accepted[key]=v
            elif kind=='execution_partial_observation_unavailable':
                fields={'kind','key','trial','admission_sha256','request_id','original_return_row','evidence_sha256','evidence'}
                if set(e)!=fields or key not in executions or key in execution_partial or key in execution_not_started or returned.get(key,{}).get('outcome')!='unknown':raise ValueError('partial disposition needs one unknown execution')
                if e['original_return_row']!=return_rows.get(key) or e['trial']!=executions[key]['trial'] or e['admission_sha256']!=executions[key]['binding'] or e['admission_sha256']!=trials[e['trial']]['admission']:raise ValueError('partial execution source binding')
                if key!=digest({'trial':e['trial'],'request':e['request_id'],'kind':'execution'}):raise ValueError('partial request identity mismatch')
                if digest(e['evidence'])!=e['evidence_sha256']:raise ValueError('partial evidence identity')
                binding=load_module('partial_execution_proof','engine-partial-execution.py').validate_evidence(e['evidence'])
                if binding!={k:e[k] for k in ('key','trial','admission_sha256','request_id','original_return_row')}:raise ValueError('partial evidence source mismatch')
                execution_partial[key]=e
            elif kind=='execution_not_started':
                fields={'kind','key','trial','admission_sha256','request_id','original_return_row','evidence_sha256','evidence','review_sha256','review'}
                if set(e)!=fields or key not in executions or key in execution_not_started or key in execution_partial or returned.get(key,{}).get('outcome')!='unknown':raise ValueError('not-started requires one existing unknown execution return')
                if e['original_return_row']!=return_rows.get(key) or e['trial']!=executions[key]['trial'] or e['admission_sha256']!=executions[key]['binding'] or e['admission_sha256']!=trials[e['trial']]['admission']:raise ValueError('not-started execution/admission/return binding mismatch')
                if key!=digest({'trial':e['trial'],'request':e['request_id'],'kind':'execution'}):raise ValueError('not-started request identity mismatch')
                if len(canonical(e['evidence']))>262144 or digest(e['evidence'])!=e['evidence_sha256'] or digest(e['review'])!=e['review_sha256']:raise ValueError('not-started evidence/review hash mismatch')
                binding=load_module('execution_not_started_proof','engine-execution-not-started.py').validate_evidence(e['evidence'])
                if binding!={k:e[k] for k in ('key','trial','admission_sha256','request_id','original_return_row')}:raise ValueError('not-started evidence source mismatch')
                review=e['review']
                if not isinstance(review,dict) or set(review)!={'version','decision','evidence_sha256','reviewer','report_sha256'} or review['version']!='independent-execution-disposition-review@v1' or review['decision']!='confirmed_not_started' or review['evidence_sha256']!=e['evidence_sha256'] or any(not isinstance(review[k],str) or not 1<=len(review[k].encode())<=limit for k,limit in (('reviewer',256),)):raise ValueError('not-started independent review required')
                identity(review['report_sha256'])
                execution_not_started[key]=e
            elif kind=='bill_reconciled':
                if key not in settled or settled[key]['microusd'] is not None or key in reconciled:raise ValueError('only one existing unknown bill may be reconciled')
                if e['original_bill_row']!=null_rows.get(key) or e['microusd']!=0 or e['basis']!='exact_provider_failed_zero':raise ValueError('reconciliation basis mismatch')
                for name in ('lookup_sha256','native_journal_sha256','provider_journal_sha256','native_result_sha256','review_sha256','billing_semantics_sha256'):identity(e[name])
                if not isinstance(e['provider_request_id'],str) or len(e['provider_request_id'])!=36:raise ValueError('provider identity required')
                reconciled[key]=e
            elif kind=='price':
                q=e['value']
                if digest(q)!=key:raise ValueError('price provenance drift')
                validate_quote(q);prices[key]=q
            elif kind=='trial':
                if key in trials:raise ValueError('duplicate trial enrollment')
                if paired_sessions and (key not in paired_sessions or e.get('admission')!=paired_sessions[key]['admission'] or e.get('evaluation_suite_sha256')!=paired_sessions[key]['suite']):raise ValueError('only exact paired arm may enroll')
                if session_trial is not None and (key!=session_trial or e.get('admission')!=session_admission):raise ValueError('only exact approved session may enroll')
                if set(e) not in ({'kind','key','admission'},{'kind','key','admission','evaluation_suite_sha256'}):raise ValueError('trial enrollment shape')
                if 'evaluation_suite_sha256' in e:
                    identity(e['evaluation_suite_sha256'])
                    if e['evaluation_suite_sha256']!=(paired_sessions[key]['suite'] if paired_sessions else suite_hash):raise ValueError('trial suite not granted by campaign')
                identity(e['admission']);trials[key]=e
            elif kind in ('call','execution'):
                if e['trial'] not in trials:raise ValueError('unenrolled trial')
                if unlimited_policy and session_version!=UNLIMITED_GRANT:raise ValueError('unlimited effects require fresh scoped grant')
                if paired_sessions and (e['trial'] not in paired_sessions or trials[e['trial']]['admission']!=paired_sessions[e['trial']]['admission']):raise ValueError('paired budget cannot fund other trial')
                if session_suite is not None and (e['trial']!=session_trial or trials[e['trial']].get('admission')!=session_admission or trials[e['trial']].get('evaluation_suite_sha256')!=session_suite):raise ValueError('session budget cannot fund historical/other suite')
                if any(v['trial']==e['trial'] for v in quarantined.values()):raise ValueError('quarantined trial permanently closed to effects')
                entries=calls if kind=='call' else executions
                if key in calls or key in executions:raise ValueError('duplicate campaign effect')
                identity(e['binding']);entries[key]=e
                if kind=='call':
                    if 'capacity_reservation' in e:
                        v=e['capacity_reservation']
                        if not self.manifest['fixture_only'] and session_version not in (*HARNESS_GRANTS,PAIR_GRANT,'native-browser-session@v6','native-browser-session@v7','native-browser-session@v8','native-browser-session@v9','native-browser-session@v10'):raise ValueError('context reservation not admitted')
                        if set(v)!={'input_tokens','completion_tokens','price_sha256'} or type(v['input_tokens']) is not int or not 0<v['input_tokens']<=capacity.INPUT_TOKENS or v['completion_tokens']!=32768 or v['price_sha256'] not in prices:raise ValueError('context reservation provenance')
                        expected=max(RESERVE,int(quote_estimate(prices[v['price_sha256']],32768,v['input_tokens']).to_integral_value(rounding=decimal.ROUND_CEILING)))
                        if e['reserved_microusd']!=expected:raise ValueError('context reservation drift')
                    elif e['reserved_microusd']!=RESERVE:raise ValueError('reservation drift')
                if session_limits is not None:
                    session_calls={k:v for k,v in calls.items() if v['trial']==session_trial};session_groups=[v for v in executions.values() if v['trial']==session_trial]
                    effective_now={**settled,**reconciled}
                    charged=sum(effective_now[k]['microusd'] if k in effective_now and effective_now[k]['microusd'] is not None else v['reserved_microusd'] for k,v in session_calls.items())
                    if exceeds(len(session_calls),session_limits['session_max_calls']) or exceeds(len(session_groups),session_limits['session_max_execution_groups']) or exceeds(charged,session_limits['session_cap_microusd']):raise ValueError('harness session count/cost allowance exceeded')
                if paired_sessions:
                    effective_now={**settled,**reconciled}
                    totals=[0,0,0]
                    for arm_id,arm in paired_sessions.items():
                        ac={k:v for k,v in calls.items() if v['trial']==arm_id};ag=[v for v in executions.values() if v['trial']==arm_id]
                        charged=sum(effective_now[k]['microusd'] if k in effective_now and effective_now[k]['microusd'] is not None else v['reserved_microusd'] for k,v in ac.items())
                        if len(ac)>arm['max_calls'] or len(ag)>arm['max_operations'] or charged>arm['cap_microusd']:raise ValueError('paired arm allowance exceeded')
                        totals=[totals[0]+len(ac),totals[1]+len(ag),totals[2]+charged]
                    if any(v>pair_limits[k] for v,k in zip(totals,('session_max_calls','session_max_execution_groups','session_cap_microusd'))):raise ValueError('combined paired allowance exceeded')
                if exceeds(len(calls),max_calls) or exceeds(len(executions),max_groups):raise ValueError('campaign count exceeded')
            elif kind in ('bill','return'):
                entries=calls if kind=='bill' else executions;results=settled if kind=='bill' else returned
                if key not in entries or key in results:raise ValueError('unknown/repeated settlement')
                if key in quarantined and kind=='bill' and e['microusd'] is not None:raise ValueError('late quarantine bill requires separate authenticated reconciliation; no output recovery')
                if kind=='bill' and e['microusd'] is not None and (type(e['microusd']) is not int or e['microusd']<0):raise ValueError('invalid actual charge')
                if kind=='return' and e['outcome'] not in ('returned','unknown'):raise ValueError('invalid execution outcome')
                if kind=='return' and 'result_sha256' in e:identity(e['result_sha256'])
                results[key]=e
                if kind=='return':return_rows[key]=row['hash']
                if kind=='bill' and e['microusd'] is None:null_rows[key]=row['hash']
            else:raise ValueError('unrecognized campaign event')
        effective={**settled,**reconciled}
        unknown_calls=[k for k in calls if k not in effective or effective[k]['microusd'] is None]
        historical_unknown_exec=[k for k in executions if k not in returned or returned[k]['outcome']=='unknown']
        unknown_exec=[k for k in historical_unknown_exec if k not in execution_not_started and k not in execution_partial]
        known=sum(v['microusd'] for v in effective.values() if v['microusd'] is not None)
        committed=known+sum(calls[k]['reserved_microusd'] for k in unknown_calls)
        overrun=any(v['microusd'] is not None and v['microusd']>calls[k]['reserved_microusd'] for k,v in effective.items()) or exceeds(committed,cost_cap)
        if session_limits is not None:
            session_charge=sum(effective[k]['microusd'] if k in effective and effective[k]['microusd'] is not None else v['reserved_microusd'] for k,v in calls.items() if v['trial']==session_trial)
            overrun |= exceeds(session_charge,session_limits['session_cap_microusd'])
        if paired_sessions:
            for arm_id,arm in paired_sessions.items():
                arm_charge=sum(effective[k]['microusd'] if k in effective and effective[k]['microusd'] is not None else v['reserved_microusd'] for k,v in calls.items() if v['trial']==arm_id)
                overrun |= arm_charge>arm['cap_microusd']
        return {'calls':len(calls),'execution_groups':len(executions),'known_cost_microusd':known,
                'committed_microusd':committed,'effective_cap_microusd':cost_cap,'unknown_calls':unknown_calls,'unknown_executions':unknown_exec,'blocking_unknown_executions':unknown_exec,'historical_unknown_executions':historical_unknown_exec,'execution_not_started':list(execution_not_started),'partial_observation_unavailable':list(execution_partial),'quarantined_unknown_calls':list(quarantined),'quarantined_trials':[v['trial'] for v in quarantined.values()],'uncertain_committed_microusd':sum(calls[k]['reserved_microusd'] for k in unknown_calls),'financially_unbounded_quarantines':list(quarantined),
                'unlimited_policy':unlimited_policy,'unlimited_policy_sha256':digest(unlimited_policy) if unlimited_policy else None,'risk_acceptances':risk_accepted,'blocking_unknown_calls':[k for k in unknown_calls if k not in risk_accepted and k not in unlimited_keys],
                'overrun':overrun,'stopped':overrun or bool([k for k in unknown_calls if k not in risk_accepted and k not in unlimited_keys] or unknown_exec) or any(time.time()>=__import__('datetime').datetime.fromisoformat(v['expires_at']).timestamp() for k,v in risk_accepted.items() if k in unknown_calls and k not in unlimited_keys),
                'effective_max_calls':max_calls,'remaining_calls':remaining(max_calls,len(calls)),'remaining_execution_groups':remaining(max_groups,len(executions)),
                'effective_max_execution_groups':max_groups,'evaluation_suite_sha256':suite_hash,
                'reconciled_bills':len(reconciled),'fixture_only':self.manifest['fixture_only'],'provider_hard_cap':False},trials,calls,executions
    def append_locked(self,f,rows,event):
        row=self.row(rows,event);self.fold(rows+[row]);f.seek(0,2);f.write(canonical(row).decode()+'\n');f.flush();os.fsync(f.fileno())
    def status(self):
        # A verified point-in-time snapshot is not admission. Mutations still take
        # the exclusive lock and revalidate the complete ledger under that lock.
        # Readers must never acquire a lock that can abort a live writer.
        for _ in range(3):
            with os.fdopen(os.open(self.path,os.O_RDONLY|os.O_NOFOLLOW),'rb') as f:
                before=os.fstat(f.fileno());data=f.read(4*1024*1024+1);after=os.fstat(f.fileno())
            if len(data)>4*1024*1024:raise ValueError('campaign ledger bound')
            if before.st_size!=after.st_size or len(data)!=after.st_size:continue
            if not data.endswith(b'\n'):raise ValueError('incomplete campaign snapshot')
            rows=[strict(line) for line in data.splitlines()]
            for i,row in enumerate(rows):
                if row!=self.row(rows[:i],row['event']):raise ValueError('campaign chain changed')
            if not rows or rows[0]['event']!={'kind':'manifest','value':self.manifest}:raise ValueError('wrong campaign history')
            if self.upgrade and not any(r['hash']==self.upgrade['ledger_prefix_head'] for r in rows):raise ValueError('upgrade ledger prefix missing')
            return self.fold(rows)[0]
        raise ValueError('campaign snapshot changed during read; retry observation')
    def enable_unlimited(self,policy,expected_hash=None):
        """Append explicit trusted instruction; never reset usage or settle unknowns."""
        key=digest(policy)
        if expected_hash is not None and key!=expected_hash:raise ValueError('unlimited policy hash')
        with self.locked() as (f,rows):
            old=next((r['event'] for r in rows if r['event']['kind']=='unlimited_policy'),None)
            if old:
                if old['key']!=key:raise ValueError('unlimited policy already recorded')
            else:self.append_locked(f,rows,{'kind':'unlimited_policy','key':key,'value':policy})
        return self.status()
    def grant_calls(self,grant,expected_hash):
        """Explicit reviewed extension only; does not create or infer user authority."""
        if digest(grant)!=expected_hash:raise ValueError('call grant approval hash mismatch')
        with self.locked() as (f,rows):
            status=self.fold(rows)[0]
            if grant.get('version') in (*HARNESS_GRANTS,PAIR_GRANT,'b91-evaluation-extension@v1','native-pursuit-extension@v1','native-pursuit-extension@v2','native-information-extension@v1','native-information-extension@v2','native-objective-extension@v1','native-objective-extension@v2','native-objective-extension@v3','native-open-guide-extension@v1','native-open-guide-extension@v2','native-browser-initial@v1','native-browser-session@v1','native-browser-session@v2','native-browser-session@v3','native-browser-session@v4','native-browser-session@v5','native-browser-session@v6','native-browser-session@v7','native-browser-session@v8','native-browser-session@v9','native-browser-session@v10'):
                validate_evaluation_grant(grant,self.hash,rows[-1]['hash'],status['effective_max_calls'],status['effective_max_execution_groups'],status['effective_cap_microusd'])
                kind='evaluation_grant'
            else:
                validate_call_grant(grant,self.hash,rows[-1]['hash'],status['effective_max_calls']);kind='call_grant'
            self.append_locked(f,rows,{'kind':kind,'key':expected_hash,'value':grant})
        return self.status()
    def preflight_run(self,*,calls,execution_groups):
        """Locked snapshot readiness; no exclusive reservation or dispatch.

        Run once before any effect; each later effect still checks current limits.
        Legacy profiles use a fixed reservation. Capacity-aware sessions use this
        value as a planning floor; each request may reserve more and stop sooner.
        """
        if type(calls) is not int or not 0<=calls<=200 or type(execution_groups) is not int or not 0<=execution_groups<=100:
            raise ValueError('run readiness bounds')
        with self.locked() as (_,rows):
            status=self.fold(rows)[0]
            if status['stopped']:raise ValueError('campaign uncertain/overrun; run not ready')
            if exceeds(calls,status['remaining_calls']) or exceeds(execution_groups,status['remaining_execution_groups']):raise ValueError('whole run exceeds campaign slots')
            required=calls*RESERVE
            if exceeds(status['committed_microusd']+required,status['effective_cap_microusd']):raise ValueError('whole run exceeds campaign cost allowance')
            return {'ready':True,'exclusive_reservation':False,'ledger_head':rows[-1]['hash'],
                    'planned_calls':calls,'planned_execution_groups':execution_groups,
                    'conservative_allowance_microusd':required,'status':status}
    def attach_upgrade(self):
        if self.upgrade is None:raise ValueError('reviewed upgrade attachment required')
        with self.locked() as (f,rows):
            if any(r['event'].get('kind')=='upgrade' and r['event']['key']==self.upgrade_hash for r in rows):return
            self.append_locked(f,rows,{'kind':'upgrade','key':self.upgrade_hash,'value':self.upgrade})
    def reconcile_failed_zero(self,trial_root,lookup_path,review,review_hash,*,dry_run=False):
        """Read-only exact historical joins, then one append. No provider lookup or retry."""
        if digest(review)!=review_hash or set(review)!={'reviewer','provider_endpoint','lookup_sha256','billing_semantics_sha256','decision'}:
            raise ValueError('exact independent reconciliation review required')
        if review['decision']!='exact_failed_zero' or not isinstance(review['reviewer'],str) or not 1<=len(review['reviewer'])<=256:
            raise ValueError('independent failed-zero review required')
        if not review['provider_endpoint'].startswith('https://api.cheaperinference.com/v1/usage/requests?request_ids='):raise ValueError('official exact request lookup required')
        identity(review['billing_semantics_sha256'])
        d=load_module('campaign_historical_driver','engine-b91-driver.py');root=Path(trial_root).resolve()
        raw=d.e.bounded_file(root/'driver.jsonl',8*1024*1024);native=[d.e.strict(line) for line in raw.splitlines()]
        a=native[0]['event']['value']
        view=object.__new__(d.Driver);view.root=root;view.file=root/'driver.jsonl';view.admission=a;view.fixture=a['mode']=='fixture'
        _,_,done=view.read() # validates full historical hash chain; no current admission or dispatch
        if a['model']!=MODEL or a['provider']!=ENDPOINT or Path(a['trial']).resolve()!=root or view.fixture!=self.manifest['fixture_only']:raise ValueError('historical trial scope mismatch')
        trial=digest({'trial':str(root),'admission':d.e.sha(a)})
        journal=d.e.Journal(root/'provider.jsonl',root/'origin.json',done['origin']['sha256'],correction_limit=a.get('schema_corrections',0))
        events=journal.events();provider_raw=d.e.bounded_file(root/'provider.jsonl',8*1024*1024)
        lookup_raw=d.e.bounded_file(Path(lookup_path),262144)
        if hashlib.sha256(lookup_raw).hexdigest()!=review['lookup_sha256']:raise ValueError('lookup evidence changed')
        lookup=d.e.strict(lookup_raw);data=lookup.get('record',{}).get('data',[])
        if len(data)!=1 or lookup['record'].get('has_more') is not False:raise ValueError('ambiguous provider lookup')
        r=data[0];provider_id=lookup.get('request_id')
        if review['provider_endpoint']!='https://api.cheaperinference.com/v1/usage/requests?request_ids='+str(provider_id):raise ValueError('lookup query identity mismatch')
        if (r.get('request_id')!=provider_id or r.get('model')!=MODEL or r.get('status')!='failed' or r.get('currency')!='USD'
                or any(type(r.get(k)) is not int or r[k]!=0 for k in ('prompt_tokens','completion_tokens','total_tokens'))
                or r.get('billing',{}).get('status')!='failed' or decimal.Decimal(str(r.get('billing',{}).get('billed_cost_usd'))) != 0
                or (r.get('billed_cost_usd') is not None and decimal.Decimal(str(r['billed_cost_usd']))!=0)):
            raise ValueError('positive terminal failed zero-charge evidence required')
        candidates=[]
        for name,value in done.items():
            receipt=value.get('receipt',{}) if isinstance(value,dict) else {}
            if name.startswith('transport_') and receipt.get('response_request_ids')==[provider_id] and receipt.get('response_status')=='received_http_error' and receipt.get('usd') is None:
                request=name.removeprefix('transport_')
                intent=next((e for e in events if e['kind']=='dispatch_intent' and e['id']==request),None)
                if intent:candidates.append((request,intent,value))
        if len(candidates)!=1:raise ValueError('provider ID not uniquely bound to original native transport')
        request,intent,value=candidates[0]
        def timestamp(text):
            t=datetime.datetime.fromisoformat(text.replace('Z','+00:00'))
            if t.tzinfo is None:raise ValueError('timezone-aware reconciliation chronology required')
            return t.timestamp()
        native_intent=next(row['event'] for row in native if row['event'].get('kind')=='intent' and row['event'].get('name')=='transport_'+request)
        dispatched=timestamp(native_intent['at']);created=timestamp(r['created_at']);retrieved=timestamp(lookup['retrieved_at'])
        # At most five seconds of provider/host clock skew, never an arbitrary older request.
        if not dispatched-5<=created<=retrieved or retrieved<dispatched:raise ValueError('reconciliation chronology mismatch')
        key=digest({'trial':trial,'request':request,'kind':'call'})
        with self.locked() as (f,rows):
            _,trials,calls,_=self.fold(rows)
            if trial not in trials or trials[trial]['admission']!=d.e.sha(a) or key not in calls or calls[key]['binding']!=intent['body_sha256']:raise ValueError('campaign/native request mismatch')
            bill=next((row for row in rows if row['event'].get('kind')=='bill' and row['event']['key']==key and row['event']['microusd'] is None),None)
            if bill is None:raise ValueError('original unknown bill missing')
            event={'kind':'bill_reconciled','key':key,'original_bill_row':bill['hash'],'microusd':0,'basis':'exact_provider_failed_zero',
                'provider_request_id':provider_id,'lookup_sha256':review['lookup_sha256'],'native_journal_sha256':hashlib.sha256(raw).hexdigest(),
                'provider_journal_sha256':hashlib.sha256(provider_raw).hexdigest(),'native_result_sha256':digest(value),'review_sha256':review_hash,
                'billing_semantics_sha256':review['billing_semantics_sha256']}
            prior=next((row['event'] for row in rows if row['event'].get('kind')=='bill_reconciled' and row['event']['key']==key),None)
            if prior is not None:
                if prior!=event:raise ValueError('reconciliation cannot be rewritten')
                return self.fold(rows)[0]
            result=self.fold(rows+[self.row(rows,event)])[0]
            if dry_run:return {'proposed_event':event,'status_after':result,'applied':False}
            self.append_locked(f,rows,event)
            return result
    def record_price(self,quote):
        with self.locked() as (f,rows):self.append_locked(f,rows,{'kind':'price','key':digest(quote),'value':quote})
    def enroll(self,trial,admission_hash,*,suite_sha256=None):
        identity(trial);identity(admission_hash)
        if suite_sha256 is not None:identity(suite_sha256)
        event={'kind':'trial','key':trial,'admission':admission_hash,**({'evaluation_suite_sha256':suite_sha256} if suite_sha256 is not None else {})}
        with self.locked() as (f,rows):
            _,trials,_,_=self.fold(rows)
            if trial in trials:
                if trials[trial]!=event:raise ValueError('trial admission changed')
                return
            self.append_locked(f,rows,event)
    def quarantine_generation(self,root,request_id,journal_sha256,review,*,dry_run=False):
        module=load_module('generation_quarantine','engine-generation-quarantine.py')
        value,launch_lock=module.inspect(self,root,request_id,journal_sha256,review)
        try:
            key=digest({'trial':value['trial'],'request':request_id,'kind':'call'})
            event={'kind':'generation_quarantined','key':key,'value':value}
            with self.locked() as (f,rows):
                prior=next((r['event'] for r in rows if r['event']['kind']=='generation_quarantined' and r['event']['key']==key),None)
                if prior is not None:
                    if prior!=event:raise ValueError('quarantine identity changed')
                    return prior
                self.fold(rows+[self.row(rows,event)])
                if not dry_run:self.append_locked(f,rows,event)
            return event
        finally:launch_lock.close()
    def accept_generation_risk(self,key,decision,disclosure):
        # Caller holds the reviewed local decision lock. This never settles cost.
        with self.locked() as (f,rows):
            old=next((r['event'] for r in rows if r['event']['kind']=='generation_risk_accepted' and r['event']['key']==key),None)
            if old:
                if old['value']['decision_sha256']!=digest(decision):raise ValueError('risk decision substitution')
                return old
            status=self.fold(rows)[0];q=next(r['event']['value'] for r in rows if r['event']['kind']=='generation_quarantined' and r['event']['key']==key)
            value={'version':'unknown-generation-risk-acceptance@v1','decision_sha256':digest(decision),'disclosure_sha256':digest(disclosure),'quarantine_sha256':digest(q),'manifest_sha256':self.hash,'ledger_prefix_head':rows[-1]['hash'],'source':q['source'],'approved_at':decision['at'],'expires_at':decision['expires_at'],'baseline_committed_microusd':status['committed_microusd'],'max_committed_microusd':status['effective_cap_microusd'],'max_calls':status['effective_max_calls'],'max_execution_groups':status['effective_max_execution_groups']}
            if any(disclosure.get(k)!=v for k,v in {'max_calls':status['effective_max_calls'],'max_execution_groups':status['effective_max_execution_groups'],'max_committed_microusd':status['effective_cap_microusd'],'baseline_committed_microusd':status['committed_microusd'],'manifest_sha256':self.hash}.items()):raise ValueError('approved allowance or accounting changed')
            if disclosure.get('proof',{}).get('value')!=q:raise ValueError('approved disposal proof changed')
            from datetime import datetime
            start=datetime.fromisoformat(decision['at']);end=datetime.fromisoformat(decision['expires_at'])
            if start.tzinfo is None or end.tzinfo is None or not start.timestamp()<=time.time()<end.timestamp() or end.timestamp()-start.timestamp()>7200:raise ValueError('approval time window invalid')
            if decision.get('choice')!='approve' or decision.get('disclosure_sha256')!=digest(disclosure) or disclosure['unknown_key']!=key:raise ValueError('explicit bound approval required')
            event={'kind':'generation_risk_accepted','key':key,'value':value};self.append_locked(f,rows,event);return event
    def require_output_eligible(self,trial):
        if trial in self.status().get('quarantined_trials',[]):raise ValueError('generation quarantined: old output and trial are permanently ineligible')
    def reserve(self,kind,trial,request,binding,*,reservation=RESERVE,capacity_reservation=None,dry_run=False):
        from datetime import datetime
        identity(trial);identity(binding)
        key=digest({'trial':trial,'request':request,'kind':kind})
        with self.locked() as (f,rows):
            status,_,calls,executions=self.fold(rows)
            if any(not datetime.fromisoformat(v['approved_at']).timestamp()<=time.time()<datetime.fromisoformat(v['expires_at']).timestamp() for k,v in status.get('risk_acceptances',{}).items() if k in status['unknown_calls'] and not status.get('unlimited_policy')):raise ValueError('accepted-risk fresh allowance expired or not yet active')
            if key in calls or key in executions:raise ValueError('effect already reserved; reconcile, never retry')
            if status['stopped']:raise ValueError('campaign uncertain/overrun; no new effects')
            if kind=='call' and (exceeds(status['calls']+1,status['effective_max_calls']) or exceeds(status['committed_microusd']+reservation,status['effective_cap_microusd'])):raise ValueError('campaign call/dollar limit')
            if kind=='execution' and exceeds(status['execution_groups']+1,status['effective_max_execution_groups']):raise ValueError('campaign execution limit')
            if kind not in ('call','execution'):raise ValueError('effect kind')
            event={'kind':kind,'key':key,'trial':trial,'binding':binding,**({'reserved_microusd':reservation,**({'capacity_reservation':capacity_reservation} if capacity_reservation else {})} if kind=='call' else {})}
            if dry_run:self.fold(rows+[self.row(rows,event)])
            else:self.append_locked(f,rows,event)
        return key
    def settle(self,kind,key,value):
        with self.locked() as (f,rows):
            self.append_locked(f,rows,{'kind':kind,'key':key,**({'microusd':value} if kind=='bill' else {'outcome':value})})
    def reconcile_execution_partial(self,key,evidence):
        identity(key)
        proof=load_module('partial_execution_proof','engine-partial-execution.py')
        binding=proof.validate_evidence(evidence)
        if binding['key']!=key:raise ValueError('partial key mismatch')
        event={'kind':'execution_partial_observation_unavailable',**binding,'evidence_sha256':digest(evidence),'evidence':evidence}
        with proof.locked_source(evidence), self.locked() as (f,rows):
            proof.validate_current_evidence(evidence)
            self.append_locked(f,rows,event)
        return self.status()
    def reconcile_execution_not_started(self,key,evidence,review):
        """Record independently proven non-start; keep original return/count immutable."""
        identity(key)
        proof=load_module('execution_not_started_proof','engine-execution-not-started.py')
        binding=proof.validate_evidence(evidence)
        if binding.get('key')!=key:raise ValueError('not-started key mismatch')
        event={'kind':'execution_not_started',**binding,'evidence_sha256':digest(evidence),'evidence':evidence,'review_sha256':digest(review),'review':review}
        with self.locked() as (f,rows):
            proof.validate_current_evidence(evidence)
            self.append_locked(f,rows,event)
        return self.status()
    def run_call(self,trial,request,body,timeout,transport,*,payload_limit=32768,preflight_only=False):
        if (body.get('model')!=MODEL or type(body.get('max_completion_tokens')) is not int or body.get('max_completion_tokens') not in (8192,32768) or body.get('stream') is not False
            or payload_limit not in (16384,32768,capacity.PAYLOAD_BYTES) or len(json.dumps(body,allow_nan=False,ensure_ascii=(payload_limit==16384)).encode())>payload_limit or type(timeout) not in (int,float) or not 0<timeout<=240):raise ValueError('campaign model/request bound')
        with self.locked() as (_,rows):
            quotes=[r['event']['value'] for r in rows if r['event']['kind']=='price']
            grants=[r['event']['value'] for r in rows if r['event']['kind']=='evaluation_grant']
            exact_capacity_trial=bool(grants) and (any(x['trial']==trial for x in grants[-1].get('arms',[])) if grants[-1]['version']==PAIR_GRANT else grants[-1].get('session_trial_sha256')==trial)
            if body['max_completion_tokens']!=8192 and not self.manifest['fixture_only'] and (not grants or grants[-1]['version'] not in (*HARNESS_GRANTS,PAIR_GRANT,'native-browser-session@v3','native-browser-session@v4','native-browser-session@v5','native-browser-session@v6','native-browser-session@v7','native-browser-session@v8','native-browser-session@v9','native-browser-session@v10') or not exact_capacity_trial):raise ValueError('completion capacity not admitted for this trial')
        large=payload_limit==capacity.PAYLOAD_BYTES
        if large and not self.manifest['fixture_only'] and (not grants or grants[-1]['version'] not in (*HARNESS_GRANTS,PAIR_GRANT,'native-browser-session@v6','native-browser-session@v7','native-browser-session@v8','native-browser-session@v9','native-browser-session@v10') or not exact_capacity_trial):raise ValueError('large context requires exact trial admission')
        input_tokens=capacity.prompt_upper(body) if large else 32768
        if not quotes:
            if not self.manifest['fixture_only']:raise ValueError('fresh official v2 price quote required')
            estimate=decimal.Decimal(RESERVE) # Explicit fixture-only allowance; no operational quote inferred.
        else:
            quote=quotes[-1]
            if quote.get('version') not in ('b91-price-quote@v2','b91-price-quote@v3') or not 0<=time.time()-quote['retrieved_at']<=3600:raise ValueError('fresh official v2 price quote required')
            if quote.get('version')=='b91-price-quote@v3' and set(body)-{'model','stream','max_completion_tokens','messages','response_format'}:raise ValueError('tier quote request scope mismatch')
            if quote.get('version')=='b91-price-quote@v3' and not quote['pricing_scope']['effectiveAt']<=time.time()*1000<quote['pricing_scope']['expiresAt']:raise ValueError('tier price expired')
            estimate=quote_estimate(quote,body['max_completion_tokens'],input_tokens)
        if estimate>RESERVE and not large:raise ValueError('advertised rate estimate exceeds reservation')
        reservation=max(RESERVE,int(estimate.to_integral_value(rounding=decimal.ROUND_CEILING)))
        capacity_record={'input_tokens':input_tokens,'completion_tokens':body['max_completion_tokens'],'price_sha256':digest(quote)} if large and quotes else None
        key=self.reserve('call',trial,request,digest(body),reservation=reservation,capacity_reservation=capacity_record,dry_run=preflight_only)
        if preflight_only:return {'admissible':True,'reserved_microusd':reservation,'key':key,'mutated':False}
        try:result=transport(body,timeout)
        except BaseException:
            self.settle('bill',key,None);raise
        receipt=result.get('receipt',{});usd=receipt.get('usd') if receipt.get('status')=='settled' else None
        actual=None
        if usd is not None:
            try:
                amount=decimal.Decimal(str(usd))
                if not amount.is_finite() or amount<0:raise ValueError('invalid charge')
                actual=int((amount*1000000).to_integral_value(rounding=decimal.ROUND_CEILING))
            except (ValueError,decimal.InvalidOperation):actual=None
        self.settle('bill',key,actual)
        usage=receipt.get('usage',{})
        if large and (type(usage.get('prompt_tokens')) is not int or usage['prompt_tokens']>input_tokens or type(usage.get('completion_tokens')) is not int or usage['completion_tokens']>body['max_completion_tokens']):result={**result,'resource_stop':'request_token_estimate_exceeded_or_unknown'}
        if actual is None or actual>reservation:result={**result,'resource_stop':'unknown_usage' if actual is None else 'resource_overrun'}
        return result
    def run_execution(self,trial,group,admission_hash,worker):
        key=self.reserve('execution',trial,group,admission_hash)
        try:result=worker()
        except BaseException:
            self.settle('return',key,'unknown');raise
        with self.locked() as (f,rows):self.append_locked(f,rows,{'kind':'return','key':key,'outcome':'returned','result_sha256':digest(result)})
        return result


def official_preflight():
    req=urllib.request.Request(CATALOG,headers={'Accept':'application/json'})
    with urllib.request.urlopen(req,timeout=30) as response:raw=response.read(262145)
    if len(raw)>262144:raise ValueError('catalog bound')
    record=strict(raw)
    if record.get('id')!=MODEL:raise ValueError('catalog model mismatch')
    threshold=record.get('input_token_price_threshold')
    if type(threshold) is not int or threshold==0 or threshold < -1:raise ValueError('unsupported tiered pricing')
    scope=None
    if threshold>0:
        if record.get('default_pricing_scope')!=DEFAULT_PRICE_SCOPE:raise ValueError('unsupported tiered default scope')
        matches=[x for x in record.get('pricing_scopes',[]) if x.get('scope')==DEFAULT_PRICE_SCOPE]
        if len(matches)!=1:raise ValueError('ambiguous tiered pricing scope')
        scope={k:matches[0][k] for k in ('id','version','scope','tiers','effectiveAt','expiresAt')}
        rates=scope_rates(scope,threshold)

    quote={'version':'b91-price-quote@v2','model':MODEL,'input_token_price_threshold':-1,
           'rates':{key:str(record[key]) for key in PRICE_CEILINGS},'ceilings':dict(PRICE_CEILINGS),
           'retrieved_at':time.time(),'catalog_sha256':hashlib.sha256(raw).hexdigest()}
    if scope is not None:quote.update(version='b91-price-quote@v3',input_token_price_threshold=threshold,pricing_scope=scope,rates=rates)
    validate_quote(quote)
    if quote_estimate(quote)>RESERVE:raise ValueError('official quote exceeds reservation')
    return quote


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode',choices=['prepare','inspect','run','attach-upgrade','attach-grant','preflight'])
    parser.add_argument('--campaign',type=Path,required=True)
    parser.add_argument('--campaign-sha256')
    parser.add_argument('--upgrade',type=Path)
    parser.add_argument('--upgrade-sha256')
    parser.add_argument('--grant',type=Path)
    parser.add_argument('--grant-sha256')
    parser.add_argument('--planned-calls',type=int)
    parser.add_argument('--planned-execution-groups',type=int)
    parser.add_argument('--admission',type=Path)
    parser.add_argument('--admission-sha256')
    parser.add_argument('--reviews',type=Path)
    parser.add_argument('--runtime-review-admission',type=Path)
    parser.add_argument('--runtime-review-admission-sha256')
    parser.add_argument('--allow-paid-inference',action='store_true')
    parser.add_argument('--allow-target-execution',action='store_true')
    args=parser.parse_args()
    if bool(args.runtime_review_admission)!=bool(args.runtime_review_admission_sha256):parser.error('runtime review requires both admission and exact hash')
    if args.runtime_review_admission and args.mode!='run':parser.error('runtime review activation is run-only')
    root=args.campaign.resolve()
    if args.mode=='prepare':
        root.mkdir(parents=True,exist_ok=False)
        value=manifest(hashlib.sha256(os.urandom(32)).hexdigest(),root/'ledger.jsonl')
        fresh(root/'manifest.json',value)
        ledger=Ledger(root/'ledger.jsonl',value,digest(value),create=True)
        print(json.dumps({'campaign_sha256':digest(value),'status':ledger.status(),'dispatch_enabled':False}));return
    value=strict((root/'manifest.json').read_bytes())
    if args.mode in ('run','attach-upgrade','attach-grant') and args.campaign_sha256!=digest(value):parser.error('exact campaign approval hash required')
    upgrade=strict(args.upgrade.read_bytes()) if args.upgrade else None
    ledger=Ledger(root/'ledger.jsonl',value,digest(value),upgrade=upgrade,upgrade_hash=args.upgrade_sha256)
    if args.mode=='attach-upgrade':ledger.attach_upgrade();print(json.dumps(ledger.status()));return
    if args.mode=='attach-grant':
        if args.grant is None or args.grant_sha256 is None:parser.error('explicit grant file and approval hash required')
        print(json.dumps(ledger.grant_calls(strict(args.grant.read_bytes()),args.grant_sha256)));return
    if args.mode=='preflight':
        if args.planned_calls is None or args.planned_execution_groups is None:parser.error('whole-run call and execution-group counts required')
        print(json.dumps(ledger.preflight_run(calls=args.planned_calls,execution_groups=args.planned_execution_groups)));return
    if args.mode=='inspect':print(json.dumps(ledger.status(),indent=2));return
    if upgrade is not None:
        with ledger.locked() as (_,rows):
            if not any(r['event'].get('kind')=='upgrade' and r['event']['key']==args.upgrade_sha256 for r in rows):raise ValueError('attach reviewed implementation upgrade before live run')
    if not args.allow_paid_inference or not args.allow_target_execution or not args.admission or not args.reviews:
        parser.error('run requires existing trial admission/reviews and both explicit effect opt-ins')
    driver=load_module('campaign_native_driver','engine-b91-driver.py')
    admission=driver.e.strict(driver.e.bounded_file(args.admission,16384));trial_root=Path(admission['trial']).resolve()
    if args.admission_sha256!=driver.e.sha(admission):parser.error('exact trial admission hash required')
    driver.validate_admission(admission,trial_root,False)
    if admission.get('evaluation_suite_sha256') is not None:
        identity(admission['evaluation_suite_sha256'])
        if admission['evaluation_suite_sha256']!=ledger.status()['evaluation_suite_sha256']:raise ValueError('trial suite not granted by campaign')
    if admission['model']!=MODEL or value['fixture_only']:raise ValueError('campaign only admits live Luna synthetic trials')
    review_module=None;review_admission=None
    if args.runtime_review_admission:
        review_module=load_module('campaign_project_runtime_review','engine-project-runtime-review.py')
        review_admission=driver.e.strict(driver.e.bounded_file(args.runtime_review_admission,16384))
        if review_module.digest(review_admission)!=args.runtime_review_admission_sha256 or review_admission!=review_module.admission(admission,value):raise ValueError('exact source/campaign-bound runtime review admission required')
    trial=digest({'trial':str(trial_root),'admission':driver.e.sha(admission)})
    if ledger.status()['stopped']:raise ValueError('campaign uncertain/overrun; no new driver effects')
    # A metadata failure occurs before native intent, initialization or execution.
    quote=official_preflight()
    ledger.record_price(quote)
    ledger.enroll(trial,driver.e.sha(admission),suite_sha256=admission.get('evaluation_suite_sha256'))
    transport=driver.e.module('campaign_synthetic_transport','engine-synthetic-transport.py')
    def provider(body,timeout):
        # One current official quote per run invocation; reviews/resume start a new
        # invocation. Metadata failure cannot strand an already-written native intent.
        if time.time()-quote['retrieved_at']>3600:raise ValueError('price quote expired')
        request=[x for x in instance.journal().events() if x['kind']=='dispatch_intent'][-1]['id']
        stage=json.loads(body['messages'][1]['content'])['stage']
        def invoke(b,t):
            result=transport.invoke(b,t,expected_model=MODEL,payload_limit=driver.stage_payload_limit(admission,stage))
            # Price provenance contains no prompt, credentials or provider raw response.
            result=dict(result);result['campaign_price_preflight']=quote;return result
        return ledger.run_call(trial,request,body,timeout,invoke,payload_limit=driver.stage_payload_limit(admission,stage))
    def worker(controller,stage):
        return ledger.run_execution(trial,stage,driver.e.sha(admission),lambda:driver.live_worker(controller,stage))
    instance=driver.Driver(trial_root,admission,provider,worker,driver.FileReviewer(args.reviews))
    if review_module is not None:
        def review_transport(body,timeout):
            if time.time()-quote['retrieved_at']>3600:raise ValueError('price quote expired')
            return transport.invoke(body,timeout,expected_model=MODEL,payload_limit=32768)
        instance.reviewer=review_module.RuntimeReviewer(instance,review_admission,ledger,review_transport,instance.reviewer)
    try:result=instance.run()
    except (driver.Blocked,driver.e.PayloadBoundError,ValueError) as error:
        result={'status':'blocked','reason':str(error),'no_retry':True,'live_b91_passed':False}
    print(json.dumps({'driver':result,'campaign':ledger.status()},indent=2))

if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__=='__main__':main()
