"""Authored proposal-reference representation controls; no model or tool effects."""
import copy,importlib.util,json,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('proposal_references',Path(__file__).with_name('engine-proposal-references.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class ProposalReferences(unittest.TestCase):
 def setUp(self):
  self.parents=[{'id':'1'*64,'requirement':'orientation','meaning':'Keep p0 and r0 literal prose unchanged'},{'id':'2'*64,'requirement':'discovery','meaning':'Explore unfamiliar perspectives'}]
  original={'parents':['1'*64,'2'*64],'relationship':'r0 is literal narrative','aspiration':'A shared pursuit','discriminator':'A source-bound hypothesis','revises':None,'catalysts':[]}
  revised={**copy.deepcopy(original),'revises':'3'*64,'development':{'original_revision':'3'*64,'prior_revision':'3'*64,'retained_meaning':'Both parents','return_reveals':'Authored evidence only','unresolved':'Unknown benefit','proposed_development':'Hypothesis'}}
  self.request={'state_root':'a'*64,'coding':{'snapshot':'b'*64,'cases':[],'files':[{'path':'app.js','source_digest':'c'*64,'editable':True,'source':'const p0 = "r0";'}],
   'relational':{'binding':'d'*64,'inputs':{'snapshot':'b'*64,'parents':self.parents,'returns':[]},'history':[
    {'admission':'3'*64,'binding':'e'*64,'proposal':original,'current':False,'superseded':True,'withdrawn':False},
    {'admission':'4'*64,'binding':'f'*64,'proposal':revised,'current':False,'superseded':False,'withdrawn':False}]}}}
  e=m.native
  self.context={'parents':copy.deepcopy(self.parents),'relational_contract':e.relational_contract(self.request),'relational_history':copy.deepcopy(self.request['coding']['relational']['history']),
   'return_evidence':e.return_evidence.catalog(self.request),'response_contracts':{'relational':e.payload_contract(self.request,'final_proposal_only')['relational'],
    'return_basis':{'semantics':e.return_evidence.contract(),'schema':e.return_evidence.schema(self.request)}},
   'files':[{'path':'app.js','source':'const p0 = "r0";'}], 'advisory_reconsideration':{'lineage':'4'*64}}
 def test_expectation_contract_uses_exact_aliases_and_never_repairs_full_ids(self):
  spec=importlib.util.spec_from_file_location('browser_contract',Path(__file__).with_name('engine-browser-project.py'));web=importlib.util.module_from_spec(spec);spec.loader.exec_module(web)
  self.context['pursuit_expectation_contract']='expectation:{parents:[contributing IDs]}'
  self.context['response_contracts']['edits_or_abstention']=web.response_schema(['app.js'])
  before=copy.deepcopy(self.context);wire=self.projected()
  self.assertEqual(self.context,before)
  self.assertNotIn('contributing IDs',wire['pursuit_expectation_contract'])
  schema=wire['response_contracts']['edits_or_abstention']['oneOf'][1]['properties']['expectation']['properties']['parents']['items']
  self.assertEqual(schema,{'type':'string','enum':['p0','p1']})
  value={'alternatives':[{'expectation':{'parents':['p0','p1'],'anticipated':'Authored','grounds':'Authored','uncertainty':'Unknown'}}]}
  self.assertEqual(m.expand(value,wire['proposal_references'],self.request)['alternatives'][0]['expectation']['parents'],['1'*64,'2'*64])
  for parents in (['1'*64,'2'*64],['p8'],['f'*64]):
   bad=copy.deepcopy(value);bad['alternatives'][0]['expectation']['parents']=parents
   original=copy.deepcopy(bad)
   for _ in range(2):
    with self.assertRaisesRegex(ValueError,'unknown or stale expectation'):m.expand(bad,wire['proposal_references'],self.request)
   self.assertEqual(bad,original)
  stale=copy.deepcopy(self.request);stale['state_root']='9'*64
  with self.assertRaisesRegex(ValueError,'stale or substituted'):m.expand(value,wire['proposal_references'],stale)
 def test_review_provenance_authenticates_wire_native_mapping_and_restart(self):
  self.request['estimates']=[{'action':'authored','evidence':['receipt'],'decision':'retain','serves':['orientation'],'resource_cost':{'sunk_tokens_charged':1000,'source_prefix':'unchanged'}}]
  raw={'alternatives':[{'expectation':{'parents':['p0','p1'],'anticipated':'Authored','grounds':'Source prediction','uncertainty':'Unknown'}}]}
  wire={'messages':[{'content':'Authored fixture'},{'content':json.dumps(self.projected())}]}
  expanded=m.expand(raw,self.projected()['proposal_references'],self.request)
  response={'proposals':[{'kind':'edit_set'}],'pursuit_expectations':{'0':expanded['alternatives'][0]['expectation']}}
  record={'model_content':raw,'decoded_sha256':m.digest([response,{}]),'association':{'request':'a'*64,'source':{'state_root':self.request['state_root']},'request_body_sha256':m.digest(wire),'model_content_sha256':m.digest(raw)}}
  evidence=m.review_provenance(self.request,record,wire,response,{})
  self.assertEqual(evidence['model_submission'],raw);self.assertEqual(evidence['native_candidate_sha256'],m.digest(response))
  self.assertEqual(evidence,m.review_provenance(*json.loads(json.dumps([self.request,record,wire,response,{}]))))
  current=copy.deepcopy(self.request);current['estimates'][0]['resource_cost']['sunk_tokens_charged']=1200
  self.assertEqual(evidence,m.review_provenance(current,record,wire,response,{},captured=copy.deepcopy(self.request)))
  with self.assertRaises(ValueError):m.review_provenance(current,record,wire,response,{})
  forged_capture=copy.deepcopy(self.request);forged_capture['goal']='forged'
  with self.assertRaisesRegex(ValueError,'captured proposal request'):m.review_provenance(current,record,wire,response,{},captured=forged_capture)
  for field in ('action','evidence','decision','serves'):
   stale=copy.deepcopy(current);stale['estimates'][0][field]='changed'
   with self.assertRaises(ValueError):m.review_provenance(stale,record,wire,response,{},captured=self.request)
  for field,value in [('sunk_tokens_charged',999),('sunk_tokens_charged',True),('source_prefix','changed')]:
   stale=copy.deepcopy(current);stale['estimates'][0]['resource_cost'][field]=value
   with self.assertRaises(ValueError):m.review_provenance(stale,record,wire,response,{},captured=self.request)
  for field in ('state_root','goal','coding'):
   stale=copy.deepcopy(current)
   if field=='coding':stale[field]['files'][0]['source']='altered'
   else:stale[field]='altered'
   with self.assertRaises(ValueError):m.review_provenance(stale,record,wire,response,{},captured=self.request)
  for index in (0,1,2,3):
   args=copy.deepcopy([self.request,record,wire,response,{}])
   if index==0:args[0]['state_root']='9'*64
   elif index==1:args[1]['model_content']['alternatives'][0]['expectation']['parents']=['p1']
   elif index==2:
    payload=json.loads(args[2]['messages'][1]['content']);payload['proposal_references']['parents']['p0']='9'*64;args[2]['messages'][1]['content']=json.dumps(payload)
    args[1]['association']['request_body_sha256']=m.digest(args[2])
   else:args[3]['pursuit_expectations']['0']['parents']=['9'*64]
   with self.assertRaises(ValueError):m.review_provenance(*args)
  bad=copy.deepcopy(response);bad['pursuit_expectations']['0']['parents']=['9'*64];forged=copy.deepcopy(record);forged['decoded_sha256']=m.digest([bad,{}])
  with self.assertRaisesRegex(ValueError,'expectation substitution'):m.review_provenance(self.request,forged,wire,bad,{})
 def test_catalyst_aliases_preserve_exact_reviewed_history_and_parent_set(self):
  relation=self.request['coding']['relational'];relation['catalyst_references']=[{'admission':'4'*64,'parents':['1'*64,'2'*64],'historical':True}]
  self.context['relational_contract']=m.native.relational_contract(self.request)
  wire=self.projected();self.assertEqual(wire['relational_contract']['catalyst_references'][0]['admission'],'r1')
  value=self.proposal();proposal=value['alternatives'][0]['relational'];proposal.update(revises=None,catalysts=['r1']);proposal.pop('development')
  expanded=m.expand(value,wire['proposal_references'],self.request)['alternatives'][0]['relational']
  self.assertEqual(expanded['catalysts'],['4'*64]);m.native.validate_relational_lineage(self.request,expanded,'open_choice')
  for bad in [['3'*64],['4'*64,'4'*64],['f'*64]]:
   v=copy.deepcopy(expanded);v['catalysts']=bad
   with self.assertRaises(ValueError):m.native.validate_relational_lineage(self.request,v,'open_choice')
  stale=copy.deepcopy(self.request);stale['coding']['relational']['catalyst_references']=[]
  with self.assertRaises(ValueError):m.expand(value,wire['proposal_references'],stale)
 def test_success_alias_is_source_bound_and_never_accepts_host_metadata(self):
  self.request['coding']['relational']['inputs']['current_success']={'id':'5'*64,'basis':{'assessment':'6'*64}}
  self.context['current_success']=copy.deepcopy(self.request['coding']['relational']['inputs']['current_success'])
  wire=self.projected();self.assertEqual(wire['current_success']['id'],'s0')
  value={'alternatives':[{'success_preference':{'aspiration':'r1','success':'s0'}}]}
  result=m.expand(value,wire['proposal_references'],self.request)['alternatives'][0]['success_preference']
  self.assertEqual(result,{'aspiration':'4'*64,'success':'5'*64,'binding':'d'*64})
  for change in ({'success':'5'*64},{'success':'stale'},{'binding':'d'*64},{'fulfills_parents':True},{'rationale':'extra unsupported field'}):
   bad=copy.deepcopy(value);bad['alternatives'][0]['success_preference'].update(change)
   with self.assertRaises(ValueError):m.expand(bad,wire['proposal_references'],self.request)
  stale=copy.deepcopy(self.request);stale['coding']['relational']['inputs']['current_success']['id']='9'*64
  with self.assertRaises(ValueError):m.expand(value,wire['proposal_references'],stale)
  self.assertEqual(value,{'alternatives':[{'success_preference':{'aspiration':'r1','success':'s0'}}]})
 def test_choice_summary_dynamic_evidence_and_prerequisites(self):
  self.context.pop('advisory_reconsideration')
  before=copy.deepcopy((self.request,self.context));wire=self.projected();forms=wire['choice_summary']['forms']
  self.assertIn('retain',forms);self.assertIn('relationship_initial',forms);self.assertNotIn('success_preference',forms);self.assertNotIn('contrast_preference',forms)
  self.assertEqual((self.request,self.context),before)
  for kind,evidence in (('success_preference','current_success'),('contrast_preference','current_contrast')):
   self.request['coding']['relational']['inputs'][evidence]={'id':'5'*64,'basis':{'source_snapshot':'b'*64}}
   self.context[evidence]=copy.deepcopy(self.request['coding']['relational']['inputs'][evidence]);self.context['response_contracts'][kind]={'authored':'reference contract'}
   wire=self.projected();summary=wire['choice_summary'];self.assertNotIn(kind,summary['forms']);self.assertEqual(summary['reviewed_aspiration_references'],[])
   self.assertIn(kind,summary['unavailable_identity_routes'])
   self.request['coding']['relational']['identity_choices']={kind:['4'*64]}
   summary=self.projected()['choice_summary'];self.assertIn(kind,summary['forms']);self.assertEqual(summary['eligible_identity_aspirations'][kind],['r1'])
   self.request['coding']['relational'].pop('identity_choices')
   self.assertEqual(m.expand({'retain':'No supported new wanting'},wire['proposal_references'],self.request),{'retain':'No supported new wanting'})
   bad=copy.deepcopy(self.context);bad[evidence]['id']='foreign'
   with self.assertRaisesRegex(ValueError,'choice evidence'):m.project(bad,self.request)
   self.request['coding']['relational']['inputs'].pop(evidence);self.context.pop(evidence);self.context['response_contracts'].pop(kind)
  self.assertEqual(self.projected()['choice_summary'],json.loads(json.dumps(self.projected()))['choice_summary'])
 def test_choice_summary_current_guidance_child_and_reconsideration(self):
  self.context.pop('advisory_reconsideration');entry=self.request['coding']['relational']['history'][1]
  entry['current']=True;entry['binding']=self.request['coding']['relational']['binding'];self.request['coding']['relational']['guidance']=[copy.deepcopy(entry)]
  self.context['relational_history']=copy.deepcopy(self.request['coding']['relational']['history']);self.context['relational_contract']=m.native.relational_contract(self.request)
  summary=self.projected()['choice_summary'];self.assertEqual(summary['reviewed_aspiration_references'],['r1'])
  self.assertEqual('relationship_revision' in summary['forms'],bool(self.context['relational_contract']['allowed_lineage']));self.assertIn('not approved outcomes',summary['interpretation'])
  child={'id':'7'*64,'meaning':'Authored generated child','original_contributors':['1'*64,'2'*64]}
  self.request['coding']['relational']['inputs']['parents'].append(child);self.context['parents'].append(copy.deepcopy(child));self.context['relational_contract']=m.native.relational_contract(self.request)
  wire=self.projected();self.assertIn('p2',wire['proposal_references']['parents']);self.assertIn('relationship_initial',wire['choice_summary']['forms'])
  self.context['advisory_reconsideration']={'lineage':'4'*64};self.assertEqual(set(self.projected()['choice_summary']['forms']),{'abstain','relationship_revision'})
 def test_choice_summary_does_not_remove_adverse_evidence_or_unknown_user_prose(self):
  self.context.pop('advisory_reconsideration');self.context['public_brief']='User asks for retention; '+m.LEGACY_OPTIONS
  self.context['review_criticism']={'authored':'adverse outcome, unknown fulfillment'};self.context['independent_assessments']=[{'status':'unknown','source':'old'}]
  before=copy.deepcopy(self.context);wire=self.projected()
  for key in ('public_brief','review_criticism','independent_assessments','files'):self.assertEqual(wire[key],before[key])
  self.assertNotIn('brief_presentation',wire);self.assertEqual(self.context,before)
 def test_exact_host_brief_replacement_is_provenance_bound(self):
  from unittest.mock import patch
  import hashlib
  brief='Authored host task. '+m.LEGACY_OPTIONS+' Preserve uncertainty.'
  self.context['public_brief']=brief
  with patch.object(m,'LEGACY_HOST_BRIEF_SHA256',hashlib.sha256(brief.encode()).hexdigest()):
   wire=self.projected()
   self.assertEqual(wire['public_brief'],'Authored host task. Current proposal forms and prerequisites are in choice_summary. Preserve uncertainty.')
   self.assertEqual(wire['brief_presentation']['original_sha256'],hashlib.sha256(brief.encode()).hexdigest())
   self.assertEqual(self.context['public_brief'],brief)
   self.context['public_brief']=brief+' Changed user wording.'
   self.assertEqual(self.projected()['public_brief'],self.context['public_brief'])
   self.assertNotIn('brief_presentation',self.projected())
 def test_frozen_host_brief_matches_exact_replacement(self):
  import ast,hashlib
  root=Path(__file__).resolve().parents[1]
  data=json.loads((root/'experiments/relational-observation-v4/conditions.json').read_text())
  tree=ast.parse((root/'experiments/relational-observation-live-v4/prepare.py').read_text())
  assignment=next(n for n in tree.body if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='BRIEF' for t in n.targets))
  brief=eval(compile(ast.Expression(assignment.value),'<frozen host brief>','eval'),{'__builtins__':{}},{'data':data})
  self.assertEqual(hashlib.sha256(brief.encode()).hexdigest(),m.LEGACY_HOST_BRIEF_SHA256)
  self.context['public_brief']=brief;before=copy.deepcopy((self.context,self.request));wire=self.projected()
  self.assertEqual(wire['public_brief'],brief.replace(m.LEGACY_OPTIONS,'Current proposal forms and prerequisites are in choice_summary.'))
  self.assertEqual(wire['brief_presentation']['original_sha256'],m.LEGACY_HOST_BRIEF_SHA256)
  self.assertEqual((self.context,self.request),before)
  self.context['public_brief']=brief+' '
  self.assertEqual(self.projected()['public_brief'],brief+' ')
  self.assertNotIn('brief_presentation',self.projected())
 def projected(self):return m.project(self.context,self.request)
 def proposal(self):
  return {'alternatives':[{'relational':{'parents':['p0','p1'],'relationship':'New hypothesis; literal r0 unchanged','aspiration':'A shared pursuit','discriminator':'Evidence may contradict this','revises':'r1','catalysts':[],
   'development':{'original_revision':'r0','prior_revision':'r1','retained_meaning':'Both wants','return_reveals':'No observed outcome','unresolved':'Still unknown','proposed_development':'Authored development'}},
   'return_basis':{'observed':[],'inferred':[{'path':'app.js','kind':'source_prediction','statement':'Literal p0/r0 in source is not an identity reference'}],'unknown':['Benefit is unknown']}}]}
 def test_project_publishes_aliases_and_omits_copy_only_metadata(self):
  before=copy.deepcopy((self.context,self.request));wire=self.projected()
  self.assertEqual(wire['parents'][0]['id'],'p0');self.assertEqual(wire['parents'][0]['meaning'],self.parents[0]['meaning'])
  self.assertEqual(wire['relational_contract']['allowed_lineage'][0]['revises'],'r1')
  self.assertEqual(wire['relational_history'][0]['proposal']['relationship'],'r0 is literal narrative')
  self.assertEqual(wire['advisory_reconsideration']['lineage'],'r1')
  self.assertNotIn('binding',wire['return_evidence'])
  schema=wire['response_contracts']['return_basis']['schema']
  self.assertNotIn('binding',schema['required']);self.assertNotIn('digest',schema['properties']['inferred']['items']['oneOf'][0]['required'])
  self.assertEqual(wire['files'],self.context['files']);self.assertEqual((self.context,self.request),before)
 def test_revision_expands_and_passes_unchanged_native_validators(self):
  wire=self.projected();raw=self.proposal();before=copy.deepcopy(raw)
  result=m.expand(raw,wire['proposal_references'],self.request);item=result['alternatives'][0]
  m.native.validate_relational_lineage(self.request,item['relational'],'open_choice')
  m.native.return_evidence.validate(item['return_basis'],self.request)
  self.assertEqual(item['relational']['development']['original_revision'],'3'*64)
  self.assertEqual(item['relational']['development']['prior_revision'],'4'*64)
  self.assertEqual(item['return_basis']['inferred'][0]['digest'],'c'*64)
  self.assertEqual(item['relational']['relationship'],raw['alternatives'][0]['relational']['relationship'])
  self.assertEqual(raw,before)
 def test_initial_abstention_inspection_and_code_do_not_acquire_references(self):
  catalog=self.projected()['proposal_references']
  for raw in ({'abstain':'Insufficient evidence'}, {'inspect':{'path':'app.js','offset':0}}, {'alternatives':[{'edits':[{'path':'app.js','old_text':'p0','new_text':'r0'}],'rationale':'Keep literal aliases'}]}):
   self.assertEqual(m.expand(raw,catalog,self.request),raw)
  raw=self.proposal();rel=raw['alternatives'][0]['relational'];rel['revises']=None;rel.pop('development');raw['alternatives'][0].pop('return_basis')
  value=m.expand(raw,catalog,self.request);m.native.validate_relational_lineage(self.request,value['alternatives'][0]['relational'],'open_choice')
 def test_wrong_long_duplicate_or_cross_request_refs_rejected(self):
  catalog=self.projected()['proposal_references']
  for field,value in [('parents',['1'*64,'p1']),('parents',['p0','p0']),('revises','4'*64),('revises','r99')]:
   raw=self.proposal();raw['alternatives'][0]['relational'][field]=value
   with self.assertRaises(ValueError):m.expand(raw,catalog,self.request)
  for mutation in ({**self.request,'state_root':'9'*64},):
   with self.assertRaises(ValueError):m.expand(self.proposal(),catalog,mutation)
  tampered=copy.deepcopy(catalog);tampered['parents']['p0']='9'*64
  with self.assertRaises(ValueError):m.expand(self.proposal(),tampered,self.request)
 def test_host_metadata_is_rejected_even_if_it_happens_to_be_correct(self):
  catalog=self.projected()['proposal_references']
  for field in ('binding','digest'):
   raw=self.proposal();basis=raw['alternatives'][0]['return_basis']
   if field=='binding':basis[field]=m.native.return_evidence.catalog(self.request)['binding']
   else:basis['inferred'][0][field]='c'*64
   with self.assertRaisesRegex(ValueError,'model supplied host'):m.expand(raw,catalog,self.request)
  raw=self.proposal();raw['state_root']=self.request['state_root']
  with self.assertRaises(ValueError):m.expand(raw,catalog,self.request)
 def test_stale_parent_ancestry_and_source_still_fail_native_validation(self):
  catalog=self.projected()['proposal_references'];raw=self.proposal();raw['alternatives'][0]['relational']['revises']='r0'
  value=m.expand(raw,catalog,self.request)
  with self.assertRaises(ValueError):m.native.validate_relational_lineage(self.request,value['alternatives'][0]['relational'],'open_choice')
  raw=self.proposal();raw['alternatives'][0]['return_basis']['inferred'][0]['path']='protected.js'
  with self.assertRaises(ValueError):m.expand(raw,catalog,self.request)
  other=copy.deepcopy(self.request);other['coding']['files'][0]['source_digest']='9'*64
  with self.assertRaises(ValueError):m.expand(self.proposal(),catalog,other)
 def test_context_or_ambiguous_inventory_drift_rejected(self):
  bad=copy.deepcopy(self.context);bad['return_evidence']['binding']='9'*64
  with self.assertRaises(ValueError):m.project(bad,self.request)
  bad=copy.deepcopy(self.request);bad['coding']['files'].append(copy.deepcopy(bad['coding']['files'][0]))
  with self.assertRaises(ValueError):m.catalog(bad)
 def test_replay_same_request_is_deterministic_without_mutation(self):
  wire=self.projected();before=json.dumps((self.context,self.request),sort_keys=True)
  self.assertEqual(m.expand(self.proposal(),wire['proposal_references'],self.request),m.expand(self.proposal(),wire['proposal_references'],self.request))
  self.assertEqual(json.dumps((self.context,self.request),sort_keys=True),before)

if __name__=='__main__':unittest.main()
