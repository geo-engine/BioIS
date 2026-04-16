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
import {
  NDVIProcessOutputs,
  ProcessesApi,
  PointGeoJsonType,
  PointGeoJsonInputMediaType,
  InlineOrRefData,
  QualifiedInputValue,
} from '@geoengine/biois';
import { CommonModule } from '@angular/common';
import { ColorBreakpoint, NumberIndicatorComponent } from './number-indicator.component';

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
    NumberIndicatorComponent,
  ],
})
export class DashboardComponent {
  private readonly breakpointObserver = inject(BreakpointObserver);
  private readonly activatedRoute = inject(ActivatedRoute);
  private readonly userService = inject(UserService);

  readonly processId: Signal<string | undefined>;

  private readonly mockInputs: NDVIProcessOutputs = {
    ndvi: null,
    kNdvi: null,
    inputs: {
      coordinate: {
        value: {
          type: PointGeoJsonType.Point,
          coordinates: [0, 0],
        },
        mediaType: PointGeoJsonInputMediaType.ApplicationGeojson,
      },
      year: 0,
      month: 0,
    },
  };

  readonly result: ResourceRef<NDVIProcessOutputs> = resource({
    params: () => ({
      processId: this.processId(),
    }),
    defaultValue: this.mockInputs,
    loader: async ({ params }) => {
      const api = new ProcessesApi(this.userService.apiConfiguration());
      if (!params.processId) return this.mockInputs;

      const result = await api.results(params.processId);

      if (result instanceof Blob) {
        throw new Error('Expected NDVIProcessOutputs but received HttpFile');
      }

      // Transform Results (with InlineOrRefData wrappers) to NDVIProcessOutputs
      const outputs: NDVIProcessOutputs = {
        ndvi: null,
        kNdvi: null,
        inputs: this.mockInputs.inputs,
      };

      // Extract values from InlineOrRefData wrappers
      if ('ndvi' in result) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const ndviData = result['ndvi'];
        outputs.ndvi = typeof ndviData === 'number' ? ndviData : null;
      }
      if ('kNdvi' in result) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const kNdviData = result['kNdvi'];
        outputs.kNdvi = typeof kNdviData === 'number' ? kNdviData : null;
      }
      if ('inputs' in result) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        const inputsData = result['inputs'];
        // Extract the value from QualifiedInputValue wrapper
        if (this.isQualifiedInputValue(inputsData) && inputsData.value) {
          outputs.inputs = inputsData.value as typeof outputs.inputs;
        }
      } else {
        throw new Error('Expected "inputs" in result but it was missing');
      }

      return outputs;
    },
  });

  readonly colspan = toSignal(
    this.breakpointObserver
      .observe(Breakpoints.Handset)
      .pipe(map(({ matches }) => (matches ? 2 : 1))),
  );

  readonly ndviColorMap: Array<ColorBreakpoint> = [
    { min: -1, max: 0, color: '#8B4513' }, // Barren ground/cities - brown
    { min: 0, max: 0.1, color: '#A0522D' }, // Very little vegetation - saddle brown
    { min: 0.1, max: 0.3, color: '#DAA520' }, // Sparse vegetation - goldenrod
    { min: 0.3, max: 0.6, color: '#9ACD32' }, // Moderate vegetation - yellow-green
    { min: 0.6, max: 0.9, color: '#32CD32' }, // Healthy crops - lime green
    { min: 0.9, max: 1, color: '#008000' }, // Dense vegetation - dark green
  ];

  constructor() {
    this.processId = toSignal(
      this.activatedRoute.params.pipe(
        map((params) => ('resultId' in params ? (params['resultId'] as string) : undefined)),
      ),
    );
  }

  private isQualifiedInputValue(data: InlineOrRefData): data is QualifiedInputValue {
    return typeof data === 'object' && data !== null && 'value' in data;
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
