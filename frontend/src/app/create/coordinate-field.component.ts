import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  input,
  signal,
  untracked,
} from '@angular/core';
import { FieldTree, FormField } from '@angular/forms/signals';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { GeoJSONFeature, GeoJSONPoint } from '@geoengine/biois';
import { MapComponent } from '../map/map.component';
import { emptyGeoJsonFeatureCollection, GeoJsonFeatureCollection } from '../util/geo-json';

/** Decimals of the coordinates set via the map (6 decimals ≈ 0.1 m) */
const COORDINATE_DECIMALS = 6;

/** Delay before typed coordinates are shown on the map, so it does not jump while typing */
const TYPING_DEBOUNCE_MS = 800;

/** A single coordinate that can be set on a map or entered as longitude and latitude. */
@Component({
  selector: 'app-coordinate-field',
  template: `
    <app-map
      [features]="collection()"
      (featuresChange)="onFeaturesChange($event)"
      [geometryTypes]="['Point']"
      [maxFeatures]="1"
      [showFeatureList]="false"
    />

    <div class="coordinates">
      @for (coordinateValue of ['Longitude', 'Latitude']; track $index; let index = $index) {
        <mat-form-field>
          <mat-label>{{ coordinateValue }}</mat-label>
          <input matInput type="number" step="any" [formField]="field().coordinates[index]" />
          @for (error of field().coordinates[index]().errors(); track error) {
            <mat-error>{{ error.message }}</mat-error>
          }
        </mat-form-field>
      }

      @for (error of field().coordinates().errors(); track error) {
        <mat-error>{{ error.message }}</mat-error>
      }
    </div>
  `,
  styles: `
    :host {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    .coordinates {
      display: flex;
      flex-wrap: wrap;
      gap: 0.5rem;
    }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [FormField, MapComponent, MatFormFieldModule, MatInputModule],
})
export class CoordinateFieldComponent {
  readonly field = input.required<FieldTree<GeoJSONPoint, string>>();

  /** The point on the map, which follows typed coordinates with a delay */
  readonly collection = signal<GeoJsonFeatureCollection>(emptyGeoJsonFeatureCollection());

  /** The point of the form, or no feature while the coordinates are incomplete, e.g., while typing */
  private readonly formCollection = computed((): GeoJsonFeatureCollection => {
    const point = this.field()().value();
    return pointCollection(point.coordinates);
  });

  private initialized = false;

  constructor() {
    effect((onCleanup) => {
      const collection = this.formCollection();
      if (sameCoordinates(collection, untracked(this.collection))) return;

      // show the initial coordinate right away
      if (!this.initialized) {
        this.initialized = true;
        this.collection.set(collection);
        return;
      }

      const timeout = setTimeout(() => this.collection.set(collection), TYPING_DEBOUNCE_MS);
      onCleanup(() => clearTimeout(timeout));
    });
  }

  onFeaturesChange(collection: GeoJsonFeatureCollection): void {
    const geometry = collection.features.at(-1)?.geometry;
    // keep the coordinate if the point was removed, since the process requires one
    if (!geometry || (geometry.type as string) !== 'Point') return;

    const [lon, lat] = (geometry as GeoJSONPoint).coordinates;
    const coordinates = [roundCoordinate(lon), roundCoordinate(lat)];

    // changes on the map are no typing, so they are applied immediately
    this.collection.set(pointCollection(coordinates));

    const value = this.field()().value;
    value.set({ ...value(), coordinates });
  }
}

function pointCollection(coordinates: number[]): GeoJsonFeatureCollection {
  const [lon, lat] = coordinates;
  if (!Number.isFinite(lon) || !Number.isFinite(lat)) return emptyGeoJsonFeatureCollection();

  return {
    type: 'FeatureCollection',
    features: [
      {
        type: 'Feature',
        geometry: { type: 'Point', coordinates: [lon, lat] },
        properties: {},
      } as GeoJSONFeature,
    ],
  };
}

function sameCoordinates(a: GeoJsonFeatureCollection, b: GeoJsonFeatureCollection): boolean {
  const coordinates = (collection: GeoJsonFeatureCollection): string =>
    JSON.stringify(collection.features.map((feature) => feature.geometry));
  return coordinates(a) === coordinates(b);
}

export function roundCoordinate(value: number): number {
  const factor = 10 ** COORDINATE_DECIMALS;
  return Math.round(value * factor) / factor;
}
