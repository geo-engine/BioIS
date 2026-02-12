import { ChangeDetectionStrategy, Component, inject, OnInit } from '@angular/core';
import { UserService } from '../user.service';
import { ActivatedRoute } from '@angular/router';
import { firstValueFrom } from 'rxjs';

@Component({
  selector: 'app-signin.component',
  imports: [],
  templateUrl: './signin.component.html',
  styleUrl: './signin.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SigninComponent implements OnInit {
  private readonly userService = inject(UserService);
  private readonly activatedRoute = inject(ActivatedRoute);

  ngOnInit(): void {
    const _ = this._ngOnInit();
  }

  async _ngOnInit(): Promise<void> {
    const queryParams = await firstValueFrom(this.activatedRoute.queryParams);

    // if (Object.keys(queryParams).length === 0) {
    //   await this.userService.oidcRedirect();
    //   return;
    // }

    if (!('code' in queryParams && 'session_state' in queryParams)) {
      await this.userService.oidcRedirect();
      return;
    }

    console.debug('code', queryParams['code']);
    console.debug('session_state', queryParams['session_state']);
  }
}
