import copy,importlib.util,json,tempfile,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('rx',Path(__file__).with_name('engine-retrieval-export.py'));x=importlib.util.module_from_spec(s);s.loader.exec_module(x)
class Export(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.d=Path(self.tmp.name)/'p';x.prepare(self.d);self.o=x.read(self.d/'origin-local-only.json');self.q=x.native(self.d/'state','request-v2')
 def tearDown(self):self.tmp.cleanup()
 def file(self,n,v):p=self.d/n;x.save(p,v);return p
 def retrieve(self):
  req={'state_root':self.q['state_root'],'corpus':self.o['corpus'],'ids':['R2','R5']}
  rid=x.native(self.d/'state','retrieve',self.file('retrieve.json',req));q=x.native(self.d/'state','request-v2');obs=q['retrieval_observations'][0]
  pin={'request':req,'pre_root':req['state_root'],'native_receipt':rid,'native_request_digest':obs['request_digest'],'observation_sha256':x.sha(obs)}
  return q,pin
 def test_index_only_and_no_private_fields(self):
  ctx=x.project(self.q,self.o,expected=x.sha(self.q),stage='select');text=json.dumps(x.body(ctx,'select'))
  for r in x.frozen()['records']:self.assertNotIn(r['assertion'],text)
  self.assertNotIn(str(self.d),text)
  q=copy.deepcopy(self.q);q['private']='SECRET';q['history']=[{'private':'SECRET'}];q['instructions']='SECRET'
  with self.assertRaises(ValueError):x.project(q,self.o,expected=x.sha(self.q),stage='select')
  self.assertEqual(x.native(self.d/'state','request-v2'),self.q)
 def test_receipt_bytes_and_wrong_identity(self):
  q,pin=self.retrieve();ctx=x.project(q,self.o,expected=x.sha(q),stage='reflect',receipt=pin);text=json.dumps(x.body(ctx,'reflect'))
  extra=copy.deepcopy(q);extra['private']='SECRET';extra['history']=[{'private':'SECRET'}];extra['instructions']='SECRET'
  self.assertNotIn('SECRET',json.dumps(x.project(extra,self.o,expected=x.sha(extra),stage='reflect',receipt=pin)))
  for r in x.frozen()['records']:
   self.assertEqual(r['assertion'] in text,r['id'] in ('R2','R5'))
  for mutate in (lambda p:p.update(native_receipt='0'*64),lambda p:p['request'].update(corpus='0'*64),lambda p:p.update(pre_root='0'*64)):
   p=copy.deepcopy(pin);mutate(p)
   with self.assertRaises(ValueError):x.project(q,self.o,expected=x.sha(q),stage='reflect',receipt=p)
  bad=copy.deepcopy(q);bad['retrieval_observations'][0]['records'][0]['bytes'][0]=33
  with self.assertRaises(ValueError):x.project(bad,self.o,expected=x.sha(bad),stage='reflect',receipt=pin)
  with self.assertRaises(Exception):x.native(self.d/'state','retrieve',self.d/'retrieve.json')
 def test_real_native_reviewed_memory_and_next_projection(self):
  q,pin=self.retrieve();sub={'basis':q['basis'],'generator':'authored-test','response':{'protocol':q['protocol'],'state_root':q['state_root'],'attention_binding':q['attention_binding'],'proposals':[{'kind':'new_candidate','description':'Authored possible comparison, not emergence','serves':[q['parents'][0]['id']],'rationale':'Source assertions remain unverified'}]}}
  pid=x.native(self.d/'state','submit-possibility',self.file('sub.json',sub));pq=x.native(self.d/'state','request-v2');pending={pid:x.sha(pq['pending_reviews'][0])}
  ctx=x.project(pq,self.o,expected=x.sha(pq),stage='review',receipt=pin,pending=pending);self.assertLess(len(json.dumps(x.body(ctx,'review')).encode()),32768)
  rv={'state_root':pq['state_root'],'proposal':pid,'reviewer':'independent-authored-test','decision':'admit','reason':'PRIVATE_REVIEW_SENTINEL authored wiring only'}
  x.native(self.d/'state','review',self.file('review.json',rv));aq=x.native(self.d/'state','request-v2');item=aq['possibility_memory'][0];reviewed={item['revision']:x.sha(item)}
  ctx=x.project(aq,self.o,expected=x.sha(aq),stage='next',receipt=pin,reviewed=reviewed)
  self.assertNotIn('PRIVATE_REVIEW_SENTINEL',json.dumps(x.body(ctx,'next')));self.assertEqual(aq['parents'],self.q['parents'])
  self.assertEqual(aq,x.native(self.d/'state','request-v2'))
  with self.assertRaises(ValueError):x.project(aq,self.o,expected=x.sha(aq),stage='next',receipt=pin)
 def test_authority_scope_and_stale_stop(self):
  for change in (lambda q:q['authority'].update(executions=1),lambda q:q['parents'][0].update(active=False),lambda q:q['visible_records'].append({'private':True}),lambda q:q.update(phase='retrieval_context_stale')):
   q=copy.deepcopy(self.q);change(q)
   with self.assertRaises(ValueError):x.project(q,self.o,expected=x.sha(q),stage='select')
if __name__=='__main__':unittest.main()

class Driver(unittest.TestCase):
 def test_four_stage_fixture_driver_replay_and_accounting(self):
  from unittest.mock import patch
  import time
  s=importlib.util.spec_from_file_location('driver',Path(__file__).with_name('engine-retrieval-validation.py'));d=importlib.util.module_from_spec(s);s.loader.exec_module(d)
  class Ledger:
   def __init__(self):self.calls=0;self.ops=0
   def preflight_run(self,**kw):
    self.expected=kw;return {'status':{'calls':192,'execution_groups':57,'known_cost_microusd':161431}}
   def record_price(self,q):pass
   def enroll(self,*a):pass
   def run_call(self,trial,req,body,timeout,transport,**kw):
    self.calls+=1;ctx=json.loads(body['messages'][1]['content'])['context'];stage=ctx['stage']
    if stage=='select':v={'state_root':ctx['state_root'],'corpus':ctx['corpus'],'ids':['R3'],'reason':'Authored selection fixture'}
    elif stage=='review':v={'state_root':ctx['state_root'],'proposal':ctx['pending_reviews'][0]['proposal'],'reviewer':'independent-authored-fixture','decision':'admit','reason':'Wiring only, not semantic endorsement'}
    else:
     props=[] if stage=='next' else [{'kind':'new_candidate','description':'Authored possibility; no real outcome','serves':[ctx['parents'][0]['id']],'rationale':'The fictional assertion remains unverified'}]
     v={'basis':ctx['basis'],'generator':'authored-fixture','response':{'protocol':ctx['protocol'],'state_root':ctx['state_root'],'attention_binding':ctx['attention_binding'],'proposals':props}}
    return {'value':v,'receipt':{'status':'settled','usd':'0'}}
   def run_execution(self,a,b,c,worker):self.ops+=1;return worker()
   def status(self):return {'fixture_only':True,'calls':self.calls,'operations':self.ops}
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp)/'run';x.prepare(root);plan=x.read(root/'manifest.json');grant={'plan':x.sha(plan),'max_calls':4,'max_operations':1,'reservation_microusd':200000,'deadline':time.time()+3600,'scope':'frozen-fiction-retrieval@v1','runner_sha256':d.hashlib.sha256(Path(d.__file__).read_bytes()).hexdigest()};gp=Path(tmp)/'grant.json';x.save(gp,grant);l=Ledger()
   with patch.object(d,'ledger',return_value=l),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}):
    r=d.run(root,gp);self.assertEqual(r['reason'],'completed_next_preview_no_further_adoption');self.assertEqual((l.calls,l.ops),(4,1));self.assertEqual(l.expected,{'calls':4,'execution_groups':1})
    replay=d.run(root,gp);self.assertFalse(replay['dispatch']);self.assertEqual((l.calls,l.ops),(4,1))
