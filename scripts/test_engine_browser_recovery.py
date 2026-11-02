"""Authored local recovery flow; no provider calls or historical mutation."""
import copy,importlib.util,json,os,shutil,tempfile,unittest
from pathlib import Path
from test_engine_browser_adapter import m as a,campaign
rec=a.load('recovery_checks','engine-browser-recovery.py')
@unittest.skipUnless(os.getenv('TSR_BROWSER_RECOVERY_SOURCE'),'explicit preserved source for local authored recovery')
class Recovery(unittest.TestCase):
 def setUp(self):
  self.source=Path(os.environ['TSR_BROWSER_RECOVERY_SOURCE']);self.proof=json.loads((a.e.ROOT/'experiments/browser-game-recovery-v1/source-proof.json').read_text())
 def test_exact_prefix_and_negative_controls(self):
  checked=rec.verify(self.source,self.proof,a);self.assertEqual(checked['view']['branch'],'reconsider')
  for field,value in [('source_snapshot','f'*64),('observations_sha256','f'*64),('pending_stage','other'),('execution_name','execution_other')]:
   with self.subTest(field=field),self.assertRaises(ValueError):rec.verify(self.source,{**self.proof,field:value},a)
  bad=copy.deepcopy(self.proof);bad['native_files']['coding-work/app.js']='f'*64
  with self.assertRaises(ValueError):rec.verify(self.source,bad,a)
  bad=copy.deepcopy(self.proof);bad['files']['adapter/adapter.jsonl']='f'*64
  with self.assertRaises(ValueError):rec.verify(self.source,bad,a)
 def test_unknown_native_cost_and_appended_dispatch_rejected(self):
  from unittest.mock import patch
  original=a.d.Native
  class UnknownCost:
   def __init__(self,path):self.real=original(path)
   def budget(self):return {**self.real.budget(),'unknown_external_cost_requests':1}
   def request(self):return self.real.request()
   def cli(self,*args):return self.real.cli(*args)
  with patch.object(a.d,'Native',UnknownCost),self.assertRaisesRegex(ValueError,'unknown native effect'):rec.verify(self.source,self.proof,a)
  root=Path(tempfile.mkdtemp(dir=a.e.ROOT/'target',prefix='recovery-tamper-'));self.addCleanup(shutil.rmtree,root)
  shutil.copytree(self.source/'native',root/'native')
  for name in self.proof['files']:
   p=root/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(self.source/name,p)
  p=root/'adapter/adapter.jsonl';rows=[json.loads(l) for l in p.read_text().splitlines()];rows.append(a.d.Driver.row(rows,{'kind':'intent','name':'dispatch_forged','at':'2026-10-29T00:00:00Z'}));p.write_text(''.join(json.dumps(r)+'\n' for r in rows));proof=copy.deepcopy(self.proof);proof['files']['adapter/adapter.jsonl']=rec.sha(p)
  with self.assertRaisesRegex(ValueError,'pre-dispatch'):rec.verify(root,proof,a)
 def test_missing_external_settlement_blocks_campaign(self):
  manifest=json.loads((a.e.ROOT/'target/tsr-integrated-campaign-1/manifest.json').read_text());rows=[json.loads(l) for l in Path(manifest['ledger_path']).read_text().splitlines()]
  last=max(i for i,r in enumerate(rows) if r['event']['kind']=='return');rows[last]['event']['outcome']='unknown';rebuilt=[]
  for row in rows:rebuilt.append(campaign.Ledger.row(rebuilt,row['event']))
  ledger=object.__new__(campaign.Ledger);ledger.manifest=manifest;ledger.hash=campaign.digest(manifest)
  self.assertTrue(ledger.fold(rebuilt)[0]['stopped'])
 @unittest.skipUnless(os.getenv('TSR_BROWSER_RECOVERY_FLOW'),'explicit authored sealed integration')
 def test_reconsider_fresh_choice_review_execution_and_restart(self):
  root=Path(tempfile.mkdtemp(dir=a.e.ROOT/'target',prefix='recovery-authored-'));self.addCleanup(shutil.rmtree,root)
  # Prepared derivative contains genuine old returns + attributed independent feedback;
  # all new model responses below are authored test stubs, not live behavior.
  prepared=a.e.ROOT/'target/browser-game-recovery-authored-check'
  shutil.copytree(prepared/'native',root/'native');c=a.d.Native(root/'native')
  old=json.loads((prepared/'adapter-admission.json').read_text());origin=copy.deepcopy(old['origin']);origin['directory']=str(c.directory)
  manifest=campaign.manifest('a'*64,root/'ledger',fixture=True);ledger=campaign.Ledger(root/'ledger',manifest,campaign.digest(manifest),create=True)
  ad=a.admission(root/'adapter',origin,c.cli('project','config'),manifest,old['expires_at'],max_decisions=6,max_executions=1,correction_allowance=1)
  bodies=[];seen_proposal=False
  def transport(body,timeout):
   nonlocal seen_proposal
   data=json.loads(body['messages'][1]['content']);bodies.append(data)
   if 'criteria' in data:value={'verdict':'accept','findings':{k:{'finding':'met','basis':'static_source','evidence':[data['binding']['evidence_ids'][0]],'explanation':'Authored wiring judgment only','limitations':'Not model behavior or player satisfaction'} for k in data['criteria']}}
   elif 'advisory_reconsideration' in data:
    prior=data['relational_history'][-1];p=prior['proposal'];lineage=data['relational_contract']['allowed_lineage'][-1]
    value={'alternatives':[{'relational':{**p,'revises':prior['admission'],'development':{'original_revision':lineage['original_revision'],'prior_revision':lineage['prior_revision'],'retained_meaning':p['aspiration'],'return_reveals':'Authored test retains meaning against the actual bounded browser return; no new semantic claim.','unresolved':'Functional defects remain a separate independent assessment.','proposed_development':None}},'return_basis':{'binding':data['return_evidence']['binding'],'observed':[x['id'] for x in data['return_evidence']['cases']][:8],'inferred':[],'unknown':['Player satisfaction is unknown.']}}]}
   else:
    seen_proposal=True;self.assertIn('review_criticism',data)
    value={'alternatives':[{'edits':[{'path':'app.js','old_text':'(() => {','new_text':'(() => {\n  // Authored recovery wiring fixture; no gameplay repair claim.'}],'rationale':'Authored harmless fixture edit, not an engine repair or claim to fix defects'}]}
   return {'value':value,'receipt':{'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.1}}
  adapter=a.Adapter(root/'adapter',ad,c,ledger,transport,worker=lambda view:adapter.resume_process(view));self.assertEqual(adapter.native('request',None)['branch'],'reconsider')
  ca={'version':'local-project-cycle@v3','journal':str(root/'cycle'),'scope':origin['public']['scope'],'max_decisions':6,'max_executions':1,'completion_tokens':32768,'input_tokens':a.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'local_source_profile':'browser-scoped-source@v1','expires_at':ad['expires_at'],'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
  cycle=a.loop.Cycle(root/'cycle',ca,adapter.native,adapter.proposal,adapter.review);result=cycle.run()
  self.assertTrue(seen_proposal);self.assertEqual(result['status'],'review_required');self.assertEqual(ledger.status()['execution_groups'],1)
  before=(root/'ledger').read_bytes();self.assertEqual(a.loop.Cycle(root/'cycle',ca,adapter.native,adapter.proposal,adapter.review).run(),result);self.assertEqual((root/'ledger').read_bytes(),before)
  self.assertEqual(rec.tree(self.source/'native'),self.proof['native_files'])
if __name__=='__main__':unittest.main()
