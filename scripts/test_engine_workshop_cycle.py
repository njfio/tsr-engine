"""Authored fake provider + real controller; no network or semantic success claims."""
import datetime as dt
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
from types import SimpleNamespace
import unittest
import test_engine_synthetic_cycle as h

def setup(folder,scenario='workshop'):
    initializer=h.module('workshop_initializer','scripts/engine-workshop-prepare.py')
    # TemporaryDirectory itself exists; initialize a fresh child for production parity.
    folder.rmdir()
    initializer.prepare(folder,(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat(),scenario)
    d=h.module('workshop_setup','scripts/engine-directions.py');c=d.load_synthetic_cycle()
    return d,c,d.cli(folder,'direction-request')

def worker(folder,pause,runner=None,model=None):
    folder=Path(folder);d=h.module('workshop_worker','scripts/engine-directions.py');c=d.load_synthetic_cycle();t=c.load_transport()
    out=folder/'synthetic-external'
    def checkpoint(name):
        n=len(list(out.glob('intent-*.json')))
        if pause==f'{name}-{n}':os._exit(75)
    c.checkpoint=checkpoint
    def invoke(body,timeout):
        with (folder/'fake-calls').open('a') as f:f.write('fixture\n');f.flush();os.fsync(f.fileno())
        w=json.loads(body['messages'][1]['content']);ctx=w['context']
        assert 'semantic_boundary' in ctx
        if 'review' in ctx:
            value={k:ctx['review'][k] for k in ('state_root','return_commitment','observation_revision','evidence_key')}
            old=ctx['existing'][0] if ctx['existing'] else None
            p={'parents':ctx['obligation_ids'],'relationship':'OFFLINE FIXTURE: hypothetical relationship.',
               'aspiration':'OFFLINE FIXTURE: a test aspiration, not a winning answer.',
               'discriminator':'OFFLINE FIXTURE: interpretation remains unknown.',
               'revises':old['revision'] if old else None,'catalysts':[],'development':None}
            if old:
                p.update({k:old[k] for k in ('relationship','aspiration','discriminator')})
                p['development']={'original_revision':old['origin']['revision'],'prior_revision':old['revision'],
                    'retained_meaning':'OFFLINE FIXTURE unchanged.', 'return_reveals':'OFFLINE FIXTURE recorded event only.',
                    'unresolved':'OFFLINE FIXTURE meaning remains unknown.','proposed_development':None}
            value['proposals']=[] if ((folder/'EMPTY').exists() and not old) or len(ctx.get('active_obligation_ids',ctx['obligation_ids']))<2 else [p]
        elif 'frozen_possibilities' in ctx:
            pending=ctx['frozen_possibilities'];value={'state_root':ctx['state_root'],'proposal_digest':d.proposal_digest(pending),
                'assessments':[{'proposal_id':d.proposal_digest(node),'forecasts':dict(zip(ctx['obligation_ids'],[(5+len(ctx['archive']['response_evidence']['recorded']))/6,1])),
                                'supports':[ctx['obligation_ids'][0]]} for node in pending['directions']]}
        else:
            recorded={r['response_id'] for r in ctx['archive']['response_evidence']['recorded']}
            record=next(r for r in ctx['archive']['world']['responses'] if r['id'] not in recorded);rid=record['id']
            value={'state_root':ctx['state_root'],'directions':[{'purpose':'OFFLINE FIXTURE deterministic test operation, not model discovery.',
                'action':{'operation':'archive','event':{'kind':'record_response','response':rid,'challenge':record['challenge'],'revision':record['revision']}}}]}
        t.validate(value,w['response_schema'])
        return {'value':value,'receipt':{'status':'settled','usage_status':'reported','usd':'0',
            'usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'finish_reason':'stop','elapsed_seconds':.001}}
    t.invoke=invoke;c.load_transport=lambda:t;d.load_synthetic_cycle=lambda:c
    args=SimpleNamespace(directory=folder,model=model or c.MODEL,synthetic_admission=folder/'admission.json',stop_file=folder/'STOP',expire_remaining=None)
    print(json.dumps(runner(args,d,c,invoke) if runner else d.run(args)))

class WorkshopCycle(unittest.TestCase):
    def process(self,folder,pause='none'):
        return subprocess.run([sys.executable,str(Path(__file__).resolve()),'--worker',str(folder),pause],capture_output=True,text=True,timeout=35)
    def test_five_stages_and_restarts(self):
        pauses=['none']+[f'{point}-{n}' for n in range(1,6) for point in ('outcome','applied')]+['controller_receipt-1','controller_receipt-3','controller_receipt-4']
        for pause in pauses:
            with self.subTest(pause=pause),tempfile.TemporaryDirectory() as temp:
                folder=Path(temp);d,c,initial=setup(folder)
                first=self.process(folder,pause);self.assertEqual(first.returncode,0 if pause=='none' else 75,first.stderr)
                result=self.process(folder);self.assertEqual(result.returncode,0,result.stderr)
                terminal=json.loads(result.stdout);self.assertEqual(terminal['status'],'proposal_only_complete')
                self.assertEqual(terminal['calls_dispatched'],5);self.assertEqual(terminal['acknowledged_actions'],1)
                self.assertEqual(len((folder/'fake-calls').read_text().splitlines()),5)
                j=folder/'synthetic-external';intents=[c.read(j/f'intent-{i}.json') for i in range(1,6)]
                self.assertEqual([i['phase'] for i in intents],['relational','possibilities','assessment','relational','possibilities_final'])
                self.assertEqual(sum(i['completion_reserved'] for i in intents),65536)
                self.assertEqual(sum(i['wall_reserved'] for i in intents),1200)
                final=d.cli(folder,'direction-request');self.assertEqual(final['cycles'],initial['cycles']+1)
                self.assertEqual(len(final['archive']['events']),5)
                for key in ('standing_desires','authority'):self.assertEqual(final[key],initial[key])
                self.assertFalse((j/'apply-intent-5.json').exists())
                for n in range(1,6):
                    self.assertLessEqual(len(json.dumps(c.read(j/f'request-{n}.json')).encode()),16384)
                self.assertEqual(self.process(folder).returncode,0)
                self.assertEqual(d.cli(folder,'direction-request'),final)
                self.assertEqual(len((folder/'fake-calls').read_text().splitlines()),5)
    def test_empty_stops_without_mutation_and_replay(self):
        with tempfile.TemporaryDirectory() as temp:
            folder=Path(temp);d,c,initial=setup(folder);(folder/'EMPTY').touch()
            result=self.process(folder);self.assertEqual(result.returncode,0,result.stderr)
            self.assertEqual(json.loads(result.stdout)['status'],'empty_initial_aspiration')
            self.assertEqual(d.cli(folder,'direction-request'),initial)
            self.process(folder);self.assertEqual(len((folder/'fake-calls').read_text().splitlines()),1)
    def test_initial_and_admission_drift_rejected(self):
        import copy
        with tempfile.TemporaryDirectory() as temp:
            folder=Path(temp);d,c,initial=setup(folder)
            for mutation in ('events','due','history'):
                bad=copy.deepcopy(initial)
                if mutation=='events':bad['archive']['events'].pop()
                elif mutation=='due':bad['relational_request']['due']=False
                else:bad['relational_request']['history']=[{'fixture':'not permitted'}]
                with self.subTest(mutation=mutation),self.assertRaises(ValueError):c.workshop_initial(bad)
            args=SimpleNamespace(directory=folder,model=c.MODEL)
            a=c.read(folder/'admission.json')
            for key,value in [('proposal_only_tail',False),('stop_empty_initial_aspiration',False),('actions',2),('calls',6),('fixture_sha256','0'*64)]:
                bad=copy.deepcopy(a);bad[key]=value
                with self.subTest(key=key),self.assertRaises(ValueError):c.validate_admission(bad,args)

    def test_unknown_never_retries_at_each_stage(self):
        for n in range(1,6):
            with self.subTest(stage=n),tempfile.TemporaryDirectory() as temp:
                folder=Path(temp);d,c,initial=setup(folder)
                self.assertEqual(self.process(folder,f'intent-{n}').returncode,75)
                before=d.cli(folder,'direction-request');count=len((folder/'fake-calls').read_text().splitlines()) if (folder/'fake-calls').exists() else 0
                result=self.process(folder);self.assertEqual(result.returncode,0,result.stderr)
                self.assertEqual(json.loads(result.stdout)['status'],'uncertain_generation')
                self.assertEqual(d.cli(folder,'direction-request'),before)
                self.assertEqual(len((folder/'fake-calls').read_text().splitlines()) if (folder/'fake-calls').exists() else 0,count)

if __name__=='__main__':
    if len(sys.argv)>1 and sys.argv[1]=='--worker':worker(sys.argv[2],sys.argv[3])
    else:unittest.main()
