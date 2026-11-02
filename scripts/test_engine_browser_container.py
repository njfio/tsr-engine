#!/usr/bin/env python3
"""Explicit opt-in real browser verification of authored infrastructure fixtures.
No model transport, campaign ledger, game code or writable host runtime mount.
"""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
import os
IMAGE='sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883'
ROOT=Path(__file__).resolve().parents[1]
@unittest.skipUnless(os.getenv('TSR_BROWSER_CONTAINER_AUTHORED')=='1','explicit isolated authored browser test opt-in')
class BrowserContainer(unittest.TestCase):
    def test_protected_harness_browser_and_resource_boundaries(self):
        with tempfile.TemporaryDirectory(prefix='browser-authored-',dir=ROOT/'target') as tmp:
            p=Path(tmp)
            shutil.copyfile(ROOT/'scripts/engine-browser-harness.cjs',p/'browser-harness.cjs')
            html='<link rel="stylesheet" href="style.css"><button id="start">Start</button><p id="status">Idle</p><script src="app.js"></script>'
            js="document.querySelector('#start').onclick=()=>document.querySelector('#status').textContent='Ready';document.onkeydown=()=>document.querySelector('#status').textContent='Moved';"
            probe={'version':'browser-probe@v1','assets':['index.html','style.css','app.js'],'entry':'index.html','actions':[{'kind':'click','value':'#start'},{'kind':'key','value':'ArrowLeft'}],'checks':[{'selector':'#status','text':'Moved'}]}
            (p/'style.css').write_text('button { color: navy; }')
            (p/'probe.json').write_text(json.dumps(probe))
            def run(label,source,markup=html):
                (p/'index.html').write_text(markup);(p/'app.js').write_text(source)
                before={f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in p.iterdir()}
                cmd=['docker','run','--rm','--init','--pull=never','--network=none','--read-only','--cap-drop=ALL','--security-opt=no-new-privileges','--pids-limit=256','--memory=2g','--cpus=2','--user=501:20','--tmpfs=/tmp:rw,nosuid,nodev,exec,size=256m','--mount',f'type=bind,src={p},dst=/workspace,readonly','--workdir=/workspace','--entrypoint=node',IMAGE,'browser-harness.cjs','probe.json','--container-only']
                result=subprocess.run(cmd,capture_output=True,timeout=45)
                self.assertEqual(before,{f.name:hashlib.sha256(f.read_bytes()).hexdigest() for f in p.iterdir()})
                report=json.loads(result.stdout)
                self.assertEqual(report['files'],{n:before[n] for n in probe['assets']})
                evidence.append({'case':label,'command':cmd,'exit':result.returncode,'report':report,'stderr':result.stderr.decode(),'protected_files_unchanged':True})
                return result,report
            evidence=[]
            r,v=run('authored_click_and_key',js);self.assertEqual(r.returncode,0);self.assertTrue(v['passed'])
            r,v=run('external_script_denied',js,html+'<script src="https://example.invalid/no.js"></script>');self.assertEqual(r.returncode,1);self.assertFalse(v['passed']);self.assertGreater(v['error_count'],0)
            r,v=run('protected_probe_not_served',js,html+'<script src="probe.json"></script>');self.assertEqual(r.returncode,1);self.assertFalse(v['passed']);self.assertGreater(v['error_count'],0)
            r,v=run('application_error_not_pass',js+"throw Error('authored failure');");self.assertEqual(r.returncode,1);self.assertFalse(v['passed'])
            if os.getenv('TSR_BROWSER_CONTAINER_REPORT'):
                Path(os.environ['TSR_BROWSER_CONTAINER_REPORT']).write_text(json.dumps({'authored_only':True,'chromium_sandbox':False,'image':IMAGE,'cases':evidence},indent=2)+'\n')
if __name__=='__main__':unittest.main()
