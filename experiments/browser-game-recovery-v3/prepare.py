#!/usr/bin/env python3
"""Fresh source-bound derivative after a proven pre-dispatch projection failure."""
import argparse,copy,datetime,hashlib,importlib.util,json,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('a',ROOT/'scripts/engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
rec=a.load('browser_recovery','engine-browser-recovery.py')
def prepare(out,source,campaign,*,fixture=False):
 out=out.resolve();source=source.resolve();proof=json.loads(Path(__file__).with_name('source-proof.json').read_text())
 if rec.tree(source/'native')!=proof['native_files'] or any(rec.sha(source/n)!=h for n,h in proof['files'].items()):raise ValueError('source prefix drift')
 completed={}
 for name in ('adapter','cycle'):
  rs=[json.loads(l) for l in (source/name/(name+'.jsonl')).read_text().splitlines()];pending={};done={}
  for i,row in enumerate(rs):
   if row!=a.d.Driver.row(rs[:i],row['event']):raise ValueError('source journal chain')
   ev=row['event']
   if ev['kind']=='intent':pending[ev['name']]=ev
   if ev['kind']=='result':
    if ev['name'] not in pending or ev['intent']!=a.e.sha(pending.pop(ev['name'])):raise ValueError('source unmatched effect')
    done[ev['name']]=ev['value']
  if (pending if name=='adapter' else set(pending)!={proof['pending_stage']}):raise ValueError('source pending mismatch')
  completed[name]=done
 if any(n.startswith('execution_') for n in completed['adapter']):raise ValueError('unexpected execution')
 if completed['cycle']['reconsider_0_review']['judgment']['decision']!='approve' or 'reconsider_0_advance' not in completed['cycle']:raise ValueError('missing admitted relationship review')
 m=a.load('recovery_campaign','engine-b91-campaign.py');rows=[json.loads(l) for l in Path(campaign['ledger_path']).read_text().splitlines()]
 for i,row in enumerate(rows):
  if row!=m.Ledger.row(rows[:i],row['event']):raise ValueError('campaign chain')
 ledger=object.__new__(m.Ledger);ledger.manifest=campaign;ledger.hash=m.digest(campaign);status,trials,calls,executions=ledger.fold(rows)
 if status['stopped'] or (status['calls'],status['execution_groups'],status['known_cost_microusd'])!=(282,76,267746):raise ValueError('settled campaign baseline')
 if len([x for x in calls.values() if x['trial']==proof['trial']])!=5 or any(x['trial']==proof['trial'] for x in executions.values()):raise ValueError('failed source effect inventory')
 out.mkdir(exist_ok=False);shutil.copytree(source/'native',out/'native')
 def save(name,value):(out/name).write_text(json.dumps(value,indent=2)+'\n')
 prior=json.loads((source/'adapter-admission.json').read_text());c=a.d.Native(out/'native');request=c.request();budget=c.budget();view=c.cli('project-lifecycle-view')
 if c.cli('project','cycle','request')['branch']!='propose':raise ValueError('must resume ordinary native choice')
 if any(budget[k] for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')) or budget['stopped']:raise ValueError('unknown native effects')
 extension={'state_root':request['state_root'],'previous':view['admission'],'limits':{'model_calls':budget['model_calls']+182,'model_tokens':budget['tokens_charged']+197400000,'wall_ms':budget['elapsed_wall_ms']+7200000,'patch_bytes':budget['patch_bytes_charged']+104857600,'external_cost_microusd':budget['external_cost_charged_microusd']},'tool_ms_ceiling':budget['tool_ms_charged']+7200000,'reason':'Same user-requested game; new derivative after verified pre-dispatch failure; no completed execution replay; campaign still enforces remaining budget'}
 extended=c.json_cli(('project-resource-extend',),extension,'--request');save('resource-extension.json',extension)
 r=c.request();inputs=r['coding']['relational']['inputs'];native_view=c.cli('project','cycle','request')
 before=a.d.Native(source/'native').request()
 if native_view['branch']!='propose' or r['coding']['snapshot']!=proof['source_snapshot'] or r['coding']['relational']!=before['coding']['relational']:raise ValueError('meaning/source history changed')
 public=copy.deepcopy(prior['origin']['public']);public.update(parents=inputs['parents'],source_snapshots=[r['coding']['snapshot']],public_returns=[a.e.sha(x) for x in inputs['returns']],public_relationships=[a.e.sha(x) for x in r['coding']['relational']['history']])
 critique=copy.deepcopy(prior['origin']['prior_review_feedback'])
 # Preserve source-bound independent feedback and the actual semantic criticism.
 rejected=[v for n,v in completed['adapter'].items() if n.startswith('review_rejection_')]
 if len(rejected)!=1:raise ValueError('missing unique prior review criticism')
 critique['assessment']+=' Prior separate runtime review (fallible judgment, not a new requirement; incremental proposals remain allowed) rejected the reset-only proposal as incomplete: '+rejected[0]['report']['findings']['stage_adequacy']['explanation']
 origin={'directory':str(c.directory),'public':public,'prior_review_feedback':critique,'initial_sources':{f['path']:f['source'] for f in r['coding']['files'] if f['editable']}}
 expires=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=2)).isoformat();ad=a.admission(out/'adapter',origin,c.cli('project','config'),campaign,expires,max_decisions=81,max_executions=98,correction_allowance=20)
 ca={'version':'local-project-cycle@v3','journal':str(out/'cycle'),'scope':public['scope'],'max_decisions':81,'max_executions':98,'completion_tokens':32768,'input_tokens':a.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'local_source_profile':'browser-scoped-source@v1','expires_at':expires,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
 save('adapter-admission.json',ad);save('cycle-admission.json',ca);save('attributed-feedback.json',json.loads((source/'attributed-feedback.json').read_text()));save('source-lineage.json',{'source':str(source),'source_native_files':proof['native_files'],'source_proof_sha256':rec.sha(Path(__file__).with_name('source-proof.json')),'source_snapshot':proof['source_snapshot'],'source_unchanged':rec.tree(source/'native')==proof['native_files'],'no_completed_execution_repeated':True,'start_branch':native_view['branch'],'history_preserved':True})
 if rec.tree(source/'native')!=proof['native_files']:raise ValueError('original changed')
 result={'adapter_sha256':a.e.sha(ad),'suite_sha256':a.e.sha({'public':public,'configuration':ad['configuration_sha256']}),'calls':182,'operations':98,'corrections':20,'expires_at':expires,'provider_calls':0,'target_executions':0};save('prepared.json',result);return result
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--source',type=Path,required=True);p.add_argument('--campaign-manifest',type=Path,required=True);p.add_argument('--packet-sha256',required=True);args=p.parse_args()
 manifest=Path(__file__).with_name('manifest.json')
 if rec.sha(manifest)!=args.packet_sha256:raise ValueError('packet drift')
 if any(rec.sha(ROOT/n)!=h for n,h in json.loads(manifest.read_text())['files'].items()):raise ValueError('source pins')
 print(json.dumps(prepare(args.output,args.source,json.loads(args.campaign_manifest.read_text())),indent=2))
