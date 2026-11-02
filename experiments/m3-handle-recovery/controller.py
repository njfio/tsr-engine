"""Versioned continuation: original assignment0 is carried unchanged, never rerun."""
from pathlib import Path
import importlib.util,json,os,sys,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'experiments/m3-handle-live'));sys.path.insert(0,str(HERE))
spec=importlib.util.spec_from_file_location('frozen_controller',ROOT/'experiments/m3-handle-live/controller.py');c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
import guard
old=c.old;budget=c.budget;sha=c.sha;bind_accounting=c.bind_accounting;classify=c.classify;verify_handle_evidence=c.verify_handle_evidence
MANIFEST=ROOT/'docs/m3-handle-live-amendment-v1.json'
class Operations(c.Operations):
    def __init__(self,m):
        super().__init__(m)
        self.old.require_idle=lambda:guard.require_idle(ROOT/m['observer'],m['sha256'][m['observer']],ROOT/m['state']/'guard-checks.jsonl' if (ROOT/m['state']).exists() else None)
    def guard(self,initial=False):
        super().guard(initial)
        previous=json.loads((ROOT/self.m['prior_terminal']).read_text())
        if previous['status']!='halted-review-required' or len(previous['results'])!=1:raise ValueError('prior terminal scope')
        if initial:
            if self.ledger()!= (ROOT/self.m['prior_evidence']/'ledger.jsonl').read_bytes():raise ValueError('new or unresolved charges since prior audit')
            if any((ROOT/a['input']/'record').exists() or list((ROOT/a['input']).glob('startup-*')) for a in self.m['arms'][1:]):raise ValueError('remaining assignment already attempted')

def validate_scope(m):
    original=json.loads((ROOT/'docs/m3-handle-live-execution-freeze.json').read_text())
    for key,value in original.items():
        if key not in ('state','launch_baseline','sha256','arms') and m[key]!=value:raise ValueError('changed original setting: '+key)
    if len(m['arms'])!=12:raise ValueError('assignment count')
    for i,(before,after) in enumerate(zip(original['arms'],m['arms'])):
        expected=before.copy()
        if i:expected['broker']=f'experiments/m3-handle-recovery/arm-{i}.py'
        if after!=expected:raise ValueError('changed assignment scope')
    if any(m['sha256'].get(p)!=h for p,h in original['sha256'].items()):raise ValueError('original pin changed')
    if m['absolute_deadline_epoch']>1792352700:raise ValueError('deadline extended')

def deadline_remaining(epoch,monotonic):
    return min(epoch-time.time(),monotonic-time.monotonic())

def coordinate(m,state,ops):
    from decimal import Decimal
    from datetime import datetime
    validate_scope(m)
    if state.exists():raise ValueError('one-shot state exists; no resume/retry')
    expected=['old','new','new','old','old','new','new','old','old','new','new','old']
    if len(m['arms'])!=12 or [a['interface'] for a in m['arms']]!=expected or len({a['task'] for a in m['arms']})!=1 or any(a['mode']!='record' or a['model']!='deepseek-v4.1-flash' for a in m['arms']):raise ValueError('paired scope')
    started=time.monotonic();epoch_deadline=m['absolute_deadline_epoch']
    monotonic_deadline=started+max(0,epoch_deadline-time.time())
    ops.guard(initial=True);state.mkdir();old.sync_directory(state.parent);old.save_new(state/'plan.json',m);results=[json.loads((ROOT/m['prior_decision']).read_text())]
    if results[0]['index']!=0:raise ValueError('prior assignment identity')
    try:
        for i,arm in enumerate(m['arms'][1:],start=1):
            if deadline_remaining(epoch_deadline,monotonic_deadline)<1900:raise ValueError('global deadline headroom')
            ops.guard();before=ops.ledger();attempt=state/str(i);attempt.mkdir();old.sync_directory(state)
            old.save_new(attempt/'intent.json',arm)
            if deadline_remaining(epoch_deadline,monotonic_deadline)<1900:raise ValueError('global deadline headroom before dispatch')
            old.save_new(state/'current.json',dict(index=i,deadline=time.time()+1800,global_deadline=epoch_deadline,manifest_sha256=sha(ROOT/'docs/m3-handle-live-amendment-v1.json')))
            ops.m['engine']=arm['engine']
            process=ops.run(arm,attempt);ops.guard()
            (state/'current.json').unlink()
            if deadline_remaining(epoch_deadline,monotonic_deadline)<=0:raise ValueError('global deadline reached')
            checked=ops.export_and_verify(arm,process,attempt/'evidence')
            after=ops.ledger();ids=bind_accounting(before,after,checked['diagnostics'])
            reservations=[json.loads(l) for l in after[len(before):].splitlines() if json.loads(l)['kind']=='reserve']
            if any(r['model']!=arm['model'] for r in reservations):raise ValueError('model accounting mismatch')
            if checked['result'] and (checked['result'].get('task')!=arm['task'] or checked['result'].get('system')!=arm['mode']):raise ValueError('result identity')
            outcome=classify(checked,process)
            freshness=verify_handle_evidence(arm,checked,attempt/'evidence'/'freshness') if arm['interface']=='new' else None
            result=checked['result']
            record=dict(freshness=freshness,closure_reasons=checked.get('closure_reasons',[]),last_diagnostic_code=checked['diagnostics'][-1].get('code') if checked['diagnostics'] else None,index=i,task=arm['task'],mode=arm['mode'],pair_index=arm['pair_index'],interface=arm['interface'],model=arm['model'],outcome=outcome,requests=len(ids),latency_seconds=(datetime.fromisoformat(process['finished'])-datetime.fromisoformat(process['started'])).total_seconds(),settled_usd=str(budget.history.total(after)-budget.history.total(before)),schema_failures=sum(d.get('code')!='ok' and d.get('stage')=='decode' for d in checked['diagnostics']),stale_digest_failures=int(outcome=='stale-digest-or-oversize'),final_oracle_completed=bool(result and result.get('oracle')),final_oracle_success=bool(result and (result.get('oracle') or {}).get('evidence',{}).get('verified_success')),proposal_kinds=checked['proposal_kinds'],visible_test_exit_codes=checked['visible_test_exit_codes'],evidence_sha256=checked['manifest_sha256'])
            if deadline_remaining(epoch_deadline,monotonic_deadline)<=0:raise ValueError('global deadline after export')
            old.save_new(attempt/'decision.json',record);results.append(record);print(json.dumps(record),flush=True)
        if deadline_remaining(epoch_deadline,monotonic_deadline)<=0:raise ValueError('global deadline before completion')
        old.save_new(state/'terminal.json',dict(status='completed',results=results,aggregate_usd=str(budget.history.total(ops.ledger()))))
    except Exception as error:
        old.save_new(state/'terminal.json',dict(status='halted-review-required',results=results,error_type=type(error).__name__,reason=str(error)));raise
    return results


if __name__=='__main__':
    if sys.argv[1:]!=['--approved-continuation']:raise ValueError('authorization flag missing')
    m=json.loads(MANIFEST.read_text());lock_fd=guard.lock()
    try:coordinate(m,ROOT/m['state'],Operations(m))
    finally:os.close(lock_fd)
