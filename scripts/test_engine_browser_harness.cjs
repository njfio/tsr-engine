'use strict';
// Authored Playwright double only. Never executes the application or launches a browser.
const assert=require('node:assert/strict'),fs=require('node:fs'),os=require('node:os'),path=require('node:path');
const {validate,run}=require('./engine-browser-harness.cjs');
const c={version:'browser-probe@v1',assets:['index.html','app.js'],entry:'index.html',actions:[{kind:'click',value:'#start'},{kind:'key',value:'ArrowLeft'},{kind:'wait',value:1}],checks:[{selector:'#start',text:'Play'}]};
for(const bad of [{...c,assets:['../secret.html']},{...c,assets:['index.html','index.html']},{...c,actions:[{kind:'eval',value:'x'}]},{...c,actions:[{kind:'wait',value:1001}]},{...c,extra:true}])assert.throws(()=>validate(bad));
validate({...c,assets:['index.html','app.min.js']});
assert.throws(()=>validate({...c,assets:['index.html','.hidden.js']}));
let routeHandler,closed=0,expectedSandbox=true;const events={};const routed=[];
const page={setDefaultTimeout:()=>{},on:(k,f)=>events[k]=f,keyboard:{press:async()=>{}},waitForTimeout:async()=>{},screenshot:async()=>Buffer.from('authored pixels'),locator:()=>({click:async()=>{},innerText:async()=> 'Play'+'.'.repeat(300)}),goto:async()=>{await routeHandler({request:()=>({url:()=> 'http://tsr.invalid/index.html'}),fulfill:async v=>routed.push(v),abort:async()=>assert.fail('valid asset blocked')});}};
const chromium={launch:async options=>{assert.equal(options.chromiumSandbox,expectedSandbox);assert.equal(options.args,undefined);return {newContext:async options=>{assert.equal(options.serviceWorkers,'block');assert.equal(options.acceptDownloads,false);return {route:async(_,fn)=>routeHandler=fn,newPage:async()=>page};},close:async()=>closed++};}};
(async()=>{const cwd=process.cwd(),dir=fs.mkdtempSync(path.join(os.tmpdir(),'tsr-browser-authored-'));try{
process.chdir(dir);fs.writeFileSync('index.html','abc');fs.writeFileSync('app.js','// not executed');
const good=await run(c,chromium);assert.equal(good.passed,true);assert.equal(good.files['index.html'],'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');assert.equal(good.observations.find(x=>x.kind==='text').abbreviated,true);assert.equal(good.observations.find(x=>x.kind==='text').original_length,304);assert(routed[0].headers['Content-Security-Policy'].includes("connect-src 'none'"));
const old=page.goto;page.goto=async()=>{await old();for(let i=0;i<1000;i++)events.console({type:()=> 'error',text:()=> 'authored diagnostic'});await routeHandler({request:()=>({url:()=> 'http://tsr.invalid/probe.json'}),abort:async()=>{},fulfill:async()=>assert.fail('protected file exposed')});};
const bad=await run(c,chromium);assert.equal(bad.passed,false);assert.equal(bad.errors.length,8);assert.equal(bad.errors[0],'console error: authored diagnostic');assert.equal(bad.error_count,1001);assert.equal(closed,2);
expectedSandbox=false;await run(c,chromium,{containerOnly:true});
await assert.rejects(run(c,chromium,{containerOnly:'false'}),/mode contract/);
fs.unlinkSync('app.js');fs.symlinkSync('/etc/passwd','app.js');await assert.rejects(run(c,chromium),/symlink/);
process.chdir(cwd);console.log('PASS: authored harness controls (not browser execution)');
}finally{process.chdir(cwd);fs.rmSync(dir,{recursive:true,force:true});}})().catch(e=>{console.error(e);process.exitCode=1;});
