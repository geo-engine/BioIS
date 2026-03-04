import { effect, Injectable, Signal, signal } from '@angular/core';
import {
  createConfiguration,
  ServerConfiguration,
  UserSession,
  UserApi,
  Configuration,
  AuthMethodsConfiguration,
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
        const userSession = JSON.parse(storedUser) as UserSession; /* TODO: proper type checking */
        if (sessionIsValid(userSession)) this.user.set(userSession);
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

  logout(): void {
    // TODO: this is currently only used to clear the session on the frontend,
    // but we should also invalidate the session on the backend
    this.user.set(undefined);
  }

  async oidcRedirect(): Promise<void> {
    // TODO: try out `angular-auth-oidc-client` or `oidc-client-ts` instead of implementing OIDC ourselves

    const userApi = new UserApi(configuration());
    const redirectUri = oidcRedirectUri();
    const oidcUrl = await userApi.authRequestUrlHandler(redirectUri);

    window.location.href = oidcUrl;
  }

  apiConfiguration(): Configuration {
    const authMethods = {
      accessToken: 'Bearer ' + this.user()?.id /* TODO: handle missing/expired token */,
    };
    const config = createConfiguration({
      baseServer: new ServerConfiguration('/api', {}),
      // authMethods: authMethods as AuthMethodsConfiguration,
      authMethods: {
        default: {
          getName: () => 'default',
          applySecurityAuthentication: (context) => {
            context.setHeaderParam('Authorization', authMethods.accessToken);
            // TODO: separate this?
            context.setHeaderParam('Prefer', 'respond-async');
          },
        },
      } as AuthMethodsConfiguration,
    });
    // if (this.user()) {
    //   config.baseServer = this.user().accessToken;
    // }
    return config;
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
    baseServer: new ServerConfiguration('/api', {}),
    // ...options,
  });
}

function sessionIsValid(userSession: UserSession, now: number = Date.now()): boolean {
  // const validUntil = Temporal.Instant.from(userSession.validUntil);
  // const now = Temporal.Now.instant();
  const validUntil = Date.parse(userSession.validUntil);
  return now < validUntil;
}

// eslint-disable-next-line @typescript-eslint/naming-convention
export const __TEST__ = {
  sessionIsValid,
};
