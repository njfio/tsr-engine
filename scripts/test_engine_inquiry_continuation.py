"""Authored continuation wiring, fake transport and temporary fixture reads only."""
import copy,json,tempfile,time,unittest,hashlib,subprocess
from pathlib import Path
from unittest.mock import patch
import test_engine_exploration_validation as f
import test_engine_synthetic_transport as wire
d,x=f.d,f.x
class Continuation(unittest.TestCase):
 def parent(self,root):
  gp=f.Validation().prepare(root);l=f.Ledger()
  with patch.object(d,'ledger',return_value=l),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}):
   self.assertEqual(d.run(root,gp)['reason'],'completed_next_preview_no_further_adoption')
  return x.native(root/'state','question-request')
 def prepare(self,out,parent):
  result=x.continuation.prepare(x,out,parent,{'calls':197,'groups':57,'settled_microusd':163331,'caps':[220,60,5000000]})
  grant=x.read(out/'grant-template.json');grant['deadline']=time.time()+3600;x.save(out/'grant.json',grant)
  return out/'grant.json'
 def test_two_cycles_same_runner_real_wire_native_and_replay(self):
  with tempfile.TemporaryDirectory() as tmp:
   parent=Path(tmp)/'parent';q0=self.parent(parent);snapshot=x.continuation.tree_hash(parent/'state');out=Path(tmp)/'next';gp=self.prepare(out,parent)
   self_test=self
   class Next(f.Ledger):
    def preflight_run(self,**kw):
     assert kw=={'calls':8,'execution_groups':1}
     return {'status':{'calls':197,'execution_groups':57,'known_cost_microusd':163331}}
    def run_call(self,*args,**kw):
     result=super().run_call(*args,**kw);value=result['value'];body=args[2];ctx=json.loads(body['messages'][1]['content'])['context'];stage=ctx['stage']
     if stage=='inquiry':value['inquiry']['question']='Authored follow-up: what remains unknown after prior access?'
     if stage=='select':
      value['request']['ids']=[next(v['id'] for v in ctx['index'] if v['id'] not in {r['id'] for r in ctx['retrieved']})]
      with self_test.assertRaises(subprocess.CalledProcessError):x.model_contract(out/'state','select')
      with self_test.assertRaises(subprocess.CalledProcessError):x.model_contract(out/'state','select',inquiry_focus='0'*64)
      bad=copy.deepcopy(value);bad['inquiry']=q0['reviewed_information_questions'][0]['proposal']
      with self_test.assertRaises(subprocess.CalledProcessError):x.contract_command(out/'state','check-model-response','select',bad,inquiry_focus=ctx['inquiry']['proposal'])
     if stage=='reflect':value['response']['proposals'][0]['description']='Authored continuation possibility retains old and new receipt limits'
     envelope=wire.Transport().envelope();envelope['model']='gpt-6-luna';envelope['choices'][0]['message']['content']=json.dumps(value)
     with patch.object(d.t.broker,'credential',return_value='FIXTURE_CREDENTIAL'),patch.object(d.t.urllib.request,'build_opener') as op:
      op.return_value.open.return_value=wire.Response(json.dumps(envelope).encode());r=args[4](body,1)
      assert json.loads(op.return_value.open.call_args.args[0].data)==body
      return r
   l=Next()
   with patch.object(d,'ledger',return_value=l),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}):
    result=d.run(out,gp);self.assertEqual(result['reason'],'completed_next_preview_no_further_adoption',result)
    self.assertEqual((l.calls,l.ops),(6,1));self.assertFalse(d.run(out,gp)['dispatch'])
   q=x.native(out/'state','question-request')
   self.assertEqual(len(q['retrieval_observations']),2);self.assertEqual(len(q['information_questions']),2)
   self.assertEqual(q['parents'],q0['parents']);self.assertEqual(q['authority'],q0['authority'])
   self.assertEqual(snapshot,x.continuation.tree_hash(parent/'state'));self.assertEqual(q0,x.native(parent/'state','question-request'))
   first=json.loads(l.bodies[0]['messages'][1]['content'])['context']
   self.assertEqual(len(first['possibility_memory']),2);self.assertEqual(len(first['receipt_history']),1)
   self.assertEqual(first['basis']['kind'],'retrieval_observation');self.assertNotIn(str(parent),json.dumps(l.bodies))
   last=json.loads(l.bodies[-1]['messages'][1]['content'])['context'];self.assertEqual(len(last['receipt_history']),2)
   self.assertEqual(last['receipt_facts']['receipt'],q['basis']['receipt'])
   reordered=copy.deepcopy(q);reordered['retrieval_observations'].reverse()
   origin=x.read(out/'exploration-origin-local-only.json');inquiry={'proposal':x.read(out/'retrieval-request.json')['inquiry'],'value':x.read(out/'inquiry.json')}
   pins={item['revision']:x.sha(item) for item in q['possibility_memory']}
   projected=x.project(reordered,origin,expected=x.sha(reordered),stage='next',inquiry=inquiry,receipt=x.read(out/'receipt-pin.json'),reviewed=pins)
   self.assertEqual(projected['receipt_facts']['receipt'],q['basis']['receipt'])
   self.assertEqual(len(projected['receipt_history']),2)

 def test_scope_stale_receipts_lineage_and_no_export_of_unknown_fields(self):
  with tempfile.TemporaryDirectory() as tmp:
   parent=Path(tmp)/'parent';self.parent(parent);out=Path(tmp)/'next';self.prepare(out,parent)
   q=x.native(out/'state','question-request');o=x.read(out/'exploration-origin-local-only.json')
   project=lambda z:x.project(z,o,expected=x.sha(z),stage='inquiry')
   base=project(q);self.assertEqual(base,project(q))
   for mutate in [lambda z:z['parents'][0].update(active=False),lambda z:z['retrieval_observations'][0]['records'][0]['bytes'].append(1),lambda z:z['possibility_memory'][0]['candidate'].update(description='forged'),lambda z:z['authority'].update(executions=1),lambda z:z['retrieval_observations'][0].update(request_digest='0'*64),lambda z:z.update(state_root='0'*64)]:
    bad=copy.deepcopy(q);mutate(bad)
    with self.assertRaises(ValueError):project(bad)
   extra=copy.deepcopy(q);extra['private_unrelated']='DO_NOT_EXPORT'
   with self.assertRaises(ValueError):project(extra) # complete pinned initial request changed
   x.continuation.validate_source(x,o)
   terminal=x.read(parent/'terminal.json');terminal['reason']='tampered';(parent/'terminal.json').write_text(json.dumps(terminal))
   with self.assertRaisesRegex(ValueError,'predecessor drift'):x.continuation.validate_source(x,o)
 def test_unknown_or_interrupted_continuation_never_replays(self):
  for mode in ('unknown','interrupted'):
   with tempfile.TemporaryDirectory() as tmp:
    parent=Path(tmp)/'parent';self.parent(parent);out=Path(tmp)/'next';gp=self.prepare(out,parent)
    class Unknown(f.Ledger):
     def preflight_run(self,**kw):return {'status':{'calls':197,'execution_groups':57,'known_cost_microusd':163331}}
    l=Unknown(mode)
    with patch.object(d,'ledger',return_value=l),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}):
     if mode=='interrupted':
      with self.assertRaises(KeyboardInterrupt):d.run(out,gp)
      with self.assertRaisesRegex(ValueError,'no redispatch'):d.run(out,gp)
     else:
      self.assertIn('stopped:',d.run(out,gp)['reason']);self.assertFalse(d.run(out,gp)['dispatch'])
     self.assertEqual((l.calls,l.ops),(1,0))
 def test_source_symlink_and_derivative_destination_rejected(self):
  with tempfile.TemporaryDirectory() as tmp:
   parent=Path(tmp)/'parent';self.parent(parent)
   with self.assertRaisesRegex(ValueError,'separate'):self.prepare(parent/'nested',parent)
   (parent/'state'/'escape').symlink_to('/tmp')
   with self.assertRaisesRegex(ValueError,'symlink'):self.prepare(Path(tmp)/'next',parent)
if __name__=='__main__':unittest.main()
