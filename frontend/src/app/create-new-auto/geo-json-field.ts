import {
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  input,
  model,
  signal,
  viewChild,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatFormFieldModule } from '@angular/material/form-field';
import { FormValueControl, ValidationError, WithOptionalFieldTree } from '@angular/forms/signals';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatInputModule } from '@angular/material/input';
import { MatCardModule } from '@angular/material/card';
import { MatListModule } from '@angular/material/list';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { DndDirective } from '../util/drag-and-drop.directive';
import {
  FeatureCollectionGeoJsonInput,
  GeoJSONFeatureCollection,
  GeoJSONFeatureCollectionTypeEnum,
  GeoJsonInputMediaType,
} from '@geoengine/biois';
import { Ajv } from 'ajv';

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
          <h3>Drag & Drop File</h3>
          <p>or click to <b>browse</b></p>
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

    @for (error of errors(); track error) {
      <mat-error>{{ error.message }}</mat-error>
    }
    @if (errorValue(); as error) {
      <mat-error>{{ error }}</mat-error>
    }
  `,
  styles: `
    .dropzone {
      width: 100%;
      padding: 1rem;
      text-align: center;
      border: 2px dashed var(--mat-sys-primary);
      transition: all 0.3s ease;
      cursor: pointer;

      &.fileover {
        border-color: var(--mat-sys-secondary); /* Material Secondary Color */
        background-color: var(--mat-sys-surface-container);
        transform: scale(1.02);
      }
    }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    CommonModule,
    MatFormFieldModule,
    MatInputModule,
    MatCheckboxModule,
    DndDirective,
    MatCardModule,
    MatListModule,
    MatIconModule,
    MatButtonModule,
  ],
})
export class GeoJsonFormFieldComponent implements FormValueControl<
  FeatureCollectionGeoJsonInput | Error
> {
  readonly title = input.required<string>();
  readonly geoJsonSchema = input.required<Record<string, unknown>>();
  // readonly geoJsonSchema = input.required<z.ZodTypeAny>();

  readonly value = model.required<FeatureCollectionGeoJsonInput | Error>();
  readonly errors = input.required<readonly WithOptionalFieldTree<ValidationError>[]>();

  readonly fileName = signal<string | undefined>(undefined);
  readonly fileInput = viewChild<ElementRef<HTMLInputElement>>('fileInput');

  readonly geoJsonValidator = computed(() =>
    new Ajv({
      dynamicRef: true,
      allErrors: true,
      verbose: true,
      loadSchema: loadSchema,
    }).compileAsync(this.geoJsonSchema()),
  );

  async onFileDropped(files: FileList): Promise<void> {
    if (files.length <= 0) return;
    await this.handleFileSelection(files[0]);
  }

  async onFileSelected(event: Event): Promise<void> {
    const inputElement = event.target as HTMLInputElement;
    if (!inputElement.files?.length) return;
    await this.handleFileSelection(inputElement.files[0]);
  }

  triggerBrowse(): void {
    const inputEl = this.fileInput()?.nativeElement;
    if (!inputEl) return;
    inputEl.click();
  }

  removeFile(): void {
    this.fileName.set(undefined);
    this.value.set(new Error('No file selected')); // Set error state when file is removed

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

    let jsonContent: Record<string, unknown>;
    try {
      jsonContent = {
        value: JSON.parse(content) as JSON,
        mediaType: GeoJsonInputMediaType.ApplicationGeojson,
      };
    } catch (error) {
      // console.error('Error parsing GeoJSON file:', error);
      this.value.set(new Error('Invalid JSON file', { cause: error })); // Set error if parsing fails
      return;
    }

    const validate = await this.geoJsonValidator();
    const valid = validate(jsonContent);

    if (!valid) {
      // console.error('GeoJSON validation errors:', validate.errors);
      this.value.set(new Error('Invalid GeoJSON format', { cause: validate.errors }));
      return;
    }

    // TODO: use the zod schema for validation instead of Ajv, to avoid maintaining two separate schemas and validators

    // const parseResult = this.geoJsonSchema().safeParse(jsonContent);

    // console.log('GeoJSON validation result:', parseResult);

    // if (!parseResult.success) {
    //   console.error('GeoJSON validation errors:', parseResult.error);
    //   this.value.set(new Error('Invalid GeoJSON format', { cause: parseResult.error }));
    //   return;
    // }

    this.value.set(jsonContent as unknown as FeatureCollectionGeoJsonInput);
  }
}

async function readFileContents(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (): void => resolve(reader.result as string);
    reader.onerror = (): void => reject(reader.error ?? new Error('Unknown file reading error'));
    reader.readAsText(file);
  });
}

export function emptyGeoJsonFeatureCollection(): GeoJSONFeatureCollection {
  const featureCollection = new GeoJSONFeatureCollection();
  featureCollection.type = GeoJSONFeatureCollectionTypeEnum.FeatureCollection;
  featureCollection.features = [];
  return featureCollection;
}

export function emptyGeoJsonFeatureCollectionInput(): FeatureCollectionGeoJsonInput {
  return {
    value: emptyGeoJsonFeatureCollection(),
    mediaType: GeoJsonInputMediaType.ApplicationGeojson,
  };
}

async function loadSchema(uri: string): Promise<Record<string, unknown>> {
  const res = await fetch(uri);
  if (!res.ok) throw new Error('Loading error: ' + res.status);
  return res.json() as Promise<Record<string, unknown>>;
}
