"""Authored UI fixtures; actual isolated browser, never engine-generated output."""
import hashlib,importlib.util,json,os,subprocess,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];PACKET=ROOT/'experiments/relational-observation-v1'
s=importlib.util.spec_from_file_location('packet',PACKET/'prepare.py');p=importlib.util.module_from_spec(s);s.loader.exec_module(p)
class Packet(unittest.TestCase):
 def test_frozen_projection_has_only_eligible_condition_and_uncapped_policy(self):
  p.check();data=json.loads((PACKET/'conditions.json').read_text())
  for i in range(3):
   projection=p.projection(i);self.assertEqual(projection['condition'],data['conditions'][i])
   self.assertNotIn('conditions',projection)
   self.assertNotIn(data['conditions'][i]['facts'][0],(PACKET/'observe.cjs').read_text())
  policy=json.loads((PACKET/'manifest.json').read_text())['policy'];self.assertTrue(all(v is None for k,v in policy.items() if k.startswith('cumulative_')))
 def test_three_condition_session_stop_resume_and_idempotence(self):
  import types
  spec=importlib.util.spec_from_file_location('session',ROOT/'scripts/engine-browser-session.py');session=importlib.util.module_from_spec(spec);spec.loader.exec_module(session)
  with tempfile.TemporaryDirectory(dir=ROOT/'target') as folder:
   root=Path(folder).resolve();done={};applied=[];dispatch=[];returns=[]
   events=[p.projection(i) for i in range(3)]
   def feedback(i):
    if 'feedback_'+str(i) not in done:applied.append(events[i]['condition']['id']);done['feedback_'+str(i)]={}
   a=types.SimpleNamespace(root=root/'adapter',admission={'feedback_events':events},read=lambda:([],{},done),ledger=types.SimpleNamespace(status=lambda:{'stopped':False}),complete_pending_return=lambda:returns.append(tuple(applied)),apply_feedback=feedback,native=lambda *args:{'branch':'propose'},proposal=None,review=None)
   class Cycle:
    def __init__(self,path,*args):self.path=Path(path)
    def run(self):
     self.path.mkdir(exist_ok=True);record=self.path/'authored-result.json'
     if record.exists():return json.loads(record.read_text())
     dispatch.append(self.path.name);value={'status':'proposal_retention'};record.write_text(json.dumps(value))
     if len(dispatch)==1:(root/'STOP').write_text('Authored user Stop fixture')
     return value
   with self.assertRaisesRegex(ValueError,'session stopped'):session.run(a,{'journal':str(root)},cycle_type=Cycle)
   self.assertEqual(applied,['opening']);self.assertEqual(dispatch,['checkpoint-0'])
   (root/'STOP').unlink() # Explicit authored resume, not a real user Stop.
   result=session.run(a,{'journal':str(root)},cycle_type=Cycle)
   self.assertEqual(result['status'],'completed_checkpoints');self.assertEqual(applied,['opening','storm','reopening']);self.assertEqual(len(dispatch),3)
   self.assertIn(('opening',),returns);self.assertIn(('opening','storm'),returns)
   self.assertEqual(session.run(a,{'journal':str(root)},cycle_type=Cycle),result);self.assertEqual(len(dispatch),3)
 @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','existing isolated runtime opt-in')
 def test_control_observer_different_ui_and_negative_fixtures(self):
  results=[]
  with tempfile.TemporaryDirectory(dir=ROOT/'target',prefix='observation-packet-') as d:
   work=Path(d)
   for index,kind in [(0,'links'),(1,'buttons'),(2,'buttons'),(0,'missing'),(0,'duplicate')]:
    condition=p.projection(index)['condition'];controls=[];panels=[]
    for task in condition['tasks']:
     ident=task['id'];text=' | '.join(task['required_visible'])
     if kind!='missing':
      control=(f'<a data-tsr-task="{ident}" href="#{ident}">{ident}</a>' if kind=='links' else f'<button data-tsr-task="{ident}" onclick="document.getElementById(\'{ident}\').hidden=false">{ident}</button>')
      controls.append(control)
      if kind=='duplicate' and ident=='room':controls.append(control)
     panels.append(f'<section id="{ident}"'+(' hidden' if kind=='buttons' else '')+f'>{text}</section>')
    (work/'index.html').write_text('<meta charset="utf-8"><title>Authored harness fixture</title>'+''.join(controls+panels))
    # Inline scripts are CSP-blocked; wire real fixture handlers through permitted file.
    if kind=='buttons':
     html=(work/'index.html').read_text();import re
     html=re.sub(r' onclick="[^"]*"','',html);(work/'index.html').write_text(html+'<script src="app.js"></script>')
    (work/'app.js').write_text("document.querySelectorAll('button[data-tsr-task]').forEach(b=>b.onclick=()=>document.getElementById(b.dataset.tsrTask).hidden=false);")
    (work/'style.css').write_text('/* Authored fixture, no engine output */')
    before={n:hashlib.sha256((work/n).read_bytes()).hexdigest() for n in ['index.html','app.js','style.css']}
    cmd=['docker','run','--rm','--init','--pull=never','--network=none','--read-only','--cap-drop=ALL','--security-opt=no-new-privileges','--pids-limit=256','--memory=2g','--cpus=2','--user','501:20','--tmpfs=/tmp:rw,nosuid,nodev,exec,size=256m','--mount',f'type=bind,src={work},dst=/project,readonly','--mount',f'type=bind,src={PACKET/"observe.cjs"},dst=/verify.cjs,readonly','--entrypoint=','sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883','node','/verify.cjs']
    run=subprocess.run(cmd,capture_output=True,timeout=90);self.assertEqual(run.returncode,0,run.stderr.decode());r=json.loads(run.stdout);self.assertEqual(r['source'],before);self.assertEqual(len(r['tasks']),6)
    if kind in ('links','buttons'):
     self.assertTrue(all(t['outcome']=='enter_dispatched' for t in r['tasks']));self.assertTrue(all(text.lower() in t['after_text'].lower() for t in r['tasks'] for task in condition['tasks'] if task['id']==t['task'] for text in task['required_visible']))
    else:self.assertTrue(any(t['outcome']=='unobserved' for t in r['tasks']));self.assertNotIn('not_met',str(r))
    self.assertEqual(before,{n:hashlib.sha256((work/n).read_bytes()).hexdigest() for n in before});results.append({'fixture':kind,'condition':condition['id'],'observations':r})
  (ROOT/'target/relational-observation-authored-results.json').write_text(json.dumps(results,indent=2)+'\n')
if __name__=='__main__':unittest.main()
