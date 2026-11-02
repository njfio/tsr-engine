#!/usr/bin/env python3
"""Copy a reviewed terminal garden prefix; prepare three calls/one simulated action only."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
from types import SimpleNamespace
s=importlib.util.spec_from_file_location('garden_driver',Path(__file__).with_name('engine-directions.py'));d=importlib.util.module_from_spec(s);s.loader.exec_module(d);c=d.load_synthetic_cycle()
def inventory(root):
    result={}
    for p in sorted(root.rglob('*')):
        if p.is_symlink():raise ValueError('source symlink')
        if p.is_file():result[str(p.relative_to(root))]=hashlib.sha256(p.read_bytes()).hexdigest()
    return result
def prepare(parent,directory,review_path,expires):
    parent=Path(parent).resolve(strict=True);directory=Path(directory).resolve()
    if directory==parent or directory.is_relative_to(parent) or parent.is_relative_to(directory):raise ValueError('disjoint continuation directory required')
    review=c.read(review_path)
    terminal=c.read(parent/'synthetic-external/terminal.json');a=c.read(parent/'admission.json')
    if (terminal['status']!='proposal_only_complete' or terminal['calls_dispatched']!=5 or terminal['acknowledged_actions']!=1
        or terminal['unknown_application_outcome'] or terminal['unknown_usage_or_cost'] or terminal['unmatched_intents']
        or a.get('scenario')!='garden' or a['model']!=c.LUNA):raise ValueError('settled terminal garden required')
    if (set(review)!={'parent','terminal_sha256','admission_sha256','approved','reviewer'} or review['parent']!=str(parent)
        or review['terminal_sha256']!=c.digest(terminal) or review['admission_sha256']!=c.digest(a)
        or review['approved'] is not True or not isinstance(review['reviewer'],str) or not 1<=len(review['reviewer'])<=256):raise ValueError('exact independent parent review required')
    journal=parent/'synthetic-external';expected=a['initial_root'];actions=0
    if len(list(journal.glob('intent-*.json')))!=5:raise ValueError('parent call inventory')
    for n,phase in enumerate(['relational','possibilities','assessment','relational','possibilities_final'],1):
        intent=c.read(journal/f'intent-{n}.json');outcome=c.read(journal/f'outcome-{n}.json');applied=c.read(journal/f'applied-{n}.json')
        if (intent['phase']!=phase or intent['state_root']!=expected or intent['admission_hash']!=c.digest(a)
            or intent['payload_hash']!=c.digest(c.read(journal/f'request-{n}.json')) or intent['schema_hash']!=c.digest(c.read(journal/f'schema-{n}.json'))
            or outcome['intent_hash']!=c.digest(intent) or outcome['receipt']['status']!='settled' or outcome['receipt']['usage_status']!='reported'
            or outcome['receipt'].get('usd') is None or applied['intent_hash']!=c.digest(intent) or applied['outcome_hash']!=c.digest(outcome)):
            raise ValueError('parent provider prefix mismatch')
        if phase in ('relational','assessment'):
            controller=c.read(journal/f'controller-{n}.json');apply=c.read(journal/f'apply-intent-{n}.json')
            if (applied['controller_hash']!=c.digest(controller) or controller['apply_hash']!=c.digest(apply)
                or controller['outcome_hash']!=c.digest(outcome) or controller['before_root']!=expected
                or controller['after_root']!=applied['after_root'] or apply['value_hash']!=c.digest(c.read(journal/f'apply-value-{n}.json'))):raise ValueError('parent controller prefix mismatch')
        elif applied['controller_hash'] is not None or applied['after_root']!=expected:raise ValueError('parent proposal mutation')
        actions+=applied['action_count'];expected=applied['after_root']
    if expected!=terminal['state_root'] or actions!=1:raise ValueError('parent terminal mismatch')
    before=inventory(parent);directory.mkdir(exist_ok=False)
    shutil.copytree(parent,directory/'parent-evidence')
    for name in ('cas','events.sqlite'):
        source=parent/name;target=directory/name
        if source.is_dir():shutil.copytree(source,target)
        else:shutil.copyfile(source,target)
    if inventory(parent)!=before or inventory(directory/'parent-evidence')!=before:raise ValueError('copy changed')
    if c.storage_inventory(directory)!=c.storage_inventory(parent):raise ValueError('native copy mismatch')
    state=d.cli(directory,'direction-request');c.scope(state,d,'garden');c.continuation_initial(state)
    if state['state_root']!=terminal['state_root']:raise ValueError('terminal source drift')
    c.save(directory/'parent-review.json',review)
    binding={'parent':str(parent),'files':before,'review_sha256':c.digest(review),'preparer_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    args=SimpleNamespace(directory=directory,model=c.LUNA,calls=3,synthetic_actions=1,synthetic_scenario='garden',synthetic_continuation=binding,synthetic_expires=expires,prepare_synthetic_admission=directory/'admission.json')
    result=c.prepare(args,d.__dict__)
    if inventory(parent)!=before:raise ValueError('parent modified')
    return result
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--parent',type=Path,required=True);p.add_argument('--directory',type=Path,required=True);p.add_argument('--review',type=Path,required=True);p.add_argument('--expires',required=True)
    a=p.parse_args();print(json.dumps(prepare(a.parent,a.directory,a.review,a.expires),indent=2))
