'use strict';
// Independent generic observations; no authored gameplay or game-state mutation.
const fs=require('fs'),crypto=require('crypto');
const {chromium}=require('/opt/tsr/node_modules/playwright');
(async()=>{const assets=new Map(['index.html','style.css','app.js'].map(n=>[n,fs.readFileSync('/project/'+n)]));
const browser=await chromium.launch({headless:true,chromiumSandbox:false});
try{const context=await browser.newContext({viewport:{width:1000,height:900},serviceWorkers:'block',acceptDownloads:false});const errors=[],checks=[],trace=[];
await context.route('**/*',async route=>{const u=new URL(route.request().url()),n=u.pathname.slice(1);if(u.origin==='http://tsr.invalid'&&!u.search&&assets.has(n))return route.fulfill({body:assets.get(n),contentType:{html:'text/html',css:'text/css',js:'text/javascript'}[n.split('.').pop()],headers:{'Content-Security-Policy':"default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'none'; object-src 'none'; frame-src 'none'; base-uri 'none'; form-action 'none'"}});errors.push('blocked-resource');await route.abort();});
const page=await context.newPage();page.setDefaultTimeout(3000);page.on('pageerror',e=>errors.push(e.message));page.on('dialog',async d=>{errors.push('dialog');await d.dismiss();});
await page.goto('http://tsr.invalid/index.html');const text=()=>page.locator('body').innerText();trace.push({phase:'initial',text:await text(),title:await page.title()});
await page.locator('[data-tsr=start]').click();await page.waitForTimeout(100);checks.push({name:'start exposes Playing interface',pass:(await page.locator('[data-tsr=status]').innerText()).includes('Playing')});
for(const key of ['ArrowRight','ArrowLeft','Space','ArrowUp','z','x']){await page.keyboard.down(key);await page.waitForTimeout(120);await page.keyboard.up(key);trace.push({input:key,text:await text()});}
await page.locator('[data-tsr=start]').click();trace.push({phase:'restart click',text:await text()});const desktop=await page.screenshot({fullPage:true});
await page.setViewportSize({width:390,height:844});checks.push({name:'mobile viewport has no horizontal overflow',pass:await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)});trace.push({phase:'mobile',text:await text()});const mobile=await page.screenshot({fullPage:true});checks.push({name:'no page errors or outside resource attempts',pass:errors.length===0,observed:errors});
console.log(JSON.stringify({version:'independent-browser-observations@v1',source:Object.fromEntries([...assets].map(([n,b])=>[n,crypto.createHash('sha256').update(b).digest('hex')])),checks,trace,limitations:'Generic controls only, not proof of shooting/traversal/collisions/progression or enjoyment. No state injection.',screenshot_base64:desktop.toString('base64'),mobile_screenshot_base64:mobile.toString('base64')}));
}finally{await browser.close();}})().catch(e=>{console.error(e.stack);process.exitCode=1;});
