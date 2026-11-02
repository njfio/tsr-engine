"""Replay authenticated settled content, never a provider invocation or new judgment."""
import copy,decimal

def latest(adapter):
    rows,pending,done=adapter.read()
    dispatches=[k[9:] for k in done if k.startswith('dispatch_')]
    if not dispatches:return None
    ident=dispatches[-1];record=done.get('request_body_'+ident);dispatch=done['dispatch_'+ident]
    if not record:return None
    kind=record.get('kind')
    if not kind:return None # Historical content without typed dispatch identity stays fail-closed.
    if any(k in done for k in ('generated_'+ident,'reviewed_'+ident,'rejected_'+ident,'provider_zero_'+ident,kind)):return None
    if 'candidate' not in dispatch:raise ValueError('retained response content unavailable; no redispatch')
    if dispatch.get('resource_stop'):raise ValueError('retained response resource stop')
    if set(pending)-{'settle_'+ident}:raise ValueError('unknown nested effect; no settled replay')
    source=adapter.recovery_source()
    if source!=record['source']:raise ValueError('settled response source changed')
    index=next(sum(r['event']['kind']=='result' and r['event'].get('name','').startswith(('dispatch_','not_dispatched_')) for r in rows[:i]) for i,r in enumerate(rows) if r['event'].get('name')=='dispatch_'+ident and r['event']['kind']=='result')
    sha=adapter.recovery_sha
    if ident!=sha({'admission':sha(adapter.admission),'kind':kind,'index':index,'body':sha(record['body'])}) or record['body_sha256']!=sha(record['body']) or dispatch['candidate_sha256']!=sha(dispatch['candidate']):raise ValueError('settled response identity drift')
    meter=dispatch['receipt'];usage=meter.get('usage',{})
    if meter.get('usage_status')!='reported' or meter.get('usd') is None or any(type(usage.get(k)) is not int for k in ('prompt_tokens','completion_tokens')):raise ValueError('unknown response usage')
    elapsed=meter.get('elapsed_seconds')
    if type(elapsed) not in (int,float) or elapsed<0:raise ValueError('unknown response duration')
    cost=int((decimal.Decimal(meter['usd'])*1000000).to_integral_value(rounding=decimal.ROUND_CEILING))
    with adapter.ledger.locked() as (_,campaign):
        status=adapter.ledger.fold(campaign)[0]
        if status['stopped']:raise ValueError('campaign unresolved; no settled replay')
        key=adapter.recovery_sha({'trial':adapter.trial,'request':ident,'kind':'call'})
        bills=[r['event'] for r in campaign if r['event'].get('key')==key and r['event']['kind']=='bill']
        if len(bills)!=1 or bills[0].get('microusd')!=cost:raise ValueError('settled campaign bill mismatch')
    return {'request':ident,'kind':kind,'index':index,'record':record,'dispatch':dispatch,'usage':{'request_id':ident,'prompt_tokens':usage['prompt_tokens'],'output_tokens':usage['completion_tokens'],'duration_ns':int(elapsed*1e9),'external_cost_microusd':cost,'outcome':'measured'}}

def recover(adapter):
    if not getattr(adapter,'launch_lock_owned',False):raise ValueError('exclusive launch ownership required')
    adapter.recovery_load('response_recovery_stop','engine-browser-predispatch.py').check_stop(adapter)
    item=latest(adapter)
    if item is None:return 0
    ident=item['request'];name='settle_'+ident;_,pending,done=adapter.read()
    if name in done:return 0
    def settle():
        adapter.recovery_load('response_recovery_stop2','engine-browser-predispatch.py').check_stop(adapter)
        return adapter.controller.json_cli(('project-resource-settle',),item['usage'],'--usage')
    if name in pending:
        value=settle();adapter.append({'kind':'result','name':name,'intent':adapter.recovery_sha(pending[name]),'value':value})
    else:adapter.op(name,settle)
    return 1

def replay(adapter,kind,body):
    adapter.recovery_load("settled_replay_stop","engine-browser-predispatch.py").check_stop(adapter)
    item=latest(adapter)
    if item is None:return None
    if item['kind']!=kind or item['record']['body']!=body:raise ValueError('retained response callback changed')
    done=adapter.read()[2];ident=item['request'];settled=done.get('settle_'+ident)
    if settled is None or any(settled.get(k,0) for k in ('stopped','unknown_usage','unsettled_requests','unknown_compute_requests','unknown_external_cost_requests')):raise ValueError('native usage unresolved')
    adapter.last_ident=ident;adapter.last_result={'value':copy.deepcopy(item['dispatch']['candidate']),'receipt':{**copy.deepcopy(item['dispatch']['receipt']),'status':'settled'}}
    receipt={**item['dispatch']['receipt'],'status':'settled','mode':'fixture' if adapter.fixture else 'accounted'}
    return ident,copy.deepcopy(item['dispatch']['candidate']),receipt,item['index']
