'use strict';
// Authored empty-install API; no provider, native mutation or real project state.
const fs=require('fs'),assert=require('assert'),{chromium}=require('/opt/tsr/node_modules/playwright');
(async()=>{
 const browser=await chromium.launch({headless:true,chromiumSandbox:false});
 try{
  const page=await browser.newPage(),errors=[],unexpected=[];
  page.on('pageerror',e=>errors.push(String(e)));
  await page.route('**/*',async route=>{
   const url=new URL(route.request().url());
   if(url.pathname==='/api/bootstrap')return route.fulfill({contentType:'application/json',body:JSON.stringify({csrf:'authored-token',projects:[],budget:{available:false,reason:'AUTHORED unconfigured campaign'},recovery:{required:false}})});
   const files={'/':'index.html','/app.js':'app.js','/style.css':'style.css','/vendor/react.js':'vendor/react.js','/vendor/react-dom.js':'vendor/react-dom.js'};
   if(!files[url.pathname]){unexpected.push(url.pathname);return route.fulfill({status:404,body:'unexpected fixture request'});}
   return route.fulfill({contentType:url.pathname.endsWith('.js')?'text/javascript':url.pathname.endsWith('.css')?'text/css':'text/html',body:fs.readFileSync('/project/ui/engine/'+files[url.pathname])});
  });
  await page.goto('http://127.0.0.1:8766/');
  await page.getByRole('heading',{name:'What do you want to make?',exact:true}).waitFor();
  await page.getByLabel('Describe your project').fill('AUTHORED unsent fresh-install draft');
  await page.waitForTimeout(5500);
  assert.equal(await page.getByLabel('Describe your project').inputValue(),'AUTHORED unsent fresh-install draft');
  await page.reload();await page.getByLabel('Describe your project').waitFor();
  assert.equal(await page.getByLabel('Describe your project').inputValue(),'AUTHORED unsent fresh-install draft');
  await page.evaluate(()=>{localStorage.setItem('tsr.project','missing-old-id');localStorage.setItem('tsr.draft.missing-old-id','AUTHORED preserved old draft');});
  await page.reload();await page.getByLabel('Describe your project').waitFor();
  await page.waitForFunction(()=>localStorage.getItem('tsr.project')==='new');
  assert.equal(await page.getByLabel('Describe your project').inputValue(),'AUTHORED unsent fresh-install draft');
  await page.getByLabel('Describe your project').fill('AUTHORED latest draft after stale selection');
  await page.reload();await page.getByLabel('Describe your project').waitFor();
  assert.equal(await page.getByLabel('Describe your project').inputValue(),'AUTHORED latest draft after stale selection');
  assert.equal(await page.evaluate(()=>localStorage.getItem('tsr.draft.missing-old-id')),'AUTHORED preserved old draft');
  assert.equal(await page.getByRole('alert').count(),0);assert.deepEqual(unexpected,[]);assert.deepEqual(errors,[]);
  console.log(JSON.stringify({fixture_only:true,checks:['empty installation renders new-project flow','no nonexistent historical project request','poll and reload preserve unsent draft','stale selection migrates without deleting old or latest draft','no JavaScript or load error'],provider_calls:0}));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exit(1)});
