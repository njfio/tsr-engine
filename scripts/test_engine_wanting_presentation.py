"""Offline message-contract checks, not evidence of model behavior or preference quality."""
import copy,json,unittest
from scripts.test_engine_browser_adapter import m

class WantingPresentation(unittest.TestCase):
 def body(self,branch):
  adapter=object.__new__(m.Adapter)
  adapter.body=lambda system,context:{'messages':[{'role':'system','content':system},{'role':'user','content':json.dumps(context)}]}
  context={'current_source':'authored','findings':{'crossing':'unknown'},'adverse_evidence':['authored collision'],'proposal_responsibilities':m.web.RESPONSIBILITIES}
  before=copy.deepcopy(context);body=adapter.proposal_body({'branch':branch},context)
  self.assertEqual(context,before);self.assertEqual(json.loads(body['messages'][1]['content']),before)
  return body
 def test_ordinary_and_reconsideration_distinguish_wanting_from_fact(self):
  for branch in ('propose','reconsider'):
   with self.subTest(branch=branch):
    text=self.body(branch)['messages'][0]['content']
    self.assertIn('does not need a defect, corrective evidence or observed success',text)
    self.assertIn('distinct from asserting an empirical fact',text)
    self.assertIn('independent review',text)
    self.assertIn('no novelty, activity, relationship or preferred design is required, and none is ruled out',text)
    self.assertIn('none is preferred',text)
 def test_retention_and_component_authority_remain_separate(self):
  text=self.body('propose')['messages'][0]['content']
  self.assertIn('both artifact and meaning',text);self.assertIn('keeping the artifact alone does not preclude',text)
  self.assertIn('{retain:reason, development_declined:reason}',text)
  self.assertIn('explicit dependency and separate reviews',text);self.assertIn('executes only with authorization',text)
  advisory=self.body('reconsider')['messages'][0]['content'];self.assertIn('no execution',advisory)
 def test_reviewer_permits_supported_facts_but_rejects_fabrication(self):
  review=m.load('wanting_review_contract','engine-project-runtime-review.py')
  fidelity=review.CRITERIA['parent_fidelity']
  self.assertNotIn('Reject invented user testimony, empirical claims,',fidelity)
  self.assertIn('fabricated or unsupported empirical claims',fidelity)
  self.assertIn('Supported empirical statements may inform the rationale',fidelity)
  self.assertIn('never establish unobserved fulfillment',fidelity)
  self.assertIn('not whether it repairs an artifact defect',review.CRITERIA['stage_adequacy'])
  self.assertIn('Direct execution claims require exact observed case IDs',review.CRITERIA['epistemic_grounding'])
 def test_shared_roles_keep_obligations_uncertainty_and_no_required_change(self):
  roles=m.web.RESPONSIBILITIES
  self.assertIn('without a corrective return',roles['generator'])
  self.assertIn('preserved obligations and honest uncertainty',roles['review_stage'])
  self.assertIn('retaining both remains valid',roles['review_stage'])
  self.assertIn('Do not invent tests',roles['evidence'])
  self.assertIn('not acceptance or execution authority',roles['controller'])
 def test_legacy_reflection_contracts_are_not_rewritten(self):
  reflection=m.load('wanting_reflection_contract','engine-pursuit-reflection.py')
  self.assertIn('Do not invent desires',reflection.capture_system({'contract_version':'qualitative-evidence@v2'}))
  current=reflection.capture_system({'contract_version':'qualitative-evidence@v3'})
  self.assertNotIn('Do not invent desires',current)
  self.assertIn('Optional possibilities may suggest generated wanting',current)
  self.assertIn('separate ordinary proposal and independent admission',current)

if __name__=='__main__':unittest.main()
