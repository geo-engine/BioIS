import { ChangeDetectionStrategy, Component, computed, input, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatTableModule } from '@angular/material/table';
import { MatChipsModule } from '@angular/material/chips';
import { RowOverflowDirective } from './row-overflow.directive';

@Component({
  selector: 'app-data-resource-table',
  template: `
    <table mat-table [dataSource]="rows()">
      @for (column of columns(); track column.key) {
        <ng-container [matColumnDef]="column.key" [sticky]="column.isPrimaryKey">
          <th mat-header-cell *matHeaderCellDef>{{ column.name }}</th>
          @switch (column.type) {
            @case (ColumnType.String) {
              <td mat-cell *matCellDef="let element">
                <div class="cell-content" [innerHTML]="asHtmlString(element[column.key])"></div>
              </td>
            }
            @case (ColumnType.Url) {
              <td mat-cell *matCellDef="let element">
                <a class="cell-content" [href]="element[column.key]" target="_blank">{{
                  element[column.key]
                }}</a>
              </td>
            }
            @case (ColumnType.Number) {
              <td mat-cell *matCellDef="let element">
                <span class="cell-content">{{ element[column.key] | number: '1.0-2' }}</span>
              </td>
            }
            @case (ColumnType.Boolean) {
              <td mat-cell *matCellDef="let element">
                <mat-chip class="cell-content">
                  {{ element[column.key] ? 'TRUE' : 'FALSE' }}
                </mat-chip>
              </td>
            }
            @case (ColumnType.List) {
              <td mat-cell *matCellDef="let element">
                <div class="cell-content">
                  <ul>
                    @for (item of asList(element[column.key]); track item) {
                      <li>
                        <small>{{ item }}</small>
                      </li>
                    } @empty {
                      <span class="empty">empty</span>
                    }
                  </ul>
                </div>
              </td>
            }
            <!-- Prevent unhandled cases -->
            @default never;
          }
        </ng-container>
      }

      <tr mat-header-row *matHeaderRowDef="columnKeys(); sticky: true"></tr>
      <tr
        mat-row
        *matRowDef="let row; columns: columnKeys()"
        #rowOverflow="rowOverflow"
        appRowOverflow=".cell-content"
        [appRowOverflowExpanded]="isExpanded(row)"
        class="expandable-row"
        [class.can-expand]="rowOverflow.canExpand()"
        [class.is-expanded]="isExpanded(row)"
        (click)="rowOverflow.canExpand() && toggleRow(row)"
      ></tr>
    </table>
  `,
  styles: `
    @use '@angular/material' as mat;

    :host {
      display: block;
      max-height: 100vh;
      overflow: auto;
    }

    td {
      @include mat.chips-overrides(
        (
          hover-state-layer-color: transparent,
          focus-state-layer-color: transparent,
          outline-color: var(--mat-sys-text),
          label-text-color: var(--mat-sys-text),
        )
      );

      padding-top: 1rem;
      padding-bottom: 1rem;
      vertical-align: top; /* Keeps text nicely aligned at the top during expansion */

      .cell-content {
        max-height: calc(2 * 1.4em); /* Limits text to roughly 2 lines */
        line-height: 1.4;
        overflow: hidden;

        word-break: normal;
        text-overflow: ellipsis;

        transition: max-height 0.25s ease-out;
      }
    }

    tr {
      height: auto !important; /* Allow rows to adapt dynamically to child heights */

      &.can-expand {
        cursor: pointer;

        &:hover {
          background-color: var(--mat-sys-surface-variant);
        }
      }

      &.is-expanded {
        background-color: var(--mat-sys-surface-variant);

        .cell-content {
          max-height: none;
        }
      }
    }

    .empty {
      font-style: italic;
      color: color-mix(in srgb, var(--mat-sys-on-surface) 38%, transparent);
    }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [CommonModule, MatChipsModule, MatTableModule, RowOverflowDirective],
})
export class DataResourceTableComponent {
  readonly columns = input.required<Column[]>();
  readonly rows = input.required<Array<Row>>();

  readonly columnKeys = computed(() => this.columns().map((column) => column.key));

  readonly ColumnType = ColumnType;

  readonly expandedElement = signal<Row | null>(null);

  asHtmlString(value: unknown): string {
    if (typeof value === 'string') {
      return value.replaceAll('\n', '<br>');
    }
    return String(value);
  }

  asList(value: unknown): Array<unknown> {
    if (Array.isArray(value)) return value;
    if (typeof value === 'object' && value !== null) return Object.values(value);
    return [];
  }

  isExpanded(element: Row): boolean {
    return this.expandedElement() === element;
  }

  toggleRow(element: Row): void {
    this.expandedElement.set(this.isExpanded(element) ? null : element);
  }
}

export type Row = Record<string, unknown>;

export interface Column {
  name: string;
  key: string;
  type: ColumnType;
  isPrimaryKey: boolean;
}

export enum ColumnType {
  String = 'string',
  Number = 'number',
  Boolean = 'boolean',
  Url = 'url',
  List = 'list',
}

export function columnTypeOfField(
  type?: 'string' | 'number' | 'integer' | 'boolean' | 'list',
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
    case 'list':
      columnType = ColumnType.List;
      break;
  }

  if (columnType !== ColumnType.String || !firstValue) return columnType;

  if (typeof firstValue === 'string' && firstValue.startsWith('http')) {
    columnType = ColumnType.Url;
  }

  return columnType;
}

export function tableColumnInfoFromValue(
  schema: Record<string, unknown>,
  data: Array<Record<string, unknown>>,
): Array<Column> {
  if (!('fields' in schema)) return [];

  const fields = schema['fields'] as [
    {
      name: string;
      type?: 'string' | 'number' | 'integer' | 'boolean' | 'list';
      title?: string;
    },
  ];

  const primaryKey = new Array<string>();
  if ('primaryKey' in schema) {
    const primaryKeyField = schema['primaryKey'];
    if (typeof primaryKeyField === 'string') primaryKey.push(primaryKeyField);
    else if (Array.isArray(primaryKeyField)) {
      const stringPrimaryKeys = primaryKeyField.filter(
        (key): key is string => typeof key === 'string',
      );
      primaryKey.push(...stringPrimaryKeys);
    }
  }

  return fields.map((field) => {
    const sampleValue = data[0]?.[field.name];
    const columnType = columnTypeOfField(field.type, sampleValue);
    return {
      name: field.title ?? field.name,
      key: field.name,
      type: columnType,
      isPrimaryKey: primaryKey.includes(field.name),
    };
  });
}
