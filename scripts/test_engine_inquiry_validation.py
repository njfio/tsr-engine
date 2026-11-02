"""Authored fake-provider controls through the real native inquiry path; no live evidence."""
import copy,importlib.util,json,tempfile,time,unittest
from pathlib import Path
from unittest.mock import patch
s=importlib.util.spec_from_file_location('iv',Path(__file__).with_name('engine-inquiry-validation.py'));d=importlib.util.module_from_spec(s);s.loader.exec_module(d)
x=d.x
class Ledger:
 def __init__(self,mode='complete'):self.calls=0;self.ops=0;self.mode=mode;self.bodies=[]
 def preflight_run(self,**kw):
  assert kw=={'calls':6,'execution_groups':1}
  return {'status':{'calls':193,'execution_groups':57,'known_cost_microusd':161719}}
 def record_price(self,q):pass
 def enroll(self,*a):pass
 def status(self):return {'fixture_only':True,'calls':self.calls,'operations':self.ops}
 def run_execution(self,a,b,c,worker):
  self.ops+=1
  if self.mode=='operation_unknown':raise RuntimeError('fixture uncertain read; no repeat')
  return worker()
 def run_call(self,trial,req,body,timeout,transport,**kw):
  self.calls+=1;self.bodies.append(body)
  payload=json.loads(body['messages'][1]['content']);ctx=payload['context'];stage=ctx['stage'];contract=payload['response_contract']
  if self.mode=='unknown':return {'value':None,'receipt':{'status':'unknown'}}
  if self.mode=='interrupted':raise KeyboardInterrupt('fixture crash after intent')
  if stage=='inquiry':
   value={'inquiry':None if self.mode=='abstain' else {'state_root':ctx['state_root'],'attention':ctx['inquiry_binding'],'parents':ctx['eligible_parents'],'question':'Authored question about the indexed accounts','uncertainty':'Bodies unknown; truth unverified','attribution':'model:gpt-6-luna:inquiry','origin':'model_proposed','answer_source':{'kind':'corpus_assertions','corpus':ctx['corpus'],'rationale':'Authored control concerns assertions only'}}}
  elif stage in ('inquiry_review','review'):
   pid=ctx['inquiry']['proposal'] if stage=='inquiry_review' else ctx['pending_reviews'][0]['proposal']
   value={'state_root':ctx['state_root'],'proposal':pid,'reviewer':'model:gpt-6-luna:inquiry-review' if stage=='inquiry_review' else 'gpt-6-luna/independent-context-review','decision':'unknown' if self.mode=='review_unknown' else 'admit','reason':'Authored review control, not semantic validation'}
  elif stage=='select':value={'inquiry':ctx['inquiry']['proposal'],'request':{'state_root':ctx['state_root'],'corpus':ctx['corpus'],'ids':['R3']},'relevance':'Authored relevance hypothesis only'}
  else:value={'basis':ctx['basis'],'generator':'authored-fixture-reflection','response':{'protocol':ctx['protocol'],'state_root':ctx['state_root'],'attention_binding':ctx['attention_binding'],'proposals':[] if stage=='next' else [{'kind':'new_candidate','description':'Authored next possibility retains uncertainty','serves':ctx['eligible_parents'],'rationale':'Retrieved fiction is not empirical truth'}]}}
  if self.mode=='expire_after_select' and stage=='select':self.expire()
  return {'value':value,'receipt':{'status':'settled','usd':'0'}}
class Validation(unittest.TestCase):
 def prepare(self,root):
  x.prepare(root);m=x.read(root/'inquiry-manifest.json');g={'plan':x.sha(m),'max_calls':6,'max_operations':1,'reservation_microusd':300000,'deadline':time.time()+3600,'scope':'frozen-fiction-native-inquiry@v1','runner_sha256':d.hashlib.sha256(Path(d.__file__).read_bytes()).hexdigest()};gp=root/'grant.json';x.save(gp,g);return gp
 def test_full_native_path_reviews_and_replay(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t)/'run';gp=self.prepare(root);l=Ledger()
   with patch.object(d,'ledger',return_value=l),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}):
    result=d.run(root,gp)
    self.assertEqual(result['reason'],'completed_next_preview_no_further_adoption',result)
    self.assertEqual((l.calls,l.ops),(6,1));self.assertFalse(d.run(root,gp)['dispatch'])
    self.assertEqual((l.calls,l.ops),(6,1))
   for i,b in enumerate(l.bodies):
    self.assertLessEqual(len(x.canonical(b)),32768)
    text=json.dumps(b)
    self.assertNotIn(str(root),text)
    for record in x.r.frozen()['records']:
     self.assertEqual(record['assertion'] in text,i>=3 and record['id']=='R3')
   q=x.native(root/'state','question-request')
   self.assertEqual(q,x.native(root/'state','question-request'))
   self.assertEqual(len(q['retrieval_observations']),1)
   self.assertEqual(len(q['reviewed_information_questions']),1)
   self.assertEqual(len(q['possibility_memory']),2)
   inquiry={'proposal':q['reviewed_information_questions'][0]['proposal'],'value':x.read(root/'inquiry.json')}
   pin=x.read(root/'receipt-pin.json');reviewed={p['revision']:x.sha(p) for p in q['possibility_memory']}
   for mutate in [lambda v:v['retrieval_observations'][0].update(inquiry='0'*64),lambda v:v['retrieval_questions'][0]['inquiry'].update(question='altered question'),lambda v:v['information_questions'][0].update(applicable=False)]:
    bad=copy.deepcopy(q);mutate(bad)
    with self.assertRaises(ValueError):x.project(bad,x.read(root/'origin-local-only.json'),expected=x.sha(bad),stage='next',inquiry=inquiry,receipt=pin,reviewed=reviewed)

 def test_stops_unknown_and_interrupt_never_repeat(self):
  for mode,calls,ops in [('abstain',1,0),('review_unknown',2,0),('unknown',1,0),('operation_unknown',3,1),('interrupted',1,0)]:
   with self.subTest(mode=mode),tempfile.TemporaryDirectory() as t:
    root=Path(t)/'run';gp=self.prepare(root);l=Ledger(mode)
    with patch.object(d,'ledger',return_value=l),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}):
     if mode=='interrupted':
      with self.assertRaises(KeyboardInterrupt):d.run(root,gp)
      with self.assertRaisesRegex(ValueError,'reconcile only'):d.run(root,gp)
     else:
      d.run(root,gp);self.assertFalse(d.run(root,gp)['dispatch'])
     self.assertEqual((l.calls,l.ops),(calls,ops))
 def test_deadline_and_preflight_drift_stop_before_effects(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t)/'run';gp=self.prepare(root);l=Ledger('expire_after_select');clock=[time.time()]
   l.expire=lambda:clock.__setitem__(0,x.read(gp)['deadline']+1)
   with patch.object(d,'ledger',return_value=l),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}),patch.object(d.time,'time',side_effect=lambda:clock[0]):
    result=d.run(root,gp)
    self.assertIn('inspection deadline',result['reason']);self.assertEqual((l.calls,l.ops),(3,0))
    self.assertFalse(d.run(root,gp)['dispatch'])
  with tempfile.TemporaryDirectory() as t:
   root=Path(t)/'run';gp=self.prepare(root);l=Ledger();clock=[time.time()];original=x.body;assembled=[0]
   def slow_body(*args):
    b=original(*args);assembled[0]+=1
    if assembled[0]==2:clock[0]=x.read(gp)['deadline']+1
    return b
   with patch.object(d,'ledger',return_value=l),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}),patch.object(d.time,'time',side_effect=lambda:clock[0]),patch.object(x,'body',side_effect=slow_body):
    result=d.run(root,gp);self.assertIn('dispatch deadline',result['reason']);self.assertEqual((l.calls,l.ops),(0,0))
  with tempfile.TemporaryDirectory() as t:
   root=Path(t)/'run';gp=self.prepare(root)
   (root/'inquiry-body.json').write_text('{}')
   with patch.object(d,'ledger') as ledger:
    with self.assertRaisesRegex(ValueError,'payload drift'):d.run(root,gp)
    ledger.assert_not_called()
 def test_projection_private_fields_stale_and_authority(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t)/'run';self.prepare(root);o=x.read(root/'origin-local-only.json');q=x.native(root/'state','question-request')
   altered=copy.deepcopy(q);altered['history']=[{'secret':'PRIVATE_SENTINEL'}];altered['private']='PRIVATE_SENTINEL'
   ctx=x.project(altered,o,expected=x.sha(altered),stage='inquiry');self.assertNotIn('PRIVATE_SENTINEL',json.dumps(x.body(ctx,'inquiry')))
   with self.assertRaisesRegex(ValueError,'stale'):x.project(altered,o,expected=x.sha(q),stage='inquiry')
   for mutate in [lambda v:v['authority'].update(executions=1),lambda v:v['native_attention'].update(selected=[]),lambda v:v.update(visible_records=[{'private':'data'}])]:
    v=copy.deepcopy(q);mutate(v)
    with self.assertRaises(ValueError):x.project(v,o,expected=x.sha(v),stage='inquiry')
if __name__=='__main__':unittest.main()
