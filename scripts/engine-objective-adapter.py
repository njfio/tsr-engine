#!/usr/bin/env python3
"""Bounded finite-service evaluation profile of shared project transport and Cycle.

The fixed packet checker owns domain criteria, never candidate generation or selection.

No standalone launcher or implicit authority. A fresh reviewed origin and campaign
suite grant are mandatory. Conditions come only from the native current projection.
"""
import copy,json,time,subprocess
from pathlib import Path
import importlib.util
s=importlib.util.spec_from_file_location('objective_base',Path(__file__).with_name('engine-project-cycle-adapter.py'));base=importlib.util.module_from_spec(s);s.loader.exec_module(base)
e=base.e;d=base.d;loop=base.loop;load=base.load
shape=base.load('objective_shape','engine-project-output.py')

def pins():return {**base.pins(),'engine-objective-adapter.py':d.file_sha(Path(__file__)),'engine-output-release.py':d.file_sha(Path(__file__).with_name('engine-output-release.py'))}
def admission(root,origin,campaign,expires_at):
 return {'version':'synthetic-objective-adapter@v2','directory':str(Path(root).resolve()),'origin':origin,'campaign_sha256':e.sha(campaign),'model':base.MODEL,'max_calls':24,'max_decisions':9,'max_executions':5,'max_service_rounds':3,'correction_allowance':2,'expires_at':expires_at,'implementation':pins(),'binary_sha256':d.file_sha(e.ROOT/'target/debug/tsr'),'output_profile':'strict-json-bound@v2'}

# Fixed reviewed evaluator implementations only; no model-selected module/path.
CHECKER_PROFILES={
 'finite-shortlist@v1':('experiments/objective-service-v1/oracle.py','finite_selection_valid'),
 'guide-opportunity@v1':('experiments/guide-opportunity-v1/oracle.py','minimum_obligations_met'),
 'open-guide@v1':('experiments/open-guide-v1/oracle.py','minimum_obligations_met'),
}
def receipt_checker(origin):
 profile=origin.get('receipt_checker_profile','finite-shortlist@v1')
 if profile not in CHECKER_PROFILES:raise ValueError('unsupported receipt checker profile')
 relative,verdict=CHECKER_PROFILES[profile];path=e.ROOT/relative
 if any(p.is_symlink() for p in (path,*path.parents)) or not path.resolve().is_relative_to(e.ROOT.resolve()):raise ValueError('receipt checker path boundary')
 if origin.get('receipt_checker_sha256')!=d.file_sha(path):raise ValueError('independent receipt checker drift')
 return path,verdict

def freeze_origin(controller,public_task,suite_sha256,*,checker_profile=None):
 """Local preparation only; this does not grant export or execution authority."""
 e.digest(suite_sha256)
 request=controller.request();coding=request['coding'];config=controller.cli('project','config');objective=controller.cli('project','objective-status')
 if objective['context']['round']!=0 or objective['context']['next']!='propose':raise ValueError('fresh objective origin required')
 editable=[f for f in coding['files'] if f.get('editable')]
 if len(editable)!=1 or config['cargo'] or len(e.canonical(public_task))>8192:raise ValueError('bounded non-Cargo single source origin')
 origin={'version':'public-objective-origin@v1','directory':str(controller.directory),'scope':request['next_attention']['scope'],'requirements_revision':coding['relational']['inputs']['requirements_revision'],'parents':coding['relational']['inputs']['parents'],'initial_source':editable[0]['source'],'editable_path':editable[0]['path'],'protected':{f['path']:f['source_digest'] for f in coding['files'] if not f.get('editable')},'config_sha256':e.sha(config),'objective_admission':objective['context']['admission'],'schedule':objective['context']['schedule'],'public_task':copy.deepcopy(public_task),'suite_sha256':suite_sha256,'receipt_checker_sha256':d.file_sha(e.ROOT/'experiments/objective-service-v1/oracle.py')}
 if checker_profile is not None:
  if checker_profile not in CHECKER_PROFILES:raise ValueError('unsupported receipt checker profile')
  origin.update(version='public-objective-origin@v2',receipt_checker_profile=checker_profile,receipt_checker_sha256=d.file_sha(e.ROOT/CHECKER_PROFILES[checker_profile][0]))
 receipt_checker(origin)
 return origin

class Adapter(base.Adapter):
 release_profiles=base.Adapter.release_profiles+('synthetic-objective-adapter@v2',)
 def __init__(self,*args,**kwargs):
  super().__init__(*args,**kwargs)
  self.release=True;self.information=False;self.local_advisory=False;self.fixture=bool(self.ledger.manifest.get('fixture_only'))
  if self.fixture and self.injected_transport is None:raise ValueError('fixture transport mandatory')
 def validate_current_admission(self):
  a=self.admission
  if a!=admission(self.root,a['origin'],self.ledger.manifest,a['expires_at']):raise ValueError('objective admission drift')
  if time.time()>=d.stamp(a['expires_at']) or any(p.is_symlink() for p in (self.root,*self.root.parents)):raise ValueError('objective expired/path boundary')
  o=a['origin'];keys={'version','directory','scope','requirements_revision','parents','initial_source','editable_path','protected','config_sha256','objective_admission','schedule','public_task','suite_sha256','receipt_checker_sha256'}
  if o.get('version')=='public-objective-origin@v2':keys.add('receipt_checker_profile')
  if set(o)!=keys or o['version'] not in ('public-objective-origin@v1','public-objective-origin@v2') or str(self.controller.directory)!=o['directory']:raise ValueError('objective origin contract')
  e.digest(o['suite_sha256'])
  receipt_checker(o)
 def enroll(self):return self.op('enroll',lambda:self.ledger.enroll(self.trial,e.sha(self.admission),suite_sha256=self.admission['origin']['suite_sha256']) or {'enrolled':True})
 def inherited(self):
  """Only a separately verified continuation may supply historical provenance."""
  return {"generated":[],"reviews":[],"receipts":[]}
 def generated(self):
  return self.inherited()["generated"]+super().generated()
 def project(self,view):
  self.validate_current_admission();o=self.admission['origin'];r=self.controller.request();c=r['coding'];relation=c['relational'];inputs=relation['inputs'];obj=self.controller.cli('project','objective-status')
  if (view['state_root']!=r['state_root'] or view['scope']!=o['scope'] or view['source_snapshot']!=c['snapshot'] or inputs['requirements_revision']!=o['requirements_revision'] or e.sha(self.controller.cli('project','config'))!=o['config_sha256']):raise ValueError('objective source/root/criteria drift')
  if obj['context']['admission']!=o['objective_admission'] or obj['context']['schedule']!=o['schedule']:raise ValueError('objective schedule drift')
  if inputs['parents']!=o['parents'] or inputs.get('public_desires') or inputs.get('supplemental') or c.get('supporting_context') or c.get('information'):raise ValueError('unadmitted public state')
  files=c['files'];editable=[f for f in files if f.get('editable')]
  if len(editable)!=1 or editable[0]['path']!=o['editable_path'] or {f['path']:f['source_digest'] for f in files if not f.get('editable')}!=o['protected']:raise ValueError('source inventory')
  work=self.controller.directory/'coding-work';inventory=list(work.rglob('*'))
  if any(p.is_symlink() for p in inventory) or {p.relative_to(work).as_posix() for p in inventory if p.is_file()}not in ({o['editable_path'],*o['protected']},{o['editable_path'],*o['protected'],'fixture-probe'}):raise ValueError('private/unadmitted files')
  allowed={o['initial_source']}
  for response in self.generated():
   for proposal in response.get('proposals',[]):
    if proposal['kind']=='edit_set':allowed.update(p['new_text'] for p in proposal['patches'])
  if editable[0]['source'] not in allowed:raise ValueError('source lineage')
  relations=[e.relational_identity(p['proposal']) for response in self.generated() for p in response.get('proposals',[]) if p['kind']=='relational']
  if any(e.relational_identity(h['proposal']) not in relations for h in relation['history']):raise ValueError('unadmitted relationship')
  admitted=[{'record':r} for r in self.inherited()['reviews']]+[v for k,v in self.read()[2].items() if k.startswith('objective_admitted_')]
  native_reviews=obj['context']['round_reviews']
  if len(admitted)!=len(native_reviews) or obj['context']['final_review'] is not None:raise ValueError('foreign objective assessment')
  for actual,owned in zip(native_reviews,admitted):
   if actual!=owned['record']:raise ValueError('foreign/private objective review content')
  owned_receipts=self.inherited()['receipts']+[v.get('result',v)['next']['objective']['context']['receipts'][-1] for k,v in self.read()[2].items() if k.startswith('service_execution_')]
  if obj['context']['receipts']!=owned_receipts:raise ValueError('foreign native service receipt')
  obj=copy.deepcopy(obj)
  for receipt in obj['context']['receipts']:
   if set(receipt)!={'receipt','prepared','attempt','round','condition','source_snapshot','previous_output','previous_output_digest','output','exit','complete','runner_receipts','choice','meaning'}:raise ValueError('objective receipt projection changed')
   for identity in receipt['runner_receipts']:e.digest(identity)
  # No complete Request, config, private acceptance review, or future condition bodies.
  projection={'public_task':o['public_task'],'parents':inputs['parents'],'source':{k:editable[0][k] for k in ('path','source','source_digest','inspection_handle')},'objective':obj,'relational':{k:relation[k] for k in ('binding','guidance','history','proposal_contract')},'relational_contract':e.relational_contract(r),'relational_fields':e.payload_contract(r,'final_proposal_only')['relational'],'binding':{'state_root':r['state_root'],'source_snapshot':c['snapshot'],'attention_binding':r['next_attention']['binding']},'return_evidence':e.return_evidence.catalog(r),'return_basis_contract':e.return_evidence.contract(),'limits':{'source_replacement_utf8_bytes':2048,'native_response_utf8_bytes':8192,'protected_tests':'withheld; immutable','meaning':'Service stdout and exit describe invocation only. Same-process oracle can be bypassed; independent review must assess raw output, source and public criteria. No success or new wanting is required.'}}
  # Derive facts only after owned receipt/source checks; shared by every request family.
  projection['independent_receipt_checks']=[self.receipt_assessment(r) for r in obj['context']['receipts']]
  if view['branch']=='reconsider':projection['reconsideration']=view['reconsideration']
  if len(e.canonical(projection))>28000:raise d.Blocked('complete public objective context exceeds export bound; no truncation')
  return r,projection
 def proposal(self,view):
  request,context=self.project(view)
  context['objective']['choice_schema']=copy.deepcopy(context['objective']['choice_schema'])
  context['objective']['choice_schema']['properties'].pop('binding')
  context['objective']['choice_schema']['required'].remove('binding')
  context['output_envelope']={'version':base.bound_output.VERSION,'instruction':'Omit only these host-owned paths: service.choice.binding, relational.binding, edit_set.patches[*].source_digest and edit_set.patches[*].inspection_handle. Keep return_basis.binding and all selected parent, lineage and evidence references exactly as supplied; these remain required semantic citations.'}
  context['response_contract']={'service':{'kind':'service','choice':'exact objective.choice_schema'},'edit_set':{'kind':'edit_set','rationale':'nonblank <=512 bytes','patches':[{'path':'exact source.path','old_text':'exact full source.source','new_text':'complete replacement <=2048 UTF-8 bytes'}]},'relational':{'kind':'relational','proposal':'complete relational_fields and exact relational_contract lineage identities','return_basis':'required for revisions: return_basis_contract and return_evidence exact IDs; absent for initial relation'},'abstain':'top-level {abstain:nonblank <=512 bytes}'}
  body=self.body('Choose useful possibilities from active desires, reviewed relationship and observed returns. Return strict JSON {alternatives:[one or more native suggestion objects from response_contract]} or {abstain:text}. Service and relational suggestions must each be alone; editable alternatives may compete under native preview. No required sequence, forced disappointment, chosen IDs or new wanting. In reconsideration only one revision of the named lineage or abstention is allowed. Do not invent outcomes. Merely representing both desires is not evidence of culmination.',context)
  return self.checked('proposal',body,lambda v:self.decode_proposal(v,request,context,view))
 def decode_proposal(self,value,request,context,view):
  shape.proposal_shape(value)
  if 'abstain' in value:return value,{}
  proposals=copy.deepcopy(value['alternatives']);basis={}
  for p in proposals:
   if not isinstance(p,dict):shape.fail('$.alternatives','object_required',expected_type='object')
   if p.get('kind') not in ('service','edit_set','relational'):raise ValueError('objective action not admitted')
   if p['kind']=='service':
    if len(proposals)!=1:raise ValueError('service must be sole deferred suggestion')
    if isinstance(p.get('choice'),dict) and 'binding' in p['choice']:raise ValueError('model supplied trusted service binding; no repair')
    schema=copy.deepcopy(context['objective']['choice_schema']);schema['properties'].pop('binding',None);schema['required']=[k for k in schema['required'] if k!='binding']
    shape.check(p,{'type':'object','required':['kind','choice'],'properties':{'kind':{'const':'service'},'choice':schema}})
    p['choice']['binding']=context['objective']['binding']
   if p['kind']=='edit_set':
    source=context['source']
    if isinstance(p.get('patches'),list):
     for patch in p['patches']:
      if isinstance(patch,dict):
       if set(patch)&{'source_digest','inspection_handle'}:raise ValueError('model supplied trusted edit metadata; no repair')
       patch.update(source_digest=source['source_digest'],inspection_handle=source['inspection_handle'])
    patch_schema={'type':'object','required':['path','source_digest','inspection_handle','old_text','new_text'],'properties':{'path':{'const':source['path']},'source_digest':{'const':source['source_digest']},'inspection_handle':{'const':source['inspection_handle']},'old_text':{'const':source['source']},'new_text':{'type':'string','maxLength':2048}}}
    shape.check(p,{'type':'object','required':['kind','patches','rationale'],'properties':{'kind':{'const':'edit_set'},'patches':{'type':'array','minItems':1,'maxItems':1,'items':patch_schema},'rationale':{'type':'string','maxLength':512}}},'$.alternatives['+str(proposals.index(p))+']')
   if p['kind']=='relational':
    if not isinstance(p.get('proposal'),dict):shape.fail('$.alternatives.proposal','object_required',expected_type='object')
    if p['proposal'].get('revises') is not None:
     basis[str(proposals.index(p))]=e.return_evidence.validate(p.pop('return_basis',None),request)
    if 'binding' in p:raise ValueError('model supplied trusted relational binding; no repair')
    if set(p)!={'kind','proposal'}:raise ValueError('relational source binding')
    p['binding']=context['relational']['binding']
    e.validate_relational_lineage(request,p['proposal'],'open_choice')
  response={'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':proposals}
  self.native('preview',{'response':response});return response,basis
 def receipt_assessment(self,receipt):
  # Exact reviewed data checker, never generated code or a configurable command.
  path,verdict=receipt_checker(self.admission['origin'])
  spec=importlib.util.spec_from_file_location('frozen_objective_data_checker',path);checker=importlib.util.module_from_spec(spec);spec.loader.exec_module(checker)
  bound={k:receipt[k] for k in ('receipt','source_snapshot','condition','previous_output','output','complete','exit')}
  try:
   assessment=checker.assess(receipt['condition']['input'],receipt['previous_output'],receipt['output']) if receipt['complete'] and isinstance(receipt['output'],str) else {verdict:False,'reason':'No complete service output'}
  except (ValueError,TypeError) as error:assessment={verdict:False,'reason':'Output not assessable: '+str(error)}
  return {'input_sha256':e.sha(bound),'checker_sha256':self.admission['origin']['receipt_checker_sha256'],'receipt':receipt['receipt'],'assessment':assessment,'limits':'Independent finite receipt-data check, not source repeatability proof, subjective truth, task acceptance or authority.'}
 def review(self,view,response):
  _,context=self.project(view);stage=response.get('objective_stage')
  if stage:
   if stage not in ('round_review','objective_review') or context['objective']['context']['next']!=stage:raise ValueError('review stage drift')
   choices=['met','not_met','unknown'] if stage=='round_review' else ['accept','reject','abstain']
   props={'binding':{'const':context['objective']['binding']},'decision':{'enum':choices},'evidence':{'type':'string','maxLength':512}}
   receipts=context['objective']['context']['receipts']
   if stage=='round_review':props['receipt']={'const':receipts[-1]['receipt']}
   criteria={'criterion_'+str(i):text for i,text in enumerate(context['public_task']['criteria'])}
  else:
   if response not in self.generated():raise ValueError('foreign candidate')
   props={'response_digest':{'const':view['response_digest']},'decision':{'enum':['approve','reject','unknown','abstain']},'evidence':{'type':'string','maxLength':512}}
   receipts=context['objective']['context']['receipts']
   criteria={'relevance':'Every candidate plausibly serves active contributors; no prescribed strategy or novelty.','authority':'Only admitted edits, service or advisory relation; no criteria or permission changes.','evidence':'Truthful source-bound observations versus predictions and unknowns; no unobserved passing claim.','contract':'Exact native schema and all public task restrictions preserved.'}
  ids=[r['receipt'] for r in receipts]
  finding={'type':'object','required':['finding','basis','receipts','explanation','limitations'],'properties':{'finding':{'enum':['met','not_met','unknown']},'basis':{'enum':['static_source','observed_service_receipt','unknown']},'receipts':{'type':'array','maxItems':3,'items':{'enum':ids}},'explanation':{'type':'string','maxLength':512},'limitations':{'type':'string','maxLength':512}}}
  props['findings']={'type':'object','required':list(criteria),'properties':{k:finding for k in criteria}}
  schema={'type':'object','required':list(props),'properties':props}
  basis=next((r['return_basis'] for k,r in self.read()[2].items() if k.startswith('generated_') and r['response']==response),{})
  body=self.body('Separate independent review invocation. Assess EVERY criterion with exact source/receipt provenance. Same-model errors may correlate. Exit zero alone cannot establish semantic correctness: generated code shares process with protected harness. The independent checker establishes only listed finite properties, never repeatability or relationship fulfillment. Repeatability needs source assessment; preserve unknown if unsupported. Artifact smoke is not service evidence. Proposed relations preserve contributors; no novelty required. met/approve/accept requires all findings met; reject/not_met requires a not_met finding; unknown requires an unknown finding. Abstention is legitimate. No new authority.',{'context':context,'candidate':response,'return_basis':basis,'criteria':criteria,'response_schema':schema})
  def validate(value):
   shape.check(value,schema)
   findings=value['findings'];dec=value['decision'];states=[v['finding'] for v in findings.values()]
   if dec in ('approve','met','accept') and any(x!='met' for x in states):raise ValueError('unsupported positive assessment')
   if dec in ('reject','not_met') and 'not_met' not in states:raise ValueError('unsupported rejection')
   if dec=='unknown' and 'unknown' not in states:raise ValueError('unsupported uncertainty')
   for f in findings.values():
    if f['basis']=='observed_service_receipt' and not f['receipts']:raise ValueError('observed assessment without receipt')
   if stage=='round_review' and dec=='met' and not context['independent_receipt_checks'][-1]['assessment'][receipt_checker(self.admission['origin'])[1]]:raise ValueError('semantic assessment contradicts independent receipt check')
  ident,value,receipt=self.checked('review',body,validate,review=True)
  self.op('reviewed_'+ident,lambda:{'report':value,'context_digest':e.sha(context),'candidate':response,'receipt_checks':context.get('independent_receipt_checks',[])})
  # Full report is retained. Native attestation uses its explicit smaller API.
  judgment={k:v for k,v in value.items() if k!='findings'}
  judgment.update(reviewer='separate-runtime-invocation:'+ident,generator=base.MODEL)
  if not stage:judgment.pop('evidence')
  self.verify_owned_review(judgment,None if stage else response)
  return {'judgment':judgment,'receipt':receipt}
 def native(self,kind,value):
  self.validate_current_admission();c=self.controller
  if kind=='request':return c.cli('project','cycle','request')
  if kind=='response_context':return c.cli('project','cycle','response-context')
  if kind=='preview':return c.json_cli(('project','cycle','preview'),value['response'],'--response')
  if kind=='acknowledge':return c.json_cli(('project','cycle','acknowledge'),value)
  if kind=='objective_review':
   review=value['review'];self.verify_owned_review(review);owned=[(k,r) for k,r in self.read()[2].items() if k.startswith('reviewed_') and r['report'].get('receipt')==review['receipt']]
   if len(owned)!=1 or review['reviewer']!='separate-runtime-invocation:'+owned[0][0][len('reviewed_'):] or review['generator']!=base.MODEL or any(owned[0][1]['report'].get(k)!=review[k] for k in ('binding','decision','receipt','evidence')):raise ValueError('unattributed objective import')
   def admit():
    result=c.json_cli(('project','objective-review'),review)
    return {'record':result['context']['round_reviews'][-1],'context_binding':result['binding']}
   return self.op('objective_admitted_'+review['receipt'],admit)
  if kind=='advance':
   self.verify_owned_review(value['review'],value['response'])
   result=c.json_cli(('project','cycle','advance'),value)
   p=value['response']['proposals']
   if len(p)==1 and p[0]['kind']=='relational' and result.get('review_required'):
    record=result['proposal_record'];current=self.native('request',None)
    self.op('qualitative_'+record,lambda:c.cli('project','relational','admit','--state-root',current['state_root'],'--proposal',record,'--index','0'))
    return {**result,'review_required':False,'qualitative_admitted':True,'next':self.native('request',None)}
   return result
  if kind not in ('resume','service_resume'):raise ValueError('unsupported objective native callback')
  if self.fixture and self.worker is None:raise ValueError('fixture worker mandatory')
  view=self.native('request',None)
  if view['branch']!=kind or view['binding']!=value['binding']:raise ValueError('execution binding drift')
  done=self.read()[2];operations=[k for k in done if k.startswith(('execution_','service_execution_'))]
  if len(operations)>=self.admission['max_executions'] or sum(k.startswith('service_execution_') for k in operations)>=self.admission['max_service_rounds'] and kind=='service_resume' or sum(k.startswith('execution_') for k in operations)>=2 and kind=='resume':raise d.Blocked('objective operation ceiling')
  timeout=90 if kind=='service_resume' else self.resume_timeout(view)
  if time.time()+timeout>d.stamp(self.admission['expires_at']):raise d.Blocked('objective execution deadline')
  self.enroll();ident=e.sha({'admission':e.sha(self.admission),'kind':kind,'binding':value['binding']})
  def execute():
   if self.worker:return self.worker(view)
   result=subprocess.run([str(e.ROOT/'target/debug/tsr'),'engine','--directory',str(c.directory),'project','cycle',kind.replace('_','-'),'--binding',value['binding']],capture_output=True,timeout=timeout)
   if result.returncode or len(result.stdout)>1048576:raise d.Blocked('objective execution uncertain/refused; never retry')
   return e.strict(result.stdout)
  return self.op(('service_execution_' if kind=='service_resume' else 'execution_')+ident,lambda:self.ledger.run_execution(self.trial,ident,e.sha(self.admission),execute))
