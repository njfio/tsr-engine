"""Authored cross-language contract fixtures, no live provider or target commands."""
import copy
import json
import unittest
import test_engine_b91_project_envelope as base
m=base.m

class Lineage(unittest.TestCase):
    def setUp(self):
        self.f=base.ProjectEnvelope();self.f.setUp();self.addCleanup(self.f.doCleanups)
        self.c=self.f.controller;self.request=self.c.request();self.p=self.f.relation(self.request)['relational']
    def test_real_native_initial_absent_null_and_invalid_development(self):
        for development in ('absent',None,{}):
            p=copy.deepcopy(self.p)
            if development!='absent':p['development']=development
            response=m.local.bind_proposal(self.request,json.dumps({'relational':p}))
            if development=={}:
                with self.assertRaisesRegex(m.StageValidationError,'initial_development_forbidden'):m.validate_relational_lineage(self.request,p,m.STAGES[0])
                with self.assertRaises(ValueError):self.c.validate(response)
            else:
                m.validate_relational_lineage(self.request,p,m.STAGES[0])
                self.assertEqual(self.c.validate(response)['assessments'][0]['disposition'],'deferred')
        self.assertEqual(self.c.request()['state_root'],self.request['state_root'])
    def history(self):
        r=copy.deepcopy(self.request);relation=r['coding']['relational'];original='1'*64;latest='2'*64
        relation['history']=[{'admission':original,'binding':'3'*64,'proposal':copy.deepcopy(self.p),'withdrawn':False,'superseded':True,'current':False},
                             {'admission':latest,'binding':'4'*64,'proposal':{**self.p,'revises':original},'withdrawn':False,'superseded':False,'current':False}]
        p={**self.p,'revises':latest,'development':{'original_revision':original,'prior_revision':latest,
           'retained_meaning':'Fixture meaning','return_reveals':'Fixture return','unresolved':'Unknown','proposed_development':None}}
        return r,p
    def test_revision_references_and_strict_negative_matrix_all_stages(self):
        for stage in m.STAGES:
            r,p=self.history();m.validate_relational_lineage(r,p,stage)
            variants=[('unknown_stale_or_withdrawn_revision',lambda q:q.update(revises='f'*64)),
                      ('unknown_stale_or_withdrawn_revision',lambda q:q.update(revises='1'*64)),
                      ('invalid_revision_ancestry',lambda q:q['development'].update(original_revision='prose')),
                      ('invalid_revision_ancestry',lambda q:q['development'].update(prior_revision=None)),
                      ('revision_development_required',lambda q:q.update(development=None)),
                      ('invalid_parent_references',lambda q:q.update(parents=['f'*64,'e'*64])),
                      ('no_insight_meaning_drift',lambda q:q.update(aspiration='replacement')),
                      ('no_insight_meaning_drift',lambda q:q.update(relationship=q['relationship']+' '))]
            for code,mutate in variants:
                q=copy.deepcopy(p);mutate(q)
                with self.subTest(stage=stage,code=code),self.assertRaisesRegex(m.StageValidationError,code):m.validate_relational_lineage(r,q,stage)
            q=copy.deepcopy(p);q['development'].pop('proposed_development');m.validate_relational_lineage(r,q,stage)
            for change in ('withdrawn','same_binding'):
                changed=copy.deepcopy(r);entry=changed['coding']['relational']['history'][-1]
                if change=='withdrawn':entry['withdrawn']=True
                else:entry['binding']=changed['coding']['relational']['binding']
                self.assertEqual(m.relational_contract(changed)['allowed_lineage'],[])
    def test_independent_chains_and_current_parent_availability(self):
        r,p=self.history();relation=r['coding']['relational']
        other=copy.deepcopy(relation['history'][0]);other.update(admission='5'*64,superseded=False)
        relation['history'].append(other)
        refs=m.relational_contract(r)['allowed_lineage']
        self.assertEqual({x['original_revision'] for x in refs},{'1'*64,'5'*64})
        forged=copy.deepcopy(p);forged['development']['original_revision']='5'*64
        with self.assertRaisesRegex(m.StageValidationError,'invalid_revision_ancestry'):m.validate_relational_lineage(r,forged,m.STAGES[0])
        relation['inputs']['parents']=relation['inputs']['parents'][:1]
        self.assertEqual(m.relational_contract(r)['allowed_lineage'],[])
        with self.assertRaisesRegex(m.StageValidationError,'invalid_parent_references'):m.validate_relational_lineage(r,p,m.STAGES[0])
    def test_all_stage_projections_explicit_initial_no_fabricated_references(self):
        fake=self.f.fake()
        for stage in m.STAGES:
            request,body=m.project(self.f.journal,fake,stage)
            ctx=json.loads(body['messages'][1]['content']);contract=ctx['relational_contract']
            self.assertEqual(contract['allowed_lineage'],[])
            self.assertEqual(contract['initial']['revises'],None)
            self.assertEqual(contract['source_binding'],request['coding']['relational']['binding'])
            self.assertEqual(contract['state_root'],request['state_root'])
            self.assertEqual(contract['parent_ids'],self.p['parents'])
            self.assertIn('never prose',body['messages'][0]['content'])
            self.assertIn('proposed_development null/absent requires copying prior relationship, aspiration, discriminator verbatim',contract['revision']['no_new_insight'])
            self.assertIn('exact equality',contract['revision']['no_new_insight'])
if __name__=='__main__':unittest.main()
