"""Local exact-receipt crash controls; no provider or execution effects."""
import copy,datetime,json,shutil,tempfile,unittest
from pathlib import Path
from test_engine_browser_adapter import m,campaign

class Recovery(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory(dir=m.e.ROOT/'target');self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)
  source=m.e.ROOT/getattr(self,'source_fixture','target/v2-tsr-evidence-feedback')
  if not source.exists():self.skipTest('preserved source-bound evidence checkpoint unavailable')
  shutil.copytree(source/'native',self.root/'native');self.c=m.d.Native(self.root/'native')
  b=self.c.budget();life=self.c.cli('project-lifecycle-view')
  self.c.json_cli(('project-resource-extend',),{'state_root':self.c.request()['state_root'],'previous':life['admission'],'limits':{'model_calls':b['model_calls']+20,'model_tokens':b['tokens_charged']+2000000,'wall_ms':b['elapsed_wall_ms']+600000,'patch_bytes':b['patch_bytes_charged']+1048576,'external_cost_microusd':b['external_cost_charged_microusd']},'tool_ms_ceiling':b['tool_ms_charged']+120000,'reason':'Authored local receipt recovery only'},'--request')
  old=json.loads((source/'adapter-admission.json').read_text());origin=copy.deepcopy(old['origin']);origin['directory']=str(self.c.directory)
  q=self.c.request();coding=q['coding'];origin['initial_sources']={f['path']:f['source'] for f in coding['files'] if f['editable']}
  origin['public'].update(source_snapshots=[coding['snapshot']],public_returns=[m.e.sha(v) for v in coding['relational']['inputs']['returns']],public_relationships=[m.e.sha(v) for v in coding['relational']['history']]);origin.pop('prior_review_feedback',None)
  self.manifest=campaign.manifest('a'*64,self.root/'ledger',fixture=True);self.ledger=campaign.Ledger(self.root/'ledger',self.manifest,campaign.digest(self.manifest),create=True)
  self.policy={'version':'authorized-feedback-cycle@v1','authorization_id':'authored-recovery','max_continuations':3}
  self.a=m.admission(self.root/'adapter',origin,self.c.cli('project','config'),self.manifest,(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(minutes=30)).isoformat(),max_decisions=4,max_executions=0,feedback_cycle=self.policy)
 def adapter(self):return m.Adapter(self.root/'adapter',self.a,self.c,self.ledger,lambda *_:self.fail('No model dispatch permitted'))
 def test_committed_transition_recovered_without_redispatch(self):
  a=self.adapter();before=a.native('request',None);self.assertEqual(before['branch'],'review_required')
  def crash(phase,name):
   if phase=='after_effect' and name=='session_continue_0':raise RuntimeError('Authored interruption after native commit')
  a.hook=crash
  with self.assertRaisesRegex(RuntimeError,'Authored interruption'):a.continue_session({'binding':before['binding'],'index':0,'reason':'initial_checkpoint'})
  self.assertTrue(a.read()[1]);fresh=self.adapter();self.assertEqual(fresh.recover_session_transition(),1)
  record=fresh.native('session_transition_receipt',{'index':0});self.assertEqual(record['reason'],'initial_checkpoint');self.assertEqual(fresh.recover_session_transition(),0)
  self.assertEqual(self.ledger.status()['calls'],0);self.assertEqual(self.c.request()['coding']['snapshot'],before['source_snapshot'])
  wrong=copy.deepcopy(record['transition']['instruction']);wrong['message_id']='substitution'
  with self.assertRaises(Exception):self.c.json_cli(('project','cycle','continuation-receipt'),wrong)
  # No recovery of an outer callback unless the exact native transition exists.
  ca={'version':'local-project-cycle@v3','journal':str(self.root/'cycle'),'scope':before['scope'],'max_decisions':4,'max_executions':0,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','feedback_cycle':self.policy}
  cycle=m.loop.Cycle(ca['journal'],ca,fresh.native,lambda *_:self.fail('proposal'),lambda *_:self.fail('review'))
  cycle.append({'kind':'intent','name':'session_continue_0','at':m.d.iso_now()})
  self.assertEqual(cycle.recover_pending(),1);self.assertFalse(cycle.read()[1]);self.assertEqual(cycle.read()[2]['session_continue_0'],record)
 def test_consumed_and_changed_context_receipts_reject(self):
  a=self.adapter();v=a.native('request',None);record=a.continue_session({'binding':v['binding'],'index':0,'reason':'initial_checkpoint'})
  self.c.cli('project','cycle','close-continuation','--state-root',self.c.request()['state_root'],'--continuation',record['result']['continuation'])
  with self.assertRaises(Exception):a.native('session_transition_receipt',{'index':0})
  v=a.native('request',None);record=a.continue_session({'binding':v['binding'],'index':1,'reason':'after_reflection'})
  status=self.c.cli('project','acceptance','status');parents=self.c.request()['coding']['relational']['inputs']['parents']
  self.c.json_cli(('project','acceptance','feedback'),{'requirements_revision':status['requirements_revision'],'decision_record':status['decision_record'],'guidance':{p['requirement']:'Authored changed context for stale receipt rejection' for p in parents}})
  with self.assertRaises(Exception):a.native('session_transition_receipt',{'index':1})
 def test_preview_timeout_retries_only_local_unchanged_once(self):
  import subprocess
  from unittest.mock import patch
  a=self.adapter();original=m.base.Adapter.native;attempts=[]
  def injected(obj,kind,value):
   if kind=='preview':
    attempts.append(kind)
    if len(attempts)!=2:raise subprocess.TimeoutExpired(['rollback-only-preview'],120)
    return {'authored':'local preview result'}
   return original(obj,kind,value)
  with patch.object(m.base.Adapter,'native',injected):
   self.assertEqual(a.native('preview',{'response':{}}),{'authored':'local preview result'})
   with self.assertRaises(subprocess.TimeoutExpired):a.native('preview',{'response':{}})
  self.assertEqual(len(attempts),3);self.assertEqual(self.ledger.status()['calls'],0)
 def test_missing_native_receipt_stops_without_mutation(self):
  a=self.adapter();v=a.native('request',None)
  instruction={'binding':v['binding'],'message_id':'authored-recovery:0','reason':'Authored only'}
  a.op('continuation_input_0',lambda:{'purpose':'initial_checkpoint','instruction':instruction,'evidence_anchor':a.evidence_anchor(),'criticism_keys':[],'before':{k:v[k] for k in ('state_root','source_snapshot','scope')}})
  a.append({'kind':'intent','name':'session_continue_0','at':m.d.iso_now()})
  with self.assertRaises(Exception):self.adapter().recover_session_transition()
  self.assertEqual(self.c.cli('project','cycle','request')['binding'],v['binding']);self.assertEqual(self.ledger.status()['calls'],0)
 def test_startup_scope_identity_check_and_idempotency(self):
  a=self.adapter();launcher=m.load('startup','engine-native-cycle.py');grant=a.scope_grant('a'*64,'Authored independent review',500000);digest=campaign.digest(grant)
  before=(self.root/'ledger').read_bytes();launcher.ensure_scope_grant(self.ledger,campaign,a,grant,digest,check_only=True);self.assertEqual(before,(self.root/'ledger').read_bytes())
  with self.assertRaises(ValueError):launcher.ensure_scope_grant(self.ledger,campaign,a,{**grant,'session_admission_sha256':'f'*64},digest)
  launcher.ensure_scope_grant(self.ledger,campaign,a,grant,digest);after=(self.root/'ledger').read_bytes();launcher.ensure_scope_grant(self.ledger,campaign,a,grant,digest);self.assertEqual(after,(self.root/'ledger').read_bytes())
 def test_common_decision_ordinal_excludes_continuation(self):
  self.assertEqual(m.loop.Cycle.decision_count({'advance_0':{},'session_continue_0':{},'advance_1':{},'reconsider_2_acknowledge':{}}),3)

class CompletionWitness(unittest.TestCase):
 def test_latest_receipt_required_not_merely_temporal_order(self):
  execution={'state':{'goal_completion':{'snapshot':'now','test_receipt':'latest'}}}
  self.assertFalse(m.return_witness(execution,{})['reviewed_return'])
  self.assertFalse(m.return_witness(execution,{'0':{'snapshot':'now','resolved_observations':[{'receipt':'old'}]}})['reviewed_return'])
  self.assertFalse(m.return_witness(execution,{'0':{'snapshot':'old','resolved_observations':[{'receipt':'latest'}]}})['reviewed_return'])
  self.assertTrue(m.return_witness(execution,{'0':{'snapshot':'now','resolved_observations':[{'receipt':'latest'}]}})['reviewed_return'])

if __name__=='__main__':unittest.main()

class CapacityGrant(unittest.TestCase):
 def test_explicit_full_session_extension_preserves_baseline_and_limits(self):
  with tempfile.TemporaryDirectory(dir=m.e.ROOT/'target') as directory:
   path=Path(directory)/'ledger';manifest=campaign.manifest('a'*64,path,fixture=True);ledger=campaign.Ledger(path,manifest,campaign.digest(manifest),create=True)
   st=ledger.status()
   with ledger.locked() as (_,rows):head=rows[-1]['hash']
   g={'version':'native-browser-harness@v2','manifest_sha256':ledger.hash,'ledger_prefix_head':head,'previous_max_calls':st['effective_max_calls'],'max_calls':max(st['effective_max_calls'],200),'previous_max_execution_groups':st['effective_max_execution_groups'],'max_execution_groups':max(st['effective_max_execution_groups'],100),'cap_microusd':max(st['effective_cap_microusd'],10000000),'reservation_microusd':50000,'authority_sha256':'b'*64,'reviewer':'Authored explicit full-session authority','suite_manifest_sha256':'c'*64,'source_class':'newly-authored-nonprivate-browser-project','session_trial_sha256':'d'*64,'session_admission_sha256':'e'*64,'baseline':{'calls':0,'execution_groups':0,'settled_microusd':0},'session_max_calls':200,'session_max_execution_groups':100,'session_cap_microusd':10000000}
   for patch in ({'session_max_calls':201},{'session_max_execution_groups':101},{'session_cap_microusd':10000001},{'baseline':{'calls':1,'execution_groups':0,'settled_microusd':0}}):
    bad={**g,**patch};before=path.read_bytes()
    with self.assertRaises(ValueError):ledger.grant_calls(bad,campaign.digest(bad))
    self.assertEqual(path.read_bytes(),before)
   ledger.grant_calls(g,campaign.digest(g));result=ledger.status();self.assertEqual(result['calls'],0);self.assertGreaterEqual(result['remaining_calls'],200);self.assertGreaterEqual(result['remaining_execution_groups'],100)
   with self.assertRaises(ValueError):ledger.grant_calls(g,campaign.digest(g))

class CandidateIdentity(unittest.TestCase):
 def test_transport_identity_does_not_hide_repetition_but_evidence_changes_do(self):
  r={'proposals':[{'kind':'relational','binding':'old','proposal':{'relationship':'authored'}}]}
  b={'0':{'basis':{'binding':'old','unknown':['uncertain']},'resolved_observations':[{'receipt':'receipt1'}]}}
  changed=copy.deepcopy(r);changed['proposals'][0]['binding']='new'
  b2=copy.deepcopy(b);b2['0']['basis']['binding']='new'
  self.assertEqual(m.semantic_candidate(r,b),m.semantic_candidate(changed,b2))
  b2['0']['basis']['unknown']=['corrected substantive limitation'];self.assertNotEqual(m.semantic_candidate(r,b),m.semantic_candidate(r,b2))
  b2=copy.deepcopy(b);b2['0']['resolved_observations'][0]['receipt']='receipt2';self.assertNotEqual(m.semantic_candidate(r,b),m.semantic_candidate(r,b2))

class RejectionIdentity(unittest.TestCase):
 def test_corrected_basis_gets_own_exact_criticism(self):
  from unittest.mock import patch
  a=object.__new__(m.Adapter);a.admission={'max_review_rejections':3,'origin':{}};a.validate_current_admission=lambda:None
  from types import SimpleNamespace
  eligibility=[];a.trial='authored-rejection-fixture';a.ledger=SimpleNamespace(require_output_eligible=lambda trial:eligibility.append(trial))
  view={'state_root':'root','source_snapshot':'source','scope':'scope'};done={};a.read=lambda:('',{},done);a.evidence_anchor=lambda:'anchor'
  a.op=lambda key,fn:done.setdefault(key,fn());response={'proposals':[]}
  def owned(judgment,response):return {'generated_request':judgment['reviewer'],'report':{'explanation':judgment['reviewer']}}
  a.verify_owned_review=owned
  with patch.object(m.base.Adapter,'native',lambda *_:view):
   for ident,unknown in [('first','old uncertainty'),('second','corrected uncertainty')]:
    done['generated_'+ident]={'response':response,'return_basis':{'unknown':[unknown]}}
    result=a.native('reject_proposal',{'source':view,'response':response,'judgment':{'decision':'reject','response_digest':'same-native-payload','reviewer':ident}})
    self.assertEqual(result['report']['explanation'],ident)
  self.assertEqual(len([k for k in done if k.startswith('review_rejection_')]),2)
  self.assertEqual(eligibility,[a.trial,a.trial])

class CheckpointRecovery(Recovery):
 def test_namespaced_continuation_crash_and_long_authorization(self):
  self.policy['authorization_id']='a'*200
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=4,max_executions=0,feedback_cycle=self.policy)
  a=self.adapter();before=a.native('request',None);cp='e'*64;key=cp+'_0'
  def crash(phase,name):
   if phase=='after_effect' and name=='session_continue_'+key:raise RuntimeError('authored namespaced interruption')
  a.hook=crash
  request={'binding':before['binding'],'index':0,'reason':'initial_checkpoint','checkpoint_id':cp}
  with self.assertRaisesRegex(RuntimeError,'namespaced'):a.continue_session(request)
  fresh=self.adapter();self.assertEqual(fresh.recover_session_transition(),1)
  record=fresh.native('session_transition_receipt',{'index':0,'checkpoint_id':cp})
  self.assertLessEqual(len(record['transition']['instruction']['message_id'].encode()),256)
  with self.assertRaises(ValueError):fresh.native('session_transition_receipt',{'index':0,'checkpoint_id':'f'*64})
  self.assertEqual(fresh.continue_session(request),record['transition'])
  with self.assertRaisesRegex(ValueError,'replay request mismatch'):fresh.continue_session({**request,'binding':'f'*64})
  self.assertEqual(self.ledger.status()['calls'],0)
