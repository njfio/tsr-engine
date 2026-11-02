"""Offline control tests; these do not count as model-generation evidence."""
import importlib.util
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('directions', Path(__file__).with_name('engine-directions.py'))
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)


class Response:
    def __enter__(self): return self
    def __exit__(self, *args): pass
    def read(self, limit):
        return json.dumps({'models': [{'name': 'installed', 'size': 10, 'digest': 'test'}]}).encode()


class DirectionsControls(unittest.TestCase):
    def invoke(self, path, events, calls=2, state="satisfied", batch=None, batch_calls=None):
        (path / 'local-directions.jsonl').write_text(''.join(json.dumps(e) + '\n' for e in events))
        args = SimpleNamespace(directory=path, model='installed', calls=calls, endpoint='http://127.0.0.1:11435', batch=batch, batch_calls=batch_calls)
        with patch.object(d.urllib.request, 'build_opener') as build, patch.object(d, 'cli') as cli:
            build.return_value.open.return_value = Response()
            cli.return_value = {'status': state}
            result = d.run(args)
            self.assertEqual(build.return_value.open.call_count, 1)  # local inventory only
            return result

    def config(self):
        return {'status': 'config', 'model': 'installed', 'endpoint': 'http://127.0.0.1:11435', 'calls': 2}

    def test_controller_refusal_prefix_preserves_fail_fast_before_second_record(self):
        with tempfile.TemporaryDirectory() as tmp:
            events=[self.config(), {'status':'started','phase':'assessment'},
                    {'status':'possibilities_rejected','state_root':'r', 'stop_on_invalid':True}]
            result=self.invoke(Path(tmp),events,state='deliberate')
            self.assertEqual(result['status'],'invalid_offer_stop')

    def forecast_state(self):
        return {'state_root':'r1','obligation_ids':['original','child'], 'obligation_roots':[0,0],
                'selected_desires':[0,1], 'observation':[0,0], 'notebook':{'known':[]}, 'available_notes':[{'id':'a','topic':'x'},{'id':'b','topic':'x'}],
                'standing_desires':[{'aim':{'predicate':'account','topic':'x'}},{'aim':{'predicate':'include_notes','notes':['a']}}]}

    def forecast_offer(self):
        return {'state_root':'r1','directions':[{'purpose':'generated', 'supports':['child','original'],
            'forecasts':{'child':1.0,'original':0.5}, 'action':{'operation':'compose','title':'x','claims':['a']}}]}

    def test_identified_forecasts_are_order_independent_and_do_not_modify_raw_offer(self):
        state=self.forecast_state(); raw=self.forecast_offer()
        result=d.normalize_forecasts(state,raw)
        self.assertEqual(result['directions'][0]['anticipated'],[0.5,1.0])
        self.assertEqual(result['directions'][0]['supports'],[1,0])
        self.assertNotIn('anticipated',raw['directions'][0])
        raw['directions'][0]['forecasts']={'original':0.5,'child':1.0}
        self.assertEqual(d.normalize_forecasts(state,raw),result)
        self.assertEqual(d.normalize_forecasts(state,raw),result)

    def test_identified_forecasts_reject_stale_missing_unknown_duplicate_and_nonfinite(self):
        import copy
        state=self.forecast_state()
        for mutation in (lambda x:x.update(state_root='old'),
                         lambda x:x['directions'][0]['forecasts'].pop('child'),
                         lambda x:x['directions'][0]['forecasts'].update(unknown=0),
                         lambda x:x['directions'][0]['forecasts'].update(child=float('nan')),
                         lambda x:x['directions'][0]['forecasts'].update(child=10**400),
                         lambda x:x['directions'][0].update(supports=['child','child'])):
            offer=copy.deepcopy(self.forecast_offer()); mutation(offer)
            with self.assertRaises(ValueError): d.normalize_forecasts(state,offer)
        with self.assertRaises(ValueError):
            d.local.runner.codec.strict_json('{"child":0,"child":1}')

    def test_refinement_forecast_consistency_is_rejection_not_correction(self):
        state=self.forecast_state(); offer=self.forecast_offer()
        offer['directions'][0]['forecasts']['original']=0
        with self.assertRaisesRegex(ValueError,'subset expected covered count'):
            d.normalize_forecasts(state,offer)
        self.assertEqual(offer['directions'][0]['forecasts']['original'],0)
        offer['directions'][0]['forecasts']['original']=0.9
        self.assertEqual(d.normalize_forecasts(state,offer)['directions'][0]['anticipated'],[0.9,1.0])
        state['standing_desires'][1]['aim']['notes']=['a','b']
        with self.assertRaisesRegex(ValueError,'subset expected covered count'):
            d.normalize_forecasts(state,offer)

    def test_inactive_inspection_want_does_not_disable_account_prerequisite(self):
        state={'selected_desires':[1,2],'active_desires':[1,2],'notebook':{'known':[]},
               'standing_desires':[{'aim':{'predicate':'understand'}},{'aim':{'predicate':'account'}},{'aim':{'predicate':'concise'}}]}
        self.assertIn('continuation',d.schema(state)['properties']['directions']['items']['required'])
        state['active_desires']=[0,1,2]
        self.assertNotIn('continuation',d.schema(state)['properties']['directions']['items']['required'])

    def test_inactive_refinement_does_not_constrain_current_original_forecast(self):
        state=self.forecast_state();state['active_desires']=[0];state['selected_desires']=[0]
        offer=self.forecast_offer();node=offer['directions'][0];node['supports']=['original']
        node['forecasts']={'original':1,'child':0}
        self.assertEqual(d.normalize_forecasts(state,offer)['directions'][0]['anticipated'],[1,0])
        context=d.identified_context(state,dict(state,recent_returns=[]))
        self.assertFalse(context['obligations'][1]['active'])
        self.assertTrue(context['obligations'][0]['active'])
        node['supports']=['child']
        with self.assertRaisesRegex(ValueError,'unselected support'):d.normalize_forecasts(state,offer)

    def test_first_and_terminal_forecasts_remain_separate(self):
        state=self.forecast_state(); offer=self.forecast_offer(); node=offer['directions'][0]
        node['continuation']={'action':node['action'], 'forecasts':dict(node['forecasts'])}
        node['action']={'operation':'inspect','notes':['a']}
        node['forecasts']={'original':0,'child':0}
        converted=d.normalize_forecasts(state,offer)['directions'][0]
        self.assertEqual(converted['anticipated'],[0,0])
        self.assertEqual(converted['continuation']['anticipated'],[0.5,1])
        node['continuation']['forecasts']['original']=0
        with self.assertRaisesRegex(ValueError,'terminal'):
            d.normalize_forecasts(state,offer)

    def test_complement_bound_and_malformed_shapes_are_terminal_validation_errors(self):
        import copy
        state=self.forecast_state(); offer=self.forecast_offer()
        offer['directions'][0]['forecasts']={'original':1,'child':0}
        with self.assertRaisesRegex(ValueError,'complement capacity'): d.normalize_forecasts(state,offer)
        for mutate in (lambda x:x.update(directions=None), lambda x:x.update(directions=[None]),
                       lambda x:x['directions'][0].pop('supports'),
                       lambda x:x['directions'][0].update(supports=[[]]),
                       lambda x:x['directions'][0].update(continuation=None)):
            bad=copy.deepcopy(self.forecast_offer()); mutate(bad)
            with self.assertRaises(ValueError): d.normalize_forecasts(state,bad)

    def test_model_context_labels_historical_forecasts_and_parent_identity(self):
        state=self.forecast_state(); state['standing_desires'][1]['parents']=[0]
        context=dict(state,observation_updates=[{'before':[0,0],'after':[0.5,1]}],recent_returns=[{'before':[0,0],'after':[0.5,1], 'contrast':[0],
             'prior_plan':{'anticipated':[0.5,1],'action':{'operation':'compose'}}}])
        result=d.identified_context(state,context)
        self.assertEqual(result['obligations'][1]['parents'],['original'])
        self.assertEqual(result['recent_returns'][0]['prior_plan']['forecasts'],{'original':0.5,'child':1})
        self.assertEqual(result['recent_returns'][0]['contrast'],['original'])
        self.assertNotIn('observation',result)
        self.assertEqual(result['observation_updates'][0]['after'],{'original':0.5,'child':1})
        self.assertEqual(context['recent_returns'][0]['prior_plan']['anticipated'],[0.5,1])

    def test_identified_schema_binds_current_version_and_all_ids(self):
        state=self.forecast_state(); schema=d.forecast_schema(state)
        self.assertEqual(schema['properties']['state_root']['const'],'r1')
        self.assertEqual(schema['properties']['directions']['maxItems'],2)
        props=schema['properties']['directions']['items']['properties']
        self.assertNotIn('anticipated',props)
        self.assertEqual(props['forecasts']['required'],['original','child'])
        self.assertEqual(props['supports']['items']['enum'],['original','child'])
        # Refinement admission appends an identity without changing old identities.
        state['obligation_ids'].append('later');state['standing_desires'].append({'aim':{'predicate':'concise'}})
        self.assertEqual(d.forecast_schema(state)['properties']['directions']['items']['properties']['forecasts']['required'],['original','child','later'])

    def test_restart_cannot_raise_frozen_call_budget(self):
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaisesRegex(ValueError, 'frozen local run budget'):
                self.invoke(Path(tmp), [self.config()], calls=3)

    def test_uncertain_call_is_never_duplicated(self):
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaisesRegex(ValueError, 'uncertain previous local call'):
                self.invoke(Path(tmp), [self.config(), {'status': 'started'}])

    def test_exhausted_budget_does_not_generate(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.invoke(Path(tmp), [self.config(), {'status': 'started'}, {'status': 'returned'}, {'status': 'started'}, {'status': 'returned'}], state='deliberate')
            self.assertEqual(result['status'], 'local_call_budget')

    def test_last_budgeted_call_can_still_be_terminal(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = self.invoke(Path(tmp), [self.config(), {'status': 'started'}, {'status': 'returned'}, {'status': 'started'}, {'status': 'returned'}])
            self.assertEqual(result['status'], 'satisfied')

    def test_observed_return_clears_stale_rejection_feedback(self):
        old={'status':'rejected','reason':'old state'}
        current={'status':'rejected','reason':'current state'}
        self.assertEqual(d.pending_rejections([old,{'status':'returned'},current]),[current])
        self.assertEqual(d.pending_rejections([old,{'status':'returned'}]),[])

    def test_provider_schema_uses_current_eligible_indices_without_supplying_actions(self):
        state={'selected_desires':[1,4], 'standing_desires':[{}]*5}
        properties=d.schema(state)['properties']['directions']['items']['properties']
        self.assertEqual(properties['supports']['items']['enum'],[1,4])
        self.assertEqual(properties['anticipated']['minItems'],5)
        self.assertEqual(properties['anticipated']['maxItems'],5)
        self.assertNotIn('enum',properties['action'])
        future=properties['continuation']['properties']
        self.assertEqual(future['anticipated']['minItems'],5)
        self.assertEqual(future['anticipated']['maxItems'],5)
        self.assertNotIn('enum',future['action'])
        self.assertNotIn('continuation',d.schema(state)['properties']['directions']['items']['required'])

    def test_account_only_contract_requires_plan_when_no_direct_action_is_available(self):
        state={'selected_desires':[0,1], 'notebook':{'known':[]}, 'observation':[0,0], 'standing_desires':[
            {'aim':{'predicate':'account'}}, {'aim':{'predicate':'concise'}}]}
        item=d.schema(state)['properties']['directions']['items']
        self.assertIn('continuation',item['required'])
        self.assertNotIn('enum',item['properties']['anticipated']['items'])
        self.assertNotIn('enum',item['properties']['continuation']['properties']['anticipated']['items'])
        self.assertNotIn('enum',item['properties']['action'])
        state['notebook']['known']=['a']
        self.assertNotIn('continuation',d.schema(state)['properties']['directions']['items']['required'])
        state['notebook']['known']=[]
        state['standing_desires'][0]['aim']['predicate']='understand'
        self.assertNotIn('continuation',d.schema(state)['properties']['directions']['items']['required'])

    def test_observation_rederives_eligible_actions_without_prescribing_a_candidate(self):
        state={'selected_desires':[0], 'standing_desires':[{'aim':{'predicate':'account'}}],
               'notebook':{'known':[]},'observation':[0], 'available_notes':[{'id':'a'},{'id':'b'}]}
        def props(): return d.schema(state)['properties']['directions']['items']['properties']
        p=props()
        self.assertEqual(len(p['action']['anyOf']),1)
        self.assertEqual(p['action']['anyOf'][0]['properties']['notes']['items']['enum'],['a','b'])
        self.assertEqual(p['continuation']['properties']['action']['properties']['claims']['items']['enum'],['a','b'])
        state['notebook']['known']=['a']
        p=props();self.assertEqual(len(p['action']['anyOf']),2)
        self.assertEqual(p['action']['anyOf'][0]['properties']['notes']['items']['enum'],['b'])
        self.assertEqual(p['action']['anyOf'][1]['properties']['claims']['items']['enum'],['a'])
        state['notebook']['known']=['a','b']
        p=props();self.assertEqual(len(p['action']['anyOf']),1)
        self.assertNotIn('continuation',p)
        self.assertEqual(p['action']['anyOf'][0]['properties']['operation']['const'],'compose')
        self.assertNotIn('const',p['action']['anyOf'][0]['properties']['title'])
        self.assertNotIn('const',p['action']['anyOf'][0]['properties']['claims'])

    def test_singleton_frontier_permits_one_real_alternative(self):
        state={'selected_desires':[0], 'standing_desires':[{'aim':{'predicate':'account'}}],
               'notebook':{'known':[]},'observation':[0], 'available_notes':[{'id':'a'}]}
        self.assertEqual(d.schema(state)['properties']['directions']['minItems'],1)
        state['available_notes'].append({'id':'b'})
        self.assertEqual(d.schema(state)['properties']['directions']['minItems'],2)

    def test_single_remaining_support_cannot_repeat_even_without_unique_items_support(self):
        state={'selected_desires':[1], 'standing_desires':[{}]*5}
        support=d.schema(state)['properties']['directions']['items']['properties']['supports']
        # Local grammar backends may ignore uniqueItems. Cardinality still excludes
        # the observed [1,1] offer without changing or supplying any action.
        self.assertEqual(support['minItems'],1)
        self.assertEqual(support['maxItems'],1)
        self.assertEqual(support['items']['enum'],[1])

    def test_refinement_schema_preserves_unsupported_parent_uncertainty(self):
        state={'recent_returns':[{'contrast':[0,1]}], 'standing_desires':[{'aim':{'predicate':'account'}},{'aim':{'predicate':'concise'}}], 'observation':[0.5,0.5]}
        array=d.refinement_schema(state)['properties']['refinements']
        self.assertEqual(array['items']['properties']['parent']['enum'],[0])
        state['observation'][0]=1.0
        self.assertEqual(d.refinement_schema(state)['properties']['refinements']['maxItems'],0)

    def test_explicit_batch_preserves_history_and_is_idempotent(self):
        history=[self.config(),{'status':'started'},{'status':'returned'},{'status':'started'},{'status':'rejected'}]
        limit,grant=d.batch_allowance(history,'verify-schema',3,2)
        self.assertEqual(limit,5)
        self.assertEqual(len(history),5)
        self.assertEqual(d.batch_allowance(history+[grant],'verify-schema',3,2),(5,None))
        with self.assertRaisesRegex(ValueError,'frozen'):
            d.batch_allowance(history+[grant],'verify-schema',4,2)
        with self.assertRaisesRegex(ValueError,'existing allowance'):
            d.batch_allowance(history+[grant],'another',3,2)

    def test_new_allowance_cannot_bypass_uncertain_call(self):
        with tempfile.TemporaryDirectory() as tmp:
            with self.assertRaisesRegex(ValueError,'uncertain previous local call'):
                self.invoke(Path(tmp),[self.config(),{'status':'started'}],batch='resume',batch_calls=2)

    def test_satisfied_restart_does_not_generate(self):
        with tempfile.TemporaryDirectory() as tmp:
            self.assertEqual(self.invoke(Path(tmp), [self.config()])['status'], 'satisfied')

    def test_explicit_provider_failure_is_terminal_but_timeout_remains_uncertain(self):
        for error,terminal in [(d.urllib.error.HTTPError('http://127.0.0.1:11435/api/chat',500,'failed',{},None),True),(TimeoutError('timeout'),False)]:
            with tempfile.TemporaryDirectory() as tmp:
                directory=Path(tmp)
                args=SimpleNamespace(directory=directory,model='installed',calls=1,endpoint='http://127.0.0.1:11435')
                state={'status':'deliberate','state_root':'root','recent_returns':[], 'selected_desires':[0], 'standing_desires':[{'aim':{'predicate':'concise'}}], 'obligation_ids':['o'], 'obligation_roots':[0], 'observation':[0]}
                with patch.object(d.urllib.request,'build_opener') as build, patch.object(d,'cli',return_value=state) as cli:
                    build.return_value.open.side_effect=[Response(),error]
                    if terminal:
                        self.assertEqual(d.run(args)['status'],'provider_failed')
                    else:
                        with self.assertRaises(TimeoutError): d.run(args)
                    self.assertTrue(all(call.args[1]=='direction-request' for call in cli.call_args_list))
                events=[json.loads(l) for l in (directory/'local-directions.jsonl').read_text().splitlines()]
                self.assertEqual(sum(e['status']=='started' for e in events),1)
                self.assertEqual(events[-1]['status'],'provider_failed' if terminal else 'started')
                with patch.object(d.urllib.request,'build_opener') as build, patch.object(d,'cli',return_value=state):
                    build.return_value.open.return_value=Response()
                    if terminal:
                        self.assertEqual(d.run(args)['status'],'local_call_budget')
                    else:
                        with self.assertRaisesRegex(ValueError,'uncertain'): d.run(args)
                    self.assertEqual(build.return_value.open.call_count,1)

    def test_stop_during_inference_preserves_offer_without_applying_action(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp); stop=directory/'collector.stop'
            args=SimpleNamespace(directory=directory,model='installed',calls=1,endpoint='http://127.0.0.1:11435',stop_file=stop)
            state={'status':'deliberate','state_root':'root','recent_returns':[], 'selected_desires':[0], 'standing_desires':[{'aim':{'predicate':'concise'}}], 'obligation_ids':['o'], 'obligation_roots':[0], 'observation':[0]}
            class Generated(Response):
                def read(self, limit):
                    stop.touch()
                    return json.dumps({'done':True,'done_reason':'stop','message':{'content':json.dumps({'directions':[], 'state_root':'root'})}}).encode()
            with patch.object(d.urllib.request,'build_opener') as build, patch.object(d,'cli',return_value=state) as cli:
                build.return_value.open.side_effect=[Response(),Generated()]
                self.assertEqual(d.run(args)['status'],'operator_stop')
                self.assertTrue(all(call.args[1]=='direction-request' for call in cli.call_args_list))
            events=[json.loads(l) for l in (directory/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual(events[-1]['status'],'stopped_before_apply')
            self.assertEqual(sum(e['status']=='started' for e in events),1)
            self.assertTrue((directory/'directions-1.json').exists())
            stop.unlink()
            with patch.object(d.urllib.request,'build_opener') as build, patch.object(d,'cli',return_value=state) as cli:
                build.return_value.open.return_value=Response()
                self.assertEqual(d.run(args)['status'],'local_call_budget')
                self.assertEqual(build.return_value.open.call_count,1)
                self.assertTrue(all(call.args[1]=='direction-request' for call in cli.call_args_list))

    def test_stop_after_durable_intent_prevents_inference_dispatch(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp); stop=directory/'collector.stop'
            args=SimpleNamespace(directory=directory,model='installed',calls=1,endpoint='http://127.0.0.1:11435',stop_file=stop)
            state={'status':'deliberate','state_root':'root','recent_returns':[], 'selected_desires':[0], 'standing_desires':[{'aim':{'predicate':'concise'}}], 'obligation_ids':['o'], 'obligation_roots':[0], 'observation':[0]}
            count=0
            def stopping(fd):
                nonlocal count
                count+=1
                if count==2: stop.touch()
            with patch.object(d.os,'fsync',side_effect=stopping), patch.object(d.urllib.request,'build_opener') as build, patch.object(d,'cli',return_value=state):
                build.return_value.open.return_value=Response()
                self.assertEqual(d.run(args)['status'],'operator_stop')
                self.assertEqual(build.return_value.open.call_count,1) # inventory, no inference
            events=[json.loads(l) for l in (directory/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual(events[-1]['status'],'stopped_before_dispatch')


class ActionContracts(unittest.TestCase):
    def state(self):
        state=DirectionsControls().forecast_state()
        state['notebook']['known']=['a']
        state['available_notes'].append({'id':'c','topic':'x'})
        return state

    def plan(self):
        return {'directions':[{'action':{'operation':'inspect','notes':['b']},
            'continuation':{'action':{'operation':'compose','title':'generated','claims':['a','b']}}}]}

    def test_disjoint_schema_never_allows_compose_continuation(self):
        schema=d.direction_request_schema(self.state())
        variants=schema['properties']['directions']['items']['anyOf']
        self.assertEqual(len(variants),2)
        for variant in variants:
            operation=variant['properties']['action']['properties']['operation']['const']
            if operation=='compose':
                self.assertNotIn('continuation',variant['properties'])
                self.assertFalse(variant['additionalProperties'])
            elif 'continuation' in variant['properties']:
                self.assertIn('continuation',variant['required'])
                self.assertEqual(variant['properties']['continuation']['properties']['action']['properties']['operation']['const'],'compose')
        # No known evidence: only genuinely planned inspection for coverage wants.
        state=self.state();state['notebook']['known']=[]
        variants=d.direction_request_schema(state)['properties']['directions']['items']['anyOf']
        self.assertEqual(len(variants),1)
        self.assertIn('continuation',variants[0]['required'])
        # All known: direct composition only.
        state['notebook']['known']=['a','b','c']
        variants=d.direction_request_schema(state)['properties']['directions']['items']['anyOf']
        self.assertEqual(len(variants),1)
        self.assertNotIn('continuation',variants[0]['properties'])

    def test_direct_inspection_requires_a_selected_unsatisfied_knowledge_predicate(self):
        state=self.state()
        self.assertFalse(d.inspection_can_support(state))
        state['standing_desires'][0]['aim']={'predicate':'understand','topic':'x'}
        self.assertTrue(d.inspection_can_support(state))
        variants=d.direction_request_schema(state)['properties']['directions']['items']['anyOf']
        self.assertEqual(len(variants),3)
        state['standing_desires'][0]['aim']['topic']='unrelated'
        self.assertFalse(d.inspection_can_support(state))
        state['standing_desires'][0]['aim']={'predicate':'inspect_notes','notes':['a']}
        self.assertFalse(d.inspection_can_support(state))
        state['standing_desires'][0]['aim']['notes']=['b']
        self.assertTrue(d.inspection_can_support(state))
        state['active_desires']=[1]
        self.assertFalse(d.inspection_can_support(state))
        state['active_desires']=[0,1];state['obligation_roots']=[0,1];state['selected_desires']=[1]
        self.assertFalse(d.inspection_can_support(state))
        # A selected refinement inherits its original's knowledge scope, not its prose.
        state['standing_desires'][0]['aim']={'predicate':'understand','topic':'x'}
        state['obligation_roots']=[0,0]
        self.assertTrue(d.inspection_can_support(state))
        state['active_desires']=[1]
        self.assertFalse(d.inspection_can_support(state))

    def test_dependencies_are_validated_without_repairing_candidates(self):
        import copy
        offer=self.plan();before=copy.deepcopy(offer)
        d.validate_action_contract(self.state(),offer)
        self.assertEqual(offer,before)
        for change in [
            lambda n:n.update(action={'operation':'compose','title':'x','claims':['a']}),
            lambda n:n['action'].update(notes=['a']),
            lambda n:n['action'].update(notes=['b','b']),
            lambda n:n['action'].update(notes=['outside']),
            lambda n:n['continuation']['action'].update(claims=['a']),
            lambda n:n['continuation']['action'].update(claims=['c']),
            lambda n:n['continuation'].update(action={'operation':'inspect','notes':['c']}),
            lambda n:n.update(continuation=None),
            lambda n:n['action'].update(command='forbidden')]:
            bad=copy.deepcopy(offer);change(bad['directions'][0]);snapshot=copy.deepcopy(bad)
            with self.assertRaises(ValueError): d.validate_action_contract(self.state(),bad)
            self.assertEqual(bad,snapshot)
        for value in [None,{},'bad',[None]]:
            with self.assertRaises(ValueError): d.validate_action_contract(self.state(),{'directions':value})

    def test_direct_actions_and_uncertain_predictions_are_not_rewritten(self):
        state=self.state()
        for action in [{'operation':'inspect','notes':['b','c']},
                       {'operation':'compose','title':'independently generated title','claims':['a']}]:
            offer={'directions':[{'action':action,'forecasts':{'original':0.42,'child':0.71}}]}
            d.validate_action_contract(state,offer)
            self.assertEqual(offer['directions'][0]['forecasts'],{'original':0.42,'child':0.71})


class RevisionContext(unittest.TestCase):
    def test_measured_contract_does_not_reveal_uninspected_content(self):
        state={'notebook':{'known':['a'],'title':'A title'},'available_notes':[
            {'id':'a','text':'three source words'}, {'id':'b','text':None}]}
        result=d.action_contract(state)
        self.assertEqual(result['known_claim_word_counts'],{'a':3,'b':None})
        self.assertEqual(result['current_title_word_count'],2)
        self.assertEqual(result['inspect']['preserves'],['claims','title'])
        self.assertEqual(result['compose']['preserves'],['known'])
        self.assertEqual(result['compose']['continuation'],'forbidden')
        state['available_notes'][1]['text']='still hidden despite accidentally provided text'
        self.assertIsNone(d.action_contract(state)['known_claim_word_counts']['b'])

    def test_rejected_proposal_is_version_bound_and_clears_after_return(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)
            offer={'state_root':'r1','directions':[{'action':{'operation':'inspect','notes':['a']}}]}
            (path/'directions-1.json').write_text(json.dumps(offer))
            events=[{'status':'started'},{'status':'rejected'}]
            self.assertEqual(d.last_rejected_proposal(path,events,{'state_root':'r1'})['directions'],offer['directions'])
            self.assertIn('legacy',d.last_rejected_proposal(path,events,{'state_root':'r1'})['integrity'])
            events[-1]['proposal_sha256']='changed'
            with self.assertRaisesRegex(ValueError,'digest mismatch'):
                d.last_rejected_proposal(path,events,{'state_root':'r1'})
            events[-1]['proposal_sha256']=''
            with self.assertRaisesRegex(ValueError,'digest mismatch'):
                d.last_rejected_proposal(path,events,{'state_root':'r1'})
            events[-1].pop('proposal_sha256')
            self.assertIsNone(d.last_rejected_proposal(path,events,{'state_root':'r2'}))
            self.assertIsNone(d.last_rejected_proposal(path,events+[{'status':'returned'}],{'state_root':'r1'}))


if __name__ == '__main__':
    unittest.main()

class InterruptedGeneration(unittest.TestCase):
    def setup_run(self, directory):
        payload={'messages':[{'content':'observation'}]}
        digest=d.hashlib.sha256(json.dumps(payload,sort_keys=True).encode()).hexdigest()
        name='request-1-test.json'; (directory/name).write_text(json.dumps(payload))
        intent={'status':'started','state_root':'r','phase':'direction','request_file':name,'envelope_sha256':digest}
        events=[{'status':'config','calls':1},intent]
        (directory/'local-directions.jsonl').write_text(''.join(json.dumps(e)+'\n' for e in events))
        return events

    def test_before_send_and_after_send_are_both_unknown_and_consume_slot(self):
        # A durable intent cannot prove whether bytes reached the provider.
        for phase in ('before-send','after-send'):
            with self.subTest(phase=phase), tempfile.TemporaryDirectory() as tmp:
                directory=Path(tmp);self.setup_run(directory)
                with patch.object(d,'cli',return_value={'state_root':'r'}):
                    receipt=d.reconcile_generation(directory,'driver ended; generation abandoned')
                self.assertEqual(receipt['provider_outcome'],'unknown')
                self.assertEqual(receipt['budget_slot'],'consumed')
                events=[json.loads(x) for x in (directory/'local-directions.jsonl').read_text().splitlines()]
                self.assertEqual(sum(e['status']=='started' for e in events),1)
                self.assertEqual(d.batch_allowance(events,None,None,1),(1,None))
                with self.assertRaisesRegex(ValueError,'unresolved'), patch.object(d,'cli'):
                    d.reconcile_generation(directory,'repeat')

    def test_saved_response_abandoned_without_applying(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);self.setup_run(directory)
            response=directory/'directions-1.json';response.write_text('{"state_root":"r","directions":[]}')
            original=response.read_bytes()
            with patch.object(d,'cli',return_value={'state_root':'r'}) as cli:
                receipt=d.reconcile_generation(directory,'saved proposal deliberately abandoned')
                cli.assert_called_once_with(directory.resolve(),'direction-request')
            self.assertIn('not applied',receipt['provider_outcome'])
            self.assertEqual(response.read_bytes(),original)

    def test_raw_provider_response_before_validation_is_preserved_on_abandonment(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);self.setup_run(directory)
            response=directory/'provider-response-1.json'
            response.write_text(json.dumps(d.provider_response_metadata({'done':False,'message':{'content':'unparsed'}})))
            original=response.read_bytes()
            with patch.object(d,'cli',return_value={'state_root':'r'}) as cli:
                receipt=d.reconcile_generation(directory,'response saved before interrupted validation')
                cli.assert_called_once_with(directory.resolve(),'direction-request')
            self.assertIn('not applied',receipt['provider_outcome'])
            self.assertEqual(receipt['budget_slot'],'consumed')
            self.assertEqual(response.read_bytes(),original)
            self.assertIn('provider_response_sha256',receipt)

    def test_applied_or_changed_state_never_marked_unapplied(self):
        for changed, artifact in ((True,False),(False,True)):
            with tempfile.TemporaryDirectory() as tmp:
                directory=Path(tmp);self.setup_run(directory)
                if artifact:(directory/'turn-1.json').write_text('{}')
                before=(directory/'local-directions.jsonl').read_bytes()
                with patch.object(d,'cli',return_value={'state_root':'changed' if changed else 'r'}), self.assertRaises(ValueError):
                    d.reconcile_generation(directory,'investigating')
                self.assertEqual((directory/'local-directions.jsonl').read_bytes(),before)

    def test_relational_interruption_preserves_response_and_refuses_applied_receipt(self):
        for applied in (False,True):
            with tempfile.TemporaryDirectory() as tmp:
                directory=Path(tmp);events=self.setup_run(directory);events[-1]['phase']='relational'
                journal=directory/'local-directions.jsonl';journal.write_text(''.join(json.dumps(e)+'\n' for e in events))
                response=directory/'relational-offer-1.json';response.write_text('{"state_root":"r","proposals":[]}')
                original=response.read_bytes()
                if applied:(directory/'relational-review-1.json').write_text('{}')
                with patch.object(d,'cli',return_value={'state_root':'r'}):
                    if applied:
                        with self.assertRaisesRegex(ValueError,'application receipt'):d.reconcile_generation(directory,'inspect interrupted relational stage')
                    else:
                        receipt=d.reconcile_generation(directory,'abandon saved relation without apply')
                        self.assertIn('not applied',receipt['provider_outcome']);self.assertEqual(receipt['budget_slot'],'consumed')
                self.assertEqual(response.read_bytes(),original)

    def test_changed_envelope_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);self.setup_run(directory)
            (directory/'request-1-test.json').write_text('{}')
            with self.assertRaisesRegex(ValueError,'digest mismatch'):
                d.reconcile_generation(directory,'investigating')

    def test_live_driver_lock_and_terminal_receipts_cannot_be_bypassed(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);self.setup_run(directory)
            journal=directory/'local-directions.jsonl'
            with journal.open('r+') as held:
                d.fcntl.flock(held,d.fcntl.LOCK_EX | d.fcntl.LOCK_NB)
                with self.assertRaises(BlockingIOError), patch.object(d,'cli') as cli:
                    d.reconcile_generation(directory,'must not interrupt live driver')
                cli.assert_not_called()
            for status in ('returned','rejected','stopped_before_dispatch','generation_abandoned'):
                self.setup_run(directory)
                with journal.open('a') as out: out.write(json.dumps({'status':status})+'\n')
                with self.assertRaisesRegex(ValueError,'unresolved'):
                    d.reconcile_generation(directory,'terminal request cannot be reclassified')



class CompactDecisionContext(unittest.TestCase):
    def test_expiration_is_durable_no_refund_and_precedes_new_batch(self):
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory);journal=path/'local-directions.jsonl'
            events=[{'status':'config','calls':2},{'status':'started'},{'status':'generation_abandoned'}]
            journal.write_text(''.join(json.dumps(e)+'\n' for e in events))
            expired=d.expire_allowance(path,'Separate new admission; expire old slot')
            self.assertEqual(expired['calls'],1)
            history=[json.loads(x) for x in journal.read_text().splitlines()]
            self.assertEqual(d.batch_allowance(history,'new',4,2)[0],5)
            self.assertEqual(d.expire_allowance(path,'repeat')['status'],'no_unused_allowance')
            self.assertEqual(history,[json.loads(x) for x in journal.read_text().splitlines()])
            with self.assertRaises(ValueError):d.batch_allowance(history+[expired],None,None,2)
            journal.write_text(json.dumps({'status':'config','calls':2})+'\n'+json.dumps({'status':'started'})+'\n')
            with self.assertRaises(ValueError):d.expire_allowance(path,'uncertain')

    def test_deferred_inspection_frozen_without_terminal_content(self):
        import copy
        state=DirectionsControls().forecast_state()
        state.update(state_root='r1',observation_revision='revision',minimum_directions=1)
        state['notebook']={'known':[], 'claims':[], 'title':''}
        node={'purpose':'Resolve missing admitted evidence','action':{'operation':'inspect','notes':['a']},'deferred_observation':'revision'}
        proposal={'state_root':'r1','directions':[node]}
        d.validate_possibilities(state,proposal,{'prior_rejected_offers':[]})
        variants=d.direction_request_schema(state)['properties']['directions']['items']['anyOf']
        self.assertTrue(any('deferred_observation' in v['required'] for v in variants))
        forecasts={i:0 for i in state['obligation_ids']}
        response={'state_root':'r1','proposal_digest':d.proposal_digest(proposal),'assessments':[{'proposal_id':d.proposal_digest(node),'forecasts':forecasts,'supports':[state['obligation_ids'][0]]}]}
        completed=d.complete_assessment(state,proposal,response)
        self.assertEqual(completed['directions'][0]['action'],node['action'])
        self.assertEqual(d.normalize_forecasts(state,completed)['directions'][0]['deferred_observation'],'revision')
        self.assertNotIn('continuation',completed['directions'][0])
        for bad in ('stale','compose','terminal','known','scope'):
            altered=copy.deepcopy(proposal);n=altered['directions'][0]
            if bad=='stale':n['deferred_observation']='old'
            elif bad=='compose':n['action']={'operation':'compose','title':'x','claims':['a']}
            elif bad=='terminal':n['continuation']={'action':{'operation':'compose','title':'x','claims':['a']}}
            elif bad=='known':n['action']['notes']=[]
            else:n['action']['notes']=['outside']
            with self.assertRaises(ValueError):d.validate_possibilities(state,altered,{'prior_rejected_offers':[]})

    def test_historical_feedback_binds_receipt_and_allows_forecast_correction(self):
        import copy
        import hashlib
        state=DirectionsControls().forecast_state()
        state['state_root']='r1'
        state['notebook']={'known':['a'],'claims':['a'],'title':'existing'}
        state['observation']=[0.5,1]
        node={'action':{'operation':'inspect','notes':['b']},
              'forecasts':{'original':1,'child':1},
              'continuation':{'action':{'operation':'compose','claims':['a','b'],'title':'future'}}}
        offer={'state_root':'r1','directions':[node]}
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)
            raw=json.dumps(offer).encode()
            (path/'directions-1.json').write_bytes(raw)
            events=[{'status':'started'},{'status':'rejected','proposal_sha256':hashlib.sha256(raw).hexdigest()}]
            rejected=d.last_rejected_proposal(path,events,state)
            context=dict(state,recent_returns=[],rejected_proposal_for_feedback=rejected)
            before=copy.deepcopy(context)
            result=d.direction_context(state,context)
            item=result['unchanged_effect_feedback'][0]
            self.assertNotIn('direction_index',item)
            self.assertEqual(item['historical_direction_index'],0)
            self.assertEqual(item['historical_action'],node['action'])
            self.assertEqual(item['historical_continuation_action'],node['continuation']['action'])
            self.assertEqual(item['historical_provenance']['file_sha256'],hashlib.sha256(raw).hexdigest())
            self.assertTrue(item['historical_provenance']['receipt_digest_verified'])
            self.assertIn('may retain the action',item['meaning'])
            self.assertNotIn('generate a different',item['meaning'])
            self.assertEqual(context,before)
            self.assertEqual(result,d.direction_context(state,context))
            self.assertIsNone(d.last_rejected_proposal(path,events,dict(state,state_root='new')))

    def test_residual_projection_preserves_lineage_and_does_not_supply_actions(self):
        import copy
        state=DirectionsControls().forecast_state()
        state['notebook']={'known':['a'],'claims':['a'],'title':'Current title'}
        state['available_notes'][0]['text']='observed evidence'
        state['standing_desires'].append({'aim':{'predicate':'concise','max_words':3}})
        state['obligation_ids'].append('brevity');state['obligation_roots'].append(2)
        state['observation']=[0.5,1,0.75];state['selected_desires']=[0,2]
        context=dict(state,recent_returns=[],tensions=[{'desire':0,'text':'duplicate'}],selection={'settings':{'candidate_kinds':['unavailable']},'kernel':'retained'})
        before=copy.deepcopy(context)
        result=d.direction_context(state,context)
        self.assertEqual(context,before)
        self.assertEqual(result['obligations'][0]['measurement']['missing'],['b'])
        self.assertEqual(result['obligations'][1]['original_id'],'original')
        self.assertFalse(result['obligations'][1]['selected'])
        self.assertEqual(result['obligations'][2]['measurement']['current_words'],4)
        self.assertNotIn('tensions',result)
        self.assertEqual(result['selection'],{'kernel':'retained'})
        self.assertNotIn('directions',result)
        self.assertNotIn('action',result)
        self.assertEqual(result,d.direction_context(state,context))

    def test_noop_feedback_is_bound_to_unchanged_components_not_authored_repair(self):
        state=DirectionsControls().forecast_state()
        state['notebook']={'known':['a'],'claims':['a'],'title':'existing'}
        state['observation']=[0.5,1]
        offer={'directions':[{'action':{'operation':'compose','title':'existing','claims':['a']},
                             'forecasts':{'original':1,'child':1}}]}
        checks=d.unchanged_effect_feedback(state,offer)
        self.assertTrue(checks[0]['same_account'])
        self.assertEqual(checks[0]['unchanged_component_mismatches'],
                         [{'obligation':'original','proposed':1,'unchanged_observation':0.5}])
        offer['directions'][0]['action']['title']='different'
        self.assertEqual(d.unchanged_effect_feedback(state,offer),[])
        offer['directions'][0]['action']={'operation':'inspect','notes':['b']}
        self.assertFalse(d.unchanged_effect_feedback(state,offer)[0]['same_account'])
        self.assertEqual(state['notebook']['title'],'existing')

        for value in (None, 1, {}, [None], [{'action':{'operation':'compose','claims':[{}]},'forecasts':{}}]):
            self.assertEqual(d.unchanged_effect_feedback(state,{'directions':value}),[])

class Reconsideration(unittest.TestCase):
    def node(self):
        return {'purpose':'first wording','action':{'operation':'compose','title':'two words','claims':['a','b']},'forecasts':{'goal':0.5},'supports':['goal']}

    def test_identity_ignores_wording_order_and_forecasts_but_tracks_effects(self):
        import copy
        a=self.node(); b=copy.deepcopy(a)
        b.update(purpose='new words',forecasts={'goal':1},supports=[])
        b['action'].update(title='other words',claims=['b','a'])
        self.assertEqual(d.possibility_identity(a),d.possibility_identity(b))
        b['action']['claims']=['a']
        self.assertNotEqual(d.possibility_identity(a),d.possibility_identity(b))
        b=copy.deepcopy(a);b['action']['title']='one'
        self.assertNotEqual(d.possibility_identity(a),d.possibility_identity(b))
        a={'action':{'operation':'inspect','notes':['x','y']},'continuation':self.node()}
        b=copy.deepcopy(a);b['action']['notes'].reverse()
        self.assertEqual(d.possibility_identity(a),d.possibility_identity(b))
        b['continuation']['action']['claims']=['a']
        self.assertNotEqual(d.possibility_identity(a),d.possibility_identity(b))

    def test_rejected_offer_is_not_all_individual_failures_and_changed_state_reopens(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);offer={'state_root':'old','directions':[self.node()]}
            path=directory/'directions-1.json';path.write_text(json.dumps(offer))
            events=[{'status':'started'},{'status':'rejected','reason':'tsr engine: no supported expected progress: weak-support fallback abstained; desires remain unmet',
                'proposal_sha256':d.hashlib.sha256(path.read_bytes()).hexdigest()}]
            no_progress_reason=events[1]['reason']
            for rejection in ('inconsistent forecasts', 'unknown support identity', 'tsr engine: need at least 2 valid distinct alternatives', 'tsr engine: no supported expected progress: weak-support fallback abstained; estimated benefit insufficient'):
                events[1]['reason']=rejection
                diagnostic=d.reconsideration_context(directory,events,{'state_root':'old','notebook':{'title':'two words','claims':['a','b']}})
                # An effect-changing action must remain eligible even after estimate-driven abstention.
                corrected={'state_root':'old','directions':[self.node()]}
                corrected['directions'][0]['action']['title']='different length now'
                d.require_reconsideration(corrected,diagnostic)
                productive=d.reconsideration_context(directory,events,{'state_root':'old','notebook':{'title':'a longer title','claims':['a','b']}})
                self.assertEqual(productive['prior_rejected_offers'][0]['nonproductive_possibilities'],[])
                d.require_reconsideration(offer,productive)
            events[1]['reason']=no_progress_reason
            history=d.reconsideration_context(directory,events,{'state_root':'old','notebook':{'title':'two words','claims':['a','b']}})
            with self.assertRaisesRegex(ValueError,'repeat mechanically unchanged'):
                d.require_reconsideration(offer,history)
            new=self.node();new['action']['claims']=['a']
            d.require_reconsideration({'directions':[self.node(),new]},history)
            reopened=d.reconsideration_context(directory,events,{'state_root':'changed'})
            self.assertEqual(reopened['prior_rejected_offers'],[])
            d.require_reconsideration(offer,reopened)
            self.assertEqual(history,d.reconsideration_context(directory,events,{'state_root':'old','notebook':{'title':'two words','claims':['a','b']}}))
            path.write_text('{}')
            with self.assertRaisesRegex(ValueError,'digest mismatch'):
                d.reconsideration_context(directory,events,{'state_root':'old','notebook':{'title':'two words','claims':['a','b']}})

    def test_schema_allows_honest_inability_without_fake_direction(self):
        variants=d.proposal_response_schema(DirectionsControls().forecast_state())['anyOf']
        self.assertEqual(variants[1]['required'],['state_root','inability'])
        self.assertFalse(variants[1]['additionalProperties'])
        self.assertEqual(variants[1]['properties']['inability']['maxLength'],512)

    def test_inability_is_terminal_and_never_applied(self):
        class Provider:
            def __init__(self, value):self.value=value
            def __enter__(self):return self
            def __exit__(self,*args):pass
            def read(self,limit):return json.dumps(self.value).encode()
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp)
            state=DirectionsControls().forecast_state();state.update(status='deliberate',recent_returns=[])
            args=SimpleNamespace(directory=directory,model='installed',calls=1,endpoint='http://127.0.0.1:11435')
            response={'done':True,'done_reason':'stop','message':{'content':json.dumps({'state_root':'r1','inability':'No useful different proposal identified.'})}}
            with patch.object(d,'cli',return_value=state) as cli, patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider({'models':[{'name':'installed','size':1}]}),Provider(response)]
                result=d.run(args)
            self.assertEqual(result['status'],'generation_inability')
            self.assertTrue(all(call.args[1]=='direction-request' for call in cli.call_args_list))
            events=[json.loads(x) for x in (directory/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual(events[-1]['status'],'generation_inability')
            self.assertEqual(sum(x['status']=='started' for x in events),1)
            self.assertTrue((directory/'inability-1.json').exists())

class TwoStageProposal(unittest.TestCase):
    def state(self):
        return dict(DirectionsControls().forecast_state(),status='deliberate',recent_returns=[])

    def proposal(self):
        return {'state_root':'r1','directions':[
            {'purpose':'generated A','action':{'operation':'inspect','notes':['a']},'continuation':{'action':{'operation':'compose','title':'A','claims':['a']}}},
            {'purpose':'generated B','action':{'operation':'inspect','notes':['b']},'continuation':{'action':{'operation':'compose','title':'B','claims':['b']}}}]}

    def assessment(self, proposal):
        return {'state_root':'r1','proposal_digest':d.proposal_digest(proposal),'assessments':[
            {'proposal_id':d.proposal_digest(node),'forecasts':{'original':0,'child':0},'supports':['original'],
             'continuation_forecasts':{'original':0.5,'child':1 if i==0 else 0}}
            for i,node in enumerate(proposal['directions'])]}

    def test_assessment_preserves_frozen_actions_and_identity_under_reordering(self):
        import copy
        proposal=self.proposal();before=copy.deepcopy(proposal);assessment=self.assessment(proposal)
        assessment['assessments'].reverse()
        value=d.complete_assessment(self.state(),proposal,assessment)
        self.assertEqual(proposal,before)
        for old,new in zip(proposal['directions'],value['directions']):
            self.assertEqual(old['action'],new['action'])
            self.assertEqual(old['continuation']['action'],new['continuation']['action'])
            self.assertEqual(old['purpose'],new['purpose'])
        d.normalize_forecasts(self.state(),value)

    def test_stale_changed_unknown_duplicate_or_contradictory_assessments_rejected(self):
        import copy
        proposal=self.proposal()
        mutations=[lambda v:v.update(state_root='old'),lambda v:v.update(proposal_digest='changed'),
            lambda v:v['assessments'][0].update(action={'operation':'inspect','notes':['b']}),
            lambda v:v['assessments'][0].update(proposal_id='unknown'),
            lambda v:v['assessments'][0].update(supports=['unknown']),
            lambda v:v['assessments'][0]['continuation_forecasts'].update(original=0,child=1),
            lambda v:v['assessments'].append(copy.deepcopy(v['assessments'][0]))]
        for mutate in mutations:
            value=self.assessment(proposal);mutate(value)
            with self.assertRaises(ValueError):d.complete_assessment(self.state(),proposal,value)
        state=self.state();state['state_root']='new'
        with self.assertRaises(ValueError):d.complete_assessment(state,proposal,self.assessment(proposal))

    def test_first_stage_has_no_forecast_support_and_requires_distinct_effects(self):
        import copy
        p=self.proposal();history={'prior_rejected_offers':[]}
        d.validate_possibilities(self.state(),p,history)
        for field in ('forecasts','supports'):
            changed=copy.deepcopy(p);changed['directions'][0][field]={}
            with self.assertRaises(ValueError):d.validate_possibilities(self.state(),changed,history)
        same=copy.deepcopy(p);same['directions'][1]=copy.deepcopy(same['directions'][0]);same['directions'][1]['purpose']='other wording'
        with self.assertRaisesRegex(ValueError,'action effects'):d.validate_possibilities(self.state(),same,history)
        schema=d.possibility_schema(self.state())['anyOf'][0]
        for variant in schema['properties']['directions']['items']['anyOf']:
            self.assertNotIn('forecasts',variant['properties'])
            self.assertNotIn('supports',variant['properties'])

    def test_different_continuations_do_not_disguise_duplicate_first_action(self):
        import copy
        proposal=self.proposal(); history={'prior_rejected_offers':[]}
        original=copy.deepcopy(proposal)
        proposal['directions'][1]['action']=copy.deepcopy(proposal['directions'][0]['action'])
        proposal['directions'][1]['continuation']['action']={'operation':'compose','title':'Different length','claims':['a']}
        self.assertNotEqual(d.possibility_identity(proposal['directions'][0]),
                            d.possibility_identity(proposal['directions'][1]))
        with self.assertRaisesRegex(ValueError,'distinct first-action'):
            d.validate_possibilities(self.state(),proposal,history)
        # The original distinct admitted inspections remain valid and unchanged.
        d.validate_possibilities(self.state(),original,history)
        self.assertEqual(original,self.proposal())

    def test_restart_and_interrupted_assessment_preserve_frozen_proposal(self):
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);path=directory/'possibilities-1.json';path.write_text(json.dumps(self.proposal()))
            receipt={'status':'possibilities_recorded','request_number':1,'state_root':'r1','response_sha256':d.hashlib.sha256(path.read_bytes()).hexdigest()}
            events=[{'status':'started'},receipt]
            frozen=d.pending_possibilities(directory,events)
            self.assertEqual(frozen,self.proposal())
            events += [{'status':'started','phase':'assessment'}]
            self.assertEqual(d.pending_possibilities(directory,events),frozen)
            events += [{'status':'generation_abandoned','provider_outcome':'unknown'}]
            self.assertEqual(d.pending_possibilities(directory,events),frozen)
            self.assertIsNone(d.pending_possibilities(directory,events+[{'status':'possibilities_stale'}]))
            self.assertIsNone(d.pending_possibilities(directory,events+[{'status':'returned'}]))
            self.assertIsNone(d.pending_possibilities(directory,events+[{'status':'generation_inability'}]))
            path.write_text('{}')
            with self.assertRaisesRegex(ValueError,'digest mismatch'):d.pending_possibilities(directory,events)

    def test_two_budgeted_calls_preserve_candidates_through_controller_submission(self):
        class Provider:
            def __init__(self,value):self.value=value
            def __enter__(self):return self
            def __exit__(self,*args):pass
            def read(self,limit):return json.dumps(self.value).encode()
        proposal=self.proposal();assessment=self.assessment(proposal);state=self.state()
        replies=[{'models':[{'name':'installed','size':1}]}]+[
            {'done':True,'done_reason':'stop','message':{'content':json.dumps(value)}} for value in (proposal,assessment)]
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);applied=[]
            def controller(path,command,*args):
                if command=='direction-request':return state
                self.assertEqual(command,'deliberate')
                value=json.loads(Path(args[1]).read_text());applied.append(value)
                return {'next':state,'turn':{'chosen':0,'before':[0,0],'after':[0,0]}}
            args=SimpleNamespace(directory=directory,model='installed',calls=2,endpoint='http://127.0.0.1:11435',two_stage=True)
            with patch.object(d,'cli',side_effect=controller),patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider(x) for x in replies]
                result=d.run(args)
            self.assertEqual(result['status'],'local_call_budget')
            self.assertEqual(len(applied),1)
            for original,submitted in zip(proposal['directions'],applied[0]['directions']):
                self.assertEqual(original['action'],submitted['action'])
                self.assertEqual(original['continuation']['action'],submitted['continuation']['action'])
            events=[json.loads(x) for x in (directory/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual([x['phase'] for x in events if x['status']=='started'],['possibilities','assessment'])
            self.assertTrue((directory/'assessment-2.json').exists())
            self.assertIsNone(d.pending_possibilities(directory,events))

    def test_controller_abstention_is_terminal_without_claiming_action_return(self):
        class Provider:
            def __init__(self,value):self.value=value
            def __enter__(self):return self
            def __exit__(self,*args):pass
            def read(self,limit):return json.dumps(self.value).encode()
        proposal=self.proposal();assessment=self.assessment(proposal);state=self.state()
        replies=[{'models':[{'name':'installed','size':1}]}]+[
            {'done':True,'done_reason':'stop','message':{'content':json.dumps(value)}} for value in (proposal,assessment)]
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);applied=[]
            def controller(path,command,*args):
                if command=='direction-request':return state
                self.assertEqual(command,'deliberate')
                value=json.loads(Path(args[1]).read_text());applied.append(value)
                return {'status':'selection_abstained','next':state,'selection_decision':{'reason':'near_tie_abstention'}}
            args=SimpleNamespace(directory=directory,model='installed',calls=2,endpoint='http://127.0.0.1:11435',two_stage=True)
            with patch.object(d,'cli',side_effect=controller),patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider(x) for x in replies]
                result=d.run(args)
            self.assertEqual(result['status'],'selection_abstained')
            self.assertEqual(len(applied),1)
            for original,submitted in zip(proposal['directions'],applied[0]['directions']):
                self.assertEqual(original['action'],submitted['action'])
                self.assertEqual(original['continuation']['action'],submitted['continuation']['action'])
            events=[json.loads(x) for x in (directory/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual([x['phase'] for x in events if x['status']=='started'],['possibilities','assessment'])
            self.assertEqual(events[-1]['phase'],'selection_abstention')
            self.assertNotIn('after',events[-1])
            self.assertTrue((directory/'assessment-2.json').exists())
            self.assertIsNone(d.pending_possibilities(directory,events))

    def test_one_slot_freezes_without_applying_or_implicitly_spending_second(self):
        class Provider:
            def __init__(self,value):self.value=value
            def __enter__(self):return self
            def __exit__(self,*args):pass
            def read(self,limit):return json.dumps(self.value).encode()
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);args=SimpleNamespace(directory=directory,model='installed',calls=1,endpoint='http://127.0.0.1:11435',two_stage=True)
            response={'done':True,'done_reason':'stop','message':{'content':json.dumps(self.proposal())}}
            with patch.object(d,'cli',return_value=self.state()) as cli,patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider({'models':[{'name':'installed','size':1}]}),Provider(response)]
                result=d.run(args)
            self.assertEqual(result['status'],'assessment_pending')
            self.assertTrue(all(x.args[1]=='direction-request' for x in cli.call_args_list))
            events=[json.loads(x) for x in (directory/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual(sum(x['status']=='started' for x in events),1)
            self.assertEqual(d.pending_possibilities(directory,events),self.proposal())

    def test_controller_abstention_returns_to_composition_not_endless_reassessment(self):
        class Provider:
            def __init__(self,value):self.value=value
            def __enter__(self):return self
            def __exit__(self,*args):pass
            def read(self,limit):return json.dumps(self.value).encode()
        state=self.state();proposal=self.proposal()
        replies=[{'models':[{'name':'installed','size':1}]}]+[
            {'done':True,'done_reason':'stop','message':{'content':json.dumps(value)}}
            for value in (proposal,self.assessment(proposal),{'state_root':'r1','inability':'No new useful possibilities identified.'})]
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp)
            def controller(path,command,*args):
                if command=='direction-request':return state
                raise ValueError('tsr engine: no supported expected progress: weak-support fallback abstained; desires remain unmet')
            args=SimpleNamespace(directory=directory,model='installed',calls=3,endpoint='http://127.0.0.1:11435',two_stage=True)
            with patch.object(d,'cli',side_effect=controller),patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider(x) for x in replies]
                result=d.run(args)
            self.assertEqual(result['status'],'generation_inability')
            events=[json.loads(x) for x in (directory/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual([x['phase'] for x in events if x['status']=='started'],['possibilities','assessment','possibilities'])
            self.assertTrue(any(x['status']=='possibilities_rejected' for x in events))
            self.assertIsNone(d.pending_possibilities(directory,events))

    def test_assessment_only_never_executes_and_preserves_state(self):
        class Provider:
            def __init__(self,value):self.value=value
            def __enter__(self):return self
            def __exit__(self,*args):pass
            def read(self,limit):return json.dumps(self.value).encode()
        for invalid in [False,True]:
            state=self.state();proposal=self.proposal();assessment=self.assessment(proposal)
            if invalid:assessment['proposal_digest']='wrong'
            replies=[{'models':[{'name':'installed','size':1}]}]+[{'done':True,'done_reason':'stop','message':{'content':json.dumps(v)}} for v in (proposal,assessment)]
            with tempfile.TemporaryDirectory() as tmp:
                directory=Path(tmp)
                def controller(path,command,*args):
                    self.assertEqual(command,'direction-request','comparison cannot execute/refine')
                    return state
                args=SimpleNamespace(directory=directory,model='installed',calls=2,endpoint='http://127.0.0.1:11435',two_stage=True,assess_only=True,possibility_projection='no-historical-narratives-v1')
                with patch.object(d,'cli',side_effect=controller),patch.object(d.urllib.request,'build_opener') as build:
                    build.return_value.open.side_effect=[Provider(x) for x in replies]
                    result=d.run(args)
                self.assertFalse(result['execution_authorized'])
                self.assertEqual(result['status'],'assessment_only_invalid_offer' if invalid else 'assessment_validated_no_execution')
                events=[json.loads(x) for x in (directory/'local-directions.jsonl').read_text().splitlines()]
                self.assertEqual(sum(x['status']=='started' for x in events),2)
                starts=[x for x in events if x['status']=='started']
                saved=json.loads((directory/starts[0]['request_file']).read_text())
                projected=json.loads(saved['messages'][1]['content'])
                self.assertEqual(starts[0]['context_contract'],projected['context_projection']['contract'])
                self.assertEqual(starts[0]['context_contract'],'possibilities-no-historical-narratives@v1')
                full=json.loads((directory/starts[0]['full_possibility_context_file']).read_text())
                assessment_payload=json.loads((directory/starts[1]['request_file']).read_text())
                assessment_context=json.loads(assessment_payload['messages'][1]['content'])
                assessment_context.pop('frozen_possibilities')
                self.assertEqual(assessment_context,full)
                self.assertFalse(any(x['status']=='returned' for x in events))
                if not invalid:self.assertIsNone(d.pending_possibilities(directory,events))


    def test_assessment_only_search_stops_at_first_inability(self):
        class Provider:
            def __init__(self,value):self.value=value
            def __enter__(self):return self
            def __exit__(self,*args):pass
            def read(self,limit):return json.dumps(self.value).encode()
        with tempfile.TemporaryDirectory() as tmp:
            state=self.state();args=SimpleNamespace(directory=Path(tmp),model='installed',calls=2,endpoint='http://127.0.0.1:11435',two_stage=True,assess_only=True,possibility_search='bounded-test',possibility_temperature=0.7,possibility_seed=7301,possibility_samples=2)
            replies=[{'models':[{'name':'installed','size':1}]},{'done':True,'done_reason':'stop','message':{'content':json.dumps({'state_root':'r1','inability':'No valid possibilities generated.'})}}]
            with patch.object(d,'cli',return_value=state) as cli,patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider(x) for x in replies]
                result=d.run(args)
            self.assertEqual(result['status'],'generation_inability')
            self.assertEqual(build.return_value.open.call_count,2) # metadata plus ONE generation
            self.assertTrue(all(call.args[1]=='direction-request' for call in cli.call_args_list))
            events=[json.loads(x) for x in (Path(tmp)/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual(sum(x['status']=='started' for x in events),1)


class DiverseSampling(unittest.TestCase):
    def args(self):
        return SimpleNamespace(two_stage=True, possibility_search='bounded-search',
                               possibility_temperature=0.7, possibility_seed=7301, possibility_samples=3)

    def test_operator_constraint_certificate_is_not_a_generation_hint(self):
        state=DirectionsControls().forecast_state()
        context=dict(state,recent_returns=[],constraint_status={'status':'proven_joint_conflict'})
        self.assertNotIn('constraint_status',d.identified_context(state,context))
        self.assertIn('constraint_status',context)

    def test_frozen_config_and_stage_separation(self):
        args=self.args(); config=d.sampling_config(args,[])
        self.assertEqual(d.sampling_config(args,[config]),config)
        args.possibility_temperature=0.8
        with self.assertRaisesRegex(ValueError,'frozen'): d.sampling_config(args,[config])
        for phase in ['assessment','refinement','direction']:
            options, receipt=d.sampling_options(phase,config,[])
            self.assertEqual(options,{'temperature':0,'num_ctx':32768,'num_predict':2048})
            self.assertIsNone(receipt)
        for value in [float('nan'),float('inf'),0,-1,1.1,True]:
            args=self.args();args.possibility_temperature=value
            with self.assertRaises(ValueError):d.sampling_config(args,[])

    def test_seeds_restart_and_consumed_uncertain_slots(self):
        config=d.sampling_config(self.args(),[]);events=[]
        for index in range(3):
            options,receipt=d.sampling_options('possibilities',config,events)
            self.assertEqual(options['seed'],7301+index)
            self.assertEqual(receipt['sample'],index+1)
            self.assertEqual(d.sampling_options('possibilities',config,list(events)),(options,receipt))
            events.extend([{'status':'started','phase':'possibilities','sampling':receipt},
                           {'status':'generation_abandoned','provider_outcome':'unknown'}])
        with self.assertRaisesRegex(ValueError,'exhausted'):d.sampling_options('possibilities',config,events)
        self.assertEqual(d.sampling_options('assessment',config,events)[0]['temperature'],0)

    def test_bounded_inability_search_preserves_attempts_and_restart_budget(self):
        class Provider:
            def __init__(self,value):self.value=value
            def __enter__(self):return self
            def __exit__(self,*args):pass
            def read(self,limit):return json.dumps(self.value).encode()
        inventory={'models':[{'name':'installed','size':1}]}
        reply={'done':True,'done_reason':'stop','message':{'content':json.dumps({'state_root':'r1','inability':'No alternative found.'})}}
        with tempfile.TemporaryDirectory() as tmp:
            directory=Path(tmp);args=self.args()
            args.directory=directory;args.model='installed';args.calls=2;args.endpoint='http://127.0.0.1:11435'
            state=DirectionsControls().forecast_state();state.update(status='deliberate',recent_returns=[])
            with patch.object(d,'cli',return_value=state) as cli,patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider(inventory),Provider(reply),Provider(reply)]
                result=d.run(args)
                self.assertEqual(result['status'],'local_call_budget')
                self.assertTrue(all(c.args[1]=='direction-request' for c in cli.call_args_list))
            events=[json.loads(x) for x in (directory/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual(sum(e['status']=='generation_inability' for e in events),2)
            self.assertEqual([e['sampling']['options']['seed'] for e in events if e['status']=='started'],[7301,7302])
            self.assertTrue((directory/'inability-1.json').exists() and (directory/'inability-2.json').exists())
            with patch.object(d,'cli',return_value=state),patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider(inventory)]
                self.assertEqual(d.run(args)['status'],'local_call_budget')
                self.assertEqual(build.return_value.open.call_count,1)

class DecisionFeedback(unittest.TestCase):
    def history(self):
        effect={'action':{'operation':'compose','claims':['a'],'title_words':2}}
        return {'state_root':'r1','prior_rejected_offers':[
            {'possibilities':[effect], 'nonproductive_possibilities':[effect],
             'offer_rejection_excerpt':'no supported expected progress',
             'request_number':n,'proposal_sha256':str(n)} for n in [1,2]]}

    def test_dedup_retains_provenance_and_offer_scope(self):
        history=self.history(); summary=d.decision_reconsideration(history)
        self.assertEqual(len(summary['effects']),1)
        group=summary['effects'][0]
        self.assertEqual(len(group['evidence']),2)
        self.assertEqual(len(group['offer_failures']),1)
        self.assertTrue(group['mechanically_unchanged_now'])
        self.assertIn('whole offer',group['offer_failures'][0]['scope'])
        self.assertIn('partial progress',summary['permitted'])
        self.assertEqual(d.decision_reconsideration(history),summary)
        self.assertEqual(len(history['prior_rejected_offers']),2)

    def test_changed_effect_is_not_blanket_rejected_and_changed_state_clears(self):
        history=self.history()
        # Offline counterexample only, never supplied to a live generator.
        revised={'action':{'operation':'compose','claims':['a'],'title_words':3}}
        history['prior_rejected_offers'][0]['possibilities'].append(revised)
        summary=d.decision_reconsideration(history)
        self.assertFalse(summary['effects'][1]['mechanically_unchanged_now'])
        self.assertIn('no observed return',summary['unknown'])
        d.require_reconsideration({'directions':[{'action':{'operation':'compose','claims':['a'],'title':'three word title'}}]},history)
        fresh=d.decision_reconsideration({'state_root':'r2','prior_rejected_offers':[]})
        self.assertEqual(fresh['effects'],[])

    def test_raw_duplicate_feedback_removed_without_losing_measured_feedback(self):
        state=DirectionsControls().forecast_state()
        context=dict(state,recent_returns=[],reconsideration=self.history(),
                     last_rejected_offer='old failure',rejected_proposal_for_feedback=DirectionsControls().forecast_offer())
        result=d.direction_context(state,context)
        self.assertNotIn('last_rejected_offer',result)
        self.assertNotIn('rejected_proposal_for_feedback',result)
        self.assertIn('unchanged_effect_feedback',result)
        self.assertIn('rejected_proposal_for_feedback',context)

class GenerationLimitReport(unittest.TestCase):
    def test_report_binds_state_config_and_outcomes_without_claiming_infeasibility(self):
        search={'name':'trial'}
        started={'status':'started','phase':'possibilities','state_root':'r1',
                 'model':'installed','digest':'digest','envelope_sha256':'request',
                 'sampling':{'search':'trial','sample':1,'options':{'seed':7}}}
        events=[started,{'status':'generation_inability','response_sha256':'response'}]
        report=d.generation_limit_report(search,events,{'state_root':'r1'})
        self.assertEqual(report['attempts'][0]['outcome'],'generation_inability')
        self.assertEqual(report['attempts'][0]['response_sha256'],'response')
        self.assertIn('not infeasibility',report['meaning'])
        self.assertEqual(d.generation_limit_report(search,events,{'state_root':'r2'})['attempts'],[])
        self.assertEqual(d.generation_limit_report({'name':'other'},events,{'state_root':'r1'})['attempts'],[])
        self.assertEqual(events[0],started)
        for status in ['provider_failed','stopped_before_dispatch','stopped_before_apply']:
            terminal=d.generation_limit_report(search,[started,{'status':status}],{'state_root':'r1'})
            self.assertEqual(terminal['attempts'][0]['outcome'],status)

class ProviderComparison(unittest.TestCase):
    def args(self):
        return SimpleNamespace(model='alternative',calls=4,two_stage=True,
                               provider_comparison='compare',provider_digest='pinned')
    def events(self):
        return [{'status':'config','model':'original','endpoint':'local','calls':4}]
    def test_explicit_digest_bound_provider_change_without_state_reset(self):
        args=self.args();events=self.events()
        record=d.provider_comparison(args,events,'local',{'digest':'pinned'},'r1',None)
        self.assertEqual(record['initial_state_root'],'r1')
        self.assertEqual(d.provider_comparison(args,events+[record],'local',{'digest':'pinned'},'r2',None),record)
        self.assertEqual(events,self.events())
        for change in [('provider_comparison',None),('provider_digest','other'),('calls',5),('two_stage',False)]:
            bad=self.args();setattr(bad,*change)
            with self.assertRaises(ValueError):d.provider_comparison(bad,events,'local',{'digest':'pinned'},'r1',None)
        with self.assertRaisesRegex(ValueError,'pending'):
            d.provider_comparison(args,events,'local',{'digest':'pinned'},'r1',{'state_root':'r1'})
        with self.assertRaises(ValueError):d.provider_comparison(args,events,'elsewhere',{'digest':'pinned'},'r1',None)
        args.model='third'
        with self.assertRaisesRegex(ValueError,'frozen'):
            d.provider_comparison(args,events+[record],'local',{'digest':'pinned'},'r1',None)
    def test_carry_adds_only_new_slots_and_freezes_grant(self):
        events=self.events()+[{'status':'started'}]*3
        with self.assertRaises(ValueError):d.batch_allowance(events,'additional',5,4)
        limit,grant=d.batch_allowance(events,'additional',5,4,True)
        self.assertEqual(limit-3,6)
        self.assertEqual(d.batch_allowance(events+[grant],'additional',5,4,True),(9,None))
        with self.assertRaises(ValueError):d.batch_allowance(events+[grant],'additional',6,4,True)

class RejectedProviderEvidence(unittest.TestCase):
    def test_rejected_provider_payload_preserved_before_validation_without_execution(self):
        import hashlib
        class Provider:
            def __init__(self,value):self.value=value
            def __enter__(self):return self
            def __exit__(self,*args):pass
            def read(self,limit):return json.dumps(self.value).encode()
        for response in [
            {'done':False,'done_reason':'length','message':{'content':'partial response'}},
            {'done':True,'done_reason':'stop','message':{'content':'not JSON'}},
            {'done':True,'done_reason':'stop','message':{'content':'{}','tool_calls':[{'function':{'name':'untrusted'}}]}}]:
            with self.subTest(response=response),tempfile.TemporaryDirectory() as tmp:
                directory=Path(tmp);state=DirectionsControls().forecast_state();state.update(status='deliberate',recent_returns=[])
                args=SimpleNamespace(directory=directory,model='installed',calls=1,endpoint='http://127.0.0.1:11435')
                with patch.object(d,'cli',return_value=state) as cli,patch.object(d.urllib.request,'build_opener') as build:
                    build.return_value.open.side_effect=[Provider({'models':[{'name':'installed','size':1}]}),Provider(response)]
                    d.run(args)
                    self.assertTrue(all(c.args[1]=='direction-request' for c in cli.call_args_list))
                events=[json.loads(x) for x in (directory/'local-directions.jsonl').read_text().splitlines()]
                self.assertEqual(events[-1]['status'],'rejected')
                raw=(directory/'provider-response-1.json').read_bytes()
                self.assertEqual(json.loads(raw),d.provider_response_metadata(response))
                self.assertNotIn('message',json.loads(raw))
                self.assertEqual(hashlib.sha256(raw).hexdigest(),events[-1]['provider_response_sha256'])
                self.assertFalse((directory/'turn-1.json').exists())


class PossibilityContextTests(unittest.TestCase):
    def test_state_first_retains_authority_outcomes_and_all_effects_without_forecast_prose(self):
        import copy
        context={'state_root':'state','authority':{'actions':2},'obligations':[{'id':'want','observed':0.5}],
                 'notebook':{'known':['a']},'available_notes':[{'id':'b','text':None}],
                 'action_contract':{'partial_progress':'allowed','losses':'accounted'},
                 'recent_returns':[{'before':{'want':1},'after':{'want':0.5},'progress':False,'contrast':['want'],
                                    'residual':0.2,'generator_residual':{'want':0.2},'prior_plan':{'action':{'operation':'inspect','notes':['b']},'forecasts':{'want':1}}}],
                 'reconsideration':{'effects':[{'effect':{'action':{'operation':'inspect','notes':['b']}},'mechanically_unchanged_now':False,
                    'offer_failures':[{'category':'offer_selection_abstained','scope':'historical','reason_excerpt':'OLD_OBSOLETE_FORECAST'}],
                    'evidence':[{'request_number':7,'proposal_sha256':'hash'}]}]},
                 'forecast_contract':{'legacy':'FORECAST_VALUE'},'unchanged_effect_feedback':['OLD_ERROR']}
        before=copy.deepcopy(context); result=d.possibility_context(context)
        self.assertEqual(context,before)
        self.assertEqual(result,d.possibility_context(context))
        self.assertEqual(list(result)[:3],['authority','state_root','obligations'])
        for key in ['authority','state_root','obligations','notebook','available_notes','action_contract']:
            self.assertEqual(result[key],context[key])
        history=result['recent_observed_returns'][0]
        for key in ['before','after','progress','contrast']:self.assertEqual(history[key],context['recent_returns'][0][key])
        effect=result['historical_reconsideration']['effects'][0]
        for key in ['effect','mechanically_unchanged_now','evidence']:self.assertEqual(effect[key],context['reconsideration']['effects'][0][key])
        self.assertNotIn('OLD_OBSOLETE_FORECAST',json.dumps(result))
        self.assertNotIn('FORECAST_VALUE',json.dumps(result))
        self.assertEqual(result['context_projection']['omitted_fields'],['forecast_contract','unchanged_effect_feedback'])
        changed=copy.deepcopy(context);changed['authority']['actions']=0
        self.assertEqual(d.possibility_context(changed)['authority']['actions'],0)
        self.assertNotEqual(d.possibility_context(changed)['context_projection']['full_context_sha256'],result['context_projection']['full_context_sha256'])

    def test_partial_progress_and_unknown_evidence_are_not_replaced_by_a_candidate(self):
        context={'authority':{'external':False},'state_root':'same','obligations':[{'aim':{'predicate':'account'},'observed':0.5},{'aim':{'predicate':'concise','max_words':4},'observed':0.8}],
                 'available_notes':[{'id':'unread','text':None}],'recent_returns':[], 'minimum_directions':2,
                 'action_contract':{'partial_progress':True,'independent_loss_check':True}}
        result=d.possibility_context(context)
        self.assertEqual(result['obligations'],context['obligations'])
        self.assertEqual(result['available_notes'],context['available_notes'])
        self.assertEqual(result['minimum_directions'],2)
        self.assertNotIn('directions',result)
        self.assertNotIn('forecasts',result)


class HistoricalNarrativeAblationTests(unittest.TestCase):
    def test_explicit_default_unchanged_and_only_narrative_paths_removed(self):
        import copy
        context={'authority':{'actions':2,'external':False},'state_root':'r',
          'obligations':[{'text':'preserve this narrative want','observed':0.5}],
          'available_notes':[{'id':'a','text':None}], 'action_contract':{'cost':2,'loss_limit':0},
          'recent_returns':[{'chosen_direction':'OLD PURPOSE','before':0,'after':0.5,'progress':True,'prior_plan':{'action':{'operation':'inspect','notes':['a']}}}],
          'reconsideration':{'unknown':'no inference from rejection','permitted':'partial progress',
             'effects':[{'effect':{'action':{'operation':'inspect','notes':['a']}},'mechanically_unchanged_now':False,'evidence':['receipt'],'offer_failures':[{'category':'old failure','reason_excerpt':'old reason'}]}]}}
        original=copy.deepcopy(context); baseline=d.possibility_context(context)
        self.assertEqual(d.experimental_possibility_context(context),baseline)
        got=d.experimental_possibility_context(context,'no-historical-narratives-v1')
        expected=copy.deepcopy(baseline)
        expected['recent_observed_returns'][0].pop('chosen_direction')
        expected['historical_reconsideration']['effects'][0].pop('offer_failures')
        expected['historical_reconsideration'].pop('historical_only')
        expected['context_projection']=got['context_projection']
        self.assertEqual(got,expected); self.assertEqual(context,original)
        self.assertEqual(got['context_projection']['full_context_sha256'],baseline['context_projection']['full_context_sha256'])
        self.assertEqual(got,d.experimental_possibility_context(context,'no-historical-narratives-v1'))
        changed=copy.deepcopy(context); changed['authority']['actions']=0
        self.assertEqual(d.experimental_possibility_context(changed,'no-historical-narratives-v1')['authority']['actions'],0)
        with self.assertRaises(ValueError):d.experimental_possibility_context(context,'unknown')

class RelationalDriverTests(unittest.TestCase):
    def state(self):
        state=TwoStageProposal().state()
        state['refinement_needed']=True
        state['relational_request']={'due':True,'response_schema':{'type':'object'},'evidence_key':'evidence','return_commitment':'return','observation_revision':'revision','observed_return':{'commitment':'return','chosen':{'operation':'inspect','notes':['a']},'before':{'w1':0},'after':{'w1':1},'contrast':['w2']}}
        return state

    def test_refinement_priority_preserves_fresh_guidance_and_frozen_assessment(self):
        import copy
        state={'refinement_needed':True,'relational_guidance':[], 'standing_desires':['unchanged'],
               'authority':'unchanged','cycles':1}
        before=copy.deepcopy(state)
        self.assertTrue(d.operational_refinement_due(state,None))
        self.assertFalse(d.operational_refinement_due(state,{'directions':[]}))
        state['relational_guidance']=[{'source':'generated hypothesis'}]
        self.assertFalse(d.operational_refinement_due(state,None))
        self.assertFalse(d.operational_refinement_due(state,{'directions':[]}))
        state['relational_guidance']=[]  # Controller removes stale/withdrawn guidance.
        self.assertTrue(d.operational_refinement_due(state,None))
        self.assertEqual(state,before)
        state['relational_request']={'due':True}
        self.assertFalse(d.operational_refinement_due(state,None))

    def test_relational_update_reaches_next_normal_possibility_context_without_execution(self):
        import copy
        state=self.state(); original=copy.deepcopy(state)
        proposal={'state_root':'r1','return_commitment':'return','observation_revision':'revision','evidence_key':'evidence','proposals':[]}
        captured=[]
        def controller(path,command,*args):
            nonlocal state
            if command=='direction-relate':
                self.assertEqual(json.loads(Path(args[1]).read_text()),proposal)
                state=copy.deepcopy(state);state['state_root']='r2';state['relational_request']['due']=False
                # Scripted mock receipt tests wiring, not model emergence or semantic admission.
                state['relational_guidance']={'status':'provisional','entries':[{'aspiration':'fixture relation','independent_weight':None}]}
                return state
            self.assertEqual(command,'direction-request','no action or refinement permitted in this fixture')
            return state
        class Provider:
            def __init__(self,v):self.v=v
            def __enter__(self):return self
            def __exit__(self,*a):pass
            def read(self,limit):return json.dumps(self.v).encode()
        replies=[{'models':[{'name':'installed','size':1}]}]+[{'done':True,'done_reason':'stop','prompt_eval_count':100,'eval_count':10,'total_duration':12345,'message':{'content':json.dumps(x)}} for x in [proposal,{'state_root':'r2','inability':'fixture stop'}]]
        with tempfile.TemporaryDirectory() as tmp:
            args=SimpleNamespace(directory=Path(tmp),model='installed',calls=2,endpoint='http://127.0.0.1:11435',two_stage=True)
            with patch.object(d,'cli',side_effect=controller),patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider(x) for x in replies]
                result=d.run(args)
            self.assertEqual(result['status'],'generation_inability')
            events=[json.loads(x) for x in (Path(tmp)/'local-directions.jsonl').read_text().splitlines()]
            starts=[x for x in events if x['status']=='started']
            self.assertEqual([x['phase'] for x in starts],['relational','possibilities'])
            self.assertEqual(starts[0]['context_contract'],'relational-development-request@v2')
            self.assertEqual(starts[0]['resource_reservation']['output_tokens'],2048)
            terminal=[e for e in events if e['status']=='returned'][0]
            self.assertEqual(terminal['usage']['reported']['eval_count'],10)
            self.assertEqual(terminal['usage']['status'],'reported')
            self.assertEqual(events[-1]['usage']['reported']['prompt_eval_count'],100)
            relational_payload=json.loads(json.loads((Path(tmp)/starts[0]['request_file']).read_text())['messages'][1]['content'])
            self.assertEqual(relational_payload['relational_request']['observed_return'],original['relational_request']['observed_return'])
            saved=json.loads((Path(tmp)/starts[1]['request_file']).read_text());ctx=json.loads(saved['messages'][1]['content'])
            self.assertEqual(ctx['relational_guidance'],state['relational_guidance'])
            self.assertEqual(state['standing_desires'],original['standing_desires'])
            self.assertNotIn('chosen',events[-1])
            # Exhausted journal cannot repeat relational generation on restart.
            with patch.object(d,'cli',side_effect=controller),patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.return_value=Provider(replies[0]);d.run(args)
                self.assertEqual(build.return_value.open.call_count,1)

    def test_satisfied_state_reviews_once_then_stops_without_action(self):
        import copy
        state=self.state(); state['status']='satisfied'
        state['relational_request']['review_only']=True
        calls=[]
        def controller(path,command,*args):
            nonlocal state
            calls.append(command)
            if command=='direction-relate':
                state=copy.deepcopy(state);state['state_root']='reviewed'
                state['relational_request']['due']=False
            else:self.assertEqual(command,'direction-request')
            return state
        class Provider:
            def __init__(self,v):self.v=v
            def __enter__(self):return self
            def __exit__(self,*a):pass
            def read(self,limit):return json.dumps(self.v).encode()
        response={'done':True,'done_reason':'stop','prompt_eval_count':100,'eval_count':10,
                  'total_duration':12345,'message':{'content':json.dumps({'proposals':[]})}}
        with tempfile.TemporaryDirectory() as tmp:
            args=SimpleNamespace(directory=Path(tmp),model='installed',calls=2,
                endpoint='http://127.0.0.1:11435',two_stage=True)
            with patch.object(d,'cli',side_effect=controller),patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider({'models':[{'name':'installed','size':1}]}),Provider(response)]
                result=d.run(args)
            self.assertEqual(result['status'],'satisfied')
            self.assertEqual(calls.count('direction-relate'),1)
            events=[json.loads(x) for x in (Path(tmp)/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual([e['phase'] for e in events if e['status']=='started'],['relational'])

    def test_invalid_relational_review_fail_fast_persists_across_restart(self):
        state=self.state();state['status']='satisfied'
        def controller(path,command,*args):
            if command=='direction-relate':raise ValueError('fixture invalid proposal')
            self.assertEqual(command,'direction-request')
            return state
        class Provider:
            def __init__(self,v):self.v=v
            def __enter__(self):return self
            def __exit__(self,*a):pass
            def read(self,limit):return json.dumps(self.v).encode()
        tags={'models':[{'name':'installed','size':1}]}
        response={'done':True,'done_reason':'stop','prompt_eval_count':100,'eval_count':10,
                  'total_duration':12345,'message':{'content':json.dumps({'proposals':[]})}}
        with tempfile.TemporaryDirectory() as tmp:
            args=SimpleNamespace(directory=Path(tmp),model='installed',calls=2,
                endpoint='http://127.0.0.1:11435',two_stage=True,stop_on_invalid=True)
            with patch.object(d,'cli',side_effect=controller),patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.side_effect=[Provider(tags),Provider(response)]
                self.assertEqual(d.run(args)['status'],'invalid_offer_stop')
                self.assertEqual(build.return_value.open.call_count,2)
            # The prefix ending at rejection already contains the stop decision;
            # no second write is needed before a crash/restart.
            prefix=[json.loads(x) for x in (Path(tmp)/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual(prefix[-1]['status'],'rejected')
            self.assertIs(prefix[-1]['stop_on_invalid'],True)
            # Removing the option cannot silently undo the persisted stop.
            args.stop_on_invalid=False
            with patch.object(d,'cli',side_effect=controller),patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.return_value=Provider(tags)
                self.assertEqual(d.run(args)['status'],'invalid_offer_stop')
                self.assertEqual(build.return_value.open.call_count,1)
            events=[json.loads(x) for x in (Path(tmp)/'local-directions.jsonl').read_text().splitlines()]
            self.assertEqual(sum(e['status']=='started' for e in events),1)
            self.assertTrue(any(e['status']=='rejected' for e in events))

    def test_assessment_only_never_applies_relational_stage(self):
        class Provider:
            def __enter__(self):return self
            def __exit__(self,*a):pass
            def read(self,limit):return json.dumps({'models':[{'name':'installed','size':1}]}).encode()
        with tempfile.TemporaryDirectory() as tmp:
            args=SimpleNamespace(directory=Path(tmp),model='installed',calls=2,endpoint='http://127.0.0.1:11435',two_stage=True,assess_only=True)
            with patch.object(d,'cli',return_value=self.state()) as cli,patch.object(d.urllib.request,'build_opener') as build:
                build.return_value.open.return_value=Provider();result=d.run(args)
                self.assertEqual(result['status'],'assessment_only_relational_review_required')
                self.assertEqual(build.return_value.open.call_count,1)
                self.assertTrue(all(c.args[1]=='direction-request' for c in cli.call_args_list))


class ProviderUsageTests(unittest.TestCase):
    def test_reported_resources_survive_metadata_without_body(self):
        r={'done':True,'done_reason':'stop','prompt_eval_count':120,'eval_count':33,'total_duration':123456,'eval_duration':9000,'message':{'content':'private body'}}
        got=d.provider_response_metadata(r)
        self.assertEqual(got['usage']['reported']['eval_count'],33)
        self.assertEqual(got['usage']['reported']['total_duration'],123456)
        self.assertEqual(got['usage']['status'],'reported')
        self.assertFalse(got['usage']['reservation_retained'])
        self.assertNotIn('private body',json.dumps(got))
    def test_missing_invalid_and_excess_usage_never_release_unknown(self):
        for r in [{},{'prompt_eval_count':0},{'prompt_eval_count':True,'eval_count':0},
                  {'prompt_eval_count':2,'eval_count':-1},{'prompt_eval_count':'2','eval_count':3},
                  {'prompt_eval_count':2,'eval_count':3,'total_duration':-1}]:
            self.assertEqual(d.provider_usage(r)['status'],'unknown')
            self.assertTrue(d.provider_usage(r)['reservation_retained'])
        z=d.provider_usage({'done':True,'done_reason':'stop','prompt_eval_count':0,'eval_count':0})
        self.assertEqual(z['status'],'reported')
        over=d.provider_usage({'done':True,'done_reason':'stop','prompt_eval_count':32769,'eval_count':2049})
        self.assertEqual(over['status'],'limit_exceeded')
        self.assertEqual(over['reported']['eval_count'],2049)
        self.assertTrue(over['reservation_retained'])

    def test_partial_known_breach_stops_driver_without_retry(self):
        class Provider:
            def __init__(self,v):self.v=v
            def __enter__(self):return self
            def __exit__(self,*a):pass
            def read(self,limit):return json.dumps(self.v).encode()
        for usage in [{'prompt_eval_count':32769},
                      {'prompt_eval_count':3,'eval_count':2049,'total_duration':-1}]:
            with self.subTest(usage=usage), tempfile.TemporaryDirectory() as tmp:
                state=DirectionsControls().forecast_state();state.update(status='deliberate',recent_returns=[])
                args=SimpleNamespace(directory=Path(tmp),model='installed',calls=2,endpoint='http://127.0.0.1:11435')
                response=dict(usage,done=True,done_reason='stop',message={'content':'{}'})
                with patch.object(d,'cli',return_value=state),patch.object(d.urllib.request,'build_opener') as build:
                    build.return_value.open.side_effect=[Provider({'models':[{'name':'installed','size':1}]}),Provider(response)]
                    result=d.run(args)
                    self.assertEqual(result['status'],'provider_resource_limit')
                    self.assertEqual(build.return_value.open.call_count,2) # metadata plus exactly one generation
                events=[json.loads(x) for x in (Path(tmp)/'local-directions.jsonl').read_text().splitlines()]
                self.assertEqual(sum(e['status']=='started' for e in events),1)
                self.assertEqual(events[-1]['status'],'provider_resource_limit')
                self.assertTrue(events[-1]['usage']['reservation_retained'])
                with patch.object(d,'cli',return_value=state),patch.object(d.urllib.request,'build_opener') as build:
                    build.return_value.open.return_value=Provider({'models':[{'name':'installed','size':1}]})
                    self.assertEqual(d.run(args)['status'],'provider_resource_limit')
                    self.assertEqual(build.return_value.open.call_count,1)
