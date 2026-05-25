import { ChangeDetectionStrategy, Component, inject, resource } from '@angular/core';
import { BreakpointObserver, Breakpoints } from '@angular/cdk/layout';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatButtonModule } from '@angular/material/button';
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatListModule } from '@angular/material/list';
import { MatIconModule } from '@angular/material/icon';
import { map, shareReplay } from 'rxjs/operators';
import { RouterModule, RouterOutlet } from '@angular/router';
import { rxResource } from '@angular/core/rxjs-interop';
import { ProcessesApi } from '@geoengine/biois';
import { UserService } from '../user.service';
import { processName } from '../util/processes';
import { MatTooltip } from '@angular/material/tooltip';
import { TitleService } from './title.service';

@Component({
  selector: 'app-navigation',
  templateUrl: './navigation.component.html',
  styleUrl: './navigation.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    MatToolbarModule,
    MatButtonModule,
    MatSidenavModule,
    MatListModule,
    MatIconModule,
    RouterOutlet,
    RouterModule,
    MatTooltip,
  ],
})
export class NavigationComponent {
  private readonly breakpointObserver = inject(BreakpointObserver);
  private readonly userService = inject(UserService);
  private readonly titleService = inject(TitleService);

  readonly isHandset = rxResource({
    stream: () =>
      this.breakpointObserver.observe(Breakpoints.Handset).pipe(
        map((result) => result.matches),
        shareReplay(),
      ),
    defaultValue: false,
  });

  readonly processes = resource({
    loader: async () => {
      const processApi = new ProcessesApi(this.userService.apiConfiguration());
      const processResponse = await processApi.processes();
      processResponse.processes.sort((a, b) => a.id.localeCompare(b.id));
      return processResponse;
    },
  });

  readonly processName = processName;
  readonly title = this.titleService.title;
}
