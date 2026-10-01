import {
  ChangeDetectionStrategy,
  Component,
  inject,
  input,
  output,
  resource,
  signal,
} from '@angular/core';
import { MatAutocompleteModule } from '@angular/material/autocomplete';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { GeocodingResult, GeocodingService } from './geocoding.service';

const MIN_QUERY_LENGTH = 3;
const DEBOUNCE_MS = 300;

/** A search field for addresses and places. */
@Component({
  selector: 'app-location-search',
  template: `
    <mat-form-field subscriptSizing="dynamic">
      <mat-icon matPrefix>search</mat-icon>
      <input
        matInput
        type="search"
        placeholder="Search for a place or address"
        aria-label="Search for a place or address"
        [value]="query()"
        [matAutocomplete]="autocomplete"
        (input)="query.set($event.target.value)"
        (keydown.enter)="$event.preventDefault()"
      />
      @if (query()) {
        <button
          matSuffix
          mat-icon-button
          type="button"
          aria-label="Clear search"
          (click)="query.set('')"
        >
          <mat-icon>close</mat-icon>
        </button>
      }
      <mat-autocomplete
        #autocomplete="matAutocomplete"
        autoActiveFirstOption
        [displayWith]="resultLabel"
        (optionSelected)="select($event.option.value)"
      >
        @if (results.isLoading()) {
          <mat-option disabled class="status">
            <mat-spinner diameter="18" />
            Searching…
          </mat-option>
        } @else if (results.error()) {
          <mat-option disabled class="status">Search is unavailable</mat-option>
        } @else if (results.hasValue()) {
          @for (result of results.value(); track $index) {
            <mat-option [value]="result">
              <span class="label">{{ result.label }}</span>
              @if (result.detail) {
                <span class="detail">{{ result.detail }}</span>
              }
            </mat-option>
          } @empty {
            <mat-option disabled class="status">No results</mat-option>
          }
        }
        @if (results.status() !== 'idle') {
          <mat-option disabled class="attribution">
            Search by Photon · © OpenStreetMap contributors
          </mat-option>
        }
      </mat-autocomplete>
    </mat-form-field>
  `,
  styles: `
    mat-form-field {
      width: 100%;
    }

    .label,
    .detail {
      display: block;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }

    .detail {
      font: var(--mat-sys-body-small);
      color: var(--mat-sys-on-surface-variant);
    }

    .status mat-spinner {
      display: inline-block;
      vertical-align: middle;
      margin-right: 0.5rem;
    }

    .attribution {
      min-height: 2rem;
      font: var(--mat-sys-label-small);
    }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    MatAutocompleteModule,
    MatButtonModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatProgressSpinnerModule,
  ],
})
export class LocationSearchComponent {
  /** [lon, lat] to prefer nearby results, e.g., the map center */
  readonly bias = input<[number, number]>();
  readonly locationSelected = output<GeocodingResult>();

  readonly query = signal('');
  /** The label of the selected result, which must not trigger another search */
  private readonly selectedLabel = signal<string | undefined>(undefined);

  private readonly geocoding = inject(GeocodingService);

  readonly results = resource({
    params: (): { query: string } | undefined => {
      const query = this.query().trim();
      if (query.length < MIN_QUERY_LENGTH || query === this.selectedLabel()) return undefined;
      return { query };
    },
    loader: async ({ params, abortSignal }): Promise<GeocodingResult[]> => {
      // debounce: a newer query aborts this one while waiting
      await delay(DEBOUNCE_MS, abortSignal);
      return this.geocoding.search(params.query, { bias: this.bias(), signal: abortSignal });
    },
  });

  readonly resultLabel = (result: GeocodingResult | string | null): string =>
    typeof result === 'string' ? result : (result?.label ?? '');

  select(result: GeocodingResult): void {
    this.selectedLabel.set(result.label);
    this.query.set(result.label);
    this.locationSelected.emit(result);
  }
}

function delay(ms: number, abortSignal: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(resolve, ms);
    abortSignal.addEventListener(
      'abort',
      () => {
        clearTimeout(timeout);
        reject(abortSignal.reason as Error);
      },
      { once: true },
    );
  });
}
