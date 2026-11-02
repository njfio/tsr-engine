"""Explicit reviewed representation migration; never generates or edits a response.

The original adapter remains frozen, including its failed apply intent. A separate
append-only journal records the identical native admission under corrected code.
Only proven committed interruptions recover automatically; no uncertain replay.
"""
import argparse
import fcntl
import importlib.util
import json
import os
from pathlib import Path

s=importlib.util.spec_from_file_location('migration_browser',Path(__file__).with_name('engine-browser-adapter.py'))
a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
h=a.load('migration_criterion','engine-criterion-return.py')

class FrozenReader(a.Adapter):
 def __init__(self,admission):
  self.admission=admission;self.root=Path(admission['directory']);self.file=self.root/'adapter.jsonl';self.fixture=False;self.release=True
 def append(self,*args):raise ValueError('frozen parent is read-only')
 def op(self,*args,**kwargs):raise ValueError('frozen parent cannot execute')

def retained(reader):
 _,pending,done=reader.read()
 if len(pending)!=1:raise ValueError('one failed local admission required')
 name=next(iter(pending))
 if not name.startswith('criterion_') or not name.endswith('_apply'):raise ValueError('not criterion admission')
 stem=name[:-6];capture=done[stem+'_input'];review=done[stem+'_review'];value=done[stem+'_native_input']
 # Narrow migration of unchanged uncertainty, never met/not_met or abstention.
 if review['value']['verdict']!='unknown' or any(f['finding']!='unknown' for f in review['value']['findings'].values()):raise ValueError('only retained uncertainty may migrate')
 if any(k.startswith('format_correction_') for k in done):raise ValueError('corrected historical request requires separate migration review')
 h.verify_retained(reader,stem,capture,review)
 expected={'state_root':value['state_root'],'binding':value['binding'],'frame':capture['frame'],'reviewer':'separate-runtime-invocation:'+review['request'],'generator':capture['generator'],'response_digest':a.e.sha(review['value']),'decision':'unknown','findings':h.native_findings(review['value'],capture['frame'])}
 if expected!=value:raise ValueError('native response changed')
 return value,review['request']

def check_context(context,value):
 current=context['current']
 if current is not None:
  if current['assessment']!=value:raise ValueError('different assessment committed')
  return current['id']
 if any(context[k]!=value[k] for k in ('state_root','binding','frame')):raise ValueError('stale migration source or evidence')
 return None

def advance(journal,value,context,admit,guard,hook=lambda *_:None):
 """Restart may finish a proven commit; an uncommitted pending intent stays blocked."""
 guard();rows,pending,done=journal.read();current=check_context(context(),value)
 if 'apply' in done:
  if current!=done['apply']:raise ValueError('completed migration receipt changed')
  return current
 if pending:
  if set(pending)!={'apply'} or current is None:raise ValueError('migration effect not proven committed; no replay')
  journal.append({'kind':'result','name':'apply','intent':a.e.sha(pending['apply']),'value':current})
  return current
 if current is not None:raise ValueError('assessment committed outside migration')
 intent={'kind':'intent','name':'apply','at':a.d.iso_now()};journal.append(intent);hook('after_intent')
 guard();result=admit(value);hook('after_effect')
 if check_context(context(),value)!=result:raise ValueError('migration receipt mismatch')
 journal.append({'kind':'result','name':'apply','intent':a.e.sha(intent),'value':result})
 return result

def implementation():return {**a.pins(),Path(__file__).name:a.d.file_sha(Path(__file__))}

def run(plan,expected,mode,*,hook=lambda *_:None):
 if a.e.sha(plan)!=expected or plan['version']!='criterion-representation-migration@v1':raise ValueError('reviewed plan binding')
 if plan['implementation']!=implementation() or plan['binary_sha256']!=a.d.file_sha(a.e.ROOT/'target/debug/tsr'):raise ValueError('corrected runtime drift')
 root=Path(plan['directory']).resolve()
 if not root.is_relative_to(a.e.ROOT/'target') or any(x.is_symlink() for x in (root,*root.parents)):raise ValueError('migration path')
 for path,digest in plan['files'].items():
  if a.d.file_sha(Path(path))!=digest:raise ValueError('frozen migration input drift: '+path)
 ad=a.e.strict(Path(plan['admission']).read_bytes());reader=FrozenReader(ad)
 required={plan['admission'],str(reader.file),str(Path(plan['cycle'])/'cycle.jsonl'),plan['campaign'],plan['upgrade']}
 if not required.issubset(plan['files']) or plan['previous_binary_sha256']!=ad['binary_sha256']:raise ValueError('original admission/evidence pins missing')
 if set(plan['resource_totals'])!={'model_calls','tokens_charged','tool_ms_charged','external_cost_charged_microusd'}:raise ValueError('complete retained resource totals required')
 lock=os.fdopen(os.open(reader.root/'launch.lock',os.O_WRONLY|os.O_NOFOLLOW),'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 value,request=retained(reader)
 if a.e.sha(value)!=plan['native_input_sha256'] or request!=plan['request']:raise ValueError('retained response binding')
 native_path=Path(plan['native_directory']).resolve()
 if not native_path.is_relative_to(a.e.ROOT/'target') or any(x.is_symlink() for x in (native_path,*native_path.parents)):raise ValueError('native migration path')
 native=a.d.Native(native_path)
 m=a.load('migration_campaign','engine-b91-campaign.py');manifest=a.e.strict(Path(plan['campaign']).read_bytes());upgrade=a.e.strict(Path(plan['upgrade']).read_bytes());ledger=m.Ledger(manifest['ledger_path'],manifest,m.digest(manifest),upgrade=upgrade,upgrade_hash=m.digest(upgrade))
 def guard():
  for folder in (reader.root,Path(plan['cycle']),root):
   if (folder/'STOP').exists() or any(folder.glob('checkpoint-*/STOP')):raise ValueError('migration stopped')
  for path,digest in plan['files'].items():
   if a.d.file_sha(Path(path))!=digest:raise ValueError('frozen input changed')
  state=ledger.status();budget=native.budget()
  if any(budget.get(k)!=v for k,v in plan['resource_totals'].items()):raise ValueError('resource accounting changed')
  if state['stopped'] or state['blocking_unknown_calls'] or state['blocking_unknown_executions'] or any(budget.get(k) for k in ('stopped','exceeded','unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')):raise ValueError('migration stopped or external effects unresolved')
  ledger.require_output_eligible(m.digest({'trial':str(reader.root),'admission':a.e.sha(ad)}))
 guard();context=lambda:native.cli('project','criterion-context');current=context();check_context(current,value)
 if mode=='check':return {'ready':True,'request':request,'native_input_sha256':a.e.sha(value),'provider_calls':0,'assessment_current':current['current'] is not None}
 root.mkdir(exist_ok=True);journal=object.__new__(a.d.Driver);journal.root=root;journal.file=root/'migration.jsonl';journal.admission=plan;journal.fixture=False
 if not journal.file.exists():a.e.private_save(journal.file,a.d.Driver.row([],{'kind':'admission','value':plan,'fixture_only':False}))
 receipt=advance(journal,value,context,lambda v:native.json_cli(('project','criterion-apply'),v),guard,hook)
 return {'receipt':receipt,'unchanged_response':request,'provider_calls':0,'external_executions':0,'parent_journal_unchanged':True}

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('mode',choices=('check','apply'));p.add_argument('--plan',required=True);p.add_argument('--expected-sha256',required=True);args=p.parse_args()
 print(json.dumps(run(a.e.strict(Path(args.plan).read_bytes()),args.expected_sha256,args.mode),indent=2))
