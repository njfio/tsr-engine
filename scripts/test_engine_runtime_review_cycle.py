"""Fixture reports test mechanics only. Native simulator, fake provider, no network."""
import contextlib
import copy
import datetime as dt
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import test_engine_synthetic_cycle as h
import test_engine_workshop_cycle as w

def worker(folder,mode):
    folder=Path(folder);x=h.module('review_cycle_wrapper','scripts/engine-campaign-next-choice.py');a=x.s.read(folder/'admission.json')
    rows=[json.loads(line) for line in (folder.parent/'ledger').read_text().splitlines()];manifest=rows[0]['event']['value']
    ledger=x.c.Ledger(folder.parent/'ledger',manifest,x.c.digest(manifest));ext=x.extension(manifest,a)
    def checkpoint(point):
        import os
        if mode=='pause' and point=='review_outcome':os._exit(75)
        if mode=='pause_post' and point=='review_outcome' and list((folder/'synthetic-external').glob('review-*-post_action.outcome.json')):os._exit(75)
        if mode=='recovery_pause' and point=='semantic_recovery':os._exit(75)
        if mode=='unknown_dispatch' and point=='review_intent':os._exit(75)
        if mode=='stop' and point=='review_outcome':(folder/'STOP').touch()
    x.s.checkpoint=checkpoint
    def runner(args,d,s,generate):
        def provider(body,timeout):
            user=json.loads(body['messages'][1]['content'])
            if 'binding' not in user:
                generated=generate(body,timeout)
                if mode in ('recover','recovery_pause','combined_recover') and user['context'].get('attributed_semantic_feedback') and generated['value'].get('proposals'):
                    generated['value']['proposals'][0]['aspiration']='OFFLINE FIXTURE: revised tentative aspiration after attributed critique.'
                if mode=='recover_assessment' and user['context'].get('attributed_semantic_feedback') and generated['value'].get('directions'):
                    generated['value']['directions'][0]['purpose']='OFFLINE FIXTURE: tentative bookkeeping progress only, not real learning.'
                return generated
            binding=user['binding'];kind=binding['kind'];verdict='accept'
            if mode in ('reject','unknown','abstain') and kind=='relational':verdict=mode
            if mode in ('post_reject','recover_post') and kind=='post_action':verdict='reject'
            if mode in ('recover','recovery_pause','repeat','combined_recover') and kind=='relational' and not list((folder/'synthetic-external').glob('recovery-*.json')):verdict='reject'
            if mode=='recover_assessment' and kind=='assessment' and not list((folder/'synthetic-external').glob('recovery-*.json')):verdict='unknown'
            findings={key:{'finding':'met','basis':'static_semantic','evidence':[binding['evidence_ids'][0]],'explanation':'OFFLINE authored finding','limitations':'Fixture-only structural validation; not semantic evidence.'} for key in user['criteria']}
            if verdict=='reject':findings['epistemic_grounding']['finding']='not_met'
            if verdict in ('unknown','abstain'):findings['epistemic_grounding'].update(finding='unknown',basis='unknown')
            value={'review_id':binding['review_id'],'verdict':verdict,'findings':findings}
            if mode=='invalid_schema' and kind=='relational':value['findings']['scope_integrity']['explanation']='é'*300
            if mode=='invalid_contract' and kind=='post_action':value['findings']['scope_integrity']['basis']='unknown'
            return {'value':value,'receipt':{'status':'settled','usage_status':'reported','usd':'0','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'finish_reason':'stop','elapsed_seconds':0.001}}
        return x.run(ledger,a,ext,x.s.digest(ext),provider,planned_restart=mode in ('combined_restart','combined_resume','combined_recover'))
    w.worker(folder,'none',runner=runner,model='gpt-6-luna')

class ReviewCycle(unittest.TestCase):
    def setup(self,root,recovery=False,combined=False):
        folder=root/'trial';prep=h.module('review_cycle_prepare','scripts/engine-workshop-prepare.py')
        prep.prepare(folder,(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat(),'workshop','gpt-6-luna',runtime_review=True,review_recovery=recovery,reviewed_multi=combined)
        x=h.module('review_fixture_campaign','scripts/engine-campaign-next-choice.py');m=x.c.manifest('f'*64,root/'ledger',fixture=True)
        x.c.Ledger(root/'ledger',m,x.c.digest(m),create=True)
        return folder,x
    def process(self,folder,mode):
        return subprocess.run([sys.executable,str(Path(__file__).resolve()),'--worker',str(folder),mode],capture_output=True,text=True,timeout=40)
    def test_combined_two_cycles_restart_and_terminal_replay(self):
        with tempfile.TemporaryDirectory() as temp:
            folder,x=self.setup(Path(temp),True,True);initial=x.d.cli(folder,'direction-request');a=x.s.read(folder/'admission.json')
            self.assertEqual((a['calls'],a['runtime_review']['max_calls'],a['total_call_limit'],a['actions']),(10,9,19,2))
            first=self.process(folder,'combined_restart');self.assertEqual(first.returncode,75,first.stderr)
            out=folder/'synthetic-external';self.assertTrue((out/'planned-restart.json').exists());self.assertFalse((out/'intent-3.json').exists())
            done=self.process(folder,'combined_resume');self.assertEqual(done.returncode,0,done.stderr);r=json.loads(done.stdout)
            self.assertEqual(r['acknowledged_actions'],2);self.assertEqual(r['calls_dispatched'],7);self.assertEqual(r['review_calls_dispatched'],7)
            self.assertIn(r['status'],('satisfied','action_budget_review_complete'))
            final=x.d.cli(folder,'direction-request');self.assertEqual(len(final['archive']['events']),6)
            for key in ('standing_desires','authority','active_obligation_ids'):self.assertEqual(initial[key],final[key])
            responses=[e['response'] for e in final['archive']['events'] if e['kind']=='record_response']
            self.assertEqual(len(set(responses)),2);self.assertEqual(final['observation'],[1.0,1.0])
            for n in (3,6):
                turn=x.s.read(out/f'controller-{n}.json')['result']['turn']
                self.assertGreater(turn['after'][0],turn['before'][0])
            self.assertFalse((out/'operator-boundary-intent.json').exists())
            before=(folder.parent/'ledger').read_bytes();again=self.process(folder,'combined_resume');self.assertEqual(again.returncode,0,again.stderr)
            self.assertEqual((folder.parent/'ledger').read_bytes(),before)
            self.assertEqual(x.d.cli(folder,'direction-request'),final)

    def test_combined_profile_rejects_operator_boundary_and_mixed_profiles(self):
        with tempfile.TemporaryDirectory() as temp:
            folder,x=self.setup(Path(temp),True,True);a=x.s.read(folder/'admission.json')
            from types import SimpleNamespace
            args=SimpleNamespace(directory=folder,model=a['model'])
            for key,value in [('operator_boundary',{}),('autonomy_profile','workshop-multi@v1'),('idle_profile','all-originals-withdrawn@v1'),('continuation',{'invalid':True})]:
                wrong=copy.deepcopy(a);wrong[key]=value
                with self.subTest(key=key),self.assertRaises(ValueError):x.s.validate_admission(wrong,args)

    def test_combined_recovery_restart_and_operator_stop(self):
        with tempfile.TemporaryDirectory() as temp:
            folder,x=self.setup(Path(temp),True,True)
            first=self.process(folder,'combined_recover');self.assertEqual(first.returncode,75,first.stderr)
            out=folder/'synthetic-external';self.assertTrue((out/'recovery-1.json').exists());self.assertFalse((out/'apply-intent-2.json').exists())
            before=(folder.parent/'ledger').read_bytes();(folder/'STOP').touch()
            stopped=self.process(folder,'combined_recover');self.assertEqual(stopped.returncode,0,stopped.stderr)
            self.assertEqual(json.loads(stopped.stdout)['status'],'operator_stop');self.assertEqual((folder.parent/'ledger').read_bytes(),before)
            (folder/'STOP').unlink() # Explicit evaluator resumption, not automatic recovery of withdrawal.
            done=self.process(folder,'combined_recover');self.assertEqual(done.returncode,0,done.stderr);r=json.loads(done.stdout)
            self.assertEqual(r['acknowledged_actions'],2);self.assertEqual(r['calls_dispatched'],8)
            self.assertEqual(r['review_calls_dispatched'],8);self.assertLessEqual(r['total_calls_dispatched'],19)
            self.assertEqual(len(list(out.glob('recovery-*.json'))),1)

    def test_recovery_single_restart_and_exact_repeat(self):
        for mode,expected in [('recover','proposal_only_complete'),('repeat','semantic_recovery_no_progress'),('recover_post','semantic_recovery_reflection_complete'),('recover_assessment','proposal_only_complete')]:
            with self.subTest(mode=mode),tempfile.TemporaryDirectory() as temp:
                folder,x=self.setup(Path(temp),True)
                if mode=='recover':
                    paused=self.process(folder,'recovery_pause');self.assertEqual(paused.returncode,75,paused.stderr)
                done=self.process(folder,mode);self.assertEqual(done.returncode,0,done.stderr);r=json.loads(done.stdout)
                self.assertEqual(r['status'],expected);self.assertLessEqual(r['total_calls_dispatched'],13)
                self.assertEqual(len(list((folder/'synthetic-external').glob('recovery-*.json'))),1)
                before=(folder.parent/'ledger').read_bytes();again=self.process(folder,mode);self.assertEqual(again.returncode,0,again.stderr)
                self.assertEqual((folder.parent/'ledger').read_bytes(),before)
                self.assertEqual(r['acknowledged_actions'],0 if mode=='repeat' else 1)

    def test_recovery_invalid_unknown_and_tamper_fail_closed(self):
        for mode,status in [('invalid_schema','review_invalid_report'),('unknown_dispatch','review_generation_unknown')]:
            with self.subTest(mode=mode),tempfile.TemporaryDirectory() as temp:
                folder,x=self.setup(Path(temp),True);first=self.process(folder,mode)
                if mode=='unknown_dispatch':
                    self.assertEqual(first.returncode,75,first.stderr);first=self.process(folder,'accept')
                self.assertEqual(first.returncode,0,first.stderr);self.assertEqual(json.loads(first.stdout)['status'],status)
                self.assertFalse(list((folder/'synthetic-external').glob('recovery-*.json')))
        for key in ('review_sha256','before_root','after_root','admission_hash','payload_hash','schema_hash'):
            with self.subTest(tamper=key),tempfile.TemporaryDirectory() as temp:
                folder,x=self.setup(Path(temp),True);paused=self.process(folder,'recovery_pause');self.assertEqual(paused.returncode,75,paused.stderr)
                rp=folder/('synthetic-external/intent-1.json' if key in ('admission_hash','payload_hash','schema_hash') else 'synthetic-external/recovery-1.json');v=x.s.read(rp);v[key]='0'*64;rp.write_text(json.dumps(v))
                before=(folder.parent/'ledger').read_bytes();done=self.process(folder,'recover');self.assertNotEqual(done.returncode,0)
                self.assertEqual((folder.parent/'ledger').read_bytes(),before)

    def test_accept_restart_native_action_and_replay_tamper(self):
        with tempfile.TemporaryDirectory() as temp:
            folder,x=self.setup(Path(temp));first=self.process(folder,'pause');self.assertEqual(first.returncode,75,first.stderr)
            completed=self.process(folder,'accept');self.assertEqual(completed.returncode,0,completed.stderr)
            r=json.loads(completed.stdout);self.assertEqual(r['status'],'proposal_only_complete');self.assertEqual(r['calls_dispatched'],5)
            self.assertEqual(r['review_calls_dispatched'],4);self.assertEqual(r['total_calls_dispatched'],9);self.assertEqual(r['acknowledged_actions'],1)
            before=(folder.parent/'ledger').read_bytes();replay=self.process(folder,'accept');self.assertEqual(replay.returncode,0,replay.stderr)
            self.assertEqual((folder.parent/'ledger').read_bytes(),before)
            path=folder/'synthetic-external/applied-3.json';value=x.s.read(path);value['review_outcomes']={};path.write_text(json.dumps(value))
            refused=self.process(folder,'accept');self.assertNotEqual(refused.returncode,0);self.assertEqual((folder.parent/'ledger').read_bytes(),before)
    def test_post_receipt_review_resume_does_not_repeat_action_or_call(self):
        with tempfile.TemporaryDirectory() as temp:
            folder,x=self.setup(Path(temp));paused=self.process(folder,'pause_post');self.assertEqual(paused.returncode,75,paused.stderr)
            state=x.d.cli(folder,'direction-request');self.assertEqual(len(state['archive']['events']),5)
            self.assertTrue((folder/'synthetic-external/controller-3.json').is_file());self.assertFalse((folder/'synthetic-external/applied-3.json').exists())
            done=self.process(folder,'accept');self.assertEqual(done.returncode,0,done.stderr);r=json.loads(done.stdout)
            self.assertEqual(r['status'],'proposal_only_complete');self.assertEqual(r['total_calls_dispatched'],9);self.assertEqual(r['acknowledged_actions'],1)
            self.assertEqual(x.d.cli(folder,'direction-request')['archive']['events'],state['archive']['events'])
            events=[json.loads(l)['event'] for l in (folder.parent/'ledger').read_text().splitlines()]
            self.assertEqual(sum(e['kind']=='call' for e in events),9);self.assertEqual(sum(e['kind']=='execution' for e in events),1)

    def test_pre_negative_unknown_abstain_preserve_state(self):
        for mode in ('reject','unknown','abstain'):
            with self.subTest(mode=mode),tempfile.TemporaryDirectory() as temp:
                folder,x=self.setup(Path(temp));before=x.d.cli(folder,'direction-request')
                result=self.process(folder,mode);self.assertEqual(result.returncode,0,result.stderr);r=json.loads(result.stdout)
                self.assertEqual(r['status'],'review_'+mode);self.assertEqual(r['total_calls_dispatched'],2);self.assertEqual(r['acknowledged_actions'],0)
                self.assertEqual(x.d.cli(folder,'direction-request'),before)
    def test_review_unknown_dispatch_is_not_retried(self):
        with tempfile.TemporaryDirectory() as temp:
            folder,x=self.setup(Path(temp));before=x.d.cli(folder,'direction-request')
            first=self.process(folder,'unknown_dispatch');self.assertEqual(first.returncode,75,first.stderr)
            ledger_before=(folder.parent/'ledger').read_bytes();done=self.process(folder,'accept');self.assertEqual(done.returncode,0,done.stderr)
            result=json.loads(done.stdout);self.assertEqual(result['status'],'review_generation_unknown');self.assertEqual(result['retained_reservations']['calls'],1)
            self.assertEqual((folder.parent/'ledger').read_bytes(),ledger_before);self.assertEqual(x.d.cli(folder,'direction-request'),before)
    def test_stop_after_review_and_budget_tamper_fail_closed(self):
        with tempfile.TemporaryDirectory() as temp:
            folder,x=self.setup(Path(temp));before=x.d.cli(folder,'direction-request');a=x.s.read(folder/'admission.json')
            from types import SimpleNamespace
            for key in ('max_calls','completion_tokens'):
                bad=copy.deepcopy(a);bad['runtime_review'][key]=0
                with self.assertRaises(ValueError):x.s.validate_admission(bad,SimpleNamespace(directory=folder,model='gpt-6-luna'))
            done=self.process(folder,'stop');self.assertEqual(done.returncode,0,done.stderr)
            self.assertEqual(json.loads(done.stdout)['status'],'operator_stop');self.assertEqual(x.d.cli(folder,'direction-request'),before)
            self.assertFalse((folder/'synthetic-external/apply-intent-1.json').exists())

    def test_validation_diagnostic_persisted_without_invalid_prose(self):
        for mode,kind,code,count in [('invalid_schema','relational','schema_text',0),('invalid_contract','post_action','unknown_basis_cannot_establish',1)]:
            with self.subTest(mode=mode),tempfile.TemporaryDirectory() as temp:
                folder,x=self.setup(Path(temp));result=self.process(folder,mode);self.assertEqual(result.returncode,0,result.stderr)
                terminal=json.loads(result.stdout);self.assertEqual(terminal['status'],'review_invalid_report');self.assertEqual(terminal['acknowledged_actions'],count)
                path=next((folder/'synthetic-external').glob('review-*-'+kind+'.outcome.json'))
                record=x.s.read(path);self.assertIsNone(record['value']);self.assertEqual(record['validation']['code'],code)
                self.assertNotIn('é',json.dumps(record));self.assertIsNotNone(record['validation']['candidate_sha256'])

    def test_post_negative_keeps_actual_receipt_stops_continuation(self):
        with tempfile.TemporaryDirectory() as temp:
            folder,x=self.setup(Path(temp));result=self.process(folder,'post_reject');self.assertEqual(result.returncode,0,result.stderr)
            r=json.loads(result.stdout);self.assertEqual(r['status'],'review_reject');self.assertEqual(r['acknowledged_actions'],1)
            self.assertEqual(r['calls_dispatched'],3);self.assertEqual(r['review_calls_dispatched'],3)
            self.assertTrue((folder/'synthetic-external/controller-3.json').exists());self.assertFalse((folder/'synthetic-external/intent-4.json').exists())
if __name__=='__main__':
    if len(sys.argv)>1 and sys.argv[1]=='--worker':worker(Path(sys.argv[2]),sys.argv[3])
    else:unittest.main()
