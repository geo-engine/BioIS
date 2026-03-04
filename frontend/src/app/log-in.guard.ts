import { Injectable, inject } from '@angular/core';
import {
  ActivatedRouteSnapshot,
  CanActivate,
  GuardResult,
  MaybeAsync,
  Router,
  RouterStateSnapshot,
} from '@angular/router';
import { UserService } from './user.service';

const SIGN_IN_PATH = '/app/signin';

@Injectable({
  providedIn: 'root',
})
export class LogInGuard implements CanActivate {
  private router = inject(Router);
  private userService = inject(UserService);

  canActivate(_route: ActivatedRouteSnapshot, state: RouterStateSnapshot): MaybeAsync<GuardResult> {
    if (this.userService.isLoggedIn()) {
      return true;
    }
    if (state.url === SIGN_IN_PATH) {
      return true;
    }

    return this.router.createUrlTree([SIGN_IN_PATH]);
  }
}
