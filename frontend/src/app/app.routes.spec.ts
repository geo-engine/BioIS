import { TestBed } from '@angular/core/testing';
import { provideRouter, Router } from '@angular/router';
import { MockInstance, vi } from 'vitest';
import { routes } from './app.routes';
import { UserService } from './user.service';

describe('app routes', () => {
  let router: Router;
  let userService: UserService;
  let isLoggedInSpy: MockInstance<() => boolean>;
  let logoutSpy: MockInstance<() => void>;

  beforeEach(() => {
    userService = {
      isLoggedIn: vi.fn(),
      logout: vi.fn(),
    } as unknown as UserService;

    TestBed.configureTestingModule({
      providers: [provideRouter(routes), { provide: UserService, useValue: userService }],
    });

    isLoggedInSpy = vi.spyOn(userService, 'isLoggedIn');
    logoutSpy = vi.spyOn(userService, 'logout');
    router = TestBed.inject(Router);
  });

  it('redirects app routes to sign in when the user is not logged in', async () => {
    isLoggedInSpy.mockReturnValue(false);

    await router.navigateByUrl('/app/results');

    expect(router.url).toBe('/app/signin');
  });

  it('allows app routes when the user is logged in', async () => {
    isLoggedInSpy.mockReturnValue(true);

    await router.navigateByUrl('/app/results');

    expect(router.url).toBe('/app/results');
  });

  it('redirects unknown app routes to the results page', async () => {
    isLoggedInSpy.mockReturnValue(true);

    await router.navigateByUrl('/app/unknown');

    expect(router.url).toBe('/app/results');
  });

  it('logs out on signout and redirects to home', async () => {
    isLoggedInSpy.mockReturnValue(true);

    await router.navigateByUrl('/app/signout');

    expect(logoutSpy).toHaveBeenCalledTimes(1);
    expect(router.url).toBe('/');
  });
});
