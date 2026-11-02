"""Real controller/fresh-process restart tests; authored fake provider, no network."""
import datetime as dt
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest

ROOT = Path(__file__).resolve().parents[1]
def module(name, path):
    s=importlib.util.spec_from_file_location(name,ROOT/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m

def worker(folder, pause):
    d=module('driver','scripts/engine-directions.py');c=d.load_synthetic_cycle();transport=c.load_transport()
    folder=Path(folder);admission=folder/'admission.json';out=folder/'synthetic-external'
    control=json.loads((folder/'control.json').read_text()) if (folder/'control.json').exists() else {}
    def checkpoint(name):
        number=len(list(out.glob('intent-*.json')))
        if pause==f'{name}-{number}':os._exit(75)
        if pause==f'stop-{name}-{number}':(folder/'STOP').touch()
    c.checkpoint=checkpoint
    if control.get('expired'):
        original=c.dt.datetime
        class FutureClock(original):
            @classmethod
            def now(cls,tz=None):return original.now(tz)+dt.timedelta(days=2)
        c.dt.datetime=FutureClock
    original_cli = d.cli
    def controller(directory, *args):
        if args and args[0] == 'deliberate':
            if control.get('preapply_refusal'):
                original = json.loads(Path(args[2]).read_text()); original['state_root']='0'*64
                path=folder/'stale-fixture.json';path.write_text(json.dumps(original))
                return original_cli(directory,'deliberate','--response',str(path))
            if control.get('lost_after_commit'):
                original_cli(directory,*args)
                raise TimeoutError('fixture acknowledgment lost after real commit')
            if control.get('refusal_with_storage_change'):
                (folder/'cas'/'new-fixture-blob').write_text('fixture side effect')
                original=json.loads(Path(args[2]).read_text());original['state_root']='0'*64
                path=folder/'stale-fixture.json';path.write_text(json.dumps(original))
                return original_cli(directory,'deliberate','--response',str(path))
            if control.get('unknown_before_apply'):
                raise TimeoutError('fixture transport unknown before invocation')
        return original_cli(directory,*args)
    d.cli=controller
    def invoke(body,timeout):
        with (folder/'fake-calls').open('a') as f:f.write('call\n');f.flush();os.fsync(f.fileno())
        wrapper=json.loads(body['messages'][1]['content']);ctx=wrapper['context'];schema=wrapper['response_schema']
        assert 'private_review' not in json.dumps(body)
        if 'review' in ctx:
            r=ctx['review'];value={k:r[k] for k in ('state_root','return_commitment','observation_revision','evidence_key')}
            value['proposals']=[{'parents':ctx['obligation_ids'], 'relationship':'SYNTHETIC FIXTURE: relate contestability and provenance.',
                'aspiration':'SYNTHETIC FIXTURE: retain dissent while exposing response status.',
                'discriminator':'SYNTHETIC FIXTURE: did a response occur?',
                'revises':ctx['existing'][0]['revision'] if ctx['existing'] else None,'catalysts':[],'development':None}]
            if ctx['existing']:
                old=ctx['existing'][0];proposal=value['proposals'][0]
                proposal.update({key:old[key] for key in ('relationship','aspiration','discriminator')})
                proposal['development']={'original_revision':old['origin']['revision'],'prior_revision':old['revision'],
                    'retained_meaning':'SYNTHETIC FIXTURE unchanged relationship.',
                    'return_reveals':'SYNTHETIC FIXTURE adapter event observed.',
                    'unresolved':'SYNTHETIC FIXTURE semantic meaning remains unknown.', 'proposed_development':None}
        elif 'frozen_possibilities' in ctx:
            proposal=ctx['frozen_possibilities'];value={'state_root':ctx['state_root'],'proposal_digest':d.proposal_digest(proposal),'assessments':[
                {'proposal_id':d.proposal_digest(node),'forecasts':dict(zip(ctx['obligation_ids'],[1/3,1])), 'supports':ctx['obligation_ids']} for node in proposal['directions']]}
        else:
            value={'state_root':ctx['state_root'],'directions':[{'purpose':'SYNTHETIC FIXTURE: register admitted meaning challenge.',
                   'action':{'operation':'archive','event':{'kind':'register','challenge':'meaning','revision':'translation-v1'}}}]}
        if 'frozen_possibilities' in ctx:
            if control.get('no_progress'):
                for row in value['assessments']:
                    row['forecasts']=dict(zip(ctx['obligation_ids'],[.75,.5]));row['supports']=[ctx['obligation_ids'][0]]
            if control.get('empty_support'): value['assessments'][0]['supports']=[]
            if control.get('unknown_support'): value['assessments'][0]['supports']=['not-an-obligation']
            if control.get('stale_assessment'): value['state_root']='0'*64
        elif 'review' not in ctx and control.get('no_progress'):
            value['directions']=[{'purpose':'SYNTHETIC FIXTURE additional route, no measured progress.',
                'action':{'operation':'archive','event':{'kind':'route','challenge':'access','revision':'hours-v1','actor':'caretaker'}}}]
        if not any(control.get(k) for k in ('empty_support','unknown_support','stale_assessment')): transport.validate(value,schema)
        if control.get('invalid'):value={'unexpected':'invalid output'}
        return {'value':value,'receipt':{'status':'unknown' if control.get('unknown') else 'settled','usage_status':'unknown' if control.get('missing_usage') else 'reported','usd':None if control.get('unknown') else '0.001','response_status':'received','finish_reason':'stop',
             'usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':0.001,'body_sha256':'0'*64}}
    transport.invoke=invoke;c.load_transport=lambda:transport;d.load_synthetic_cycle=lambda:c
    args=SimpleNamespace(directory=folder,model=c.MODEL,synthetic_admission=admission,stop_file=folder/'STOP',expire_remaining=control.get('withdraw'))
    result=d.run(args);print(json.dumps(result))


class SyntheticCycle(unittest.TestCase):
    def setup_run(self, folder, routed=False):
        d=module('setup_driver','scripts/engine-directions.py');c=d.load_synthetic_cycle()
        spec=c.fixture()['spec'];specpath=folder/'spec.json';specpath.write_text(json.dumps(spec))
        d.cli(folder,'direction-init','--spec',str(specpath));q=d.cli(folder,'direction-request')
        seed={'state_root':q['state_root'],'directions':[{'purpose':'SYNTHETIC FIXTURE seed return',
              'action':{'operation':'archive','event':{'kind':'register','challenge':'access','revision':'hours-v1'}},
              'forecasts':dict(zip(q['obligation_ids'],[1/6,.5])),'supports':q['obligation_ids']}]}
        seedpath=folder/'seed.json';seedpath.write_text(json.dumps(d.normalize_forecasts(q,seed)))
        d.cli(folder,'deliberate','--response',str(seedpath));initial=d.cli(folder,'direction-request')
        if routed:
            seed={'state_root':initial['state_root'],'directions':[{'purpose':'SYNTHETIC FIXTURE initial routing',
                  'action':{'operation':'archive','event':{'kind':'route','challenge':'access','revision':'hours-v1','actor':'night-reader'}},
                  'forecasts':dict(zip(initial['obligation_ids'],[1/3,.5])),'supports':[initial['obligation_ids'][0]]}]}
            seedpath.write_text(json.dumps(d.normalize_forecasts(initial,seed)))
            d.cli(folder,'deliberate','--response',str(seedpath));initial=d.cli(folder,'direction-request')
        self.assertTrue(initial['relational_request']['due'])
        args=SimpleNamespace(directory=folder,model=c.MODEL,calls=4,synthetic_actions=1,
             synthetic_expires=(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat(),
             prepare_synthetic_admission=folder/'admission.json')
        d.run(args)
        return d,c,initial
    def process(self, folder, pause='none'):
        return subprocess.run([sys.executable,str(Path(__file__).resolve()),'--worker',str(folder),pause],capture_output=True,text=True,timeout=30)
    def count(self, folder):return len((folder/'fake-calls').read_text().splitlines()) if (folder/'fake-calls').exists() else 0
    def test_complete_and_process_restarts(self):
        for pause in ['none','outcome-1','controller_receipt-1','applied-1','outcome-2','applied-2','outcome-3','controller_receipt-3','applied-3','outcome-4','controller_receipt-4','applied-4']:
            with self.subTest(pause=pause),tempfile.TemporaryDirectory() as tmp:
                folder=Path(tmp);d,c,initial=self.setup_run(folder)
                first=self.process(folder,pause);self.assertEqual(first.returncode,0 if pause=='none' else 75,first.stderr)
                final=self.process(folder);self.assertEqual(final.returncode,0,final.stderr)
                self.assertEqual(self.count(folder),4)
                q=d.cli(folder,'direction-request');self.assertEqual(q['cycles'],initial['cycles']+1)
                self.assertEqual(q['standing_desires'],initial['standing_desires']);self.assertEqual(q['authority'],initial['authority'])
                self.assertEqual(len(q['archive']['events']),2);self.assertFalse(q['relational_request']['due'])
                intents=[c.read(folder/'synthetic-external'/f'intent-{n}.json') for n in range(1,5)]
                self.assertEqual([x['phase'] for x in intents],['relational','possibilities','assessment','relational'])
                again=self.process(folder);self.assertEqual(again.returncode,0,again.stderr);self.assertEqual(self.count(folder),4)
                self.assertEqual(d.cli(folder,'direction-request'),q)
    def test_uncertain_never_retries_or_reapplies(self):
        for pause,expected in [('intent-1','uncertain_generation'),('apply_intent-3','uncertain_application'),('controller_returned-3','uncertain_application')]:
            with self.subTest(pause=pause),tempfile.TemporaryDirectory() as tmp:
                folder=Path(tmp);d,c,initial=self.setup_run(folder);first=self.process(folder,pause);self.assertEqual(first.returncode,75,first.stderr)
                count=self.count(folder);state=d.cli(folder,'direction-request');second=self.process(folder)
                self.assertEqual(second.returncode,0,second.stderr);self.assertEqual(json.loads(second.stdout)['status'],expected)
                self.assertEqual(self.count(folder),count);self.assertEqual(d.cli(folder,'direction-request'),state)
    def test_fail_closed_outputs_and_retained_reservations(self):
        for control,status,retained in [({'unknown':True},'provider_unknown',1),({'missing_usage':True},'usage_unknown',1),({'invalid':True},'invalid_or_truncated_output',0)]:
            with self.subTest(control=control),tempfile.TemporaryDirectory() as tmp:
                folder=Path(tmp);d,c,initial=self.setup_run(folder);(folder/'control.json').write_text(json.dumps(control))
                result=self.process(folder);self.assertEqual(result.returncode,0,result.stderr)
                terminal=json.loads(result.stdout);self.assertEqual(terminal['status'],status)
                self.assertEqual(terminal['retained_reservations']['calls'],retained)
                self.assertEqual(self.count(folder),1);self.assertEqual(d.cli(folder,'direction-request'),initial)
                again=self.process(folder);self.assertEqual(again.returncode,0,again.stderr);self.assertEqual(self.count(folder),1)
    def test_withdrawal_does_not_erase_unknown_intent(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder=Path(tmp);d,c,initial=self.setup_run(folder)
            self.assertEqual(self.process(folder,'intent-1').returncode,75)
            (folder/'control.json').write_text(json.dumps({'withdraw':'operator withdrawal'}))
            result=self.process(folder);self.assertEqual(result.returncode,0,result.stderr)
            terminal=json.loads(result.stdout);self.assertEqual(terminal['status'],'withdrawn')
            self.assertTrue(terminal['unknown_usage_or_cost']);self.assertEqual(terminal['retained_reservations']['calls'],1)
            self.assertEqual(terminal['retained_reservations']['completion_tokens'],8192);self.assertEqual(self.count(folder),0)
    def test_expiry_preserves_unknown_generation_and_application(self):
        for pause in ['intent-1','controller_returned-3']:
            with self.subTest(pause=pause),tempfile.TemporaryDirectory() as tmp:
                folder=Path(tmp);d,c,initial=self.setup_run(folder);self.assertEqual(self.process(folder,pause).returncode,75)
                count=self.count(folder);state=d.cli(folder,'direction-request');(folder/'control.json').write_text(json.dumps({'expired':True}))
                r=self.process(folder);self.assertEqual(r.returncode,0,r.stderr);v=json.loads(r.stdout);self.assertEqual(v['status'],'expired')
                self.assertEqual(v['unknown_usage_or_cost'],pause=='intent-1')
                self.assertEqual(v['unknown_application_outcome'],pause=='controller_returned-3')
                self.assertEqual(self.count(folder),count);self.assertEqual(d.cli(folder,'direction-request'),state)

    def test_settled_outcome_stop_resumes_without_redispatch(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder=Path(tmp);d,c,initial=self.setup_run(folder);r=self.process(folder,'stop-outcome-1')
            self.assertEqual(r.returncode,0,r.stderr);self.assertEqual(json.loads(r.stdout)['status'],'operator_stop')
            self.assertEqual(self.count(folder),1);self.assertEqual(d.cli(folder,'direction-request'),initial)
            (folder/'STOP').unlink();r=self.process(folder);self.assertEqual(r.returncode,0,r.stderr);self.assertEqual(self.count(folder),4)
    def test_drift_and_frozen_admission_fail_before_new_dispatch(self):
        for change in ['source','request','admission']:
            with self.subTest(change=change),tempfile.TemporaryDirectory() as tmp:
                folder=Path(tmp);d,c,initial=self.setup_run(folder)
                self.assertEqual(self.process(folder,'outcome-1').returncode,75)
                if change=='source':path=folder/'admission.json.source.json'
                elif change=='request':path=folder/'synthetic-external/request-1.json'
                else:path=folder/'admission.json'
                value=json.loads(path.read_text())
                if change=='admission':value['calls']=5
                else:value['tampered']=True
                path.write_text(json.dumps(value));r=self.process(folder);self.assertNotEqual(r.returncode,0)
                self.assertEqual(self.count(folder),1);self.assertEqual(d.cli(folder,'direction-request'),initial)
    def test_expiry_and_zero_action_limit(self):
        for expired in [True,False]:
            with self.subTest(expired=expired),tempfile.TemporaryDirectory() as tmp:
                folder=Path(tmp);d,c,initial=self.setup_run(folder);p=folder/'admission.json';a=json.loads(p.read_text())
                if expired:a['expires_at']='2020-01-01T00:00:00+00:00'
                else:a['actions']=0
                p.write_text(json.dumps(a));r=self.process(folder);self.assertEqual(r.returncode,0,r.stderr)
                self.assertEqual(json.loads(r.stdout)['status'],'expired' if expired else 'action_budget_review_complete')
                self.assertEqual(self.count(folder),0 if expired else 1)
                self.assertEqual(d.cli(folder,'direction-request')['cycles'],initial['cycles'])

    def test_real_controller_refusals_and_restart(self):
        for mode,code in [('no_progress','no_supported_progress'),('preapply_refusal','stale_offer')]:
            for pause in ['none','controller_refusal-3']:
                with self.subTest(mode=mode,pause=pause),tempfile.TemporaryDirectory() as tmp:
                    folder=Path(tmp);d,c,initial=self.setup_run(folder,routed=mode=='no_progress')
                    (folder/'control.json').write_text(json.dumps({mode:True}))
                    first=self.process(folder,pause)
                    self.assertEqual(first.returncode,0 if pause=='none' else 75,first.stderr)
                    r=self.process(folder);self.assertEqual(r.returncode,0,r.stderr)
                    terminal=json.loads(r.stdout);self.assertEqual(terminal['status'],'controller_refused')
                    self.assertFalse(terminal['unknown_application_outcome']);self.assertEqual(self.count(folder),3)
                    receipt=c.read(folder/'synthetic-external/controller-3.json')
                    self.assertEqual(receipt['result']['diagnostic']['code'],code)
                    self.assertEqual(receipt['storage_before'],receipt['storage_after'])
                    self.assertEqual(receipt['storage_after'],c.storage_inventory(folder))
                    self.assertEqual(d.cli(folder,'direction-request')['cycles'],initial['cycles'])
                    self.assertEqual(terminal['settled_usd'],'0.003')
                    self.assertNotIn('stderr',receipt['result']['diagnostic'])
                    import copy
                    for field,bad in [('exit_code',-15),('code','unknown'),('contract','other'),('stderr_sha256','bad'),('stderr_bytes',False)]:
                        invalid=copy.deepcopy(receipt);invalid['result']['diagnostic'][field]=bad
                        with self.assertRaises(ValueError):c.verify_refusal(invalid,c.read(folder/'synthetic-external/apply-intent-3.json'),folder,receipt['before_root'])
                    path=folder/'synthetic-external/controller-3.json';original_bytes=path.read_bytes()
                    path.unlink();self.assertNotEqual(self.process(folder).returncode,0)
                    path.write_bytes(original_bytes)
                    invalid=copy.deepcopy(receipt);invalid['result']['status']='other';path.write_text(json.dumps(invalid))
                    self.assertNotEqual(self.process(folder).returncode,0);path.write_bytes(original_bytes)
                    # Changing even an unreferenced authoritative blob invalidates proof.
                    (folder/'cas'/'drift').write_text('fixture')
                    self.assertNotEqual(self.process(folder).returncode,0)

    def test_assessment_invalid_or_stale_before_application(self):
        for mode in ['unknown_support','stale_assessment']:
            with self.subTest(mode=mode),tempfile.TemporaryDirectory() as tmp:
                folder=Path(tmp);d,c,initial=self.setup_run(folder)
                (folder/'control.json').write_text(json.dumps({mode:True}))
                r=self.process(folder);self.assertEqual(r.returncode,0,r.stderr)
                self.assertEqual(json.loads(r.stdout)['status'],'invalid_or_truncated_output')
                self.assertFalse((folder/'synthetic-external/apply-intent-3.json').exists())
                self.assertEqual(d.cli(folder,'direction-request')['cycles'],initial['cycles'])
                self.assertEqual(self.count(folder),3)

    def test_unknown_before_and_after_commit_stays_unknown(self):
        for mode,increment in [('unknown_before_apply',0),('lost_after_commit',1),('refusal_with_storage_change',0)]:
            with self.subTest(mode=mode),tempfile.TemporaryDirectory() as tmp:
                folder=Path(tmp);d,c,initial=self.setup_run(folder)
                (folder/'control.json').write_text(json.dumps({mode:True}))
                r=self.process(folder);self.assertEqual(r.returncode,0,r.stderr)
                self.assertEqual(json.loads(r.stdout)['status'],'uncertain_application')
                self.assertFalse((folder/'synthetic-external/controller-3.json').exists())
                self.assertEqual(d.cli(folder,'direction-request')['cycles'],initial['cycles']+increment)
                again=self.process(folder);self.assertEqual(json.loads(again.stdout)['status'],'uncertain_application')
                self.assertEqual(self.count(folder),3)

    def test_private_projection_and_stop(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder=Path(tmp);d,c,initial=self.setup_run(folder);initial['private_review']='DO NOT EXPORT'
            projected=c.context(initial,SimpleNamespace(**vars(d)));self.assertNotIn('DO NOT EXPORT',json.dumps(projected))
            (folder/'STOP').touch();r=self.process(folder);self.assertEqual(r.returncode,0,r.stderr)
            self.assertEqual(json.loads(r.stdout)['status'],'operator_stop');self.assertEqual(self.count(folder),0)
            (folder/'STOP').unlink();r=self.process(folder);self.assertEqual(r.returncode,0,r.stderr);self.assertEqual(self.count(folder),4)

if __name__=='__main__':
    if len(sys.argv)>1 and sys.argv[1]=='--worker':worker(sys.argv[2],sys.argv[3])
    else:unittest.main()
