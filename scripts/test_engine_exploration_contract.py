"""Authored contract instances, real native validators; no model behavior evidence."""
import copy,json,subprocess,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import test_engine_exploration_validation as fixture
Ledger,d,x=fixture.Ledger,fixture.d,fixture.x

class Contract(unittest.TestCase):
 def test_captured_live4_rejections_precise_and_serialized_without_mutation(self):
  import hashlib
  records=json.loads((Path(__file__).with_name('testdata')/'qualitative-live4-rejections.json').read_text())['records']
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp)/'run';fixture.Validation().prepare(root)
   # reflect profile can be queried on current initial state; reject envelope before semantic basis.
   for number,record in enumerate(records):
    self.assertEqual(hashlib.sha256(record['content'].encode()).hexdigest(),record['sha256'])
    value=json.loads(record['content']);before=copy.deepcopy(value)
    with self.assertRaises(subprocess.CalledProcessError) as error:
     x.contract_command(root/'state','check-model-response','reflect',value)
    detail=d.correction.native_feedback(error.exception.stderr)
    self.assertEqual(detail['path'],'$.protocol')
    self.assertEqual(detail['valid_locations'],['$.response.protocol'])
    self.assertEqual(detail['required_fields'],['basis','generator','response'])
    budget=d.correction.Budget(Path(tmp)/('journal'+str(number)),{'fixture':number},1)
    feedback=budget.feedback('reflect','unchanged',{'receipt':{'status':'settled'},'value':value},str(number),error.exception.stderr)
    original={'messages':[{'role':'system','content':'unchanged task'},{'role':'user','content':'unchanged evidence'}]}
    serialized=json.loads(json.dumps(budget.body(original,feedback)))
    self.assertEqual(serialized['messages'][:-1],original['messages'])
    self.assertEqual(json.loads(serialized['messages'][-1]['content'])['protocol_correction']['native_structure'],detail)
    self.assertEqual(value,before)
 def test_all_six_assembled_contracts_native_boundaries_and_full_flow(self):
  owner=self;seen=[];checks=[];sizes=[]
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp)/'run';gp=fixture.Validation().prepare(root)
   class Audited(Ledger):
    def run_call(self,*args,**kw):
     result=super().run_call(*args,**kw);body=args[2];payload=json.loads(body['messages'][1]['content']);ctx=payload['context'];stage=ctx['stage'];value=result['value']
     seen.append(stage);sizes.append(len(x.canonical(body)))
     owner.assertEqual(payload['response_contract'],x.model_contract(root/'state',stage))
     bad_contract=copy.deepcopy(payload['response_contract']);bad_contract['state_root']='0'*64
     with owner.assertRaisesRegex(ValueError,'contract stage/state/version'):x.body(ctx,stage,bad_contract)
     owner.assertEqual(body['response_format'],{'type':'json_object'}) # advertised guidance, not proven server enforcement
     before=x.native(root/'state','question-request')
     def check(v,valid=True):
      checks.append((stage,valid))
      if valid:owner.assertTrue(x.contract_command(root/'state','check-model-response',stage,v)['valid'])
      else:
       with owner.assertRaises(subprocess.CalledProcessError):x.contract_command(root/'state','check-model-response',stage,v)
     def bad(path,replacement):
      v=copy.deepcopy(value);p=v
      for k in path[:-1]:p=p[k]
      p[path[-1]]=replacement;check(v,False)
     check(value)
     check({**value,'unexpected':True},False)
     with owner.assertRaises(subprocess.CalledProcessError) as diagnostic:
      x.contract_command(root/'state','check-model-response',stage,{**value,'stage':stage})
     detail=d.correction.native_feedback(diagnostic.exception.stderr)
     owner.assertEqual(detail['path'],'$.stage');owner.assertEqual(detail['valid_locations'],[])
     owner.assertEqual(detail['allowed_fields'],list(payload['response_contract']['schema']['fields']))
     secret='private-key-material-not-for-feedback'
     with owner.assertRaises(subprocess.CalledProcessError) as diagnostic:
      x.contract_command(root/'state','check-model-response',stage,{**value,secret:'hidden'})
     owner.assertNotIn(secret,diagnostic.exception.stderr)
     owner.assertTrue(d.correction.native_feedback(diagnostic.exception.stderr)['field_name_withheld'])
     if stage=='inquiry':
      check({'inquiry':None})
      bad(['inquiry','origin'],'user_supplied');bad(['inquiry','origin'],'invented')
      bad(['inquiry','parents'],ctx['eligible_parents'][0]);bad(['inquiry','parents'],[])
      bad(['inquiry','parents'],['0'*64]);bad(['inquiry','state_root'],'0'*64)
      bad(['inquiry','attention'],'0'*64);bad(['inquiry','attribution'],'operator')
      bad(['inquiry','question'],'é'*257);bad(['inquiry','question'],' ')
      bad(['inquiry','exploration','considered_records'],'R1')
      bad(['inquiry','exploration','considered_records'],None)
      bad(['inquiry','exploration','considered_records'],['R1','R1'])
      bad(['inquiry','exploration','considered_records'],['outside'])
      bad(['inquiry','exploration','policy'],'0'*64)
      bad(['inquiry','exploration','possible_information_benefit'],'x'*513)
      bad(['inquiry','exploration','why_no_user_priority_needed'],None)
      bad(['inquiry','answer_source'],{'kind':'user_preference'})
      v=copy.deepcopy(value);v['inquiry']['question']='é'*256;check(v)
      v=copy.deepcopy(value);v['inquiry']['origin']='model_proposed';v['inquiry'].pop('exploration')
      for source in [None,{'kind':'user_preference'},{'kind':'user_authority'},{'kind':'external_evidence'},{'kind':'mixed'},{'kind':'unknown'},v['inquiry']['answer_source']]:
       v['inquiry']['answer_source']=source;check(v)
      v['inquiry'].pop('answer_source');check(v)
      v['inquiry']['exploration']=None;check(v)
      v['inquiry']['answer_source']={'kind':'unknown','rationale':'unsupported extra'};check(v,False)
     elif stage in ('inquiry_review','review'):
      for decision in ('admit','reject','unknown'):check({**value,'decision':decision})
      for decision in ('approve',None,[],True):bad(['decision'],decision)
      bad(['proposal'],'0'*64);bad(['state_root'],'0'*64);bad(['reviewer'],'generator')
      bad(['reason'],'é'*257);bad(['reason'],[])
     elif stage=='select':
      bad(['request','ids'],'R3');bad(['request','ids'],None)
      bad(['request','ids'],['R1','R3']);bad(['request','ids'],['R2'])
      bad(['request','corpus'],'0'*64);bad(['inquiry'],'0'*64)
      v=copy.deepcopy(value);v['request']['ids']=[];check(v)
      bad(['relevance'],' ')
     else:
      bad(['basis'],{'kind':'initial_context','context':'0'*64})
      bad(['generator'],'operator');bad(['response','state_root'],'0'*64)
      bad(['response','attention_binding'],None);bad(['response','proposals'],{})
      bad(['response','common_event_forecasts'],None)
      bad(['response','common_event_forecasts'],{'passing':1})
      v=copy.deepcopy(value);v['response']['common_event_forecasts']={};check(v)
      if stage=='reflect':
       bad(['response','proposals',0,'serves'],ctx['eligible_parents'][0])
       bad(['response','proposals',0,'description'],'x'*513)
       for change in ('clarification','development'):
        proposal={'binding':ctx['attention_binding'],'identity':ctx['eligible_parents'][0],'parents':ctx['eligible_parents'],'meaning':'Authored alternative meaning; not evidence of emergence','change':change,'basis':{'kind':'unknown'},'generator':value['generator']}
        v=copy.deepcopy(value);v['response']['proposals']=[{'kind':'public_desire','proposal':proposal}];check(v)
        proposal['revises']=None;check(v)
        proposal['basis']={'kind':'observation','observation':'0'*64};check(v,False)
       relation={'parents':ctx['eligible_parents'],'relationship':'Authored relationship','aspiration':'Authored retained wanted condition','discriminator':'Authored unresolved uncertainty'}
       v=copy.deepcopy(value);v['response']['proposals']=[{'kind':'relational','binding':ctx['attention_binding'],'proposal':relation}];check(v)
       relation.update(revises=None,catalysts=[],development=None);check(v)
       relation['catalysts']=None;check(v,False);relation['catalysts']=[]
       relation['development']={};check(v,False);relation['development']=None
       relation['revises']='0'*64;check(v,False)
     owner.assertEqual(before,x.native(root/'state','question-request'))
     return result
   l=Audited()
   with patch.object(d,'ledger',return_value=l),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}):
    result=d.run(root,gp)
    self.assertEqual(result['reason'],'completed_next_preview_no_further_adoption',result)
    self.assertFalse(d.run(root,gp)['dispatch'])
   self.assertEqual(seen,list(x.STAGES));self.assertEqual((l.calls,l.ops),(6,1))
   self.assertGreater(len(checks),70);self.assertLessEqual(max(sizes),32768)

 def test_revision_ancestry_is_required_not_null_or_omitted(self):
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp)/'run';fixture.Validation().prepare(root);state=root/'state'
   q=x.native(state,'question-request');identity=q['parents'][0]['id']
   proposal={'binding':q['attention_binding'],'identity':identity,'revises':None,'parents':[p['id'] for p in q['parents']],'meaning':'Authored changed wanting for lineage control only','change':'development','basis':{'kind':'unknown'},'generator':'gpt-6-luna/qualitative-generator'}
   value={'basis':q['basis'],'generator':proposal['generator'],'response':{'protocol':q['protocol'],'state_root':q['state_root'],'attention_binding':q['attention_binding'],'proposals':[{'kind':'public_desire','proposal':proposal}]}}
   x.contract_command(state,'check-model-response','reflect',value)
   file=root/'authored.json';x.save(file,value);pid=x.native(state,'submit',file)
   q=x.native(state,'question-request');review={'state_root':q['state_root'],'proposal':pid,'reviewer':'gpt-6-luna/independent-context-review','decision':'admit','reason':'Authored wiring control, not generated semantic evidence'}
   file=root/'authored-review.json';x.save(file,review);x.native(state,'review',file);q=x.native(state,'question-request')
   current=next(p for p in q['parents'] if p['id']==identity)
   value['basis']=q['basis'];value['response'].update(state_root=q['state_root'],attention_binding=q['attention_binding'])
   proposal.update(binding=q['attention_binding'],revises=current['revision'],meaning='Authored second lineage control')
   self.assertTrue(x.contract_command(state,'check-model-response','reflect',value)['valid'])
   for replacement in (None,'0'*64):
    proposal['revises']=replacement
    with self.assertRaises(subprocess.CalledProcessError):x.contract_command(state,'check-model-response','reflect',value)
   proposal.pop('revises')
   with self.assertRaises(subprocess.CalledProcessError):x.contract_command(state,'check-model-response','reflect',value)
   proposal['revises']=current['revision'];proposal['parents']=[identity]
   with self.assertRaises(subprocess.CalledProcessError):x.contract_command(state,'check-model-response','reflect',value)

 def test_known_scalar_failure_is_retained_stops_without_coercion(self):
  class Invalid(Ledger):
   def run_call(self,*a,**kw):
    result=super().run_call(*a,**kw)
    result['value']['inquiry']['exploration']['considered_records']='R1'
    return result
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp)/'run';gp=fixture.Validation().prepare(root);l=Invalid();before=x.native(root/'state','question-request')
   with patch.object(d,'ledger',return_value=l),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}):
    result=d.run(root,gp);self.assertTrue(result['reason'].startswith('stopped:'));self.assertFalse(d.run(root,gp)['dispatch'])
   self.assertIn('considered_records',x.read(root/'contract-rejection-1.json')['diagnostic'])
   self.assertEqual((l.calls,l.ops),(1,0));self.assertEqual(before,x.native(root/'state','question-request'))
   self.assertEqual(x.read(root/'result-1.json')['value']['inquiry']['exploration']['considered_records'],'R1')

if __name__=='__main__':unittest.main()
