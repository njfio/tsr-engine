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
budget=load('m3-two-model-budget')

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
        self.attempt_state=state
        env=self.worker.env_for(False)
        env.update(TSR_M3_PREPARATION=str(base/'preparation.json'),TSR_M3_IMAGE=self.m['sandbox_image'],TSR_M3_APPROVED_LIVE='100',TSR_M3_BROKER_SCRIPT=str(ROOT/arm['broker']),TSR_M3_PYTHON=sys.executable,TSR_M3_COST_LEDGER=str(self.ledger_path),TSR_DB=str(base/('startup-'+mode+'.sqlite')))
        record=self.worker.run(arm['task'][:12]+'-'+mode,[str(ROOT/self.m['engine']),'task','run','--mode',mode,'--tasks',str(base/'tasks.jsonl'),'--driver','cheaper-inference','--profile',str(base/'profile.toml'),'--out',str(base/mode)],env)
        if record.get('status')!='completed' or record.get('exit') not in (0,2) or record.get('errors'):raise ValueError('process/containment failure')
        rows=subprocess.check_output(['ps','-axo','pid=,pgid='],text=True,timeout=15).splitlines()
        if any(record['pid'] in [int(v) for v in r.split()] for r in rows):raise ValueError('process group remains')
        return record
    def export_and_verify(self,arm,process,destination):
        import sqlite3
        import struct
        base=ROOT/arm['input'];out=base/arm['mode'];label=arm['task'][:12]+'-'+arm['mode']
        result_path=out/('results-'+arm['mode']+'.jsonl')
        if result_path.exists():
            if process['exit']!=0:raise ValueError('result/process mismatch')
            checked=export_full(self,arm,process,destination)
        else:
            log=(self.attempt_state/(label+'.log')).read_text()
            known={x:'edit-precondition' for x in ('invalid exact edit span','edit span not found','ambiguous edit span','edit output exceeds limit')}
            known['stale or oversized edit source']='stale-digest-or-oversize'
            prefix='tsr: task run: driver: the fixture session is malformed: '
            reason=log.strip().removeprefix(prefix)
            if process['exit']!=2 or not log.startswith(prefix) or reason not in known:raise ValueError('unknown process failure')
            destination.mkdir()
            stores=list((out/'state').glob('*/*.sqlite'))
            if len(stores)!=1 or stores[0].is_symlink():raise ValueError('partial store count/type')
            dbpath=destination/(label+'-state.sqlite')
            with sqlite3.connect(stores[0].as_uri()+'?mode=ro',uri=True) as src,sqlite3.connect(dbpath) as dst:
                src.backup(dst);dst.execute('pragma journal_mode=DELETE')
            for source,target in ((out/'cas',destination/label/'cas'),(base/'cas',destination/'input-cas')):
                if source.is_symlink() or not source.is_dir():raise ValueError('partial CAS type')
                target.mkdir(parents=True)
                for p in source.iterdir():
                    if p.is_symlink() or not p.is_file() or self.digest(p.read_bytes())!=p.name:raise ValueError('partial CAS digest/type')
                    shutil.copyfile(p,target/p.name)
            for name in ('profile.toml','broker-contexts.json','preparation.json'):
                shutil.copyfile(base/name,destination/name)
            shutil.copyfile(base/'tasks.jsonl',destination/(arm['task'][:12]+'-tasks.jsonl'))
            candidate=out/'copies'/arm['task']/arm['mode']/'src/lib.rs'
            if candidate.is_symlink() or not candidate.is_file() or candidate.stat().st_size>65536:raise ValueError('partial candidate type/size')
            shutil.copyfile(candidate,destination/'candidate-lib.rs')
            raw=self.diagnostic_path.read_bytes()
            if not raw.startswith(self.diagnostic_before) or not raw.endswith(b'\n'):raise ValueError('diagnostic prefix')
            diagnostics=[json.loads(l) for l in raw[len(self.diagnostic_before):].splitlines()]
            checked=dict(result=None,diagnostics=diagnostics,failure=known[reason],native_error=reason)
            old.save_new(destination/'diagnostics.json',diagnostics)
            old.save_new(destination/'process.json',process)
            (destination/'ledger.jsonl').write_bytes(self.ledger())
        dbpath=destination/(label+'-state.sqlite')
        with sqlite3.connect(dbpath.as_uri()+'?mode=ro&immutable=1',uri=True) as db:
            if db.execute('pragma integrity_check').fetchone()[0]!='ok':raise ValueError('store integrity')
            events=[(i,json.loads(p)['kind']) for i,p in db.execute('select id,payload from events order by seq')]
        proposals=[e['candidate']['kind'] for _,e in events if e['kind']=='proposed']
        closures=[e for _,e in events if e['kind']=='task_closed']
        if checked['result'] is None:
            if closures or len(proposals)!=len(checked['diagnostics'])-1 or checked['diagnostics'][-1].get('proposal_action')!='edit':raise ValueError('partial event alignment')
            if any(d.get('code')!='ok' for d in checked['diagnostics']):raise ValueError('partial diagnostic mismatch')
        elif checked['result']['outcome']=='accepted' and (not proposals or proposals[-1]!='stop'):
            raise ValueError('accepted without submission')
        checked.update(proposal_kinds=proposals,visible_test_exit_codes=[e.get('exit_code') for _,e in events if e['kind']=='executed'],proposed_count=len(proposals),partial_root=self.digest(b''.join(struct.pack('<Q',len(i.encode()))+i.encode() for i,_ in events)))
        shutil.copyfile(self.attempt_state/(label+'.log'),destination/'process.log')
        hashes={str(p.relative_to(destination)):old.sha(p) for p in destination.rglob('*') if p.is_file() and p.name!='sha256.json'}
        old.save_new(destination/'complete-sha256.json',hashes)
        for p in destination.rglob('*'):
            if p.is_file():
                with p.open('rb') as handle:os.fsync(handle.fileno())
        for directory in sorted([p for p in destination.rglob('*') if p.is_dir()],key=lambda p:len(p.parts),reverse=True):old.sync_directory(directory)
        old.sync_directory(destination);old.sync_directory(destination.parent)
        for p,h in hashes.items():
            if old.sha(destination/p)!=h:raise ValueError('complete export changed')
        checked['manifest_sha256']=old.sha(destination/'complete-sha256.json')
        return checked


def classify(checked,process):
    ds=checked['diagnostics'];result=checked['result']
    if any(d.get('code')!='ok' for d in ds[:-1]):raise ValueError('earlier protocol error')
    if result is None:return checked['failure']
    if result['outcome']=='unavailable':
        if ds[-1].get('code') not in {'duplicate_field','choice_count','tool_shape','provider_refusal','output_truncated','proposal_limit','json_invalid','proposal_fields','proposal_type','forecast_type','smoke_scope','edit_precondition'} or ds[-1].get('stage')!='decode' or ds[-1].get('status')!='failed' or checked['proposed_count']!=len(ds)-1:raise ValueError('non-model unavailability')
        return 'schema-or-response-failure'
    if result['outcome'] not in ('accepted','rejected') or any(d.get('code')!='ok' for d in ds):raise ValueError('unrecognized outcome')
    expected=len(ds)+(1 if result.get('oracle') is None else 0)
    if checked['proposed_count']!=expected:raise ValueError('proposal/diagnostic alignment')
    return result['outcome']


def coordinate(m,state,ops):
    from decimal import Decimal
    from datetime import datetime
    if state.exists():raise ValueError('one-shot state exists; no resume/retry')
    if len(m['arms'])!=6 or [a['model'] for a in m['arms']]!=['deepseek-v4.1-flash','glm-5.3-flash']*3:raise ValueError('six attempt schedule')
    ops.guard(initial=True);state.mkdir();old.sync_directory(state.parent);old.save_new(state/'plan.json',m);results=[]
    try:
        for i,arm in enumerate(m['arms']):
            ops.guard();before=ops.ledger();attempt=state/str(i);attempt.mkdir();old.sync_directory(state)
            old.save_new(attempt/'intent.json',arm)
            process=ops.run(arm,attempt);ops.guard()
            checked=ops.export_and_verify(arm,process,attempt/'evidence')
            after=ops.ledger();ids=old.settled_diagnostics(before,after,checked['diagnostics'])
            reservations=[json.loads(l) for l in after[len(before):].splitlines() if json.loads(l)['kind']=='reserve']
            if any(r['model']!=arm['model'] for r in reservations):raise ValueError('model accounting mismatch')
            if checked['result'] and (checked['result'].get('task')!=arm['task'] or checked['result'].get('system')!=arm['mode']):raise ValueError('result identity')
            outcome=classify(checked,process)
            result=checked['result']
            record=dict(index=i,model=arm['model'],outcome=outcome,requests=len(ids),latency_seconds=(datetime.fromisoformat(process['finished'])-datetime.fromisoformat(process['started'])).total_seconds(),settled_usd=str(budget.history.total(after)-budget.history.total(before)),schema_failures=sum(d.get('code')!='ok' for d in checked['diagnostics']),stale_digest_failures=int(outcome=='stale-digest-or-oversize'),final_oracle_completed=bool(result and result.get('oracle')),final_oracle_success=bool(result and (result.get('oracle') or {}).get('evidence',{}).get('verified_success')),proposal_kinds=checked['proposal_kinds'],visible_test_exit_codes=checked['visible_test_exit_codes'],evidence_sha256=checked['manifest_sha256'])
            old.save_new(attempt/'decision.json',record);results.append(record);print(json.dumps(record),flush=True)
        old.save_new(state/'terminal.json',dict(status='completed',results=results,aggregate_usd=str(budget.history.total(ops.ledger()))))
    except Exception as error:
        old.save_new(state/'terminal.json',dict(status='halted-review-required',results=results,error_type=type(error).__name__,reason=str(error)));raise
    return results

# Full-result exporter retains receipt/root/CAS checks; adds recorded budget/blocked endings.
from contextlib import closing
import sqlite3
import struct
save_new=old.save_new
sha=old.sha
sync_directory=old.sync_directory

def export_full(self, arm, process, destination):
    base=ROOT/arm['input'];mode=arm['mode'];label=arm['task'][:12]+'-'+mode;out=base/mode
    destination.mkdir()
    # Refuse links/special files: evidence is controller-owned, never candidate paths.
    for directory in (out/'cas',):
        if directory.is_symlink() or not directory.is_dir(): raise ValueError('missing CAS')
        for p in directory.iterdir():
            if p.is_symlink() or not p.is_file(): raise ValueError('unsafe CAS entry')
    shutil.copytree(out/'cas',destination/label/'cas')
    shutil.copytree(base/'cas',destination/'input-cas')
    for name in ('profile.toml','broker-contexts.json','preparation.json'):
        shutil.copyfile(base/name,destination/name)
    result_path=out/('results-'+mode+'.jsonl')
    if result_path.is_symlink(): raise ValueError('result symlink')
    shutil.copyfile(result_path,destination/(label+'-results.jsonl'))
    shutil.copyfile(base/'tasks.jsonl',destination/(arm['task'][:12]+'-tasks.jsonl'))
    stores=list((out/'state').glob('*/*.sqlite'))
    if len(stores)!=1 or stores[0].is_symlink(): raise ValueError('store count/type')
    target=destination/(label+'-state.sqlite')
    with closing(sqlite3.connect('file:'+str(stores[0])+'?mode=ro',uri=True)) as source, closing(sqlite3.connect(target)) as dest:
        source.backup(dest);dest.execute('pragma journal_mode=DELETE')
    with closing(sqlite3.connect('file:'+str(target)+'?mode=ro&immutable=1',uri=True)) as db:
        if db.execute('pragma integrity_check').fetchone()[0]!='ok': raise ValueError('store integrity')
        events=[(i,json.loads(p)) for i,p in db.execute('select id,payload from events order by seq')]
    rows=[json.loads(line) for line in result_path.read_text().splitlines()]
    if len(rows)!=1: raise ValueError('result count')
    result=rows[0]
    root=self.digest(b''.join(struct.pack('<Q',len(i.encode()))+i.encode() for i,_ in events))
    if result.get('state_root_at_start')!=self.digest(b'') or result.get('state_root_at_end')!=root:
        raise ValueError('state root mismatch')
    closures=[e['kind']['reason'] for _,e in events if e['kind']['kind']=='task_closed']
    if len(closures)!=1 or not events or events[-1][1]['kind']['kind']!='task_closed': raise ValueError('closure evidence')
    if result.get('outcome')=='unavailable':
        if result.get('oracle') is not None or closures!=['unavailable:broker failed; see controller cost ledger for reserved charges']:
            raise ValueError('unavailable closure mismatch')
    for p in list((destination/label/'cas').iterdir())+list((destination/'input-cas').iterdir()):
        if self.digest(p.read_bytes())!=p.name: raise ValueError('CAS digest mismatch')
    receipt=self.receipts.verify_receipts(destination,label,dict(result=result,task=arm['task'],mode=mode))
    if result.get('oracle') is not None and receipt['status']!='checked': raise ValueError('oracle receipts missing')
    ordinary_stop = result.get('outcome')=='rejected' and result.get('oracle') is None and (closures[0].startswith('stop:budget') or closures[0].startswith('stop:blocked:'))
    if result.get('denials') and not ordinary_stop:raise ValueError('unexpected denial')
    if result.get('outcome') in ('accepted','rejected') and not ordinary_stop and (result.get('oracle') is None or receipt.get('status')!='checked' or closures!=['oracle:'+str(result['oracle']['accepted']).lower()]):
        raise ValueError('non-oracle rejection or closure mismatch')
    if result.get('outcome') in ('accepted','rejected') and not ordinary_stop and (result['oracle'].get('accepted') is not (result['outcome']=='accepted') or receipt.get('verified_success') is not (result['outcome']=='accepted')):
        raise ValueError('outcome/oracle receipt mismatch')
    if result.get('outcome')=='accepted' and (not receipt.get('verified_success') or closures!=['oracle:true']):
        raise ValueError('acceptance evidence')
    raw=self.diagnostic_path.read_bytes()
    if not raw.startswith(self.diagnostic_before) or not raw.endswith(b'\n'): raise ValueError('diagnostic prefix')
    diagnostics=[json.loads(line) for line in raw[len(self.diagnostic_before):].splitlines()]
    save_new(destination/'diagnostics.json',diagnostics)
    save_new(destination/'process.json',process)
    save_new(destination/'receipt-verification.json',receipt)
    (destination/'ledger.jsonl').write_bytes(self.ledger())
    hashes={str(p.relative_to(destination)):sha(p) for p in destination.rglob('*') if p.is_file()}
    for p in hashes:
        with (destination/p).open('rb') as handle: os.fsync(handle.fileno())
    save_new(destination/'sha256.json',hashes)
    for directory in sorted([p for p in destination.rglob('*') if p.is_dir()],key=lambda p:len(p.parts),reverse=True): sync_directory(directory)
    sync_directory(destination)
    sync_directory(destination.parent)
    for p,h in hashes.items():
        if sha(destination/p)!=h: raise ValueError('export changed')
    return dict(result=result, diagnostics=diagnostics,
                proposed_count=sum(e['kind']['kind']=='proposed' for _,e in events),manifest_sha256=sha(destination/'sha256.json'))

if __name__=='__main__':
    if sys.argv[1:]!=['--approved-six-attempt-comparison']:raise ValueError('authorization flag missing')
    m=json.loads((ROOT/'docs/m3-two-model-v2-freeze.json').read_text())
    coordinate(m,ROOT/m['state'],Operations(m))
