"""Offline mapping tests, authored interfaces and independent-review fixtures only."""
import copy,importlib.util,json,os,subprocess,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];PACKET=ROOT/'experiments/game-wants-v3';FIX=ROOT/'experiments/game-wants-v1/fixtures'
s=importlib.util.spec_from_file_location('game_mapping',PACKET/'mapping.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
IMAGE='sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883'
def fixture(kind):
 files={p:(FIX/p).read_text() for p in ('app.js','style.css')};files['index.html']=(FIX/(kind+'.html')).read_text()
 arrows=dict(zip(('Up','Right','Down','Left'),('ArrowUp','ArrowRight','ArrowDown','ArrowLeft')))
 quote='Arrows move.' if kind=='coupled' else 'Arrows cross; WASD moves growth body.'
 mapping={'version':'reviewed-control-map@v1','source':m.source(files),'entry':'index.html','start':'button','controls':{'crossing':arrows,'growth':dict(zip(arrows,('w','d','s','a'))) if kind=='separate' else arrows},'readouts':{'crossing':'output','growth':'output'},'documentation':{'path':'index.html','quote':quote},'limitations':'Authored deterministic discrete fixture; finite coverage only'}
 return files,mapping,review(mapping)
def review(mapping):return {'mapping_sha256':m.digest(mapping),'decision':'accept','attribution':'authored independent-review fixture, not a real runtime model review','reviewer':'authored-reviewer','author':'authored-mapper','limitations':'No human fulfillment or live review proof'}
class Mapping(unittest.TestCase):
 def test_supported_plan_exact_order_and_no_truncation(self):
  for kind in ('coupled','separate'):
   files,v,r=fixture(kind)
   for batch in ('crossing-first','growth-first','mixed-pursuit'):
    self.assertEqual(len(m.search(v,batch)),6)
    for keys in m.search(v,batch):
     p=m.plan(v,files,r,batch,keys);self.assertLessEqual(len(p['probe']['actions']),12);self.assertEqual([a['value'] for a in p['probe']['actions'] if a['kind']=='key'],keys)
    self.assertEqual(m.search(v,batch),m.search(copy.deepcopy(v),batch))
   with self.assertRaisesRegex(ValueError,'no truncation'):m.plan(v,files,r,'crossing-first',['ArrowUp']*7)
 def test_missing_ambiguous_review_and_source(self):
  files,v,r=fixture('coupled');changed={**files,'app.js':files['app.js']+'\n'}
  with self.assertRaisesRegex(ValueError,'stale'):m.validate(v,changed,r)
  with self.assertRaisesRegex(ValueError,'review binding'):m.validate(v,files,{**r,'mapping_sha256':'bad'})
  self.assertEqual(m.validate(v,files,{**r,'decision':'unknown'})['status'],'unknown')
  for kind in ('missing','ambiguous'):
   x=copy.deepcopy(v)
   if kind=='missing':x['readouts']['growth']=''
   else:x['controls']['growth']['Left']='ArrowUp'
   self.assertEqual(m.validate(x,files,review(x))['status'],'unknown')
 def test_batch_labels_match_reviewed_per_position_controls(self):
  files,v,r=fixture('separate')
  for batch,keys in [('growth-first',['ArrowUp']),('crossing-first',['w']),('mixed-pursuit',['ArrowUp','ArrowRight'])]:
   with self.assertRaisesRegex(ValueError,'batch controls'):m.plan(v,files,r,batch,keys)
  self.assertEqual(m.plan(v,files,r,'mixed-pursuit',['ArrowUp','w'])['status'],'mapped')
  files,v,r=fixture('coupled');self.assertEqual(m.plan(v,files,r,'mixed-pursuit',['ArrowUp','ArrowRight'])['status'],'mapped')
 def test_utf8_native_boundaries(self):
  files,v,r=fixture('coupled')
  for field in ('selector','key'):
   x=copy.deepcopy(v)
   if field=='selector':x['start']='é'*64
   else:x['controls']['growth']['Left']='é'*64
   self.assertEqual(m.validate(x,files,review(x))['status'],'mapped')
   if field=='selector':x['start']+='é'
   else:x['controls']['growth']['Left']+='é'
   self.assertEqual(m.validate(x,files,review(x))['status'],'unknown')
 def test_binding_rejects_partial_actions_and_stale_results(self):
  files,v,r=fixture('coupled');p=m.plan(v,files,r,'crossing-first',['ArrowUp']*2)
  observed={'version':'browser-observation@v1','files':m.source(files),'observations':p['probe']['actions']}
  self.assertIsNone(m.bind_observation(p,observed,files)['criterion_verdict'])
  for x in ({**observed,'files':{}},{**observed,'observations':observed['observations'][:-1]}):
   with self.assertRaises(ValueError):m.bind_observation(p,x,files)
 @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','existing isolated browser opt-in')
 def test_actual_mapped_controls_repeat_restart_partial_and_ambiguous(self):
  results=[]
  with tempfile.TemporaryDirectory(dir=ROOT/'target',prefix='mapped-controls-') as d:
   work=Path(d)
   for kind in ('coupled','separate'):
    files,v,r=fixture(kind)
    for n,t in files.items():(work/n).write_text(t)
    growth=['ArrowRight','ArrowUp','ArrowRight','ArrowUp','ArrowUp','ArrowUp'] if kind=='coupled' else ['d','w','d','w','w','w']
    for batch,keys in [('crossing-first',['ArrowUp']*4),('growth-first',growth),('mixed-pursuit',['ArrowUp'])]:
     plan=m.plan(v,files,r,batch,keys);(work/'probe.json').write_text(json.dumps(plan['probe']));pair=[]
     for _ in range(2):
      value=self.run_probe(work);pair.append(value);bound=m.bind_observation(plan,value,files);self.assertIsNone(bound['criterion_verdict']);results.append({'kind':kind,'batch':batch,'evidence':bound})
     self.assertEqual(pair[0],pair[1]);text=' '.join(x.get('text','') for x in pair[0]['observations'])
     if batch=='crossing-first':self.assertIn('crossing reached',text);self.assertIn('growth 0',text)
     if batch=='growth-first':self.assertIn('body 3',text);self.assertIn('crossing reached' if kind=='coupled' else 'crossing open',text)
     if batch=='mixed-pursuit':self.assertIn('crossing open',text)
    self.assertEqual(m.source(files),m.source({n:(work/n).read_text() for n in files}))
   # Selector semantics are checked by real locator behavior, not assumed from a schema.
   for selector in ('#missing','button, output'):
    files,v,r=fixture('separate');v['start']=selector;plan=m.plan(v,files,review(v),'crossing-first',['ArrowUp']);(work/'probe.json').write_text(json.dumps(plan['probe']))
    result=self.run_probe(work,unavailable=True);results.append({'selector':selector,'status':'unknown','returncode':result.returncode,'stderr':result.stderr.decode()})
  (ROOT/'target/game-wants-mapped-controls-results.json').write_text(json.dumps(results,indent=2)+'\n')
 def run_probe(self,work,unavailable=False):
  cmd=['/opt/homebrew/bin/docker','run','--rm','--init','--pull=never','--network=none','--read-only','--cap-drop=ALL','--security-opt=no-new-privileges','--pids-limit=256','--memory=2g','--cpus=2','--user','501:20','--tmpfs=/tmp:rw,nosuid,nodev,exec,size=256m','--mount',f'type=bind,src={work},dst=/project,readonly','--mount',f'type=bind,src={ROOT/"scripts/engine-browser-harness.cjs"},dst=/verify.cjs,readonly','--workdir=/project','--entrypoint=',IMAGE,'node','/verify.cjs','probe.json','--container-only']
  run=subprocess.run(cmd,capture_output=True,timeout=90)
  if unavailable:self.assertEqual(run.returncode,2);return run
  self.assertEqual(run.returncode,0,run.stderr.decode());return json.loads(run.stdout)
