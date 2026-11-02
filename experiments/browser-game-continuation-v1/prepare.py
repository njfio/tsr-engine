#!/usr/bin/env python3
"""Fresh admission on a preserved native derivative; no model or execution dispatch."""
import argparse,copy,datetime,hashlib,importlib.util,json,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('adapter',ROOT/'scripts/engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
MESSAGE_ID='Sentinel_b07cd98caee88191a9a35810f10711ed'
FEEDBACK='Keep going game needs to be more of a mashup between Pac-Man and Tetris'
def tree_hashes(root):
 paths=list(root.rglob('*'))
 if any(p.is_symlink() for p in paths):raise ValueError('source symlink')
 return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths if p.is_file()}
def prepare(out,source,campaign):
 out=out.resolve();source=source.resolve();out.mkdir(exist_ok=False)
 prior=json.loads((source/'adapter-admission.json').read_text());before=tree_hashes(source/'native');shutil.copytree(source/'native',out/'native')
 def save(name,value):(out/name).write_text(json.dumps(value,indent=2)+'\n')
 c=a.d.Native(out/'native');budget=c.budget();old_request=c.request();view=c.cli('project-lifecycle-view')
 if budget['unsettled_requests'] or budget['unresolved_tool_requests'] or budget['unknown_usage'] or budget['unknown_compute_requests']:raise ValueError('unresolved source checkpoint')
 extension={'state_root':old_request['state_root'],'previous':view['admission'],'limits':{'model_calls':budget['model_calls']+200,'model_tokens':budget['tokens_charged']+8192000,'wall_ms':budget['elapsed_wall_ms']+7200000,'patch_bytes':budget['patch_bytes_charged']+104857600,'external_cost_microusd':budget['external_cost_charged_microusd']},'tool_ms_ceiling':budget['tool_ms_charged']+7200000,'reason':'User-requested persistent game continuation; cumulative accounting retained; separate campaign enforces $10 session ceiling'}
 extended=c.json_cli(('project-resource-extend',),extension,'--request');save('resource-extension.json',extension)
 request=c.request();completion=c.cli('project','acceptance','status');checkpoint=a.web.checkpoint(request,completion,extended)
 parents=request['coding']['relational']['inputs']['parents'];meaning=next(p['meaning'] for p in parents if p['requirement']=='engaging_challenge')
 draft=a.web.feedback_draft(checkpoint,request,completion,MESSAGE_ID,{'engaging_challenge':meaning+' User feedback: '+FEEDBACK})
 save('attributed-feedback.json',{'message_id':MESSAGE_ID,'author':'user','text':FEEDBACK,'sent_at':'2026-10-29T05:12:20Z','draft':draft})
 c.json_cli(('project','acceptance','feedback'),draft['native_feedback']);r=c.request();inputs=r['coding']['relational']['inputs']
 if [p['id'] for p in inputs['parents']]!=[p['id'] for p in parents]:raise ValueError('contributors changed')
 if r['coding']['snapshot']!=request['coding']['snapshot'] or inputs['returns']!=request['coding']['relational']['inputs']['returns']:raise ValueError('history/source changed during feedback')
 public=copy.deepcopy(prior['origin']['public']);public.update(parents=inputs['parents'],source_snapshots=[r['coding']['snapshot']],public_returns=[a.e.sha(x) for x in inputs['returns']],public_relationships=[a.e.sha(x) for x in r['coding']['relational']['history']])
 public['public_brief']+=' Genuine user feedback, '+MESSAGE_ID+': '+FEEDBACK+'. End of verbatim user feedback. Operator task framing (not a user quote or empirical result): continue this existing project. Choose a coherent integration of maze pursuit/collectibles with falling-block, rotation and clearing mechanics; do not merely add an unrelated minigame. Keep original artwork/naming. This is a requested direction, not observed proof of satisfaction or a prescribed implementation.'
 origin={'directory':str(c.directory),'public':public,'initial_sources':{f['path']:f['source'] for f in r['coding']['files'] if f['editable']}}
 expires=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=2)).isoformat();ad=a.admission(out/'adapter',origin,c.cli('project','config'),campaign,expires,max_decisions=90,max_executions=100,correction_allowance=20)
 ca={'version':'local-project-cycle@v3','journal':str(out/'cycle'),'scope':public['scope'],'max_decisions':90,'max_executions':100,'local_source_profile':'browser-scoped-source@v1','expires_at':expires,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
 context=a.web.project(r,public,context_budget=24000)
 if c.cli('project','cycle','request')['branch']!='propose':raise ValueError('new feedback did not enable ordinary native proposal')
 if FEEDBACK not in json.dumps(context):raise ValueError('feedback not projected')
 save('adapter-admission.json',ad);save('cycle-admission.json',ca);save('public-preview.json',context)
 if tree_hashes(source/'native')!=before:raise ValueError('original native state changed')
 save('source-lineage.json',{'source':str(source),'source_admission':a.e.sha(prior),'source_native_files':before,'copied_before_extension':True,'same_episode':budget['episode']==extended['episode'],'history_preserved':True,'original_unchanged':True,'source_snapshot':request['coding']['snapshot']})
 result={'adapter_sha256':a.e.sha(ad),'suite_sha256':a.e.sha({'public':public,'configuration':ad['configuration_sha256']}),'calls':200,'operations':100,'corrections':20,'expires_at':expires,'provider_calls':0,'target_executions':0,'requires_reviewed_grant':True};save('prepared.json',result);return result
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--source',type=Path,required=True);p.add_argument('--campaign-manifest',type=Path,required=True);p.add_argument('--packet-sha256',required=True);args=p.parse_args()
 manifest_path=Path(__file__).with_name('manifest.json')
 if a.d.file_sha(manifest_path)!=args.packet_sha256:raise ValueError('packet drift')
 manifest=json.loads(manifest_path.read_text())
 if any(a.d.file_sha(ROOT/name)!=h for name,h in manifest['files'].items()):raise ValueError('source pin drift')
 print(json.dumps(prepare(args.output,args.source,json.loads(args.campaign_manifest.read_text())),indent=2))
