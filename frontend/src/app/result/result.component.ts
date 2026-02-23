import {
  ChangeDetectionStrategy,
  Component,
  inject,
  resource,
  ResourceRef,
  Signal,
} from '@angular/core';
import { Breakpoints, BreakpointObserver } from '@angular/cdk/layout';
import { map } from 'rxjs/operators';
import { MatGridListModule } from '@angular/material/grid-list';
import { MatMenuModule } from '@angular/material/menu';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatIconModule } from '@angular/material/icon';
import { toSignal } from '@angular/core/rxjs-interop';
import { ActivatedRoute } from '@angular/router';
import { UserService } from '../user.service';
import { NDVIProcessOutputs, ProcessesApi } from '@geoengine/biois';

@Component({
  selector: 'app-result',
  templateUrl: './result.component.html',
  styleUrl: './result.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [MatButtonModule, MatCardModule, MatGridListModule, MatIconModule, MatMenuModule],
})
export class DashboardComponent {
  private readonly breakpointObserver = inject(BreakpointObserver);
  private readonly activatedRoute = inject(ActivatedRoute);
  private readonly userService = inject(UserService);

  readonly processId: Signal<string | undefined>;

  readonly result: ResourceRef<NDVIProcessOutputs> = resource({
    params: () => ({
      processId: this.processId(),
    }),
    defaultValue: {},
    loader: async ({ params }) => {
      const api = new ProcessesApi(this.userService.apiConfiguration());
      if (!params.processId) return {};

      const result = await api.results(params.processId);
      return result;
    },
  });

  /** Based on the screen size, switch from standard to one column per row */
  readonly cards = toSignal(
    this.breakpointObserver.observe(Breakpoints.Handset).pipe(
      map(({ matches }) => {
        if (matches) {
          return [
            { title: 'Result', cols: 1, rows: 1 },
            // { title: 'Card 2', cols: 1, rows: 1 },
            // { title: 'Card 3', cols: 1, rows: 1 },
            // { title: 'Card 4', cols: 1, rows: 1 },
          ];
        }

        return [
          { title: 'Result', cols: 2, rows: 1 },
          // { title: 'Card 2', cols: 1, rows: 1 },
          // { title: 'Card 3', cols: 1, rows: 2 },
          // { title: 'Card 4', cols: 1, rows: 1 },
        ];
      }),
    ),
  );

  constructor() {
    this.processId = toSignal(
      this.activatedRoute.params.pipe(
        map((params) => ('resultId' in params ? (params['resultId'] as string) : undefined)),
      ),
    );
  }

  async download(): Promise<void> {
    const processId = this.processId();
    if (!processId) return;

    const api = new ProcessesApi(this.userService.apiConfiguration());
    const result = await api.results(processId);

    const link = document.createElement('a');
    link.href = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(result));
    link.download = `result-${processId}.json`;
    link.click();
  }
}
