"""Finite feedback/checkpoint session over the existing native cycle and ledger.

Feedback is exact prospective admission data, never generated player testimony.
Each checkpoint has its own append-only cycle; one adapter owns all call/effect
accounting. No old terminal journal is reopened, and no allowance is reset.
"""
import copy
import hashlib
import json
import fcntl
import os
from pathlib import Path


def run(adapter, cycle_admission, *, cycle_type=None):
    # Use the already loaded adapter's existing generic kernel, not a new selector.
    if cycle_type is None:
        import importlib.util
        spec=importlib.util.spec_from_file_location('session_cycle',Path(__file__).with_name('engine-project-cycle.py'))
        kernel=importlib.util.module_from_spec(spec);spec.loader.exec_module(kernel)
        cycle_type=kernel.Cycle
    root=Path(cycle_admission['journal']);root.mkdir(parents=True,exist_ok=True)
    if any(p.is_symlink() for p in (root,*root.parents)):raise ValueError('session path boundary')
    with os.fdopen(os.open(root/'session.lock',os.O_CREAT|os.O_WRONLY|os.O_NOFOLLOW,0o600),'a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        results=[]
        events=adapter.admission['feedback_events']
        for index in range(max(1,len(events))):
            if any(path.exists() for path in (root/'STOP',adapter.root/'STOP',root/('checkpoint-'+str(index))/'STOP')):raise ValueError('session stopped; no next event')
            if any(k.startswith(('meaning_admission_','format_correction_')) or (k.startswith('criterion_') and k.endswith('_apply')) for k in adapter.read()[1]):adapter.recover_session_transition()
            # Never substitute an outer receipt for a still-uncertain inner effect.
            if adapter.read()[1] or adapter.ledger.status()['stopped']:raise ValueError('unresolved nested effect; no dispatch')
            # Existing checkpoint owns its pending callback/return ordering. Never
            # prepare a new assessment ahead of recovering that durable stage.
            if hasattr(adapter,'complete_pending_return') and not (root/('checkpoint-'+str(index))/'cycle.jsonl').exists():adapter.complete_pending_return()
            if events:adapter.apply_feedback(index)
            admission=copy.deepcopy(cycle_admission)
            admission['journal']=str(root/('checkpoint-'+str(index)))
            admission['checkpoint_id']=hashlib.sha256(json.dumps({'adapter':adapter.admission,'index':index},sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
            cycle=cycle_type(admission['journal'],admission,adapter.native,adapter.proposal,adapter.review)
            result=cycle.run();results.append(result)
            if index+1<len(events) and 'feedback_'+str(index+1) in adapter.read()[2]:
                continue  # A durable successor has already consumed this checkpoint.
            current=adapter.native('request',None)
            checkpoint=result['status'] in ('review_required','advisory_abstain','advisory_unknown','advisory_rejected') and current['branch']=='review_required'
            checkpoint=checkpoint or (result['status']=='proposal_retention' and current['branch'] in ('propose','review_required') and not adapter.read()[1] and not adapter.ledger.status()['stopped'])
            if (root/'STOP').exists() or (adapter.root/'STOP').exists() or (Path(admission['journal'])/'STOP').exists():raise ValueError('session stopped; no next event')
            if not checkpoint:
                return {'status':'stopped','checkpoints':results,'reason':result['status'],'feedback_applied':index+int(bool(events))}
        return {'status':'completed_checkpoints','checkpoints':results,'feedback_applied':len(events),
                'acceptance_recorded':False,'authority':'unchanged; no automatic new feedback or session'}
