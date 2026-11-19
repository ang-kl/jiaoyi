# Jiaoyi owner reports preview

The reader serves one bundled, fictional report. An authenticated owner can view its Singapore timestamp and age, download JSON, and sign out. It runs as the separate Railway service `jiaoyi-reports`. Its image uses only this directory. The process connects to the identity provider for authentication. It has no Trading, broker, database or Engineering credentials.

## Sign in with your personal Gmail account

Use your normal Google account to configure both the owner and a web client. Google hosts the password or passkey prompt. The reader requests only identity and email, and checks Google's signature and verified email against one exact personal Gmail address. Then the reader asks for its own six-digit authenticator code. This enforces MFA even when Google's ID token omits information about how you signed in.

1. Open [Google Auth Platform](https://console.cloud.google.com/auth/overview), sign in with your Gmail account and select or create a project. Under **Branding**, enter the app name and contact email. Choose **External** under **Audience**. For the preview, leave the app in **Testing** and add your Gmail address as a test user.
2. Under **Clients**, choose **Create client** and **Web application**. Add this exact **Authorized redirect URI** and save:

   `https://jiaoyi-reports-production.up.railway.app/auth/callback`

3. Create a random authenticator setup key privately. For example, run this once on your own computer with Python installed:

   ```sh
   python3 -c "import base64,secrets; print(base64.b32encode(secrets.token_bytes(20)).decode())"
   ```

   In Google Authenticator or another TOTP app choose **Enter a setup key**, name it **Jiaoyi reports**, paste that key and choose **Time based**. Store a recovery copy privately. The reader uses standard six-digit SHA-1 codes every 30 seconds.
4. In [Railway's reports service](https://railway.com/project/26f609a2-afc0-4e94-90a2-fe87ba40a713/service/d6cd8c76-4812-4d48-9689-5a59cce0b3e9), set the following variables and apply them to **jiaoyi-reports**.

   | Variable | Value |
   | --- | --- |
   | `REPORTS_GOOGLE_CLIENT_ID` | Web client ID from Google |
   | `REPORTS_GOOGLE_CLIENT_SECRET` | Web client secret from Google |
   | `REPORTS_OWNER_GMAIL` | Exact personal `@gmail.com` address returned by Google |
   | `REPORTS_TOTP_SECRET` | Same 32-character Base32 setup key stored in your authenticator |

5. In a private browser window, [open the reports reader](https://jiaoyi-reports-production.up.railway.app), choose **Sign in with Google**, select the owner account and enter your authenticator code. Open the sample and its download. Sign out and verify the download requires sign-in again.

Google mode pins Google's issuer. Configuring any Google-mode variable selects this mode and incomplete settings leave the report locked. A successful Google login allows five minutes to complete the authenticator step. Five failed code submissions within five minutes temporarily block further attempts. Use the single-replica preview deployment. To replace a lost authenticator, provision a new key using steps 3–4 and restart only the reports service.

## Alternative: configure an existing OpenID Connect provider

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
