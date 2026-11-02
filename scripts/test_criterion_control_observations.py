"""Authored evidence projection controls; never provider or live trial evidence."""
import copy,json,types,unittest
from unittest.mock import patch
from scripts.test_engine_criterion_return import h

class Controls(unittest.TestCase):
 def fixture(self):
  cases=[{'case_id':'controls','argv':['node','browser-harness.cjs','probe.json'],'exit':0,'expected_stdout':'browser-observation@v1','passed':True,'abbreviated':False,'stderr':'','stdout':json.dumps({'version':'browser-observation@v1','observations':[{'kind':'key','value':'ArrowUp'},{'kind':'text','text':'crossing open; growth 0','abbreviated':False}]})}]
  frame={'snapshot':'s','receipt':'r','observed_tests':{'receipt':'r','snapshot':'s'}}
  request={'coding':{'kind':'browser_project@v1','snapshot':'s','relational':{'inputs':{'returns':[{'receipt':'r','snapshot':'s','observations':cases}]}}}}
  return request,frame,cases
 def test_order_raw_partial_and_no_verdict(self):
  r,f,c=self.fixture();before=copy.deepcopy(r);v=h.control_observations(r,f)
  self.assertEqual(v['cases'],c);self.assertEqual(r,before);self.assertEqual(v['parent_evidence_id'],'test_receipt');self.assertEqual(v['availability'],'available');self.assertNotIn('criterion_verdict',v);self.assertIn('partial coverage remains unknown',v['limitations'])
  r['coding']['relational']['inputs']['returns'][0]['observations'][0]['abbreviated']=True
  self.assertTrue(h.control_observations(r,f)['cases'][0]['abbreviated'])
 def test_missing_foreign_stale_duplicate_and_attributed_substitution(self):
  for change in ('missing','foreign','stale','duplicate','extra','attributed'):
   r,f,c=self.fixture();rows=r['coding']['relational']['inputs']['returns']
   if change=='missing':rows.clear()
   elif change=='foreign':rows[0]['receipt']='foreign'
   elif change=='stale':rows[0]['snapshot']='old'
   elif change=='duplicate':rows.append(copy.deepcopy(rows[0]))
   elif change=='extra':rows[0]['observations'][0]['untrusted_extra']='invented'
   else:rows[0]['observations']=[{'evidence_class':'attributed report','claim':'passed'}]
   if change in ('missing','foreign'):
    v=h.control_observations(r,f);self.assertEqual(v['availability'],'unavailable');self.assertEqual(v['cases'],[])
   else:
    with self.assertRaises(ValueError):h.control_observations(r,f)
 def test_resume_rechecks_authenticated_projection_and_exact_bytes(self):
  r,f,c=self.fixture();capture={'contract_version':'criterion-evidence@v5','frame':f,'payload':{'authenticated_control_observations':h.control_observations(r,f)}};called=[]
  def project(view):called.append(view);return copy.deepcopy(r),{}
  a=types.SimpleNamespace(project=project,native=lambda *args:'authenticated view')
  h.verify_controls(a,capture);self.assertEqual(called,['authenticated view'])
  for change in ('bytes','order','missing','source'):
   original=copy.deepcopy(r)
   if change=='bytes':r['coding']['relational']['inputs']['returns'][0]['observations'][0]['stdout']+=' '
   elif change=='order':
    row=r['coding']['relational']['inputs']['returns'][0]['observations'][0];value=json.loads(row['stdout']);value['observations'].reverse();row['stdout']=json.dumps(value)
   elif change=='missing':r['coding']['relational']['inputs']['returns']=[]
   else:r['coding']['snapshot']='changed'
   with self.assertRaises(ValueError):h.verify_controls(a,capture)
   r.clear();r.update(original)
  a.project=lambda view:(_ for _ in ()).throw(ValueError('unowned browser receipt'))
  with self.assertRaisesRegex(ValueError,'unowned'):h.verify_controls(a,capture)
 def test_legacy_not_enriched_and_v5_instructions_preserve_unknown(self):
  a=types.SimpleNamespace(project=lambda v:self.fail('legacy capture must not be reprojected'))
  for version in ('v1','v2','v3','v4'):h.verify_controls(a,{'contract_version':'criterion-evidence@'+version})
  self.assertEqual(h.capture_system({'contract_version':'criterion-evidence@v4'}),h.CONSISTENT_SYSTEM)
  self.assertEqual(h.capture_system({'contract_version':'criterion-evidence@v5'}),h.CONTROL_SYSTEM)
  self.assertIn('Independent games are not a defect',h.CONTROL_SYSTEM)
  self.assertIn('not fulfillment',h.CONTROL_SYSTEM)
 def test_ordinary_capture_review_apply_interruptions_and_changed_trace(self):
  from scripts.test_engine_criterion_consistency import shape,correction
  for interruption in ('capture','review','apply','changed_during_review'):
   request,frame,cases=self.fixture();frame.update(requirements={'x':'Authored movement criterion'},publication='p',requirements_revision='rev',evidence={'app.js':'source','test_receipt':'authored aggregate'},coverage={'x':['app.js']},prerequisites={'x':{'kind':'static_semantic'}})
   request['coding']['files']=[{'path':'app.js','source':'source','editable':True}]
   context={'frame':frame,'current':None,'state_root':'root','binding':'binding','published_context':{'version':'criterion-published-context@v1','guidance':{'x':'Authored finite control condition'},'provenance':{'publication':'p','requirements_revision':'rev','current_source':'s','status':'current_publication','grants_authority':False,'review_carried':False}}}
   done={};calls=[];applies=[];interrupted=[]
   def op(name,fn):
    if name not in done:done[name]=fn()
    trigger=(interruption=='capture' and name.endswith('_input') and 'payload' in done[name]) or (interruption=='review' and name.endswith('_review')) or (interruption=='apply' and name.endswith('_apply'))
    if trigger and not interrupted:interrupted.append(name);raise RuntimeError('authored local interruption')
    return done[name]
   def stage(a,name,system,payload,**kw):
    def once():
     calls.append(copy.deepcopy(payload));self.assertEqual(payload['authenticated_control_observations']['cases'],cases)
     if interruption=='changed_during_review':request['coding']['relational']['inputs']['returns'][0]['observations'][0]['stdout']+=' changed'
     return {'request':'authored','value':{'verdict':'unknown','findings':{'x':{'finding':'unknown','basis':'unknown','evidence':['app.js'],'explanation':'Finite coverage','limitations':'No full reachability'}}}}
    return op(name,once)
   def apply(*args):applies.append(args);context['current']={'authored':'committed'};return 'authored-commit'
   a=types.SimpleNamespace(admission={'criterion_return_profile':{'version':'public-criterion-return@v1'}},controller=types.SimpleNamespace(cli=lambda *args:copy.deepcopy(context),json_cli=apply),native=lambda *args:{'state_root':'root','source_snapshot':'s','scope':'scope'},project=lambda v:(copy.deepcopy(request),{}),read=lambda:([],{},done),recovery_sha=correction.digest,recovery_load=lambda name,file:shape if file=='engine-project-output.py' else types.SimpleNamespace(_stage=stage),op=op,narrative_limit=131072)
   with patch.object(h,'verify',lambda *args:None):
    if interruption=='changed_during_review':
     with self.assertRaisesRegex(ValueError,'control evidence changed'):h.reflect(a)
     self.assertEqual(applies,[])
    else:
     with self.assertRaisesRegex(RuntimeError,'authored local interruption'):h.reflect(a)
     h.reflect(a);self.assertIsNone(h.reflect(a));self.assertEqual(len(applies),1);self.assertEqual(len(calls),1)
 def test_v5_single_reassessment_ticket_stop_resume_and_counted_replay(self):
  import tempfile
  from pathlib import Path
  from scripts.test_engine_criterion_reassessment import Harness,response,NAME,contract_tests,pursuit
  with tempfile.TemporaryDirectory() as folder:
   a=Harness(folder,[response(),response('unknown',('unknown','met'))]);payload=contract_tests.Consistency().payload()
   r,f,c=self.fixture();payload['authenticated_control_observations']=h.control_observations(r,f)
   capture={'contract_version':'criterion-evidence@v5','payload':payload,'request_source':a.source,'frame':{'snapshot':'s'}}
   a.op(NAME.removesuffix('_review')+'_input',lambda:capture);a.crash='local_intent'
   with self.assertRaises(RuntimeError):pursuit._stage(a,NAME,h.CONTROL_SYSTEM,payload,validator=h.validate_assessment)
   fresh=Harness(folder,[response(),response('unknown',('unknown','met'))]);stop=Path(folder)/'STOP';stop.touch()
   with self.assertRaisesRegex(ValueError,'stopped'):h.recover_reassessment_record(fresh)
   self.assertEqual(fresh.ledger.status()['calls'],1);stop.unlink() # authored fixture only
   self.assertEqual(h.recover_reassessment_record(fresh),1)
   result=pursuit._stage(fresh,NAME,h.CONTROL_SYSTEM,payload,validator=h.validate_assessment)
   h.verify_retained(fresh,NAME.removesuffix('_review'),capture,result)
   before=fresh.ledger.status();self.assertEqual(before['calls'],2);self.assertEqual(before['known_cost_microusd'],2000);self.assertEqual(fresh.execution_calls,0)
   self.assertEqual(pursuit._stage(fresh,NAME,h.CONTROL_SYSTEM,payload,validator=h.validate_assessment),result);self.assertEqual(fresh.ledger.status(),before)
 def test_supplemental_exact_scoped_raw_projection(self):
  request,frame,cases=self.fixture();key='supplemental:a:r'
  row={'admission':'a','receipt':'r','snapshot':'s','checked_snapshot':'checked','bundle_digest':'bundle','source_episode_root':'episode','mapped_cases':{'crossing':['controls'],'other':['private']},'browser_cases':cases+[dict(cases[0],case_id='private',stdout='outside requested criterion scope')]}
  request['coding']['relational']['inputs']['supplemental']=[row]
  value={k:row[k] for k in ('admission','receipt','snapshot','checked_snapshot','bundle_digest','source_episode_root')};value.update(mapped_cases={'crossing':['controls']},browser_cases=cases)
  frame.update(prerequisites={'crossing':{'kind':'observed_supplemental','admission':'a','receipt':'r'}},evidence={key:json.dumps(value)})
  self.assertEqual(h.validate_supplemental_evidence(request,frame),{key})
  for mutation in ('stdout','mapping','extra','order','missing','stale'):
   bad=copy.deepcopy(value)
   if mutation=='stdout':bad['browser_cases'][0]['stdout']+=' forged'
   elif mutation=='mapping':bad['mapped_cases']['crossing']=['private']
   elif mutation=='extra':bad['browser_cases'].append(row['browser_cases'][1])
   elif mutation=='order':
    parsed=json.loads(bad['browser_cases'][0]['stdout']);parsed['observations'].reverse();bad['browser_cases'][0]['stdout']=json.dumps(parsed)
   elif mutation=='missing':bad['browser_cases']=[]
   else:bad['snapshot']='old'
   with self.assertRaises(ValueError):h.validate_supplemental_evidence(request,{**frame,'evidence':{key:json.dumps(bad)}})
  hidden=copy.deepcopy(request);hidden['coding']['relational']['inputs']['supplemental']=[]
  with self.assertRaises(ValueError):h.validate_supplemental_evidence(hidden,frame)

 def test_cumulative_exact_order_completeness_adverse_and_unknown(self):
  r,f,c=self.fixture();rows=[];evidence={};bindings=[]
  for i in range(6):
   b={'admission':'a'+str(i),'receipt':'r'+str(i)};bindings.append(b)
   raw=copy.deepcopy(c);raw[0]['stdout']=json.dumps({'version':'browser-observation@v1','observations':[{'kind':'text','text':['reached','open','collision','unavailable','open','reached'][i]}]})
   if i==3:raw[0].update(exit=2,passed=False,stderr='unavailable')
   row={**b,'snapshot':'s','checked_snapshot':'check'+str(i),'bundle_digest':'bundle'+str(i),'source_episode_root':'episode'+str(i),'mapped_cases':{'crossing':['controls']},'browser_cases':raw};rows.append(row);evidence['supplemental:'+b['admission']+':'+b['receipt']]=json.dumps(row)
  r['coding']['relational']['inputs']['supplemental']=rows;f.update(prerequisites={'crossing':{'kind':'observed_supplemental_set','observations':bindings}},evidence=evidence)
  self.assertEqual(h.validate_supplemental_evidence(r,f),set(evidence))
  for mutation in ('missing','reorder','duplicate','foreign','stale','bytes','missing_payload','owned_missing'):
   q=copy.deepcopy(r);g=copy.deepcopy(f);items=g['prerequisites']['crossing']['observations']
   if mutation=='missing':items.pop(0)
   elif mutation=='reorder':items.reverse()
   elif mutation=='duplicate':items[-1]=copy.deepcopy(items[0])
   elif mutation=='foreign':items[0]['receipt']='forged'
   elif mutation=='stale':g['snapshot']='other'
   elif mutation=='bytes':
    k=next(iter(g['evidence']));v=json.loads(g['evidence'][k]);v['browser_cases'][0]['stdout']='concealed adverse result';g['evidence'][k]=json.dumps(v)
   elif mutation=='missing_payload':g['evidence'].pop(next(iter(g['evidence'])))
   else:q['coding']['relational']['inputs']['supplemental'].pop(2)
   with self.assertRaises((ValueError,KeyError)):h.validate_supplemental_evidence(q,g)
  self.assertIn('including adverse',h.CUMULATIVE_SYSTEM)
