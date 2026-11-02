"""Exact no-dispatch transport proof. Never infer safety from an empty directory alone."""
import hashlib
from pathlib import Path

def check_stop(adapter):
    root=getattr(adapter,"recovery_session_root",None)
    if (adapter.root/"STOP").exists() or (root is not None and ((Path(root)/"STOP").exists() or any(Path(root).glob("checkpoint-*/STOP")))):
        raise ValueError("Stop blocks pre-dispatch recovery")

def inspect(adapter):
    rows,pending,done=adapter.read()
    if any(not k.startswith(('not_dispatched_','predispatch_input_')) for k in pending):raise ValueError('unknown nested effect')
    check_stop(adapter)
    candidates=[k[8:] for k in done if k.startswith('reserve_') and 'dispatch_'+k[8:] not in done and 'not_dispatched_'+k[8:] not in done]
    if not candidates:return None
    if len(candidates)!=1:raise ValueError('ambiguous undispatched reservation')
    ident=candidates[0]
    if any(e['event'].get('name')=='dispatch_'+ident for e in rows):raise ValueError('dispatch intent remains unknown')
    if set(pending)-{'not_dispatched_'+ident,'predispatch_input_'+ident}:raise ValueError('foreign pending disposition')
    record=done.get('request_body_'+ident)
    if not record or record['body_sha256']!=adapter.recovery_sha(record['body']):raise ValueError('missing exact retained request')
    current=adapter.controller.request();source={k:current['coding']['snapshot'] if k=='source_snapshot' else current['next_attention']['scope'] if k=='scope' else current[k] for k in ('state_root','source_snapshot','scope')}
    if source!=record['source']:raise ValueError('stale pre-dispatch source')
    count=sum(k.startswith(('dispatch_','not_dispatched_')) for k in done)
    kind=record.get('kind','proposal')
    if not isinstance(kind,str) or not kind or len(kind)>256:raise ValueError('invalid callback kind')
    expected=adapter.recovery_sha({'admission':adapter.recovery_sha(adapter.admission),'kind':kind,'index':count,'body':record['body_sha256']})
    if ident!=expected:raise ValueError('not an exact pending callback request')
    output=adapter.root/('output-'+str(count))
    if output.is_symlink() or not output.is_dir() or any(output.iterdir()):raise ValueError('nonempty or absent output quarantine')
    with adapter.ledger.locked() as (_,campaign):
        status=adapter.ledger.fold(campaign)[0]
        if status['stopped'] or status.get('blocking_unknown_calls') or status.get('blocking_unknown_executions'):raise ValueError('unknown campaign effect')
        key=adapter.recovery_sha({'trial':adapter.trial,'request':ident,'kind':'call'})
        if any(row['event'].get('key')==key for row in campaign):raise ValueError('campaign request already exists')
        prefix=campaign[-1]['hash']
    reserve_row=next(row for row in rows if row['event'].get('name')=='reserve_'+ident and row['event']['kind']=='result')
    evidence={'version':'proven-no-dispatch@v1','request':ident,'trial':adapter.trial,'admission':adapter.recovery_sha(adapter.admission),'source':source,'body_sha256':record['body_sha256'],'adapter_reserve_prefix':reserve_row['hash'],'campaign_prefix':prefix,'output_index':count,'no_dispatch_intent':True,'no_campaign_request':True}
    return {'request':ident,'kind':kind,'evidence':evidence,'body':record['body'],'proof':{'state_root':source['state_root'],'reservation':{'request_id':ident,'state_root':source['state_root'],'envelope':record['body_sha256'],'model':record['body']['model'],'tokens_upper':adapter.predispatch_tokens(record['body'])},'evidence':adapter.recovery_sha(evidence),'review':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}}

def recover(adapter):
    if not getattr(adapter,"launch_lock_owned",False):raise ValueError("exclusive launch ownership required")
    candidate=inspect(adapter)
    if candidate is None:return 0
    ident=candidate['request'];rows,pending,done=adapter.read();name='not_dispatched_'+ident
    prepared=done.get('predispatch_input_'+ident)
    if prepared is None:
        input_name='predispatch_input_'+ident
        if set(pending)=={input_name}:
            # Pure local capture: freshly revalidate all bindings before completing
            # the original intent; there was no provider/native side effect here.
            check_stop(adapter)
            adapter.append({'kind':'result','name':input_name,'intent':adapter.recovery_sha(pending[input_name]),'value':candidate})
            prepared=candidate;rows,pending,done=adapter.read()
        elif pending:raise ValueError('missing prepared no-dispatch proof')
        else:prepared=adapter.op(input_name,lambda:candidate)
    # Prefix may advance elsewhere, but all original evidence and source must persist.
    for key in ('request','kind','body'):
        if prepared[key]!=candidate[key]:raise ValueError('pre-dispatch proof changed')
    for key in ('source','body_sha256','adapter_reserve_prefix','output_index','trial','admission'):
        if prepared['evidence'][key]!=candidate['evidence'][key]:raise ValueError('pre-dispatch binding changed')
    def resolve():
        check_stop(adapter)
        return adapter.controller.json_cli(('project-resource-not-dispatched',),prepared['proof'],'--proof')
    if name in pending:
        value=resolve();adapter.append({'kind':'result','name':name,'intent':adapter.recovery_sha(pending[name]),'value':value})
    else:adapter.op(name,resolve)
    return 1

def ticket(adapter):
    _,pending,done=adapter.read()
    if pending:return None
    count=sum(k.startswith(('dispatch_','not_dispatched_')) for k in done)
    available=[v for k,v in done.items() if k.startswith('predispatch_input_') and 'not_dispatched_'+v['request'] in done and v['evidence']['output_index']==count-1]
    if not available:return None
    if len(available)!=1:raise ValueError('ambiguous pre-dispatch ticket')
    v=available[0];current=adapter.native('request',None)
    if v['evidence']['source']!={k:current[k] for k in ('state_root','source_snapshot','scope')}:raise ValueError('stale recovered callback')
    return v
