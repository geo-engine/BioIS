import { effect, Injectable, Signal, signal } from '@angular/core';
import {
  createConfiguration,
  ServerConfiguration,
  UserSession,
  UserApi,
  Configuration,
} from '@geoengine/biois';

const USER_SESSION_KEY = 'userSession';

@Injectable({
  providedIn: 'root',
})
export class UserService {
  protected readonly user = signal<UserSession | undefined>(undefined);

  constructor() {
    effect(() => {
      const user = this.user();
      if (user) {
        sessionStorage.setItem(USER_SESSION_KEY, JSON.stringify(user));
      } else {
        sessionStorage.removeItem(USER_SESSION_KEY);
      }
    });

    const storedUser = sessionStorage.getItem(USER_SESSION_KEY);
    if (storedUser) {
      try {
        this.user.set(JSON.parse(storedUser) as UserSession /* TODO: proper type checking */);
      } catch (error) {
        console.error('Failed to parse stored user session:', error);
      }
    }
  }

  isLoggedIn(): boolean {
    return this.user() !== undefined;
  }

  get userSession(): Signal<UserSession | undefined> {
    return this.user;
  }

  // TODO: logout after session expires, refresh session, …
  async login(auth: { state: string; code: string; sessionState: string }): Promise<void> {
    const userApi = new UserApi(configuration());
    const redirectUri = location.origin + location.pathname;
    const user = await userApi.authHandler(redirectUri, auth);

    this.user.set(user);
  }

  async oidcRedirect(): Promise<void> {
    // TODO: try out `angular-auth-oidc-client` or `oidc-client-ts` instead of implementing OIDC ourselves

    const userApi = new UserApi(configuration());
    const redirectUri = oidcRedirectUri();
    const oidcUrl = await userApi.authRequestUrlHandler(redirectUri);

    window.location.href = oidcUrl;
  }
}

/**
 * Generates the redirect URI for the OIDC code flow, which is typically the current URL of the frontend application. This URI is used both to initiate the OIDC flow and as the redirect URI registered with the identity provider.
 * @returns The redirect URI as a string.
 */
function oidcRedirectUri(): string {
  return location.origin + location.pathname;
}

function configuration(/*options: { authMethods?: OAuth2Configuration } = {}*/): Configuration {
  return createConfiguration({
    baseServer: new ServerConfiguration('http://localhost:4040', {}),
    // ...options,
  });
}
