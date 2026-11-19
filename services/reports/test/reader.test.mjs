import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash,generateKeyPairSync,sign} from 'node:crypto';
import {TOTP} from 'otpauth';
import * as oidc from 'openid-client';
import {createIdentity,readSettings} from '../oidc.mjs';
import {createReader} from '../server.mjs';

const issuer='https://identity.example', origin='https://reader.example', subject='one-owner';
const settings=readSettings({REPORTS_PUBLIC_ORIGIN:origin, REPORTS_OIDC_ISSUER:issuer,
  REPORTS_OIDC_CLIENT_ID:'test',REPORTS_OIDC_CLIENT_SECRET:'test-secret',REPORTS_OWNER_SUB:subject});
// RFC 6238 public test key (not a deployed secret).
const googleEnv={REPORTS_PUBLIC_ORIGIN:origin,REPORTS_GOOGLE_CLIENT_ID:'test',
  REPORTS_GOOGLE_CLIENT_SECRET:'test-secret',REPORTS_OWNER_GMAIL:'owner@gmail.com',
  REPORTS_TOTP_SECRET:'GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ'};
const json=x=>Response.json(x);
const b64=x=>Buffer.from(JSON.stringify(x)).toString('base64url');

async function fixture(t, options={}) {
  const cfg=options.google?readSettings(googleEnv):settings;
  const issuer=cfg.issuer.origin;
  const {publicKey,privateKey}=generateKeyPairSync('rsa',{modulusLength:2048});
  let authorize, clock=Date.now();
  const fetchProvider=async (url,opts) => {
    const path=new URL(url).pathname;
    if (path==='/.well-known/openid-configuration') return json({issuer,authorization_endpoint:issuer+'/authorize',
      token_endpoint:issuer+'/token',jwks_uri:issuer+'/keys',id_token_signing_alg_values_supported:['RS256'],
      response_types_supported:['code'],subject_types_supported:['public']});
    if (path==='/keys') return json({keys:[{...publicKey.export({format:'jwk'}),kid:'test',alg:'RS256',use:'sig'}]});
    assert.equal(path,'/token');
    const params=new URLSearchParams(opts.body);
    assert.equal(params.get('redirect_uri'),origin+'/auth/callback');
    assert.equal(createHash('sha256').update(params.get('code_verifier')).digest('base64url'),authorize.get('code_challenge'));
    const iat=Math.floor(Date.now()/1000);
    const payload={iss:issuer,aud:'test',iat,exp:iat+600,sub:subject,nonce:authorize.get('nonce'),...(options.google ? {email:'owner@gmail.com',email_verified:true} : {amr:['pwd','mfa']}),...options.claims};
    const value=b64({alg:'RS256',kid:'test'})+'.'+b64(payload);
    const sig=sign('RSA-SHA256',Buffer.from(value),privateKey);
    if(options.badSignature) sig[0]^=1;
    return json({access_token:'mock',token_type:'Bearer',id_token:value+'.'+sig.toString('base64url')});
  };
  const server=createReader({settings:options.disabled?null:cfg,identity:options.disabled?null:
    createIdentity(cfg,{[oidc.customFetch]:fetchProvider}),now:()=>clock});
  await new Promise(r=>server.listen(0,'127.0.0.1',r));
  t.after(()=>new Promise(r=>server.close(r)));
  const get=(path,opts)=>fetch('http://127.0.0.1:'+server.address().port+path,{redirect:'manual',...opts});
  async function start() {
    const login=await get('/login');
    assert.equal(login.status,303);
    authorize=new URL(login.headers.get('location')).searchParams;
    assert.equal(authorize.get('scope'),options.google?'openid email':'openid');
    assert.equal(authorize.get('code_challenge_method'),'S256');
    const callback='/auth/callback?code=from-idp&state='+authorize.get('state');
    const headers={cookie:login.headers.get('set-cookie').split(';')[0]};
    return {callback,headers};
  }
  return {get,start,now:()=>clock,advance:ms=>{clock+=ms;}};
}

test('unconfigured deployment denies data and still serves liveness',async t=>{
  const {get}=await fixture(t,{disabled:true});
  assert.deepEqual(await (await get('/health')).json(),{status:'ok'});
  assert.match(await (await get('/')).text(),/awaiting configuration/);
  assert.equal((await get('/login')).status,503);
  assert.equal((await get('/report.json')).status,401);
  assert.equal((await get('/sample.json')).status,404);
});

test('owner completes verified OIDC flow; download and logout enforce the session',async t=>{
  const {get,start}=await fixture(t);
  const {callback,headers}=await start();
  const login=await get(callback,{headers});
  assert.equal(login.status,303);
  assert.equal((await get(callback,{headers})).status,403);
  const sid=login.headers.getSetCookie().find(s=>s.startsWith('__Host-reports='));
  assert.match(sid,/HttpOnly; Secure; SameSite=Lax/);
  const auth={cookie:sid.split(';')[0]};
  const reader=await get('/',{headers:auth});
  assert.equal(reader.headers.get('referrer-policy'),'same-origin');
  assert.match(await reader.text(),/Sample session review/);
  const download=await get('/report.json',{headers:auth});
  assert.equal((await download.json()).title,'Sample session review');
  assert.match(download.headers.get('cache-control'),/no-store/);
  assert.equal((await get('/logout',{method:'POST',headers:{...auth,origin:issuer}})).status,403);
  assert.equal((await get('/logout',{method:'POST',headers:{...auth,origin}})).status,303);
  assert.equal((await get('/report.json',{headers:auth})).status,401);
});

for(const [name,opts] of Object.entries({stranger:{claims:{sub:'other'}},passwordOnly:{claims:{amr:['pwd']}},
    invalidSignature:{badSignature:true},wrongNonce:{claims:{nonce:'bad'}},wrongAudience:{claims:{aud:'other-app'}}})) {
  test('rejects '+name,async t=>{
    const {get,start}=await fixture(t,opts),{callback,headers}=await start();
    assert.equal((await get(callback,{headers})).status,403);
    assert.equal((await get('/report.json',{headers})).status,401);
  });
}

test('callbacks require original browser and state; idle sessions expire',async t=>{
  const {get,start,advance}=await fixture(t);
  let a=await start();
  assert.equal((await get(a.callback)).status,403);
  assert.equal((await get(a.callback+'x',{headers:a.headers})).status,403);
  a=await start();
  const login=await get(a.callback,{headers:a.headers});
  const cookie=login.headers.getSetCookie().find(s=>s.startsWith('__Host-reports=')).split(';')[0];
  advance(30*60*1000);
  assert.equal((await get('/report.json',{headers:{cookie}})).status,401);
});

test('Google owner must complete authenticator before reading; used codes cannot be reused',async t=>{
  const {get,start,now}=await fixture(t,{google:true});
  async function googleLogin() {
    const {callback,headers}=await start();
    const response=await get(callback,{headers});
    assert.equal(response.headers.get('location'),'/verify');
    const cookies=response.headers.getSetCookie();
    assert.ok(!cookies.some(c=>c.startsWith('__Host-reports=')));
    return cookies.find(c=>c.startsWith('__Host-reports-factor=')).split(';')[0];
  }
  const cookie=await googleLogin();
  const headers={cookie,origin,'content-type':'application/x-www-form-urlencoded'};
  assert.equal((await get('/report.json',{headers})).status,401);
  const form=await get('/verify',{headers});
  assert.equal(form.headers.get('referrer-policy'),'same-origin');
  const token=new TOTP({secret:googleEnv.REPORTS_TOTP_SECRET}).generate({timestamp:now()});
  const post={method:'POST',headers,body:'code='+token};
  assert.equal((await get('/verify',{...post,headers:{...headers,origin:'https://other.example'}})).status,403);
  const accepted=await get('/verify',post);
  assert.equal(accepted.status,303);
  const session=accepted.headers.getSetCookie().find(c=>c.startsWith('__Host-reports=')).split(';')[0];
  assert.equal((await get('/report.json',{headers:{cookie:session}})).status,200);
  assert.equal((await get('/verify',post)).status,403);
  const next=await googleLogin();
  assert.equal((await get('/verify',{...post,headers:{...headers,cookie:next}})).status,403);
});

for (const [name,claims] of Object.entries({different:{email:'other@gmail.com'},unverified:{email_verified:false},
    textualFlag:{email_verified:'true'}})) {
  test('Google refuses '+name+' account evidence',async t=>{
    const {get,start}=await fixture(t,{google:true,claims});
    const {callback,headers}=await start();
    assert.equal((await get(callback,{headers})).status,403);
    assert.equal((await get('/verify',{headers})).status,403);
  });
}

test('Google rate limits second-factor attempts across new logins',async t=>{
  const {get,start}=await fixture(t,{google:true});
  async function tryBatch(n) {
    const {callback,headers}=await start();
    const res=await get(callback,{headers});
    const cookie=res.headers.getSetCookie().find(c=>c.startsWith('__Host-reports-factor=')).split(';')[0];
    const statuses=[];
    for(let i=0;i<n;i++){
      const r=await get('/verify',{method:'POST',headers:{cookie,origin,'content-type':'application/x-www-form-urlencoded'},body:'code=bad'});
      statuses.push(r.status);
    }
    return statuses;
  }
  assert.deepEqual(await tryBatch(5),[403,403,403,403,403]);
  assert.deepEqual(await tryBatch(1),[429]);
});
