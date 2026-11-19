import {TOTP} from 'otpauth';
import {createServer} from 'node:http';
import {randomBytes} from 'node:crypto';
import {readFile} from 'node:fs/promises';
import {pathToFileURL} from 'node:url';
import {createIdentity, readSettings} from './oidc.mjs';

const sid = '__Host-reports', flow = '__Host-reports-login', step = '__Host-reports-factor';
const sample = new URL('./sample.json', import.meta.url);
const random = () => randomBytes(32).toString('base64url');
const cookie = (name, value, seconds) =>
  name + '=' + value + '; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=' + seconds;
const esc = value => String(value).replace(/[&<>"']/g, c =>
  ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'})[c]);
const page = (title, body) => '<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>' + title + '</title><style>html{background:#f5f4ee;color:#192c29;font:17px/1.6 system-ui,sans-serif}main{max-width:38rem;margin:10vh auto;padding:2rem}h1{font:2.6rem/1.1 Georgia,serif}a,button{color:#135d52}button{font:inherit;cursor:pointer}time,small{color:#495f59}</style><main><h1>' + title + '</h1>' + body + '</main></html>';

export function createReader({settings, identity = settings && createIdentity(settings), now = Date.now}) {
  const attempts = new Map();
  let session, challenge;
  let failures = [], lastCounter = -1;
  const totp = settings?.google && new TOTP({secret:settings.otpSecret, digits:6, period:30, algorithm:'SHA1'});
  return createServer(async (req, res) => {
    res.setHeader('Cache-Control','private, no-store');
    res.setHeader('Referrer-Policy','no-referrer');
    res.setHeader('X-Content-Type-Options','nosniff');
    res.setHeader('Content-Security-Policy',"default-src 'none'; style-src 'unsafe-inline'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'");
    const send = (code, text, type='text/plain; charset=utf-8') => {
      res.writeHead(code, {'Content-Type':type}); res.end(text);
    };
    const redirect = to => {res.writeHead(303,{Location:to}); res.end();};
    const getCookie = name => (req.headers.cookie || '').split(/;\s*/).find(x => x.startsWith(name+'='))?.slice(name.length+1);
    function issueSession(clear) {
      session = {id:random(),until:now()+8*3600_000,last:now()};
      res.setHeader('Set-Cookie',[cookie(clear,'',0),cookie(sid,session.id,8*3600)]);
      return redirect('/');
    }
    try {
      if (!req.url.startsWith('/') || req.url.startsWith('//') || req.url.includes('\\')) return send(400,'Invalid request');
      const url = new URL(req.url, settings?.origin ?? 'https://reports.invalid');
      const path = url.pathname;
      if (req.method === 'GET' && path === '/health') return send(200,'{"status":"ok"}','application/json');
      if (path === '/logout' && req.method === 'POST') {
        if (!settings || req.headers.origin !== settings.origin) return send(403,'Forbidden');
        if (session && session.id === getCookie(sid)) session = undefined;
        res.setHeader('Set-Cookie',cookie(sid,'',0));return redirect('/');
      }
      if (path === '/verify' && (req.method === 'GET' || req.method === 'POST')) {
        const key = getCookie(step);
        const validChallenge = () => totp && challenge && key === challenge.id && now() < challenge.until;
        if (!validChallenge()) return send(403,'Sign in with Google to continue.');
        if (req.method === 'GET') {
          // Keep the Origin header on same-origin form POSTs, including in Chromium.
          res.setHeader('Referrer-Policy','same-origin');
          return send(200,page('Authenticator code','<form method="post" action="/verify"><label>6-digit code <input name="code" inputmode="numeric" autocomplete="one-time-code" pattern="[0-9]{6}" maxlength="6" required autofocus></label> <button>Verify</button></form>'),'text/html; charset=utf-8');
        }
        if (req.headers.origin !== settings.origin ||
            req.headers['content-type']?.split(';')[0] !== 'application/x-www-form-urlencoded') return send(403,'Forbidden');
        let body='';
        for await (const chunk of req) {
          if (body.length + chunk.length > 64) return send(413,'Request too large.');
          body += chunk.toString('ascii');
        }
        // Verify again after reading the request; another request may have consumed the challenge.
        if (!validChallenge()) return send(403,'Sign in with Google to continue.');
        const at = now();
        failures = failures.filter(t => t > at - 300_000);
        if (failures.length >= 5) return send(429,'Please try again in five minutes.');
        const token = new URLSearchParams(body).get('code');
        const delta = /^[0-9]{6}$/.test(token ?? '') ? totp.validate({token,timestamp:at,window:1}) : null;
        const counter = Math.floor(at / 30_000);
        if (delta === null || counter + delta <= lastCounter) {
          failures.push(at);
          return send(403,page('Code unavailable','<p>Enter a fresh code from your authenticator.</p><a href="/verify">Try again</a>'),'text/html; charset=utf-8');
        }
        lastCounter = counter + delta;
        challenge = undefined;
        failures = [];
        return issueSession(step);
      }
      if (req.method !== 'GET') return send(405,'Method unavailable');
      if (path === '/login') {
        if (!identity) return send(503,'Owner sign-in is awaiting configuration.');
        const attempt = await identity.begin();
        for (const [k,a] of attempts) if (a.until <= now()) attempts.delete(k);
        if (attempts.size >= 256) return send(429,'Please try again later.');
        const key = random();
        attempts.set(key,{...attempt,until:now()+300_000});
        res.setHeader('Set-Cookie',cookie(flow,key,300));return redirect(attempt.url.href);
      }
      if (path === '/auth/callback') {
        const key = getCookie(flow), attempt = attempts.get(key);
        attempts.delete(key);
        res.setHeader('Set-Cookie',cookie(flow,'',0));
        if (!identity || !attempt || attempt.until <= now()) return send(403,'Sign-in expired. Start again.');
        let allowed;
        try {allowed = await identity.finish(url,attempt);} catch {allowed=false;}
        if (!allowed) return send(403,'Owner sign-in with MFA is required.');
        if (settings.google) {
          challenge = {id:random(),until:now()+300_000};
          res.setHeader('Set-Cookie',[cookie(flow,'',0),cookie(step,challenge.id,300)]);
          return redirect('/verify');
        }
        return issueSession(flow);
      }
      if (!['/','/report.json'].includes(path)) return send(404,'Page unavailable');
      if (!session || session.id !== getCookie(sid) || session.until <= now() || now()-session.last >= 1800_000) {
        if (path === '/report.json') return send(401,'Sign in to download.');
        const signIn = settings ? (settings.google ? '<a href="/login">Sign in with Google</a>' : '<a href="/login">Sign in with your identity provider</a>') : '<p>Owner sign-in is awaiting configuration.</p>';
        return send(200,page('Jiaoyi reports',signIn),'text/html; charset=utf-8');
      }
      session.last = now();
      const raw = await readFile(sample,'utf8');
      if (path === '/report.json') {
        res.setHeader('Content-Disposition','attachment; filename="sample-report.json"');
        return send(200,raw,'application/json');
      }
      const report = JSON.parse(raw);
      const stamp = new Date(report.as_of);
      const age = Math.max(0,Math.floor((now()-stamp.getTime())/60000));
      const when = new Intl.DateTimeFormat('en-SG',{timeZone:'Asia/Singapore',dateStyle:'medium',timeStyle:'short'}).format(stamp);
      // This HTML contains the sign-out form, which checks the request origin too.
      res.setHeader('Referrer-Policy','same-origin');
      return send(200,page(esc(report.title),'<time>'+esc(when)+' SGT</time> · '+age+' min ago<p>'+esc(report.summary)+'</p><p><a href="/report.json">Download JSON</a></p><form method="post" action="/logout"><button>Sign out</button></form>'),'text/html; charset=utf-8');
    } catch { return send(503,'Reports temporarily unavailable.'); }
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  let settings;
  try {settings=readSettings(process.env);} catch {console.error('Invalid reports login configuration');}
  createReader({settings}).listen(Number(process.env.PORT || 8080),'0.0.0.0');
}
