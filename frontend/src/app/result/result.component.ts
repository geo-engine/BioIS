import {
  ChangeDetectionStrategy,
  Component,
  computed,
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
  InlineOrRefData,
  Link as LinkValue,
  ProcessesApi,
  QualifiedInputValue,
  Schema,
} from '@geoengine/biois';
import { CommonModule } from '@angular/common';
import { ColorBreakpoint, NumberIndicatorComponent } from './number-indicator.component';
import { processName } from '../util/processes';
import { MatTableModule } from '@angular/material/table';
import { MatListModule } from '@angular/material/list';
import { LongTextComponent } from '../util/long-text.component';

@Component({
  selector: 'app-result',
  templateUrl: './result.component.html',
  styleUrl: './result.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    CommonModule,
    LongTextComponent,
    MatButtonModule,
    MatCardModule,
    MatGridListModule,
    MatIconModule,
    MatListModule,
    MatMenuModule,
    MatTableModule,
    NumberIndicatorComponent,
  ],
})
export class ResultComponent {
  private readonly breakpointObserver = inject(BreakpointObserver);
  private readonly activatedRoute = inject(ActivatedRoute);
  private readonly userService = inject(UserService);

  readonly resultId: Signal<string | undefined>;

  readonly result: ResourceRef<Record<string, InlineOrRefData>> = resource({
    params: () => ({
      resultId: this.resultId(),
    }),
    defaultValue: {},
    loader: async ({ params }) => {
      const api = new ProcessesApi(this.userService.apiConfiguration());
      if (!params.resultId) return {};

      const result = await api.results(params.resultId);

      if (result instanceof Blob) {
        throw new Error('Expected document output but received HttpFile');
      }

      return result;
    },
  });

  readonly fieldName = processName;
  readonly ResultType = ResultType;
  readonly ColumnType = ColumnType;

  readonly results = computed(() => {
    const result = this.result.value();
    if (!result) return [];

    return Object.entries(result).map(([key, rawValue]) => {
      const value = fixDataValue(rawValue) as unknown;
      return {
        key,
        title: this.fieldName(key),
        value: value instanceof QualifiedInputValue ? (value.value as unknown) : value,
        type: this.typeOfValue(value),
      };
    });
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
    this.resultId = toSignal(
      this.activatedRoute.params.pipe(
        map((params) => ('resultId' in params ? (params['resultId'] as string) : undefined)),
      ),
    );
  }

  asNumber(value: unknown): number {
    return value as number;
  }

  asArray(value: unknown): Array<unknown> {
    return value as Array<unknown>;
  }

  protected typeOfValue(value: InlineOrRefData): ResultType {
    if (value instanceof QualifiedInputValue) {
      if (value.mediaType === 'text/plain; spectral=ndvi') return ResultType.Ndvi;
      if (value.mediaType === 'application/json') return ResultType.Json;
      if (value.mediaType === 'application/vnd.dataresource+json') return ResultType.JsonTable;

      // TODO: special handling for…
      // … Input references
      // … Errors
      // … Documentation sources
    }

    if (value instanceof LinkValue) return ResultType.Link;

    if (Array.isArray(value)) return ResultType.Array;

    switch (typeof value) {
      case 'number':
      case 'bigint':
        return ResultType.Number;
      case 'boolean':
        return ResultType.Boolean;
      case 'object':
        if (value === null) return ResultType.String; // treat null as string
        return ResultType.Json;
      case 'string':
      case 'undefined': // fallback
      case 'symbol': // fallback
      case 'function': // fallback
        return ResultType.String;
    }
  }

  async download(): Promise<void> {
    const processId = this.resultId();
    if (!processId) return;

    const api = new ProcessesApi(this.userService.apiConfiguration());
    const result = await api.results(processId);

    const link = document.createElement('a');
    link.href = 'data:text/json;charset=utf-8,' + encodeURIComponent(JSON.stringify(result));
    link.download = `result-${processId}.json`;
    link.click();
  }

  asJsonTableRows(value: unknown): Array<Record<string, unknown>> {
    if (!value || !(typeof value === 'object') || !('data' in value) || !('schema' in value))
      return [];

    return value.data as Array<Record<string, unknown>>;
  }

  columns(value: unknown): Column[] {
    if (!value || !(typeof value === 'object') || !('data' in value) || !('schema' in value))
      return [];

    const schema = value.schema;
    if (!schema || !(typeof schema === 'object') || !('fields' in schema)) return [];

    const fields = schema['fields'] as [
      {
        name: string;
        type?: 'string' | 'number' | 'integer' | 'boolean';
        title?: string;
      },
    ];

    return fields.map((field) => {
      const sampleValue = this.asJsonTableRows(value)[0]?.[field.name];
      const columnType = columnTypeOfField(field.type, sampleValue);
      return {
        name: field.title ?? field.name,
        key: field.name,
        type: columnType,
      };
    });
  }

  columnKeys(value: unknown): string[] {
    const columns = this.columns(value);
    return columns.map((col) => col.key);
  }
}

interface Column {
  name: string;
  key: string;
  type: ColumnType;
}

enum ColumnType {
  String = 'string',
  Number = 'number',
  Boolean = 'boolean',
  Url = 'url',
}

enum ResultType {
  Boolean = 'boolean',
  Errors = 'errors',
  Input = 'input',
  Json = 'json',
  JsonTable = 'jsonTable',
  Ndvi = 'ndvi',
  Number = 'number',
  String = 'string',
  Link = 'link',
  Array = 'array',
}

/**
 * Fixes the given data value to ensure it is properly typed as either a Link or a QualifiedInputValue.
 */
export function fixDataValue(data: InlineOrRefData): InlineOrRefData {
  if (data instanceof QualifiedInputValue) return data;
  if (data instanceof LinkValue) return data;

  // type was link but not in Link class format, try to fix it
  if (typeof data === 'object' && 'href' in data && 'rel' in data) {
    const linkData = data as {
      href: string;
      rel: string;
      type?: string;
      templated?: boolean;
      varBase?: string;
      hreflang?: string;
      title?: string;
      length?: number;
    };
    const link = new LinkValue();
    link.href = linkData.href;
    link.rel = linkData.rel;
    link.type = linkData.type;
    link.templated = linkData.templated;
    link.varBase = linkData.varBase;
    link.hreflang = linkData.hreflang;
    link.title = linkData.title;
    link.length = linkData.length;
    return link;
  }

  if (typeof data === 'object' && 'value' in data && 'mediaType' in data) {
    const qualifiedValueData = data as {
      value: unknown;
      mediaType: string;
      encoding?: string;
      schema?: Schema;
    };
    const qualifiedValue = new QualifiedInputValue();
    qualifiedValue.value = qualifiedValueData.value;
    qualifiedValue.mediaType = qualifiedValueData.mediaType;
    qualifiedValue.encoding = qualifiedValueData.encoding;
    qualifiedValue.schema = qualifiedValueData.schema;
    return qualifiedValue;
  }

  return data;
}

export function columnTypeOfField(
  type?: 'string' | 'number' | 'integer' | 'boolean',
  firstValue?: unknown,
): ColumnType {
  if (!type) return ColumnType.String;

  let columnType: ColumnType;
  switch (type) {
    case 'string':
      columnType = ColumnType.String;
      break;
    case 'number':
    case 'integer':
      columnType = ColumnType.Number;
      break;
    case 'boolean':
      columnType = ColumnType.Boolean;
      break;
  }

  if (columnType !== ColumnType.String || !firstValue) return columnType;

  if (typeof firstValue === 'string' && firstValue.startsWith('http')) {
    columnType = ColumnType.Url;
  }

  return columnType;
}
