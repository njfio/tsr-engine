"""Authored fake generation, actual native simulator, shared fixture campaign. No network."""
import contextlib
import datetime as dt
import io
import json
from pathlib import Path
import tempfile
import unittest
import test_engine_synthetic_cycle as h
import test_engine_workshop_cycle as w

class NextChoice(unittest.TestCase):
    def test_native_cycle_shared_ledger_and_replay(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);folder=root/'garden'
            prep=h.module('nc_prep','scripts/engine-workshop-prepare.py')
            prep.prepare(folder,(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat(),'garden','gpt-6-luna')
            x=h.module('nc_wrapper','scripts/engine-campaign-next-choice.py');a=x.s.read(folder/'admission.json')
            manifest=x.c.manifest('a'*64,root/'ledger',fixture=True)
            ledger=x.c.Ledger(root/'ledger',manifest,x.c.digest(manifest),create=True)
            ext=x.extension(manifest,a);calls=[]
            admission_path=folder/'admission.json';original=admission_path.read_bytes()
            changed=dict(a);changed['expires_at']=(dt.datetime.now(dt.timezone.utc)+dt.timedelta(minutes=45)).isoformat()
            admission_path.write_text(json.dumps(changed))
            with self.assertRaisesRegex(ValueError,'binding mismatch'):
                x.run(ledger,a,ext,x.s.digest(ext),lambda *_:self.fail('drift dispatched'))
            admission_path.write_bytes(original)
            def runner(args,d,s,invoke):
                def fake(body,timeout):
                    self.assertEqual(body['model'],'gpt-6-luna');self.assertEqual(body['max_completion_tokens'],8192)
                    self.assertNotIn('temperature',body);self.assertNotIn('max_tokens',body)
                    calls.append(body)
                    return invoke(body,timeout)
                return x.run(ledger,a,ext,x.s.digest(ext),fake)
            out=io.StringIO()
            with contextlib.redirect_stdout(out):w.worker(folder,'none',runner=runner,model='gpt-6-luna')
            result=json.loads(out.getvalue());self.assertEqual(result['status'],'proposal_only_complete')
            self.assertEqual(len(calls),5);self.assertEqual(result['acknowledged_actions'],1)
            state=x.d.cli(folder,'direction-request')
            self.assertEqual(len(state['archive']['events']),5)
            self.assertEqual(sum(e['kind']=='record_response' for e in state['archive']['events']),1)
            before=ledger.path.read_bytes()
            x.run(ledger,a,ext,x.s.digest(ext),lambda *_:self.fail('replay dispatched'))
            self.assertEqual(ledger.path.read_bytes(),before)
            rows=[json.loads(l)['event'] for l in before.splitlines()]
            self.assertEqual(sum(e['kind']=='call' for e in rows),5)
            self.assertEqual(sum(e['kind']=='execution' for e in rows),1)
            continuation=h.module('nc_continue','scripts/engine-garden-continue.py')
            review={'parent':str(folder.resolve()),'terminal_sha256':x.s.digest(result),'admission_sha256':x.s.digest(a),'approved':True,'reviewer':'OFFLINE fixture reviewer, not live semantic evidence'}
            x.s.save(root/'review.json',review);child=root/'continued'
            original_parent=continuation.inventory(folder)
            with self.assertRaisesRegex(ValueError,'disjoint'):
                continuation.prepare(folder,folder/'nested',root/'review.json',(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat())
            self.assertEqual(continuation.inventory(folder),original_parent)
            continuation.prepare(folder,child,root/'review.json',(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat())
            ca=x.s.read(child/'admission.json');ce=x.extension(manifest,ca)
            def continued(args,d,s,invoke):return x.run(ledger,ca,ce,x.s.digest(ce),invoke)
            output=io.StringIO()
            with contextlib.redirect_stdout(output):w.worker(child,'none',runner=continued,model='gpt-6-luna')
            completed=json.loads(output.getvalue());self.assertEqual(completed['status'],'action_budget_review_complete')
            self.assertEqual(completed['calls_dispatched'],3);self.assertEqual(completed['acknowledged_actions'],1)
            self.assertEqual(len(x.d.cli(child,'direction-request')['archive']['events']),6)
            self.assertEqual(continuation.inventory(folder),original_parent)
            before=ledger.path.read_bytes();x.run(ledger,ca,ce,x.s.digest(ce),lambda *_:self.fail('continuation replay called'))
            self.assertEqual(ledger.path.read_bytes(),before)
            rows=[json.loads(l)['event'] for l in before.splitlines()]
            self.assertEqual(sum(e['kind']=='call' for e in rows),8);self.assertEqual(sum(e['kind']=='execution' for e in rows),2)
            archive=child/'parent-evidence/setup-provenance.json';archive.write_text('{}')
            with self.assertRaisesRegex(ValueError,'parent/archive drift'):x.run(ledger,ca,ce,x.s.digest(ce),lambda *_:self.fail('tamper called'))

    def test_multi_restart_withdrawal_mixed_returns_and_idle(self):self.multi_case('garden')
    def test_workshop_restart_withdrawal_preserves_scope(self):self.multi_case('workshop')
    def multi_case(self,scenario):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);folder=root/'garden'
            prep=h.module('multi_prep','scripts/engine-workshop-prepare.py')
            prep.prepare(folder,(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat(),scenario,'gpt-6-luna',multi=True)
            x=h.module('multi_wrapper','scripts/engine-campaign-next-choice.py');a=x.s.read(folder/'admission.json')
            self.assertEqual(a['max_payload_bytes'],32768);self.assertEqual(a['reserved_input_tokens_per_call'],32768)
            import copy
            from types import SimpleNamespace
            wrong=copy.deepcopy(a);wrong['max_payload_bytes']=65536
            with self.assertRaises(ValueError):x.s.validate_admission(wrong,SimpleNamespace(directory=folder,model='gpt-6-luna'))
            manifest=x.c.manifest('c'*64,root/'ledger',fixture=True);ledger=x.c.Ledger(root/'ledger',manifest,x.c.digest(manifest),create=True)
            import subprocess,sys
            command=[sys.executable,str(Path(__file__).resolve()),'--multi-worker',str(folder)]
            first=subprocess.run(command+['pause'],capture_output=True,text=True,timeout=30)
            self.assertEqual(first.returncode,75,first.stderr)
            self.assertTrue((folder/'synthetic-external/planned-restart.json').is_file())
            resumed=subprocess.run(command+['resume'],capture_output=True,text=True,timeout=30)
            self.assertEqual(resumed.returncode,0,resumed.stderr)
            result=json.loads(resumed.stdout);self.assertEqual(result['status'],'satisfied')
            ext=x.extension(manifest,a);seen=(folder/'fake-calls').read_text().splitlines()
            state=x.d.cli(folder,'direction-request');self.assertEqual(state['active_desires'],[0])
            initial=x.s.read(a['source_snapshot'])
            for key in ('authority','standing_desires'):self.assertEqual(state[key],initial[key])
            self.assertEqual(state['archive']['world'],initial['archive']['world'])
            self.assertEqual(len(state['archive']['events']),6);self.assertEqual(result['acknowledged_actions'],2)
            self.assertEqual(len(seen),result['calls_dispatched']);self.assertLessEqual(len(seen),8)
            receipts=[e for e in state['archive']['events'] if e['kind']=='record_response']
            self.assertEqual({r['response'] for r in receipts},({'trial-positive','trial-negative'} if scenario=='garden' else {'measurement','learning-offer'}))
            before=ledger.path.read_bytes();x.run(ledger,a,ext,x.s.digest(ext),lambda *_:self.fail('idle dispatched'))
            self.assertEqual(ledger.path.read_bytes(),before)

    def test_all_originals_withdrawn_zero_calls_actions_and_replay(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);folder=root/'idle'
            prep=h.module('idle_prep','scripts/engine-workshop-prepare.py')
            prep.prepare(folder,(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat(),'workshop','gpt-6-luna',idle=True)
            x=h.module('idle_wrapper','scripts/engine-campaign-next-choice.py');a=x.s.read(folder/'admission.json')
            manifest=x.c.manifest('e'*64,root/'ledger',fixture=True);ledger=x.c.Ledger(root/'ledger',manifest,x.c.digest(manifest),create=True)
            import copy,itertools
            from types import SimpleNamespace
            for pair in itertools.combinations(('continuation','autonomy_profile','idle_profile'),2):
                mixed=copy.deepcopy(a)
                for key in ('continuation','autonomy_profile','idle_profile'):mixed.pop(key,None)
                for key in pair:mixed[key]='forged'
                with self.assertRaisesRegex(ValueError,'incompatible synthetic profiles'):x.s.validate_admission(mixed,SimpleNamespace(directory=folder,model='gpt-6-luna'))
            ext=x.extension(manifest,a);result=x.run(ledger,a,ext,x.s.digest(ext),lambda *_:self.fail('idle provider call'))
            self.assertEqual(result['status'],'withdrawn_desires');self.assertEqual(result['calls_dispatched'],0);self.assertEqual(result['acknowledged_actions'],0)
            state=x.d.cli(folder,'direction-request');self.assertEqual(state['active_desires'],[]);self.assertEqual(len(state['archive']['events']),4)
            self.assertEqual(state['standing_desires'],x.s.fixture('workshop')['spec']['wants'])
            before=ledger.path.read_bytes();x.run(ledger,a,ext,x.s.digest(ext),lambda *_:self.fail('idle replay'))
            self.assertEqual(ledger.path.read_bytes(),before)
            events=[json.loads(line)['event'] for line in before.splitlines()]
            self.assertFalse(any(e['kind'] in ('call','execution') for e in events))

    def test_stop_before_withdrawal_has_no_operator_intent(self):
        import subprocess,sys
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);folder=root/'garden'
            prep=h.module('stop_prep','scripts/engine-workshop-prepare.py')
            prep.prepare(folder,(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat(),'garden','gpt-6-luna',multi=True)
            x=h.module('stop_multi','scripts/engine-campaign-next-choice.py')
            manifest=x.c.manifest('d'*64,root/'ledger',fixture=True);x.c.Ledger(root/'ledger',manifest,x.c.digest(manifest),create=True)
            result=subprocess.run([sys.executable,str(Path(__file__).resolve()),'--multi-worker',str(folder),'stop'],capture_output=True,text=True,timeout=30)
            self.assertEqual(result.returncode,0,result.stderr);self.assertEqual(json.loads(result.stdout)['status'],'operator_stop')
            self.assertFalse((folder/'synthetic-external/operator-boundary-intent.json').exists())
            self.assertEqual(x.d.cli(folder,'direction-request')['active_desires'],[0,1])

    def test_unknown_action_keeps_reservation_and_blocks_restart(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);folder=root/'garden'
            prep=h.module('nc_unknown_prep','scripts/engine-workshop-prepare.py')
            prep.prepare(folder,(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat(),'garden','gpt-6-luna')
            x=h.module('nc_unknown','scripts/engine-campaign-next-choice.py');a=x.s.read(folder/'admission.json')
            from types import SimpleNamespace
            import copy
            bad=copy.deepcopy(a);bad['stages']['assessment']['completion_tokens']=32768
            with self.assertRaises(ValueError):x.s.validate_admission(bad,SimpleNamespace(directory=folder,model='gpt-6-luna'))
            manifest=x.c.manifest('b'*64,root/'ledger',fixture=True)
            ledger=x.c.Ledger(root/'ledger',manifest,x.c.digest(manifest),create=True);ext=x.extension(manifest,a)
            original=ledger.run_execution
            def interrupted(trial,group,admission,apply):
                def lost_return():
                    apply()
                    raise RuntimeError('fixture lost acknowledgment after native application')
                return original(trial,group,admission,lost_return)
            ledger.run_execution=interrupted
            def runner(args,d,s,invoke):return x.run(ledger,a,ext,x.s.digest(ext),invoke)
            out=io.StringIO()
            with contextlib.redirect_stdout(out):w.worker(folder,'none',runner=runner,model='gpt-6-luna')
            self.assertEqual(json.loads(out.getvalue())['status'],'uncertain_application')
            self.assertTrue(ledger.status()['stopped'])
            self.assertEqual(len(x.d.cli(folder,'direction-request')['archive']['events']),5)
            before=ledger.path.read_bytes()
            with self.assertRaises(ValueError):x.run(ledger,a,ext,x.s.digest(ext),lambda *_:self.fail('unknown retried'))
            self.assertEqual(ledger.path.read_bytes(),before)
def multi_worker(folder,pause):
    import os
    folder=Path(folder);root=folder.parent
    x=h.module('multi_child','scripts/engine-campaign-next-choice.py');a=x.s.read(folder/'admission.json')
    rows=[json.loads(l) for l in (root/'ledger').read_text().splitlines()];manifest=rows[0]['event']['value']
    ledger=x.c.Ledger(root/'ledger',manifest,x.c.digest(manifest));ext=x.extension(manifest,a)
    def checkpoint(point):
        if pause=='stop' and point=='applied' and (folder/'synthetic-external/applied-4.json').exists():(folder/'STOP').touch()
    x.s.checkpoint=checkpoint
    def runner(args,d,s,invoke):
        def fake(body,timeout):
            ctx=json.loads(body['messages'][1]['content'])['context']
            if (folder/'synthetic-external/operator-boundary-receipt.json').exists():
                assert ctx['active_obligation_ids']==[ctx['obligation_ids'][0]]
                assert ctx.get('relational_guidance',[])==[]
            return invoke(body,timeout)
        return x.run(ledger,a,ext,x.s.digest(ext),fake,planned_restart=pause in ('pause','resume'))
    w.worker(folder,'none',runner=runner,model='gpt-6-luna')
if __name__=='__main__':
    import sys
    if len(sys.argv)>1 and sys.argv[1]=='--multi-worker':multi_worker(sys.argv[2],sys.argv[3])
    else:unittest.main()
