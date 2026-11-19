import * as oidc from 'openid-client';

function httpsUrl(value, originOnly = false) {
  const url = new URL(value);
  if (url.protocol !== 'https:' || url.username || url.password || url.search || url.hash ||
      (originOnly && url.pathname !== '/')) throw new Error('Invalid HTTPS URL');
  return url;
}

export function readSettings(env) {
  const issuer = env.REPORTS_OIDC_ISSUER;
  const clientId = env.REPORTS_OIDC_CLIENT_ID;
  const secret = env.REPORTS_OIDC_CLIENT_SECRET;
  const owner = env.REPORTS_OWNER_SUB;
  if (![issuer, clientId, secret, owner].every(s => s && s.trim() === s)) return null;
  const origin = env.REPORTS_PUBLIC_ORIGIN || ('https://' + (env.RAILWAY_PUBLIC_DOMAIN || ''));
  return {issuer: httpsUrl(issuer), origin: httpsUrl(origin, true).origin, clientId, secret, owner};
}

export function createIdentity(settings, fetchOptions = {}) {
  let cached;
  function config() {
    cached ??= oidc.discovery(settings.issuer, settings.clientId, settings.secret,
      undefined, {...fetchOptions, timeout: 8, execute: [oidc.enableNonRepudiationChecks]})
      .catch(error => { cached = undefined; throw error; });
    return cached;
  }
  return {
    async begin() {
      const cfg = await config();
      const state = oidc.randomState();
      const nonce = oidc.randomNonce();
      const verifier = oidc.randomPKCECodeVerifier();
      const url = oidc.buildAuthorizationUrl(cfg, {
        redirect_uri: settings.origin + '/auth/callback', scope: 'openid',
        code_challenge_method: 'S256', code_challenge: await oidc.calculatePKCECodeChallenge(verifier),
        nonce, state
      });
      return {url, state, nonce, verifier};
    },
    async finish(url, attempt) {
      const tokens = await oidc.authorizationCodeGrant(await config(), url, {
        expectedState: attempt.state, expectedNonce: attempt.nonce, pkceCodeVerifier: attempt.verifier,
        idTokenExpected: true
      });
      const claims = tokens.claims();
      return claims?.sub === settings.owner && Array.isArray(claims.amr) && claims.amr.includes('mfa');
    }
  };
}
