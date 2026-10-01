import {
  afterNextRender,
  ChangeDetectionStrategy,
  Component,
  computed,
  DestroyRef,
  effect,
  ElementRef,
  inject,
  input,
  model,
  signal,
  untracked,
  viewChild,
  ViewEncapsulation,
} from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatButtonToggleModule } from '@angular/material/button-toggle';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatTooltipModule } from '@angular/material/tooltip';
import { GeoJSONFeature, GeoJSONFeatureGeometry } from '@geoengine/biois';
import OlFeature, { FeatureLike } from 'ol/Feature';
import OlMap from 'ol/Map';
import View from 'ol/View';
import { containsExtent, Extent, isEmpty as isEmptyExtent } from 'ol/extent';
import GeoJSON from 'ol/format/GeoJSON';
import { Geometry } from 'ol/geom';
import { Draw, Modify, Snap } from 'ol/interaction';
import TileLayer from 'ol/layer/Tile';
import VectorLayer from 'ol/layer/Vector';
import { fromLonLat, toLonLat, transformExtent } from 'ol/proj';
import OSM from 'ol/source/OSM';
import VectorSource from 'ol/source/Vector';
import { Circle as CircleStyle, Fill, Stroke, Style, Text } from 'ol/style';
import {
  DrawableGeometryType,
  emptyGeoJsonFeatureCollection,
  FeatureField,
  GeoJsonFeatureCollection,
  defaultFeatureProperties,
  markAsDrawn,
} from '../util/geo-json';
import { fromResize } from '../util/resize-signal';
import { resolveCssColor, RgbColor } from '../util/theme-colors';
import { GeocodingResult } from './geocoding.service';
import { LocationSearchComponent } from './location-search.component';

/** Tool of the map: select and modify features, or draw a new geometry. */
export type MapTool = 'select' | DrawableGeometryType;

/** Property of the OpenLayers features that links them to the index in the collection. */
const FEATURE_INDEX = '_featureIndex';

/** Initial view if there are no features (Germany). */
const DEFAULT_CENTER: [number, number] = [10.45, 51.16];
/** Zoom level for search results without an extent, e.g., an address */
const LOCATION_ZOOM = 17;
const DEFAULT_ZOOM = 5;

/** Colors of the map features, which are resolved from the app theme on rendering */
interface MapColors {
  feature: RgbColor;
  selectedFeature: RgbColor;
  label: RgbColor;
  labelHalo: RgbColor;
}

/** The Material system colors to use and fallbacks (light theme) if they cannot be resolved */
const THEME_COLORS: { [K in keyof MapColors]: [cssColor: string, fallback: RgbColor] } = {
  feature: ['var(--mat-sys-primary)', [88, 99, 49]],
  selectedFeature: ['var(--mat-sys-secondary)', [32, 103, 118]],
  label: ['var(--mat-sys-on-surface)', [27, 28, 24]],
  labelHalo: ['var(--mat-sys-surface)', [252, 249, 242]],
};

function fallbackColors(): MapColors {
  return {
    feature: THEME_COLORS.feature[1],
    selectedFeature: THEME_COLORS.selectedFeature[1],
    label: THEME_COLORS.label[1],
    labelHalo: THEME_COLORS.labelHalo[1],
  };
}

function resolveThemeColors(context: HTMLElement): MapColors {
  const resolve = ([cssColor, fallback]: [string, RgbColor]): RgbColor =>
    resolveCssColor(context, cssColor, fallback);
  return {
    feature: resolve(THEME_COLORS.feature),
    selectedFeature: resolve(THEME_COLORS.selectedFeature),
    label: resolve(THEME_COLORS.label),
    labelHalo: resolve(THEME_COLORS.labelHalo),
  };
}

/**
 * A map to display, draw and edit the features of a GeoJSON FeatureCollection, including their properties.
 */
@Component({
  selector: 'app-map',
  template: `
    <div class="toolbar">
      <app-location-search [bias]="center()" (locationSelected)="zoomToLocation($event)" />
      @if (editable() && !singleGeometryType()) {
        <mat-button-toggle-group
          hideSingleSelectionIndicator
          aria-label="Map tool"
          [value]="tool()"
          (change)="tool.set($event.value)"
        >
          <mat-button-toggle value="select" matTooltip="Select and move features">
            <mat-icon>pan_tool_alt</mat-icon>
          </mat-button-toggle>
          @for (geometryType of geometryTypes(); track geometryType) {
            <mat-button-toggle [value]="geometryType" [matTooltip]="drawTooltip(geometryType)">
              <mat-icon>{{ geometryType === 'Point' ? 'add_location_alt' : 'pentagon' }}</mat-icon>
            </mat-button-toggle>
          }
        </mat-button-toggle-group>
      }
      <button
        mat-icon-button
        type="button"
        matTooltip="Zoom to features"
        [disabled]="features().features.length === 0"
        (click)="zoomToFeatures()"
      >
        <mat-icon>zoom_out_map</mat-icon>
      </button>
    </div>

    <div #map class="map"></div>

    @if (showFeatureList()) {
      @if (features().features.length > 0) {
        <ol class="feature-list">
          @for (feature of features().features; track $index; let index = $index) {
            <li
              [class.selected]="selectedIndex() === index"
              tabindex="0"
              (click)="selectedIndex.set(index)"
              (keydown.enter)="selectedIndex.set(index)"
              (focusin)="selectedIndex.set(index)"
            >
              <mat-icon
                class="geometry-icon"
                [matTooltip]="feature.geometry?.type ?? 'No geometry'"
              >
                {{ geometryIcon(feature) }}
              </mat-icon>
              @for (field of fields(); track field.key) {
                <mat-form-field subscriptSizing="dynamic">
                  <mat-label>{{ field.title }}</mat-label>
                  @if (field.enumValues; as options) {
                    <mat-select
                      [value]="propertyValue(feature, field.key)"
                      [disabled]="!editable()"
                      (selectionChange)="setProperty(index, field.key, $event.value)"
                    >
                      @for (option of options; track option) {
                        <mat-option [value]="option">{{ option }}</mat-option>
                      }
                    </mat-select>
                  } @else {
                    <input
                      matInput
                      type="text"
                      [value]="propertyValue(feature, field.key)"
                      [disabled]="!editable()"
                      (input)="setProperty(index, field.key, $event.target.value)"
                    />
                  }
                </mat-form-field>
              }
              @if (editable()) {
                <button
                  mat-icon-button
                  type="button"
                  matTooltip="Delete feature"
                  (click)="deleteFeature(index); $event.stopPropagation()"
                >
                  <mat-icon>delete</mat-icon>
                </button>
              }
            </li>
          }
        </ol>
      } @else if (editable()) {
        <p class="hint">Select a drawing tool above and click on the map to add features.</p>
      }
    }
  `,
  styleUrl: './map.component.scss',
  // OpenLayers' styles apply to elements it creates itself
  encapsulation: ViewEncapsulation.None,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    LocationSearchComponent,
    MatButtonModule,
    MatButtonToggleModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatSelectModule,
    MatTooltipModule,
  ],
})
export class MapComponent {
  readonly features = model<GeoJsonFeatureCollection>(emptyGeoJsonFeatureCollection());
  /** Geometry types that can be drawn */
  readonly geometryTypes = input<DrawableGeometryType[]>(['Point', 'Polygon']);
  /** Feature properties that can be edited */
  readonly fields = input<FeatureField[]>([]);
  readonly editable = input(true);
  /** If set, drawing more features replaces the oldest ones, e.g., to move a single point */
  readonly maxFeatures = input<number | undefined>(undefined);
  /** The list of features and their editable properties */
  readonly showFeatureList = input(true);

  readonly selectedIndex = model<number | undefined>(undefined);
  readonly tool = model<MapTool>('select');
  /**
   * For a single feature with a single geometry type (e.g., a coordinate), a click places the feature.
   * Selecting and moving it would be redundant, so this is the only tool.
   */
  readonly singleGeometryType = computed((): DrawableGeometryType | undefined => {
    const geometryTypes = this.geometryTypes();
    return this.maxFeatures() === 1 && geometryTypes.length === 1 ? geometryTypes[0] : undefined;
  });
  readonly activeTool = computed((): MapTool => this.singleGeometryType() ?? this.tool());
  /** [lon, lat] of the view, e.g., to prefer nearby search results */
  readonly center = signal<[number, number]>(DEFAULT_CENTER);

  readonly mapElement = viewChild.required<ElementRef<HTMLElement>>('map');

  private readonly format = new GeoJSON({
    dataProjection: 'EPSG:4326',
    featureProjection: 'EPSG:3857',
  });
  private readonly source = new VectorSource<OlFeature<Geometry>>();
  private readonly layer = new VectorLayer({
    source: this.source,
    style: (feature): Style => this.featureStyle(feature),
  });
  private readonly map = new OlMap({
    layers: [new TileLayer({ source: new OSM() }), this.layer],
    view: new View({ center: fromLonLat(DEFAULT_CENTER), zoom: DEFAULT_ZOOM }),
  });
  private readonly modify = new Modify({ source: this.source });
  private readonly snap = new Snap({ source: this.source });
  private draw?: Draw;

  private colors: MapColors = fallbackColors();

  /** The last collection that this component emitted, to not reset the view on own changes. */
  private lastEmitted?: GeoJsonFeatureCollection;

  constructor() {
    const size = fromResize(this.mapElement);

    afterNextRender(() => {
      this.colors = resolveThemeColors(this.mapElement().nativeElement);
      this.layer.changed();
      this.map.setTarget(this.mapElement().nativeElement);
      this.map.on('click', (event) => {
        if (this.activeTool() !== 'select') return;
        const feature = this.map.forEachFeatureAtPixel(event.pixel, (hit) => hit);
        this.selectedIndex.set(feature?.get(FEATURE_INDEX) as number | undefined);
      });
      this.modify.on('modifyend', (event) => this.updateGeometries(event.features.getArray()));
      this.map.on('moveend', () => {
        const center = this.map.getView().getCenter();
        if (center) this.center.set(toLonLat(center) as [number, number]);
      });
    });

    inject(DestroyRef).onDestroy(() => this.map.setTarget(undefined));

    // load features into the map
    effect(() => {
      const collection = this.features();
      untracked(() => this.loadFeatures(collection));
    });

    // restyle features on selection
    effect(() => {
      this.selectedIndex();
      this.fields();
      this.layer.changed();
    });

    // switch between drawing and modifying
    effect(() => {
      const tool = this.activeTool();
      const editable = this.editable();
      untracked(() => this.setInteractions(editable ? tool : undefined));
    });

    effect(() => {
      size();
      this.map.updateSize();
    });
  }

  zoomToFeatures(): void {
    const extent = this.source.getExtent();
    if (!extent || isEmptyExtent(extent)) return;

    this.map.getView().fit(extent, { padding: [32, 32, 32, 32], maxZoom: 16, duration: 250 });
  }

  zoomToLocation({ coordinate, extent }: GeocodingResult): void {
    const view = this.map.getView();

    if (extent) {
      const mapExtent: Extent = transformExtent(extent, 'EPSG:4326', view.getProjection());
      view.fit(mapExtent, { padding: [32, 32, 32, 32], maxZoom: LOCATION_ZOOM, duration: 250 });
      return;
    }

    view.animate({ center: fromLonLat(coordinate), zoom: LOCATION_ZOOM, duration: 250 });
  }

  deleteFeature(index: number): void {
    const features = this.features().features.filter((_, i) => i !== index);
    this.emit({ ...this.features(), features });

    const selectedIndex = this.selectedIndex();
    if (selectedIndex === undefined || selectedIndex < index) return;
    this.selectedIndex.set(selectedIndex === index ? undefined : selectedIndex - 1);
  }

  setProperty(index: number, key: string, value: string): void {
    const features = this.features().features.map((feature, i) =>
      i === index
        ? { ...feature, properties: { ...(feature.properties as object | null), [key]: value } }
        : feature,
    );
    this.emit({ ...this.features(), features });
  }

  propertyValue(feature: GeoJSONFeature, key: string): unknown {
    return (feature.properties as Record<string, unknown> | null)?.[key] ?? '';
  }

  geometryIcon(feature: GeoJSONFeature): string {
    return geometryIcon(feature.geometry?.type);
  }

  drawTooltip(geometryType: DrawableGeometryType): string {
    return geometryType === 'Point' ? 'Draw a point' : 'Draw a polygon (double-click to finish)';
  }

  private emit(collection: GeoJsonFeatureCollection): void {
    this.lastEmitted = collection;
    this.features.set(collection);
  }

  private loadFeatures(collection: GeoJsonFeatureCollection): void {
    const olFeatures = this.format.readFeatures(collection) as OlFeature<Geometry>[];
    olFeatures.forEach((feature, index) => feature.set(FEATURE_INDEX, index, true));

    this.source.clear(true);
    this.source.addFeatures(olFeatures);

    // keep the view if the change came from the user interacting with the map
    if (collection === this.lastEmitted) return;

    // do not jump around while a single point is edited, e.g., by typing its coordinates
    if (this.maxFeatures() === 1 && this.viewContains(this.source.getExtent())) return;

    this.zoomToFeatures();
  }

  private viewContains(extent: Extent | null): boolean {
    const size = this.map.getSize();
    if (!size || !extent || isEmptyExtent(extent)) return false;

    return containsExtent(this.map.getView().calculateExtent(size), extent);
  }

  private setInteractions(tool: MapTool | undefined): void {
    this.map.removeInteraction(this.modify);
    this.map.removeInteraction(this.snap);
    if (this.draw) {
      this.map.removeInteraction(this.draw);
      this.draw = undefined;
    }

    if (tool === undefined) return;

    if (tool === 'select') {
      this.map.addInteraction(this.modify);
    } else {
      // no `source`: the features are added via the collection
      this.draw = new Draw({ type: tool });
      this.draw.on('drawend', (event) => this.addFeature(event.feature));
      this.map.addInteraction(this.draw);
    }

    // snap must be added after the other interactions
    this.map.addInteraction(this.snap);
  }

  private addFeature(olFeature: FeatureLike): void {
    const geometry = olFeature.getGeometry() as Geometry | undefined;
    if (!geometry) return;

    const feature = markAsDrawn({
      type: 'Feature',
      geometry: this.format.writeGeometryObject(geometry) as GeoJSONFeatureGeometry,
      properties: defaultFeatureProperties(this.fields()),
    } as GeoJSONFeature);

    const maxFeatures = this.maxFeatures();
    const features = [...this.features().features, feature].slice(
      maxFeatures === undefined ? 0 : -maxFeatures,
    );
    this.emit({ ...this.features(), features });
    this.selectedIndex.set(features.length - 1);
  }

  private updateGeometries(olFeatures: FeatureLike[]): void {
    const geometries = new Map<number, GeoJSONFeatureGeometry>();
    for (const olFeature of olFeatures) {
      const geometry = olFeature.getGeometry() as Geometry | undefined;
      if (!geometry) continue;
      geometries.set(
        olFeature.get(FEATURE_INDEX) as number,
        this.format.writeGeometryObject(geometry) as GeoJSONFeatureGeometry,
      );
    }

    const features = this.features().features.map((feature, index) => {
      const geometry = geometries.get(index);
      return geometry ? { ...feature, geometry } : feature;
    });
    this.emit({ ...this.features(), features });
  }

  private featureStyle(feature: FeatureLike): Style {
    const selected = feature.get(FEATURE_INDEX) === this.selectedIndex();
    const color = selected ? this.colors.selectedFeature : this.colors.feature;

    const labelField = nameField(this.fields());
    const label: unknown = labelField ? feature.get(labelField.key) : undefined;

    const stroke = new Stroke({ color, width: selected ? 3 : 2 });
    const fill = new Fill({ color: [...color, 0.2] });

    return new Style({
      stroke,
      fill,
      image: new CircleStyle({ radius: selected ? 7 : 6, stroke, fill: new Fill({ color }) }),
      text:
        typeof label === 'string' && label !== ''
          ? new Text({
              text: label,
              offsetY: -16,
              font: '600 12px Poppins, sans-serif',
              fill: new Fill({ color: this.colors.label }),
              stroke: new Stroke({ color: this.colors.labelHalo, width: 3 }),
            })
          : undefined,
      zIndex: selected ? 1 : 0,
    });
  }
}

/**
 * The field to label the features with, i.e., the first free-text field (e.g., a name).
 * Fields with options are class labels, so they are only used if there is no other field.
 */
export function nameField(fields: FeatureField[]): FeatureField | undefined {
  return fields.find(({ enumValues }) => !enumValues) ?? fields.at(0);
}

export function geometryIcon(geometryType: string | undefined): string {
  switch (geometryType) {
    case 'Point':
    case 'MultiPoint':
      return 'place';
    case 'Polygon':
    case 'MultiPolygon':
      return 'pentagon';
    default:
      return 'help_outline';
  }
}
