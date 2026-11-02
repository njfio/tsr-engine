"""Fresh synthetic-only real-provider integration, bounded across both modes."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from types import SimpleNamespace

ROOT=Path(__file__).resolve().parents[1]
def load(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'scripts'/(name+'.py'))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
old=load('m3-continuation-controller')
budget=load('m3-exact-budget')

class Operations(old.LocalOperations):
    def __init__(self,m):
        super().__init__(m)
        self.receipts.subprocess=SimpleNamespace(check_output=lambda *a,**kw:subprocess.check_output(*a,**(kw|{'timeout':15})),run=lambda *a,**kw:subprocess.run(*a,**(kw|{'timeout':60})))
    def guard(self,initial=False):
        old.verify_freeze(self.m)
        if str(Path(sys.executable).resolve())!=self.m['python']:raise ValueError('Python mismatch')
        self.old.require_idle()
        baseline=json.loads((ROOT/self.m['baseline']).read_text())
        budget.check(self.ledger(),baseline,dispatch=initial)
        if initial and budget.history.freeze(self.ledger())!=json.loads((ROOT/self.m['launch_baseline']).read_text()):raise ValueError('launch baseline mismatch')
        minimum=self.m['initial_minimum_free_gib'] if initial else self.m['live_guard_gib']
        if shutil.disk_usage(ROOT).free<minimum*1024**3:raise ValueError('disk reserve')
        image=subprocess.check_output(['docker','image','inspect','--format','{{.Id}}',self.m['sandbox_image']],text=True,timeout=15).strip()
        if image!=self.m['sandbox_image']:raise ValueError('image mismatch')
    def run(self,arm,state):
        base=ROOT/arm['input'];mode=arm['mode']
        if (base/mode).exists() or list(base.glob('startup-'+mode+'.sqlite*')):raise ValueError('state not fresh')
        self.diagnostic_before=self.diagnostic_path.read_bytes() if self.diagnostic_path.exists() else b''
        self.worker.STATE=state
        env=self.worker.env_for(False)
        env.update(TSR_M3_PREPARATION=str(base/'preparation.json'),TSR_M3_IMAGE=self.m['sandbox_image'],TSR_M3_APPROVED_LIVE='100',TSR_M3_BROKER_SCRIPT=str(ROOT/self.m['broker']),TSR_M3_PYTHON=sys.executable,TSR_M3_COST_LEDGER=str(self.ledger_path),TSR_DB=str(base/('startup-'+mode+'.sqlite')))
        record=self.worker.run(arm['task'][:12]+'-'+mode,[str(ROOT/self.m['engine']),'task','run','--mode',mode,'--tasks',str(base/'tasks.jsonl'),'--driver','cheaper-inference','--profile',str(base/'profile.toml'),'--out',str(base/mode)],env)
        if record.get('status')!='completed' or record.get('exit')!=0 or record.get('errors'):raise ValueError('process/containment failure')
        rows=subprocess.check_output(['ps','-axo','pid=,pgid='],text=True,timeout=15).splitlines()
        if any(record['pid'] in [int(v) for v in r.split()] for r in rows):raise ValueError('process group remains')
        return record
    def export_and_verify(self,arm,process,destination):
        checked=super().export_and_verify(arm,process,destination)
        import sqlite3
        label=arm['task'][:12]+'-'+arm['mode']
        with sqlite3.connect((destination/(label+'-state.sqlite')).as_uri()+'?mode=ro&immutable=1',uri=True) as db:
            events=[json.loads(r[0])['kind'] for r in db.execute('select payload from events order by seq')]
        proposals=[e['candidate']['kind'] for e in events if e['kind']=='proposed']
        if not proposals or proposals[-1]!='stop':raise ValueError('explicit submission missing')
        checked['proposal_kinds']=proposals
        return checked

def main():
    if sys.argv[1:]!=['--approved-synthetic-integration']:raise ValueError('authorization flag missing')
    m=json.loads((ROOT/'docs/m3-integration-v3-freeze.json').read_text())
    if m['stage']!='synthetic-only-integration' or [a['mode'] for a in m['arms']]!=['record','rules'] or any(a['lineage']!='synthetic:m3-known-answer' for a in m['arms']):raise ValueError('stage scope')
    state=ROOT/m['state']
    if state.exists():raise ValueError('one-shot state exists')
    ops=Operations(m);ops.guard(initial=True);state.mkdir();old.sync_directory(state.parent)
    old.save_new(state/'plan.json',m);results=[]
    try:
        for i,arm in enumerate(m['arms']):
            ops.guard();before=ops.ledger();old.save_new(state/f'{i}-intent.json',dict(index=i,arm=arm))
            process=ops.run(arm,state);ops.guard()
            checked=ops.export_and_verify(arm,process,state/f'evidence-{i}')
            ids=old.settled_diagnostics(before,ops.ledger(),checked['diagnostics'])
            if any(d.get('code')!='ok' or d.get('status')!='completed' for d in checked['diagnostics']):raise ValueError('provider/protocol failure')
            result=checked['result']
            if result['task']!=arm['task'] or result['system']!=arm['mode'] or result['outcome']!='accepted' or not result.get('oracle',{}).get('evidence',{}).get('verified_success'):raise ValueError('synthetic full path not successful')
            if checked['proposed_count']!=len(ids):raise ValueError('call/proposal mismatch')
            if results and result['state_root_at_start']!=results[0]['start_root']:raise ValueError('starting roots differ')
            record=dict(task=arm['task'],mode=arm['mode'],outcome=result['outcome'],start_root=result['state_root_at_start'],requests=len(ids),evidence_sha256=checked['manifest_sha256'],proposal_kinds=checked['proposal_kinds'])
            old.save_new(state/f'{i}-decision.json',record);results.append(record)
        old.save_new(state/'terminal.json',dict(status='completed',results=results))
    except Exception:
        old.save_new(state/'terminal.json',dict(status='halted-review-required',results=results));raise

if __name__=='__main__':main()
