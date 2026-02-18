import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  OnInit,
  signal,
} from '@angular/core';
import { UserService } from '../user.service';
import { ActivatedRoute, Router } from '@angular/router';
import { firstValueFrom } from 'rxjs';
import { BackendError } from '../error';
import { MatCardModule } from '@angular/material/card';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatButtonModule } from '@angular/material/button';

enum SigninState {
  InProgress,
  InvalidState,
  Error,
  LoggedIn,
}

@Component({
  selector: 'app-signin.component',
  imports: [MatCardModule, MatIconModule, MatProgressSpinnerModule, MatButtonModule],
  templateUrl: './signin.component.html',
  styleUrl: './signin.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SigninComponent implements OnInit {
  private readonly userService = inject(UserService);
  private readonly activatedRoute = inject(ActivatedRoute);
  private readonly router = inject(Router);

  protected readonly userSession = this.userService.userSession;
  protected readonly error = signal<BackendError | undefined>(undefined);
  protected readonly invalidResponse = signal(false);

  readonly SigninState = SigninState;
  readonly state = computed(() => {
    const user = this.userService.userSession();
    const error = this.error();
    const invalidResponse = this.invalidResponse();

    if (user) {
      return SigninState.LoggedIn;
    }

    if (error) {
      return SigninState.Error;
    }

    if (invalidResponse) {
      return SigninState.InvalidState;
    }

    return SigninState.InProgress;
  });

  ngOnInit(): void {
    const _ = this.onInit();
  }

  protected async onInit(): Promise<void> {
    const queryParams: Partial<AuthResponseQueryParams> = await firstValueFrom(
      this.activatedRoute.queryParams,
    );

    if (this.userSession()) {
      return;
    }

    if (Object.keys(queryParams).length === 0) {
      try {
        await this.userService.oidcRedirect();
      } catch (error) {
        this.error.set(BackendError.fromError(error));
      }
      return;
    }

    if (!queryParams.code || !queryParams.session_state || !queryParams.state) {
      this.invalidResponse.set(true);
      return;
    }

    // TODO: remove query parameters from URL after processing to avoid confusion on page reload, e.g. using `location.replaceState` or `history.replaceState`

    try {
      await this.userService.login({
        code: queryParams.code,
        sessionState: queryParams.session_state,
        state: queryParams.state,
      });
      await this.navigateToApp();
    } catch (error) {
      this.error.set(BackendError.fromError(error));
    }
  }

  async navigateToApp(): Promise<void> {
    await this.router.navigate(['/app']);
  }
}

/**
 * The expected query parameters from the OIDC code flow after a successful authentication.
 */
interface AuthResponseQueryParams {
  state: string;
  code: string;
  // eslint-disable-next-line @typescript-eslint/naming-convention
  session_state: string;
}
