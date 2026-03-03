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
import { MatProgressSpinner } from '@angular/material/progress-spinner';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-result',
  templateUrl: './result.component.html',
  styleUrl: './result.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    CommonModule,
    MatButtonModule,
    MatCardModule,
    MatGridListModule,
    MatIconModule,
    MatMenuModule,
    MatProgressSpinner,
  ],
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

      if (result instanceof Blob) {
        throw new Error('Expected NDVIProcessOutputs but received HttpFile');
      }

      return result;
    },
  });

  readonly colspan = toSignal(
    this.breakpointObserver
      .observe(Breakpoints.Handset)
      .pipe(map(({ matches }) => (matches ? 2 : 1))),
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

  ndviToPercentage(ndvi: number): number {
    // NDVI ranges from -1 to 1, we want to convert it to a percentage from 0% (no vegetation) to 100% (full vegetation)
    return ((ndvi + 1) / 2) * 100;
  }

  spinnerColorVariable(ndvi: number): string {
    const cssVariableName = `--mat-progress-spinner-active-indicator-color`;
    return `${cssVariableName}: ${valueToColor(ndvi)};`;
  }
}

/**
 * Maps a value between -1 and 1 to a color.
 *
 * @param value a value between -1 and 1 representing the NDVI value
 */
function valueToColor(value: number): string {
  const classRanges = [
    { min: -1, max: 0, color: '#8B4513' }, // Barren ground/cities - brown
    { min: 0, max: 0.1, color: '#A0522D' }, // Very little vegetation - saddle brown
    { min: 0.1, max: 0.3, color: '#DAA520' }, // Sparse vegetation - goldenrod
    { min: 0.3, max: 0.6, color: '#9ACD32' }, // Moderate vegetation - yellow-green
    { min: 0.6, max: 0.9, color: '#32CD32' }, // Healthy crops - lime green
    { min: 0.9, max: 1, color: '#008000' }, // Dense vegetation - dark green
  ];

  for (const range of classRanges) {
    if (value >= range.min && value <= range.max) {
      return range.color;
    }
  }

  return '#808080'; // Fallback gray
}
