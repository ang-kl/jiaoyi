# Jiaoyi owner reports preview

The reader serves one bundled, fictional report. An authenticated owner can view its Singapore timestamp and age, download JSON, and sign out. It runs as the separate Railway service `jiaoyi-reports`. Its image uses only this directory. The process connects to the identity provider for authentication. It has no Trading, broker, database or Engineering credentials.

## Configure the existing OpenID Connect provider

1. Register a confidential web client for the authorization code flow. Register the exact callback `https://jiaoyi-reports-production.up.railway.app/auth/callback` and require MFA for the client. Enable PKCE S256 and the `openid` scope. The client uses `client_secret_post` and signed ID tokens verified against the provider's JWKS.
2. Restrict the app to the owner's identity. Obtain the owner's exact, case-sensitive `sub` for **this client** from your identity provider. An email address and a subject ID can differ.
3. Set these variables on **jiaoyi-reports** only:

   | Variable | Value |
   | --- | --- |
   | `REPORTS_OIDC_ISSUER` | Exact HTTPS issuer URL from the provider |
   | `REPORTS_OIDC_CLIENT_ID` | Registered client ID |
   | `REPORTS_OIDC_CLIENT_SECRET` | Client secret |
   | `REPORTS_OWNER_SUB` | Owner subject issued to this app |

   Railway supplies the public hostname. For a custom domain also set `REPORTS_PUBLIC_ORIGIN` to its HTTPS origin and update the registered callback. Sign-in requires the signed ID token to include the `mfa` value in the `amr` array. If your provider conveys MFA differently, first establish its documented assurance claim and test that mapping. A missing claim denies access.

4. Apply the variables to this service. Open the viewer in a private browser window, sign in as the owner, open the report, download JSON, then sign out. Confirm the download returns 401 after sign-out. A different identity must receive 403.

The landing page and minimal `/health` are public. Both report views check a server-side session. Sessions expire after 30 idle minutes or eight hours total. An instance restart or a new owner login invalidates the prior session. Use one replica for this sample. Report routes use no-store responses.

## Build and deploy this directory

Use Node 24. Run `npm ci` and `npm test`, or build this directory as the Docker context. The container build runs the read/login/download tests with a simulated identity provider and signed tokens. The runtime listens on `PORT` (default 8080).

For a Railway directory upload, use `services/reports --path-as-root` and explicitly select the project, environment, and **reports service ID**. The repo root builds the live Trading application. The reports Dockerfile and Railway configuration are in this directory. Future real-report publication requires private storage, a scoped publisher, freshness checks and an owner sign-in acceptance test. No real-report publisher is connected in this preview.
