import copy,hashlib,importlib.util,json,tempfile,time,unittest
from pathlib import Path
import live_core as c

def request(interface='old'):
    r=dict(protocol='tsr-broker@v2',id='a'*64,task=c.TASK,state_root='b'*64,state_version=0,remaining_tokens=200000,remaining_action_calls=20,progress=dict(source_state='c'*64,inspection_current=False,test_current=False,submission='stop submits the current candidate for independent final oracle evaluation; a visible test pass is not acceptance. Stop also consumes one remaining action call.'),deadline_ms=1800000,observation='',allowed_test_ids=['named-failing-test'],required_reads=[c.FILE],failing_tests=c.TESTS,failing_test_id='named-failing-test')
    if interface=='new':r.update(protocol='tsr-broker@inspection-handle-v1',inspection=None)
    return r
class Tests(unittest.TestCase):
    def test_exact_task_scope(self):
        for interface in ('old','new'):
            encode=c.configure(interface);r=request(interface);encode(r)
            for k,v in [('task','0'*64),('required_reads',['other.rs']),('id','private text'),('state_version',True),('allowed_test_ids',['shell']),('remaining_tokens',200001),('extra','secret')]:
                with self.subTest(interface=interface,key=k),self.assertRaises(c.T.BrokerFailure):encode(r|{k:v})
    def test_redaction_and_disclosure(self):
        for interface in ('old','new'):
            encode=c.configure(interface);r=request(interface);r['observation']='error at /Users/person/private/file.rs:17';wire=json.dumps(encode(r));self.assertNotIn('/Users/',wire);self.assertIn('[HOST_PATH]',wire)
            for text in ('sk-'+'x'*30,'-----BEGIN PRIVATE KEY-----','password='+'x'*25):
                r['observation']=text
                with self.assertRaises(c.T.BrokerFailure):encode(r)
    def test_bodies_and_decode(self):
        bodies=[]
        for interface in ('old','new'):
            r=request(interface);body=c.configure(interface)(r);bodies.append(body)
            self.assertEqual(body['max_tokens'],16384);self.assertEqual(body['model'],'deepseek-v4.1-flash')
            response=dict(choices=[dict(finish_reason='tool_calls',message=dict(tool_calls=[dict(type='function',function=dict(name='propose',arguments='{"action":"stop"}'))]))],usage=dict(total_tokens=1))
            self.assertEqual(c.T.decode(response,r)['reply']['proposal']['action'],'stop')
            bad=copy.deepcopy(response);bad['choices'][0]['message']['tool_calls']*=2
            with self.assertRaises(c.T.BrokerFailure):c.T.decode(bad,r)
            bad=copy.deepcopy(response);bad['choices'][0]['finish_reason']='length'
            with self.assertRaises(c.T.BrokerFailure):c.T.decode(bad,r)
        self.assertEqual(bodies[0]['messages'][0]['content'].split('Copy ')[0],bodies[1]['messages'][0]['content'].split('Copy ')[0])
    def test_budget(self):
        baseline=c.budget.history.freeze(b'')
        def ledger(n,cost='0'):
            return b''.join((json.dumps(dict(kind=k,id=str(i),usd='2' if k=='reserve' else cost))+'\n').encode() for i in range(n) for k in ('reserve','settled'))
        c.budget.check(ledger(239),baseline)
        with self.assertRaises(ValueError):c.budget.check(ledger(240),baseline)
        with self.assertRaises(ValueError):c.budget.check(ledger(2,'2'),baseline)
        with self.assertRaises(ValueError):c.budget.check(b'{"kind":"reserve","id":"x","usd":"2"}\n',baseline)
        with self.assertRaises(ValueError):c.budget.check(b'changed',baseline)
    def test_lease(self):
        previous=c.MANIFEST
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);entry=root/'entry.py';entry.write_text('');ledger=root/'ledger';state=root/'state';state.mkdir()
            c.MANIFEST=root/'freeze.json';m=dict(arms=[dict(index=0,broker=str(entry))],ledger=str(ledger),sha256={},state=str(state));c.MANIFEST.write_text(json.dumps(m))
            lease=dict(index=0,deadline=time.time()+30,global_deadline=time.time()+30,manifest_sha256=hashlib.sha256(c.MANIFEST.read_bytes()).hexdigest());(state/'current.json').write_text(json.dumps(lease))
            c.checked_manifest(0,entry,ledger)
            (state/'current.json').write_text(json.dumps(lease|{'deadline':0}))
            with self.assertRaises(c.T.BrokerFailure):c.checked_manifest(0,entry,ledger)
            (state/'current.json').write_text(json.dumps(lease));(state/'terminal.json').write_text('{}')
            with self.assertRaises(c.T.BrokerFailure):c.checked_manifest(0,entry,ledger)
        c.MANIFEST=previous
if __name__=='__main__':unittest.main()
