// Protected fixed observer. Contains no condition facts, current or future.
const tasks=[{id:"room",objective:"Access current quiet-reading information"},{id:"reading",objective:"Access another supplied reading topic"},{id:"prior",objective:"Access superseded-direction context or its truthful absence"}];
const fs=require('fs'),crypto=require('crypto');
const {chromium}=require('/opt/tsr/node_modules/playwright');
(async()=>{
 const assets=new Map(['index.html','style.css','app.js'].map(n=>[n,fs.readFileSync('/project/'+n)]));
 const source=Object.fromEntries([...assets].map(([n,b])=>[n,crypto.createHash('sha256').update(b).digest('hex')]));
 const browser=await chromium.launch({headless:true,chromiumSandbox:false});
 try{
  const context=await browser.newContext({serviceWorkers:'block',acceptDownloads:false});let blocked=0;const errors=[];
  await context.route('**/*',async r=>{const u=new URL(r.request().url()),n=u.pathname.slice(1);if(u.origin==='http://tsr.invalid'&&!u.search&&assets.has(n))return r.fulfill({body:assets.get(n),contentType:{html:'text/html',css:'text/css',js:'text/javascript'}[n.split('.').pop()],headers:{'Content-Security-Policy':"default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'none'; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'"}});blocked++;await r.abort();});
  const page=await context.newPage();page.setDefaultTimeout(2500);page.on('pageerror',e=>errors.push(String(e)));page.on('dialog',d=>d.dismiss());page.on('popup',p=>p.close());
  const reports=[];
  for(const width of [1280,320]){
   await page.setViewportSize({width,height:800});await page.goto('http://tsr.invalid/index.html',{timeout:5000});
   for(const task of tasks){
    const selector=`[data-tsr-task="${task.id}"]`,loc=page.locator(selector);let outcome='unobserved',reason='',tabs=0;
    const before=await page.locator('body').innerText();
    try{
     if(await loc.count()!==1)throw Error('Task addressability is missing or ambiguous');
     if(!await loc.isVisible()||!await loc.isEnabled())throw Error('Task affordance is not visible and enabled');
     const native=await loc.evaluate(e=>e.tagName==='BUTTON'||(e.tagName==='A'&&e.hasAttribute('href')));
     if(!native)throw Error('Task affordance is not a native link or button');
     // Real keyboard events, no focus injection, click scripting or game-state mutation.
     while(tabs<128&&!await loc.evaluate(e=>e===document.activeElement)){await page.keyboard.press('Tab');tabs++;}
     if(!await loc.evaluate(e=>e===document.activeElement))throw Error('Keyboard observation capacity reached; reachability remains unknown');
     await page.keyboard.press('Enter');await page.waitForTimeout(80);outcome='enter_dispatched';
    }catch(e){reason=String(e);}
    const after=await page.locator('body').innerText();reports.push({viewport:width,task:task.id,objective:task.objective,outcome,reason,tabs,before_text:before,after_text:after,limitations:'Enter dispatched is not proof of a successful handler or navigation. The before/after text is the actual returned observation; interpret it against current source and guidance. Already-visible information is valid; activation need not change text. No inference about a human finding, liking or learning from this content.'});
   }
  }
  const observations={source,checks:[{name:'No observed page errors',pass:errors.length===0},{name:'No unadmitted resource requests',pass:blocked===0}],tasks:reports,errors,blocked_resources:blocked,limitations:'Authored synthetic retrieval probes through actual keyboard controls; not human fulfillment, universal accessibility, or causal learning. Technical unavailability and capacity exhaustion are unknown, never automatically unmet. Source and criterion review remain independent.'};
  if(Buffer.byteLength(JSON.stringify(observations))>131072)throw Error('Observation transport capacity exceeded; no truncation');
  console.log(JSON.stringify(observations));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1)});
