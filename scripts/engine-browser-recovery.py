"""Read-only exact-prefix verification for a separately admitted browser derivative.
Never resumes the old outer journal, invokes a worker, or alters game source.
"""
import hashlib,json
from pathlib import Path

def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def tree(root):
 root=Path(root);paths=list(root.rglob('*'))
 if any(p.is_symlink() for p in paths):raise ValueError('recovery symlink')
 return {str(p.relative_to(root)):sha(p) for p in paths if p.is_file()}
def verify(source,proof,a):
 source=Path(source)
 if set(proof)!={'files','native_files','pending_stage','execution_name','source_snapshot','observations_sha256','assessment'}:raise ValueError('recovery proof shape')
 if tree(source/'native')!=proof['native_files']:raise ValueError('native prefix/source drift')
 for name,digest in proof['files'].items():
  if Path(name).is_absolute() or '..' in Path(name).parts or sha(source/name)!=digest:raise ValueError('frozen recovery evidence drift')
 def journal(path):
  rows=[json.loads(l) for l in (source/path).read_text().splitlines()];pending={};done={}
  for i,row in enumerate(rows):
   if row!=a.d.Driver.row(rows[:i],row['event']):raise ValueError('recovery journal chain')
   e=row['event']
   if e['kind']=='intent':pending[e['name']]=e
   elif e['kind']=='result':
    if e['name'] not in pending or e['intent']!=a.e.sha(pending.pop(e['name'])):raise ValueError('unmatched recovery effect')
    done[e['name']]=e['value']
  return rows,pending,done
 ar,ap,ad=journal('adapter/adapter.jsonl');cr,cp,cd=journal('cycle/cycle.jsonl')
 if ap or set(cp)!={proof['pending_stage']}:raise ValueError('not exact pre-dispatch pending stage')
 if cr[-1]['event']!=cp[proof['pending_stage']]:raise ValueError('pending stage not terminal intent')
 if proof['execution_name'] not in ad or not proof['execution_name'].startswith('execution_'):raise ValueError('completed execution missing')
 # Last adapter operation is the completed native execution; no subsequent reservation/dispatch.
 if ar[-1]['event'].get('name')!=proof['execution_name'] or ar[-1]['event']['kind']!='result':raise ValueError('effects after known completed execution')
 c=a.d.Native(source/'native');budget=c.budget();request=c.request();view=c.cli('project','cycle','request')
 if any(budget[k] for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')):raise ValueError('unknown native effect')
 if budget.get('stopped') or budget.get('exceeded'):raise ValueError('stopped or exceeded source checkpoint')
 if view['branch']!='reconsider' or request['coding']['snapshot']!=proof['source_snapshot']:raise ValueError('not preserved post-return reconsideration')
 completion=ad[proof['execution_name']]['state']['goal_completion']
 if completion['snapshot']!=proof['source_snapshot'] or completion['test_verdict'] not in ('passed','failed'):raise ValueError('completion source/outcome mismatch')
 receipt_ids=ad[proof['execution_name']]['receipts']+[completion['test_receipt']]
 if not receipt_ids or not any(r['receipt'] in receipt_ids for r in request['coding']['relational']['inputs']['returns']):raise ValueError('execution return not in current native context')
 obs_path=source/'independent-browser/observations.json'
 if sha(obs_path)!=proof['observations_sha256']:raise ValueError('observation drift')
 obs=json.loads(obs_path.read_text());files={f['path']:f['source'] for f in request['coding']['files'] if f['editable']}
 if set(obs['source'])!=set(files) or any(hashlib.sha256(files[n].encode()).hexdigest()!=h for n,h in obs['source'].items()):raise ValueError('observation/source mismatch')
 return {'request':request,'view':view,'budget':budget,'receipts':receipt_ids,'observations':obs,'source_unchanged':True}
