'use strict';
// Protected generic harness. Never require/import/eval editable application code in Node.
const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const sha = x => crypto.createHash('sha256').update(x).digest('hex');
const validPath = n => typeof n==='string' && n.length<=128 && n.split('/').every(p=>p.length && !/^[.-]/.test(p) && /^[A-Za-z0-9_.-]+$/.test(p));
const plain = x => x && typeof x === 'object' && !Array.isArray(x);
function readLocal(name, limit) {
  if (!validPath(name)) throw Error('path boundary');
  let p = process.cwd();
  for (const part of name.split('/')) { p=path.join(p,part); if(fs.lstatSync(p).isSymbolicLink()) throw Error('symlink boundary'); }
  const b=fs.readFileSync(p); if(b.length>limit) throw Error('file bound'); return b;
}
function validate(c) {
  if(!plain(c)||Object.keys(c).sort().join()!=='actions,assets,checks,entry,version'||c.version!=='browser-probe@v1') throw Error('probe contract');
  if(!Array.isArray(c.assets)||c.assets.length<1||c.assets.length>8||new Set(c.assets).size!==c.assets.length||!c.assets.includes(c.entry)||!c.entry.endsWith('.html')) throw Error('asset scope');
  for(const n of c.assets) if(!validPath(n)||! /\.(html|css|js)$/.test(n)) throw Error('asset path');
  if(!Array.isArray(c.actions)||c.actions.length>12||!Array.isArray(c.checks)||!c.checks.length||c.checks.length>8) throw Error('probe bounds');
  for(const a of c.actions) {
    if(!plain(a)||Object.keys(a).sort().join()!=='kind,value') throw Error('action shape');
    if(a.kind==='wait') {if(!Number.isInteger(a.value)||a.value<0||a.value>1000)throw Error('wait bound');}
    else if(!['click','key'].includes(a.kind)||typeof a.value!=='string'||!a.value.length||a.value.length>128) throw Error('action bound');
  }
  for(const q of c.checks)if(!plain(q)||Object.keys(q).sort().join()!=='selector,text'||typeof q.selector!=='string'||q.selector.length>128||!q.selector||typeof q.text!=='string'||q.text.length>128)throw Error('check shape');
  return c;
}
async function run(c, chromium, {containerOnly=false}={}) {
  if(typeof containerOnly!=='boolean')throw Error('sandbox mode contract');
  validate(c);
  const assets=new Map(c.assets.map(n=>[n,readLocal(n,1048576)]));
  if([...assets.values()].reduce((s,b)=>s+b.length,0)>1048576)throw Error('asset total');
  const files=Object.fromEntries([...assets].map(([n,b])=>[n,sha(b)]));
  const errors=[]; const observations=[]; let errorCount=0;
  const error = text => {errorCount++; if(errors.length<8)errors.push(text);};
  const browser=await chromium.launch({headless:true,chromiumSandbox:!containerOnly}); // Explicit owner opt-in; outer container controls remain mandatory.
  try {
    const context=await browser.newContext({serviceWorkers:'block',acceptDownloads:false,viewport:{width:800,height:600}});
    await context.route('**/*',async route=>{
      const u=new URL(route.request().url());const name=u.pathname.slice(1);
      if(u.origin==='http://tsr.invalid'&&!u.search&&assets.has(name)) {
        const ext=name.split('.').pop();return route.fulfill({status:200,body:assets.get(name),contentType:{html:'text/html',css:'text/css',js:'text/javascript'}[ext],headers:{'Content-Security-Policy':"default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'none'; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'"}});
      }
      error('blocked resource');await route.abort();
    });
    const page=await context.newPage();page.setDefaultTimeout(3000);
    page.on('pageerror',e=>error('page error: '+String(e.message).slice(0,160)));
    page.on('console',m=>{if(m.type()==='error')error('console error: '+String(m.text()).slice(0,160));});
    page.on('dialog',async d=>{error('dialog');await d.dismiss();});
    page.on('popup',async p=>{error('popup');await p.close();});
    await page.goto('http://tsr.invalid/'+c.entry,{waitUntil:'load',timeout:5000});
    observations.push({kind:'render',sha256:sha(await page.screenshot({type:'png'}))});
    for(const a of c.actions) {
      if(a.kind==='click')await page.locator(a.value).click();
      else if(a.kind==='key')await page.keyboard.press(a.value);
      else await page.waitForTimeout(a.value);
      observations.push({kind:a.kind,value:a.value});
    }
    for(const q of c.checks) {
      const full=await page.locator(q.selector).innerText(); const text=full.slice(0,256);
      observations.push({kind:'text',selector:q.selector,text,abbreviated:full.length>text.length,original_length:full.length,met:full.includes(q.text)});
    }
    observations.push({kind:'render',sha256:sha(await page.screenshot({type:'png'}))});
    return {version:'browser-observation@v1',passed:errorCount===0&&observations.filter(x=>x.kind==='text').every(x=>x.met),files,observations,errors,error_count:errorCount};
  } finally {await browser.close();}
}
if(require.main===module) (async()=>{
  try {
    if(process.argv.length!==3 && !(process.argv.length===4 && process.argv[3]==='--container-only'))throw Error('one protected probe and optional owner sandbox mode required');
    const c=validate(JSON.parse(readLocal(process.argv[2],8192)));
    const {chromium}=require('/opt/tsr/node_modules/playwright');
    const result=await run(c,chromium,{containerOnly:process.argv[3]==='--container-only'});const text=JSON.stringify(result);
    if(Buffer.byteLength(text)>4000)throw Error('observation bound');
    process.stdout.write(text+'\n');process.exitCode=result.passed?0:1;
  }catch(e){process.stderr.write('browser harness unavailable or failed: '+String(e.message).slice(0,256)+'\n');process.exitCode=2;}
})();
module.exports={validate,run};
