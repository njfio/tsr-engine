import copy, importlib.util, json, os, shutil, subprocess, tempfile, unittest
from pathlib import Path
from unittest.mock import patch
s=importlib.util.spec_from_file_location('next_shadow',Path(__file__).with_name('engine-next-proposal-shadow.py'))
m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class NextProposalShadow(unittest.TestCase):
    def setUp(self):
        if not os.environ.get('TSR_SEMANTIC_SHADOW_WORKER'):self.skipTest('explicit cfg(test) fixture worker required')
        self.tmp=tempfile.TemporaryDirectory(dir=m.ROOT/'target');self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)/'run';self.worker=Path(os.environ['TSR_SEMANTIC_SHADOW_WORKER'])
        self.identity=m.prepare(self.root,self.worker)
    def control(self):return json.loads((self.root/'native/next-control.json').read_text())
    def test_pair_privacy_projection_and_native_modes(self):
        q=m.controller(self.root).request();pair=m.contexts(q)
        off=copy.deepcopy(pair['guided']);off['current_guidance']=None
        self.assertEqual(off,pair['unguided'])
        plain=json.dumps(pair['unguided'])
        for text in list(m.DEVELOPED.values())+list(m.DEVELOPMENT.values()):self.assertNotIn(text,plain)
        self.assertNotIn('PRIVATE',json.dumps(pair));self.assertEqual(pair['guided']['binding'],pair['unguided']['binding'])
        dirty=copy.deepcopy(q);dirty['acceptance_cycle']={'private':'SECRET'}
        dirty['coding']['relational']['inputs']['returns'][0]['private']='SECRET'
        self.assertEqual(m.contexts(dirty),pair)
        next(f for f in dirty['coding']['files'] if f.get('editable'))['source']='private repository bytes'
        with self.assertRaises(ValueError):m.project(dirty)
        # A formatting-only authored control exercises the exact-edit branch, not usefulness.
        source=pair['guided']['source']
        edit={'protocol':'tsr-proposal@v1','state_root':q['state_root'],'attention_binding':q['next_attention']['binding'],
              'proposals':[{'kind':'exact_edit','patch':{'path':source['path'],'source_digest':source['source_digest'],'inspection_handle':source['inspection_handle'],'old_text':source['source'],'new_text':source['source']+'\n'},'rationale':'Authored structural control only, not a useful-action claim.'}]}
        preview=m.native_validate(self.root,'guided',edit,self.worker,m.binding(q))
        self.assertEqual(preview['assessments'][0]['disposition'],'eligible');self.assertFalse(preview['executed'])
        for field in ('inspection_handle','source_digest'):
            bad=copy.deepcopy(edit);bad['proposals'][0]['patch'][field]='0'*64
            with self.assertRaises(ValueError):m.native_validate(self.root,'guided',bad,self.worker,m.binding(q))
        before=q['state_root'];result=m.native_validate(self.root,'guided',self.control(),self.worker,m.binding(q))
        self.assertFalse(result['adopted']);self.assertFalse(result['executed']);self.assertEqual(m.controller(self.root).request()['state_root'],before)
        # The old revision validator does not acquire next-proposal authority.
        with self.assertRaises(ValueError):m.b.native_validate(self.root,'reject',self.control(),self.worker,m.b.native_binding(q))
        reordered=json.loads(json.dumps(self.control(),sort_keys=True))
        self.assertEqual(result,m.native_validate(self.root,'unguided',reordered,self.worker,m.binding(q)))
    def test_negative_native_and_private_fields(self):
        q=m.controller(self.root).request();expected=m.binding(q)
        for mutation in ('parent','path','unknown','kind','attention'):
            v=self.control()
            if mutation=='parent':v['proposals'][0]['experiment']['mapping']={'private':v['proposals'][0]['experiment']['mapping']['second']}
            elif mutation=='path':v['proposals'][0]['experiment']['harness']={'tests/original.rs':'#[test] fn boundary() {}'}
            elif mutation=='unknown':v['private_review']='SECRET'
            elif mutation=='kind':v['proposals'][0]['kind']='relational'
            else:v['attention_binding']='0'*64
            with self.assertRaises(ValueError,msg=mutation):m.native_validate(self.root,'guided',v,self.worker,expected)
        bad=copy.deepcopy(expected);bad['snapshot']='0'*64
        with self.assertRaisesRegex(ValueError,'stale'):m.native_validate(self.root,'guided',self.control(),self.worker,bad)
        self.assertEqual(m.native_validate(self.root,'guided',{'inability':'Insufficient evidence for a justified next direction.'},self.worker,expected)['native'],'no proposal')
        with self.assertRaises(ValueError):m.native_validate(self.root,'guided',{'inability':'','authority':True},self.worker,expected)
    def test_manifest_restart_no_dispatch_and_source_drift(self):
        command=['python3',str(m.ROOT/'scripts/engine-next-proposal-shadow.py'),'preflight','--directory',str(self.root),'--manifest-sha256',self.identity]
        self.assertEqual(subprocess.run(command,capture_output=True).returncode,0)
        self.assertFalse((self.root/'events.jsonl').exists())
        with self.assertRaisesRegex(ValueError,'admission'):m.run(self.root,self.identity,allow_paid=True)
        copied=Path(self.tmp.name)/'copy';shutil.copytree(self.root,copied)
        with self.assertRaises(ValueError):m.ready(copied,self.identity)
        source=self.root/'native/reject/coding-work/src/lib.rs';old=source.read_bytes();source.write_bytes(b'// changed\n')
        with self.assertRaises(ValueError):m.ready(self.root,self.identity)
        source.write_bytes(old);m.ready(self.root,self.identity)
        manifest=json.loads((self.root/'manifest.json').read_text())
        with patch.object(m.time,'time',return_value=manifest['expires_at']+1):
            with self.assertRaises(ValueError):m.ready(self.root,self.identity)
            self.assertEqual(m.b.inspect(self.root,self.identity)['calls_consumed_or_reserved'],0)
    def response(self,body,*args,**kwargs):
        self.assertEqual(body['model'],'gpt-6-luna');self.assertEqual(body['max_completion_tokens'],8192)
        return {'value':self.control(),'receipt':{'status':'settled','usd':'0.001','response_status':'received','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200}}}
    def test_profile_accounting_zero_adoption_and_restart(self):
        q=m.controller(self.root).request();calls=[]
        def paired(body,*args,**kwargs):
            result=self.response(body,*args,**kwargs);calls.append(body)
            if len(calls)==1:result['value']={'inability':'The fixture does not establish a justified new direction.'}
            return result
        m.run(self.root,self.identity,transport=paired,fixture=True)
        self.assertEqual(json.loads((self.root/'candidate-0.json').read_text())['validation']['native'],'no proposal')
        self.assertEqual(m.controller(self.root).request()['state_root'],q['state_root'])
        result=m.b.inspect(self.root,self.identity);self.assertEqual(result['calls_consumed_or_reserved'],2)
        self.assertEqual(result['known_usd_subtotal'],'0.002');self.assertEqual(result['terminal']['status'],'complete_advisory_only')
        for i in range(2):
            c=json.loads((self.root/f'candidate-{i}.json').read_text());self.assertIn('typed_next_proposal',c);self.assertFalse(c['validation']['adopted'])
        with self.assertRaises(FileExistsError):m.run(self.root,self.identity,transport=lambda *_a,**_k:self.fail('redispatch'),fixture=True)
    def test_unknown_reservation_and_post_control_runway(self):
        validate=m.native_validate;manifest=json.loads((self.root/'manifest.json').read_text());clock=[m.time.time()]
        def slow(*args,**kwargs):
            result=validate(*args,**kwargs);clock[0]=manifest['expires_at']-100;return result
        with patch.object(m.time,'time',side_effect=lambda:clock[0]),patch.object(m,'native_validate',side_effect=slow):
            with self.assertRaisesRegex(ValueError,'runway'):m.run(self.root,self.identity,transport=lambda *_a,**_k:self.fail('dispatch'),fixture=True)
        self.assertFalse((self.root/'events.jsonl').exists())
        def lost(*args,**kwargs):raise TimeoutError('fixture only')
        with self.assertRaises(TimeoutError):m.run(self.root,self.identity,transport=lost,fixture=True)
        result=m.b.inspect(self.root,self.identity);self.assertEqual(result['calls_consumed_or_reserved'],1);self.assertEqual(result['unknown_reservations'],[0])
        with self.assertRaises(FileExistsError):m.run(self.root,self.identity,transport=self.response,fixture=True)
if __name__=='__main__':unittest.main()
