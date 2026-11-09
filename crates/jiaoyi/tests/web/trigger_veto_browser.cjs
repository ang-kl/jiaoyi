// Run against HTML emitted by trigger_veto_ui.rs, never a broker-connected page.
const fs=require('node:fs');
const http=require('node:http');
const assert=require('node:assert/strict');
const {chromium}=require('playwright');
(async()=>{
  const html=fs.readFileSync(process.argv[2]);
  const checks=[];
  const server=http.createServer((req,res)=>{
    if(req.url==='/status-page.js') {res.writeHead(200,{'Content-Type':'text/javascript'});res.end(fs.readFileSync(require('node:path').join(__dirname,'../../src/trigger_pager.js')));return;}
    res.writeHead(200,{'Content-Type':'text/html; charset=utf-8'});res.end(html);
  });
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH||'/usr/bin/chromium',headless:true,args:['--no-sandbox']});
  const page=await browser.newPage({viewport:{width:1366,height:900}});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  const check=(name,yes)=>{assert.ok(yes,name);checks.push(name);};
  try {
    await page.goto(`http://127.0.0.1:${server.address().port}/`,{waitUntil:'networkidle'});
    const table=page.locator('[data-trigger-pager]').filter({has:page.locator('.entry-gate')});
    for(const label of ['Vetoed: 5d/20d macro','Vetoed: peer ETHUSD','Blocked: pair reservation','Armed: waiting','Invalidated: retraced','Blocked: prepared admission unavailable']) {
      check(label,await table.locator('.entry-gate').filter({hasText:label}).first().isVisible());
    }
    check('numeric distance coexists with veto',await table.locator('tbody tr').first().innerText().then(t=>t.includes('-0.25')&&t.includes('Vetoed:')));
    check('first20 rows visible',await table.locator('[data-trigger-row]:visible').count()===20);
    await table.locator('[data-page-next]').click();
    check('remaining5 rows visible after paging',await table.locator('[data-trigger-row]:visible').count()===5);
    check('reason is escaped, no script execution',await page.evaluate(()=>!window.bad&&!document.querySelector('.entry-gate img')));
    await page.setViewportSize({width:390,height:700});
    check('mobile table scrolls without document overflow',await table.locator('.trigger-scroll').evaluate(e=>e.scrollWidth>e.clientWidth&&document.documentElement.scrollWidth<=window.innerWidth));
    check('no JavaScript errors',errors.length===0);
    process.stdout.write(JSON.stringify({passed:checks.length,checks,scope:'Actual Rust-rendered synthetic fixture in Chromium; no trading or production access'})+'\n');
  }finally {await browser.close();await new Promise(resolve=>server.close(resolve));}
})().catch(e=>{process.stderr.write(e.stack+'\n');process.exitCode=1;});
