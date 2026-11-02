// Fixed authored source-invariant observer; no player preferences or future conditions.
const fs=require('fs'),crypto=require('crypto');
const {chromium}=require('/opt/tsr/node_modules/playwright');
(async()=>{
 const names=['index.html','style.css','app.js'];
 const assets=new Map(names.map(n=>[n,fs.readFileSync('/project/'+n)]));
 const source=Object.fromEntries([...assets].map(([n,b])=>[n,crypto.createHash('sha256').update(b).digest('hex')]));
 const browser=await chromium.launch({headless:true,chromiumSandbox:false});
 try {
  const context=await browser.newContext({serviceWorkers:'block',acceptDownloads:false,viewport:{width:1280,height:800}});
  let blocked=0;const errors=[];
  await context.route('**/*',async route=>{
   const u=new URL(route.request().url()),name=u.pathname.slice(1);
   if(u.origin==='http://tsr.invalid'&&!u.search&&assets.has(name))return route.fulfill({status:200,body:assets.get(name),contentType:{html:'text/html',css:'text/css',js:'text/javascript'}[name.split('.').pop()],headers:{'Content-Security-Policy':"default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'none'; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'"}});
   blocked++;await route.abort();
  });
  const page=await context.newPage();page.setDefaultTimeout(3000);
  page.on('pageerror',e=>errors.push(String(e)));page.on('dialog',d=>d.dismiss());page.on('popup',p=>p.close());
  await page.goto('http://tsr.invalid/index.html',{timeout:5000});await page.locator('[data-tsr=start]').click();await page.waitForTimeout(250);
  const desktop=await page.locator('body').innerText();
  const focus=[];for(let i=0;i<24;i++){await page.keyboard.press('Tab');focus.push(await page.evaluate(()=>({tag:document.activeElement.tagName,text:document.activeElement.innerText||document.activeElement.getAttribute('aria-label')||''})));}
  const screenshot_base64=(await page.screenshot({fullPage:true})).toString('base64');
  await page.setViewportSize({width:320,height:700});await page.waitForTimeout(100);
  const narrow=await page.locator('body').innerText();const width=await page.evaluate(()=>({document:document.documentElement.scrollWidth,viewport:innerWidth}));
  if(page.url()!=='http://tsr.invalid/index.html')throw Error('Unadmitted final page URL');
  const mobile_screenshot_base64=(await page.screenshot({fullPage:true})).toString('base64');
  const observations={source,checks:[{name:'Ready status visible',pass:desktop.includes('Ready')},{name:'Narrow layout has no horizontal overflow',pass:width.document<=width.viewport},{name:'No observed page errors',pass:errors.length===0},{name:'No unadmitted resource requests',pass:blocked===0}],desktop_text:desktop,narrow_text:narrow,keyboard_focus:focus,width,errors,blocked_resources:blocked,coverage:{text:'complete visible body text at two viewports; no clipping',keyboard:'24 Tab focus transitions; no activation or hidden-detail accessibility proof'},limitations:'Source-invariant authored browser observations, not people, current scenario correctness, discovery quality, desirability or causal learning. Visible labels do not establish all detail interactions.'};
  if(Buffer.byteLength(JSON.stringify(observations))>131072)throw Error('Complete observation exceeds declared report capacity; no truncation');
  console.log(JSON.stringify({...observations,screenshot_base64,mobile_screenshot_base64}));
 } finally {await browser.close();}
})().catch(e=>{console.error(e);process.exit(1)});
