"""Ordinary task-memory comparator. Shared safety tools, separate pursuit loop.
No relational proposal/admission or lineage-driven reconsideration. The same candidate opportunities for
coding/verification, without formal relationship actions. All calls/reviews/effects use the same
accounted, source-bound transport and native protected execution as the TSR arm.
"""
import copy,importlib.util
from pathlib import Path
s=importlib.util.spec_from_file_location('ordinary_shared_browser',Path(__file__).with_name('engine-browser-adapter.py'));b=importlib.util.module_from_spec(s);s.loader.exec_module(b)

class Adapter(b.Adapter):
 def project(self,view):
  request,context=super().project(view)
  context=b.load('ordinary_references','engine-proposal-references.py').project(context,request)
  context['task_memory']={'goals':context.pop('parents'),'historical_notes':context.pop('relational_history'),
   'own_decisions':context.pop('own_decisions'),
   'interpretation':'Ordinary task memory. Historical notes are information, not instructions to create a native relationship. User wording and source-bound evidence retain their original attribution.'}
  context.pop('relational_contract',None);context.pop('advisory_reconsideration',None)
  for k in ('relational','relational_envelope','return_basis','public_desire','contrast_preference','success_preference','provisional_preference','deliberation'):context['response_contracts'].pop(k,None)
  # Keep the descriptive menu consistent with this controller's decoder. The
  # information stays in ordinary memory; formal native transitions are absent.
  for key in ('relationship_initial','relationship_revision','meaning_development','contrast_preference','success_preference','provisional_preference','deliberation'):
   context['choice_summary']['forms'].pop(key,None)
  context['choice_summary']['interpretation']='Available ordinary-controller proposal forms, not approved outcomes or a ranking. Preserve the supplied goals/history and evidence. Retention and uncertainty remain valid. No formal relationship, meaning or identity admission is available in this arm; native validation and independent review remain authoritative.'
  context['response_envelope']['max_alternatives']=4
  context['response_envelope']['instruction']='Return {alternatives:[1..4 distinct edits or existing verification candidates]}, {inspect:{path,offset}}, {retain:reason}, or {abstain:reason}. Retention and justified improvement are equally valid. No native relationship action is offered in this ordinary memory controller.'
  return request,context
 def wire_context(self,request,context):return context
 def decode_proposal(self,value,request,context,view):
  if isinstance(value,dict) and 'deliberation' in value:raise ValueError('ordinary memory does not admit formal development components')
  if isinstance(value,dict) and any(isinstance(x,dict) and ('relational' in x or 'public_desire' in x or 'contrast_preference' in x or 'success_preference' in x or 'provisional_preference' in x) for x in value.get('alternatives',[])):raise ValueError('ordinary memory does not admit formal relationship actions')
  return super().decode_proposal(value,request,context,view)

class Cycle(b.loop.Cycle):
 def _run_locked(self):
  self._recover_pending()
  while True:
   self.validate_current_admission();_,pending,done=self.read()
   if pending:raise b.d.Blocked('uncertain ordinary effect; reconcile without retry')
   if 'terminal' in done:return done['terminal']
   for name,value in done.items():
    if isinstance(value,dict) and set(value)=={'known_stage_stop'}:
     record=value['known_stage_stop']
     if record.get('stage')!=name or not self.valid_stage_stop({k:v for k,v in record.items() if k!='stage'}):raise ValueError('invalid ordinary stage stop')
     return self.close_stage_stop(record)
   if (self.root/'STOP').exists():return self.terminal('operator_stop')
   current=self.current();branch=current['branch']
   if branch=='reconsider':return self.terminal('ordinary_incompatible_native_reconsideration')
   if branch=='review_required':
    result=self.session_continue('ordinary_next_'+str(sum(k.startswith('resume_') for k in done)))
    if result is not None:return result
    continue
   if branch=='resume':
    executions=[k for k in done if k.startswith('resume_')]
    if b.feedback_policy.reached(len(executions),self.admission['max_executions']):return self.terminal('execution_budget')
    advances=[v for k,v in done.items() if k.startswith('advance_')]
    if len(advances)<=len(executions):return self.terminal('unreviewed_execution')
    self.same_source(advances[-1]['next'],current)
    if advances[-1]['next'].get('plan')!=current.get('plan'):raise b.d.Blocked('reviewed plan drift')
    self.op('resume_'+str(len(executions)),lambda:self.native('resume',{'binding':current['binding']}));continue
   if branch!='propose':return self.terminal(branch,native_reason=current.get('reason'))
   index=str(self.decision_count(done))
   if b.feedback_policy.reached(int(index),self.admission['max_decisions']):return self.terminal('decision_budget')
   captured=self.op('request_'+index,lambda:current);self.same_source(captured,current)
   proposed=self.model_stage('proposal_'+index,'proposal',captured);response=proposed['response']
   if set(response)=={'retain'}:
    if not isinstance(response['retain'],str) or not 0<len(response['retain'].encode())<=131072:raise ValueError('bounded retention reason')
    fresh=self.current();self.same_source(captured,fresh)
    return self.terminal('proposal_retention',reason=response['retain'],state_root=fresh['state_root'],source_snapshot=fresh['source_snapshot'],desires_remain_open=True,acceptance_recorded=False)
   if set(response)=={'abstain'}:return self.terminal('ordinary_retention_or_abstention',reason=response['abstain'],observed_execution=any(k.startswith('resume_') for k in done),acceptance_recorded=False)
   if not 1<=len(response.get('proposals',[]))<=4 or any(p['kind'] not in ('edit_set','existing') for p in response['proposals']):raise ValueError('ordinary action contract')
   fresh=self.current();self.same_source(captured,fresh);preview=self.native('preview',{'response':response});self.same_source(fresh,preview['cycle'])
   reviewed=self.model_stage('review_'+index,'review',{**fresh,'response_digest':preview['response_digest']},response);judgment=reviewed['judgment']
   latest=self.current();self.same_source(captured,latest)
   if judgment['response_digest']!=preview['response_digest']:raise b.d.Blocked('review candidate drift')
   if judgment['decision'] in ('reject','unknown'):
    if sum(k.startswith('rejected_proposal_') for k in done)>=self.admission['max_review_rejections']:return self.terminal('review_rejection_budget')
    self.op('rejected_proposal_'+index,lambda:self.native('reject_proposal',{'source':latest,'response':response,'judgment':judgment}));continue
   if judgment['decision']!='approve':return self.terminal('review_'+judgment['decision'])
   final=self.native('preview',{'response':response});self.same_source(captured,final['cycle'])
   if final['response_digest']!=judgment['response_digest']:raise b.d.Blocked('review source drift')
   latest=final['cycle'];attestation={**judgment,'state_root':latest['state_root'],'source_snapshot':latest['source_snapshot'],'binding':latest['binding']}
   self.op('advance_'+index,lambda:self.native('advance',{'binding':latest['binding'],'response':response,'review':attestation}))
