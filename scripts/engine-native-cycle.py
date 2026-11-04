#!/usr/bin/env python3
"""Explicit launcher for an admitted synthetic native-choice cycle; no automatic setup."""
import argparse
import importlib.util
import json
import os
import fcntl
import time
from pathlib import Path


def require_recovery_not_stopped(journal):
    root=Path(journal)
    if (root/'STOP').exists() or any(root.glob('checkpoint-*/STOP')):
        raise ValueError('session/checkpoint stopped; no recovery')

def complete_fresh_direct_return(adapter,cycle,cycle_admission):
    """Assess already returned evidence before the first choice, never ahead of replay."""
    if adapter.admission.get('feedback_events') or not adapter.admission.get('criterion_return_profile'):return
    rows,pending,done=cycle.read()
    if pending or done or len(rows)!=1:return  # Existing cycle owns its callbacks and terminal.
    require_recovery_not_stopped(cycle_admission['journal'])
    if (adapter.root/'STOP').exists():raise ValueError('adapter stopped; no startup reflection')
    # Existing durable reflection owns exact retained responses and native joins.
    # On restart a still-new cycle may re-enter; current assessments are inert.
    adapter.complete_pending_return()

def refresh_price(ledger,module,*,now=None,fetch=None):
    """Read-only catalog refresh before native/provider reservation, never inference."""
    now=time.time() if now is None else now
    with ledger.locked() as (_,rows):
        quotes=[r['event']['value'] for r in rows if r['event']['kind']=='price']
    if not quotes:raise ValueError('initial independently checked price quote required')
    prior=quotes[-1];module.validate_quote(prior)
    if 0<=now-prior['retrieved_at']<3300:return
    fresh=(fetch or module.official_preflight)();module.validate_quote(fresh)
    if not 0<=now-fresh['retrieved_at']<=60 or any(fresh[k]!=prior[k] for k in ('model','rates','ceilings','input_token_price_threshold')):
        raise ValueError('official price changed or stale; no request reserved')
    ledger.record_price(fresh)

def require_headroom(admission,done,stamp,*,now=None):
    now=time.time() if now is None else now
    if admission.get('unlimited_policy_sha256') and admission['max_calls'] is None and admission['expires_at'] is None:return
    completed=sum(k.startswith('dispatch_') for k in done)
    remaining=max(0,admission['max_calls']-completed)
    # Browser sessions are maximum workloads, not a promise all200 worst-case
    # requests fit two hours. Require the next call + safe processing headroom.
    if admission.get('version') in ('browser-project-cycle-adapter@v3','browser-project-cycle-adapter@v4','browser-project-cycle-adapter@v5','browser-project-cycle-adapter@v6','browser-project-cycle-adapter@v7'):remaining=min(remaining,1)
    if now+remaining*240+240 > stamp(admission['expires_at']):
        raise ValueError('insufficient whole remaining envelope deadline headroom')

def ensure_scope_grant(ledger,module,adapter,grant,expected,*,check_only=False):
    """One explicit startup: exact reviewed grant, idempotent append, then enroll.

    No inferred grant and no mutation of an existing grant on restart.
    """
    admission_hash=module.load_module('startup_adapter','engine-browser-adapter.py').e.sha(adapter.admission)
    matched=(any(x['trial']==adapter.trial and x['admission']==admission_hash and x['max_calls']==adapter.admission['max_calls'] and x['max_operations']==adapter.admission['max_executions'] for x in grant.get('arms',[])) if grant.get('version')==getattr(module,'PAIR_GRANT',None) else grant.get('session_trial_sha256')==adapter.trial and grant.get('session_admission_sha256')==admission_hash)
    if module.digest(grant)!=expected or not matched:raise ValueError('startup grant/admission identity mismatch')
    with ledger.locked() as (_,rows):
        matches=[r['event'] for r in rows if r['event']['kind']=='evaluation_grant' and r['event']['key']==expected]
    if matches:
        if len(matches)!=1 or matches[0]['value']!=grant:raise ValueError('startup grant substitution')
    else:
        with ledger.locked() as (_,rows):
            status=ledger.fold(rows)[0]
            module.validate_evaluation_grant(grant,ledger.hash,rows[-1]['hash'],status['effective_max_calls'],status['effective_max_execution_groups'],status['effective_cap_microusd'])
        if not check_only:ledger.grant_calls(grant,expected)
    if not check_only:
        adapter.enroll();return ledger.status()
    with ledger.locked() as (_,rows):
        if matches:return ledger.fold(rows)[0]
        event={'kind':'evaluation_grant','key':expected,'value':grant}
        return ledger.fold(rows+[module.Ledger.row(rows,event)])[0]

def build_cycle(cycle_type,cycle_admission,adapter):
    """The one place a launched cycle is built. An opted-in run gets its reading turn here."""
    callback=getattr(adapter,'reading_callback',lambda:None)()
    abilities=getattr(adapter,'abilities_callback',lambda:None)()
    return cycle_type(cycle_admission['journal'],cycle_admission,adapter.native,adapter.proposal,adapter.review,**({'reading':callback} if callback else {}),**({'abilities':abilities} if abilities else {}))
def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--scope-grant',type=Path)
    p.add_argument('--scope-grant-sha256')
    p.add_argument('--browser-v1',action='store_true',help='Reviewed bounded browser profile; fresh public scope/admission required')
    p.add_argument('--release-v5',action='store_true',help='Reviewed finite objective profile; exact fresh admission and suite grant required')
    p.add_argument('--release-v4',action='store_true',help='Reviewed bounded information detour profile; requires explicit public corpus admission')
    p.add_argument('--release-v3',action='store_true',help='Require reviewed strict native output/reconsideration admission; not authority itself')
    p.add_argument('mode', choices=('check', 'run'))
    p.add_argument('--admission', type=Path, required=True)
    p.add_argument('--expected-sha256', required=True)
    p.add_argument('--cycle-admission', type=Path, required=True)
    p.add_argument('--campaign-manifest', type=Path, required=True)
    p.add_argument('--campaign-upgrade', type=Path, required=True)
    args = p.parse_args()
    spec = importlib.util.spec_from_file_location('native_cycle_adapter', Path(__file__).with_name('engine-browser-adapter.py' if args.browser_v1 else 'engine-objective-adapter.py' if args.release_v5 else 'engine-project-cycle-adapter.py'))
    a = importlib.util.module_from_spec(spec); spec.loader.exec_module(a)
    e = a.e
    admission = e.strict(e.bounded_file(args.admission))
    if e.sha(admission) != args.expected_sha256:
        raise ValueError('explicit adapter admission identity mismatch')
    cycle_admission = e.strict(e.bounded_file(args.cycle_admission))
    campaign = e.strict(e.bounded_file(args.campaign_manifest))
    if args.mode=='run' and campaign.get('fixture_only'):raise ValueError('fixture-only campaign cannot launch live effects')
    upgrade = e.strict(e.bounded_file(args.campaign_upgrade))
    m = a.load('native_cycle_campaign', 'engine-b91-campaign.py')
    ledger = m.Ledger(campaign['ledger_path'], campaign, m.digest(campaign), upgrade=upgrade, upgrade_hash=m.digest(upgrade))
    controller = a.d.Native(Path(admission['origin']['directory']))
    if sum((args.release_v3,args.release_v4,args.release_v5,args.browser_v1))>1:raise ValueError('one release profile only')
    if args.browser_v1:
        if admission.get('version')!=a.VERSION or cycle_admission.get('version')!='local-project-cycle@v3':raise ValueError('exact browser admissions required')
        if admission.get('feedback_cycle')!=cycle_admission.get('feedback_cycle'):raise ValueError('feedback cycle authorities differ')
        if admission.get('input_tokens',32768)!=cycle_admission.get('input_tokens',32768):raise ValueError('context token allocations differ')
        if admission.get('max_review_rejections',0)!=cycle_admission.get('max_review_rejections',0):raise ValueError('review recovery allowances differ')
        if admission['completion_tokens']!=cycle_admission.get('completion_tokens'):raise ValueError('browser completion allocations differ')
        if admission['max_decisions']!=cycle_admission['max_decisions']:raise ValueError('browser decision allowances differ')
    elif args.release_v5:
        if admission.get('version')!='synthetic-objective-adapter@v2' or cycle_admission.get('version')!='local-project-cycle@v5':raise ValueError('exact objective admissions required')
        if admission['max_decisions']!=cycle_admission['max_decisions'] or admission['max_service_rounds']!=cycle_admission['max_service_rounds']:raise ValueError('objective allowances differ')
    elif args.release_v3 or args.release_v4:
        suffix='v4' if args.release_v4 else 'v3'
        if admission.get('version')!='synthetic-project-cycle-adapter@'+suffix or cycle_admission.get('version')!='local-project-cycle@'+suffix:raise ValueError('exact release admissions required')
        if admission['max_decisions']!=cycle_admission['max_decisions']:raise ValueError('decision allowance mismatch')
    elif args.mode=='run':raise ValueError('legacy launch disabled')
    def no_fixture_dispatch(*_):
        raise ValueError('read-only fixture check cannot dispatch')
    ordinary=a.load('ordinary_browser','engine-browser-ordinary.py') if args.browser_v1 and admission.get('process_profile')=='ordinary-memory@v1' else None
    adapter_type=ordinary.Adapter if ordinary else a.Adapter
    cycle_type=ordinary.Cycle if ordinary else a.loop.Cycle
    adapter = adapter_type(admission['directory'], admission, controller, ledger,transport=no_fixture_dispatch if args.mode=='check' and campaign.get('fixture_only') else None,before_call=(lambda:refresh_price(ledger,m)) if args.release_v4 or args.release_v5 or args.browser_v1 else None)
    unlimited=args.browser_v1 and a.feedback_policy.unlimited(admission)
    if cycle_admission.get('unlimited_policy_sha256')!=admission.get('unlimited_policy_sha256'):raise ValueError('cycle unlimited policy mismatch')
    if (cycle_admission['scope'] != (admission['origin']['public']['scope'] if args.browser_v1 else admission['origin']['scope'])
            or (not unlimited and (cycle_admission['max_decisions'] * 2 > admission['max_calls']
            or cycle_admission['max_executions'] > admission['max_executions']
            or a.d.stamp(cycle_admission['expires_at']) > a.d.stamp(admission['expires_at'])))):
        raise ValueError('cycle exceeds reviewed adapter scope or limits')
    if cycle_admission.get('feedback_cycle')!=admission.get('feedback_cycle'):raise ValueError('adapter/cycle feedback authority mismatch')
    cycle = build_cycle(cycle_type, cycle_admission, adapter)
    launch_lock=os.fdopen(os.open(adapter.root/'launch.lock',os.O_CREAT|os.O_WRONLY|os.O_NOFOLLOW,0o600),'a')
    fcntl.flock(launch_lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
    adapter.launch_lock_owned=True
    if args.browser_v1 and admission.get('feedback_cycle') and args.mode=='run':
        adapter.recovery_session_root=Path(cycle_admission['journal'])
        require_recovery_not_stopped(cycle_admission['journal'])
        adapter.recover_settled_response()
        adapter.recover_predispatch()
        adapter.recover_session_transition()
    if adapter.read()[1] or ledger.status()['stopped']:
        raise a.d.Blocked('uncertain recorded nested effect; no retry')
    cycle.recover_pending()
    if cycle.read()[1]:raise a.d.Blocked('uncertain recorded effect; no retry')
    session_done=adapter.read()[2].get('session_completed')
    if session_done is not None:
        print(json.dumps({'result':session_done,'replayed_without_dispatch':True,'campaign':ledger.status()},indent=2));return
    done = cycle.read()[2]
    if 'terminal' in done:
        print(json.dumps({'result': done['terminal'], 'replayed_without_dispatch': True, 'campaign': ledger.status()}, indent=2))
        return
    current = adapter.native('request', None)
    payload_bytes = None
    if current['branch'] == 'propose':
        _, context = adapter.project(current)
        payload_bytes = len(e.canonical(context))
    status = ledger.status()
    if status['stopped']:
        raise a.d.Blocked('campaign reconciliation required')
    if admission.get('feedback_cycle'):
        if not args.scope_grant or not args.scope_grant_sha256:raise ValueError('feedback session requires exact reviewed startup grant')
        grant=e.strict(e.bounded_file(args.scope_grant))
        status=ensure_scope_grant(ledger,m,adapter,grant,args.scope_grant_sha256,check_only=True)
    if not unlimited and 'campaign_envelope' not in adapter.read()[2]:
        if (status['remaining_calls'] < admission['max_calls']
                or status['remaining_execution_groups'] < admission['max_executions']
                or status['committed_microusd'] + admission['max_calls'] * m.RESERVE > status.get('effective_cap_microusd',m.CAP)):
            raise a.d.Blocked('full run envelope does not fit remaining campaign')
    if args.release_v4 or args.release_v5 or args.browser_v1:
        require_headroom(admission,adapter.read()[2],a.d.stamp)
    if args.mode == 'check':
        print(json.dumps({'status': 'ready_for_independent_manifest_review', 'branch': current['branch'],
            'projection_bytes': payload_bytes, 'campaign': status, 'provider_calls': 0, 'target_executions': 0}, indent=2))
        return
    if admission.get('feedback_cycle'):ensure_scope_grant(ledger,m,adapter,grant,args.scope_grant_sha256)
    adapter.op('campaign_envelope', lambda: {'admission_sha256': e.sha(admission), 'calls': admission['max_calls'],
        'execution_groups': admission['max_executions'], **({'planning_floor_microusd':None if unlimited else admission['max_calls']*m.RESERVE,'reservation_policy':'actual serialized request token upper bound at verified prices; stop when next reservation does not fit; maximum calls are not guaranteed funded'} if admission.get('context_profile')=='luna-context-budget@v1' else {'reservation_microusd':admission['max_calls']*m.RESERVE}),
        'campaign_before': status, 'purpose': 'prospective envelope; every actual effect separately reserved; no counter reset'})
    ledger.record_price(m.official_preflight())
    if args.browser_v1:adapter.start_clock()
    if args.browser_v1:complete_fresh_direct_return(adapter,cycle,cycle_admission)
    result = a.load('browser_session','engine-browser-session.py').run(adapter,cycle_admission,cycle_type=cycle_type) if args.browser_v1 and admission.get('feedback_events') else cycle.run()
    if args.browser_v1 and admission.get('feedback_events'):adapter.op('session_completed',lambda:result)
    print(json.dumps({'result': result, 'campaign': ledger.status()}, indent=2))


if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__ == '__main__':
    main()
