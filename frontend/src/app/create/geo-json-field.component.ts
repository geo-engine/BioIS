import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  ElementRef,
  input,
  linkedSignal,
  model,
  signal,
  untracked,
  viewChild,
} from '@angular/core';
import { MatFormFieldModule } from '@angular/material/form-field';
import { FormValueControl, ValidationError, WithOptionalFieldTree } from '@angular/forms/signals';
import { MatCardModule } from '@angular/material/card';
import { MatListModule } from '@angular/material/list';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { DndDirective } from '../util/drag-and-drop.directive';
import { GeoJsonInputMediaType } from '@geoengine/biois';
import { Ajv, ErrorObject } from 'ajv';
import { JSONSchema } from 'ya-json-schema-types';
import { MapComponent } from '../map/map.component';
import {
  emptyGeoJsonFeatureCollection,
  FeatureCollectionGeoJsonInput,
  FeatureField,
  GeoJsonFeatureCollection,
  isDrawnFeature,
  missingFeatureProperties,
} from '../util/geo-json';
import { allowedGeometryTypes } from './schema-info';

@Component({
  selector: 'app-geo-json-field',
  template: `
    @if (fileName(); as name) {
      <mat-list>
        <mat-list-item>
          <mat-icon matListItemIcon>insert_drive_file</mat-icon>
          <span matListItemTitle>{{ name }}</span>
          <button
            mat-icon-button
            type="button"
            matListItemMeta
            (click)="removeFile(); $event.stopPropagation()"
          >
            <mat-icon color="warn">delete</mat-icon>
          </button>
        </mat-list-item>
      </mat-list>
    } @else {
      <mat-card
        class="dropzone"
        appDnd
        (fileDropped)="onFileDropped($event)"
        (click)="triggerBrowse()"
      >
        <mat-card-content>
          <mat-icon color="primary">cloud_upload</mat-icon>
          <span>Drag & drop a GeoJSON file or click to <b>browse</b> – or draw on the map</span>
        </mat-card-content>
      </mat-card>
    }
    <input
      #fileInput
      type="file"
      accept="application/geo+json,application/json,.json,.geojson"
      (change)="onFileSelected($event)"
      hidden
    />

    <app-map
      [features]="collection()"
      (featuresChange)="onCollectionChange($event)"
      [geometryTypes]="geometryTypes()"
      [fields]="fields()"
    />

    @for (error of errors(); track error) {
      <mat-error>{{ error.message }}</mat-error>
    }
    @if (errorValue(); as error) {
      <mat-error>{{ error }}</mat-error>
    } @else if (missingPropertiesWarning(); as warning) {
      <p class="warning">
        <mat-icon inline>warning</mat-icon>
        {{ warning }} Choose other properties above or fill in the values, otherwise the process
        reports such features as errors.
      </p>
    }
  `,
  styles: `
    :host {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    .warning {
      margin: 0;
      font: var(--mat-sys-body-small);
      color: var(--mat-sys-on-surface-variant);
    }

    .dropzone {
      width: 100%;
      text-align: center;
      border: 2px dashed var(--mat-sys-primary);
      transition: all 0.3s ease;
      cursor: pointer;

      mat-card-content {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
      }

      &.fileover {
        border-color: var(--mat-sys-secondary); /* Material Secondary Color */
        background-color: var(--mat-sys-surface-container);
        transform: scale(1.02);
      }
    }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    DndDirective,
    MapComponent,
    MatButtonModule,
    MatCardModule,
    MatFormFieldModule,
    MatIconModule,
    MatListModule,
  ],
})
export class GeoJsonFormFieldComponent implements FormValueControl<
  FeatureCollectionGeoJsonInput | Error
> {
  readonly title = input.required<string>();
  readonly geoJsonSchema = input.required<JSONSchema>();
  /** Feature properties that users must fill in, e.g., a name and a class label */
  readonly fields = input<FeatureField[]>([]);

  readonly value = model.required<FeatureCollectionGeoJsonInput | Error>();
  readonly errors = input.required<readonly WithOptionalFieldTree<ValidationError>[]>();

  readonly fileName = signal<string | undefined>(undefined);
  readonly fileInput = viewChild<ElementRef<HTMLInputElement>>('fileInput');

  /** The features on the map, which are kept even if they are (still) invalid */
  readonly collection = linkedSignal<
    FeatureCollectionGeoJsonInput | Error,
    GeoJsonFeatureCollection
  >({
    source: this.value,
    computation: (value, previous) =>
      value instanceof Error ? (previous?.value ?? emptyGeoJsonFeatureCollection()) : value.value,
  });

  readonly geometryTypes = computed(() => allowedGeometryTypes(this.geoJsonSchema()));

  readonly geoJsonValidator = computed(() =>
    new Ajv({
      dynamicRef: true,
      allErrors: true,
      verbose: true,
      loadSchema: loadSchema,
    }).compileAsync(this.geoJsonSchema() as Record<string, unknown>),
  );

  /** Only the latest validation may set the value */
  private validationRun = 0;

  /**
   * The fields by content, since callers may pass new but equal arrays on every change.
   * Otherwise, validating (i.e., setting the value) would trigger itself endlessly.
   */
  private readonly fieldsByContent = computed(() => this.fields(), {
    equal: (a, b) => JSON.stringify(a) === JSON.stringify(b),
  });

  /**
   * Uploaded features with missing properties do not invalidate the input, since the process reports them as errors.
   * Moreover, the fields can only be changed to other properties of a valid input.
   * Drawn features, however, must be complete (cf. `validate`).
   */
  readonly missingPropertiesWarning = computed(() =>
    missingFeatureProperties(
      this.collection(),
      this.fieldsByContent(),
      (feature) => !isDrawnFeature(feature),
    ),
  );

  constructor() {
    // e.g., the user selected another property for a field
    effect(() => {
      this.fieldsByContent();
      untracked(() => void this.validate());
    });
  }

  async onFileDropped(files: FileList): Promise<void> {
    if (files.length <= 0) return;
    await this.handleFileSelection(files[0]);
  }

  async onFileSelected(event: Event): Promise<void> {
    const inputElement = event.target as HTMLInputElement;
    if (!inputElement.files?.length) return;
    await this.handleFileSelection(inputElement.files[0]);
  }

  async onCollectionChange(collection: GeoJsonFeatureCollection): Promise<void> {
    this.collection.set(collection);
    await this.validate();
  }

  triggerBrowse(): void {
    const inputEl = this.fileInput()?.nativeElement;
    if (!inputEl) return;
    inputEl.click();
  }

  async removeFile(): Promise<void> {
    this.fileName.set(undefined);
    this.collection.set(emptyGeoJsonFeatureCollection());
    await this.validate();

    // Reset the input value so the same file can be re-selected if needed
    const inputEl = this.fileInput()?.nativeElement;
    if (!inputEl) return;
    inputEl.value = '';
  }

  errorValue(): Error | undefined {
    const value = this.value();
    return value instanceof Error ? value : undefined;
  }

  private async handleFileSelection(file: File): Promise<void> {
    this.fileName.set(file.name);

    const content = await readFileContents(file);

    let collection: unknown;
    try {
      collection = JSON.parse(content);
    } catch (error) {
      this.validationRun++; // discard pending validations
      this.value.set(new Error('Invalid JSON file', { cause: error })); // Set error if parsing fails
      return;
    }

    if (!isFeatureCollection(collection)) {
      this.validationRun++; // discard pending validations
      this.value.set(new Error('The file is not a GeoJSON FeatureCollection'));
      return;
    }

    this.collection.set(collection);
    await this.validate();
  }

  /** Validates the features on the map and sets the value (or an error) accordingly. */
  private async validate(): Promise<void> {
    const run = ++this.validationRun;
    const collection = this.collection();

    const geoJsonInput: FeatureCollectionGeoJsonInput = {
      value: collection,
      mediaType: GeoJsonInputMediaType.ApplicationGeojson,
    };

    const validate = await this.geoJsonValidator();
    const valid = validate(geoJsonInput);

    // TODO: use the zod schema for validation instead of Ajv, to avoid maintaining two separate schemas and validators

    if (run !== this.validationRun) return; // a newer validation is pending

    if (collection.features.length === 0) {
      this.value.set(new Error('Upload a file or draw at least one feature on the map'));
      return;
    }

    if (!valid) {
      this.value.set(new Error(invalidGeoJsonMessage(validate.errors), { cause: validate.errors }));
      return;
    }

    const missingProperty = missingFeatureProperties(
      collection,
      this.fieldsByContent(),
      isDrawnFeature,
    );
    if (missingProperty) {
      this.value.set(new Error(missingProperty));
      return;
    }

    this.value.set(geoJsonInput);
  }
}

function invalidGeoJsonMessage(errors: ErrorObject[] | null | undefined): string {
  const error = errors?.[0];
  if (!error) return 'Invalid GeoJSON format';

  // `/value/features/0/geometry` -> `features/0/geometry`
  const path = error.instancePath.replace(/^\/value\/?/, '');
  return `Invalid GeoJSON format: ${path ? `${path} ` : ''}${error.message ?? 'is invalid'}`;
}

function isFeatureCollection(json: unknown): json is GeoJsonFeatureCollection {
  return (
    typeof json === 'object' &&
    json !== null &&
    (json as Record<string, unknown>)['type'] === 'FeatureCollection' &&
    Array.isArray((json as Record<string, unknown>)['features'])
  );
}

async function readFileContents(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (): void => resolve(reader.result as string);
    reader.onerror = (): void => reject(reader.error ?? new Error('Unknown file reading error'));
    reader.readAsText(file);
  });
}

async function loadSchema(uri: string): Promise<Record<string, unknown>> {
  const res = await fetch(uri);
  if (!res.ok) throw new Error('Loading error: ' + res.status);
  return res.json() as Promise<Record<string, unknown>>;
}
