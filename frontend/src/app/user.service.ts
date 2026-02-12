import { Injectable, signal } from '@angular/core';
import { createConfiguration, ServerConfiguration, UserApi, UserSession } from '@geoengine/biois';

@Injectable({
  providedIn: 'root',
})
export class UserService {
  protected readonly user = signal<UserSession | undefined>(undefined);

  isLoggedIn(): boolean {
    return this.user() !== undefined;
  }

  // TODO: logout after session expires, refresh session, …
  async login(username: string, password: string): Promise<void> {
    const pkceVerifier = this.pkceVerifier();

    if (!pkceVerifier) {
      throw new Error('PKCE verifier not found in session storage');
    }

    const config = createConfiguration({
      baseServer: new ServerConfiguration('http://localhost:4040', {}),
    });
    const userApi = new UserApi(config);
    const user = await userApi.authHandler({
      sessionState: '',
      state: '',
      code: btoa(`${username}:${password}`),
    });
    this.user.set(user);
  }

  storePkceVerifier(verifier: string): void {
    sessionStorage.setItem('pkce_verifier', verifier);
  }

  pkceVerifier(): string | undefined {
    return sessionStorage.getItem('pkce_verifier') ?? undefined;
  }

  async oidcRedirect(): Promise<void> {
    // TODO: try out `angular-auth-oidc-client` or `oidc-client-ts` instead of implementing OIDC ourselves

    const location = window.location;
    const redirectUri = location.origin + location.pathname;

    const clientId = 'geoengine';
    const keycloakBaseUrl = 'https://auth.geoengine.io/realms/BioIS/protocol/openid-connect/auth';

    // const state = Math.random().toString(36).substring(2);
    const { verifier, challenge } = await generatePkcePair();
    this.storePkceVerifier(verifier);

    const oidcRequestParams = new URLSearchParams({
      client_id: clientId, // eslint-disable-line @typescript-eslint/naming-convention
      redirect_uri: redirectUri, // eslint-disable-line @typescript-eslint/naming-convention
      response_type: 'code', // eslint-disable-line @typescript-eslint/naming-convention
      scope: 'openid offline_access',
      code_challenge_method: 'S256', // eslint-disable-line @typescript-eslint/naming-convention
      code_challenge: challenge, // eslint-disable-line @typescript-eslint/naming-convention
    });

    const oidcUrl = `${keycloakBaseUrl}?${oidcRequestParams.toString()}`;

    console.debug(oidcRequestParams);
    console.debug('Redirecting to OIDC provider with URL:', oidcUrl);
    window.location.href = oidcUrl;
  }
}

const generatePkcePair = async (): Promise<{ verifier: string; challenge: string }> => {
  const array = crypto.getRandomValues(new Uint8Array(32));
  const verifier = btoa(String.fromCharCode(...array))
    .replace(/=/g, '')
    .replace(/\+/g, '-')
    .replace(/\//g, '_');

  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const digest = await crypto.subtle.digest('SHA-256', data);
  const challenge = btoa(String.fromCharCode(...new Uint8Array(digest)))
    .replace(/=/g, '')
    .replace(/\+/g, '-')
    .replace(/\//g, '_');

  return { verifier, challenge };
};

// const apiConfigurationWithAccessKey = (accessToken: string): Configuration =>
//   new Configuration({
//     basePath: DefaultConfig.basePath,
//     fetchApi: DefaultConfig.fetchApi,
//     middleware: DefaultConfig.middleware,
//     queryParamsStringify: DefaultConfig.queryParamsStringify,
//     username: DefaultConfig.username,
//     password: DefaultConfig.password,
//     apiKey: DefaultConfig.apiKey,
//     accessToken: accessToken,
//     headers: DefaultConfig.headers,
//     credentials: DefaultConfig.credentials,
//   });
