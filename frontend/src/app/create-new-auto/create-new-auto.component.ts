import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  input,
  resource,
  signal,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import {
  form,
  FormField,
  FieldTree,
  MaybeFieldTree,
  validateStandardSchema,
} from '@angular/forms/signals';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatButtonModule } from '@angular/material/button';
import {
  FeatureCollectionGeoJsonInput,
  GeoJsonInputMediaType,
  GeoJSONPoint,
  GeoJSONPointTypeEnum,
  Input,
  Metadata,
  Output,
  PointGeoJsonInput,
  ProcessesApi,
  Response,
} from '@geoengine/biois';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { UserService } from '../user.service';
import { Router } from '@angular/router';
import { MatIcon } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';
import { processName } from '../util/processes';
import { type JSONSchema } from 'ya-json-schema-types';
import { SimpleFormFieldComponent } from './simple-form-field';
import { GeoJsonFormFieldComponent } from './geo-json-field';
import * as z from 'zod';
import { convertJsonSchemaToZod } from 'zod-from-json-schema';
import { findByPointer } from '@jsonjoy.com/json-pointer';
import { marked } from 'marked';
import { LongTextComponent } from '../util/long-text.component';
import { PageTitleComponent } from '../navigation/page-title.component';

@Component({
  selector: 'app-create-new-auto',
  imports: [
    CommonModule,
    FormField,
    GeoJsonFormFieldComponent,
    LongTextComponent,
    MatButtonModule,
    MatCheckboxModule,
    MatFormFieldModule,
    MatIcon,
    MatInputModule,
    MatSelectModule,
    MatTooltipModule,
    PageTitleComponent,
    SimpleFormFieldComponent,
  ],
  templateUrl: './create-new-auto.component.html',
  styleUrls: ['./create-new-auto.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class CreateNewAutoComponent {
  readonly userService = inject(UserService);
  readonly router = inject(Router);

  readonly processId = input.required<string>();

  readonly formModel = signal<{
    inputs: Record<string, unknown>;
    outputs: Record<string, boolean>;
  }>({
    inputs: {},
    outputs: {},
  });

  readonly schema = computed((): z.ZodTypeAny => {
    const inputs = this.inputs();

    const zodInputs: Record<string, z.ZodTypeAny> = {};
    for (const { key, zodSchema } of inputs) {
      // console.log(`Converting JSON Schema for input "${key}" to Zod schema...`, zodSchema);
      zodInputs[key] = zodSchema;
    }

    return z.object({
      inputs: z.object(zodInputs),
      outputs: z
        .record(z.string(), z.unknown())
        .refine((outputs) => Object.keys(outputs).length > 0, {
          message: 'At least one output must be selected.',
        }),
    });
  });

  readonly form = form(this.formModel, (schema) => {
    validateStandardSchema(schema, () => this.schema());
  });

  readonly description = resource({
    params: () => ({
      processId: this.processId(),
    }),
    loader: ({ params }) => {
      const processApi = new ProcessesApi(this.userService.apiConfiguration());
      return processApi.process(params.processId);
    },
  });

  readonly processName = computed(() => {
    const title = this.description.value()?.title;
    return title ?? this.fieldName(this.processId());
  });

  readonly processDescriptionHtml = computed(() => {
    const description = this.description.value()?.description;
    if (!description) return '';

    return marked.parse(description, { async: false });
  });

  readonly inputs = computed(() => {
    const descriptionInputs = this.description.value()?.inputs;
    if (!descriptionInputs) return [];

    return Object.entries(descriptionInputs)
      .sort(([leftKey], [rightKey]) => leftKey.localeCompare(rightKey)) // TODO: get order from process description when available
      .map(([key, processInput]) => ({
        key,
        title: processInput.title ?? this.fieldName(key),
        description: processInput.description,
        type: typeFromSchema(processInput.schema as JSONSchema),
        metadata: processInput.metadata,
        schema: processInput.schema as Record<string, unknown>,
        zodSchema: jsonSchemaToZod(processInput.schema as Record<string, unknown>),
      }));
  });

  readonly outputs = computed(() => {
    const descriptionOutputs = this.description.value()?.outputs;
    if (!descriptionOutputs) return [];

    return Object.entries(descriptionOutputs).map(([key, processOutput]) => ({
      key,
      title: processOutput.title ?? this.fieldName(key),
      description: processOutput.description,
    }));
  });

  readonly relativeJsonPointerAvailableFields = computed<Record<string, string[]>>(() =>
    availableFieldsForRelativeJsonPointers(this.formModel(), this.inputs()),
  );

  readonly fieldName = processName;
  readonly FieldType = FieldType;

  constructor() {
    // initially, set all inputs
    effect(() => {
      const inputDescriptions = this.inputs();
      const inputs: Record<string, Input> = {};
      for (const { key, type, schema } of inputDescriptions) {
        switch (type) {
          case FieldType.Number:
          case FieldType.Integer:
            inputs[key] = defaultNumber(schema, 0);
            break;
          case FieldType.Boolean:
            inputs[key] = false;
            break;
          case FieldType.Coordinate:
            inputs[key] = {
              value: defaultCoordinate(schema),
              mediaType: GeoJsonInputMediaType.ApplicationGeojson,
            } as PointGeoJsonInput;
            break;
          case FieldType.GeoJson:
            inputs[key] = new Error('Missing GeoJSON input.'); // Placeholder value to indicate that the user needs to upload a file
            break;
          case FieldType.String:
          default:
            inputs[key] = defaultString(schema, '');
            break;
        }
      }
      this.formModel.update((current) => ({ ...current, inputs }));
    });

    // initially, set all outputs
    effect(() => {
      const outputDescriptions = this.outputs();
      const outputs = Object.fromEntries(outputDescriptions.map(({ key }) => [key, true]));
      this.formModel.update((current) => ({ ...current, outputs }));
    });
  }

  async onSubmit(): Promise<void> {
    const processApi = new ProcessesApi(this.userService.apiConfiguration());

    await processApi.execution(this.processId(), {
      inputs: this.formModel().inputs,
      outputs: outputForRequest(this.formModel().outputs),
      response: Response.Document,
    });

    await this.router.navigate(['/app/results']);
  }

  toggleOutput(outputKey: string, isChecked: boolean): void {
    this.formModel.update((current) => {
      const currentOutputs = current.outputs || {};
      if (isChecked) {
        // Add the key
        return { ...current, outputs: { ...currentOutputs, [outputKey]: true } };
      } else {
        // Remove the key entirely using destructuring
        const { [outputKey]: _, ...remaining } = currentOutputs;
        return { ...current, outputs: remaining };
      }
    });
  }

  /**
   * Force the type of the form input to be a `FeatureCollectionGeoJsonInput`.
   * TODO: validation
   */
  asPrimitiveInput(
    formInput: MaybeFieldTree<unknown, string>,
  ): FieldTree<string | number | boolean, string> {
    return formInput as FieldTree<string | number | boolean, string>;
  }

  /**
   * Force the type of the form input to be a `FeatureCollectionGeoJsonInput`.
   * TODO: validation
   */
  asGeoJsonInput(
    formInput: MaybeFieldTree<unknown, string>,
  ): FieldTree<FeatureCollectionGeoJsonInput, string> {
    return formInput as FieldTree<FeatureCollectionGeoJsonInput, string>;
  }

  /**
   * Force the type of the form input to be a `PointGeoJsonInput`.
   * TODO: validation
   */
  asGeoJsonPointFeature(
    formInput: MaybeFieldTree<unknown, string>,
  ): FieldTree<PointGeoJsonInput, string> {
    return formInput as FieldTree<PointGeoJsonInput, string>;
  }

  enumOptions(schema: JSONSchema | undefined): string[] {
    if (!schema || typeof schema === 'boolean' || !schema.enum || !Array.isArray(schema.enum))
      return [];

    const options = [];
    for (const value of schema.enum) {
      if (typeof value === 'string') options.push(value);
    }
    return options;
  }

  integerRangeList(schema: JSONSchema | undefined): number[] {
    if (
      !schema ||
      typeof schema === 'boolean' ||
      schema.type !== 'integer' ||
      typeof schema.minimum !== 'number' ||
      typeof schema.maximum !== 'number'
    )
      return [];

    const range = [];
    for (let i = schema.minimum; i <= schema.maximum; i++) {
      range.push(i);
    }
    return range;
  }
}

function outputForRequest(output: Record<string, boolean>): Record<string, Output> {
  return Object.fromEntries(
    Object.entries(output)
      .filter(([_, value]) => value)
      .map(([key, _]) => [key, {}]),
  );
}

/**
 * Determine the field type from the JSON schema.
 * This is a simplified version and may need to be expanded to handle more complex schemas (e.g., arrays, nested objects, etc.).
 */
function typeFromSchema(schema: JSONSchema | undefined): FieldType {
  if (!schema) return FieldType.String;

  // JSON Schema may be a boolean (true/false) or an object. If it's a boolean,
  // it doesn't have a `type` property, so handle that case first.
  if (typeof schema === 'boolean') return FieldType.String;

  if (schema.type === 'string') {
    if (schema.format === 'relative-json-pointer') return FieldType.RelativeJsonPointer;
    if (schema.enum) return FieldType.StringEnum;

    return FieldType.String;
  }
  if (schema.type === 'number') return FieldType.Number;
  if (schema.type === 'integer') {
    if (
      typeof schema.maximum === 'number' &&
      typeof schema.minimum === 'number' &&
      schema.maximum - schema.minimum <= 12
    ) {
      return FieldType.IntegerWithSmallRange;
    }
    return FieldType.Integer;
  }
  if (schema.type === 'boolean') return FieldType.Boolean;

  if (schema.type === 'object') {
    if (schema.title === 'PointGeoJsonInput') return FieldType.Coordinate;
    if (schema.title === 'FeatureCollectionGeoJsonInput') return FieldType.GeoJson;
  }

  return FieldType.String; // fallback to string if type cannot be determined
}

export enum FieldType {
  Boolean = 'boolean',
  Coordinate = 'coordinate',
  GeoJson = 'geoJson',
  Integer = 'integer',
  IntegerWithSmallRange = 'integerWithSmallRange',
  Number = 'number',
  RelativeJsonPointer = 'relativeJsonPointer',
  String = 'string',
  StringEnum = 'stringEnum',
}

function geoJsonPointFeature(coordinates: [number, number]): GeoJSONPoint {
  const point = new GeoJSONPoint();
  point.type = GeoJSONPointTypeEnum.Point;
  point.coordinates = coordinates;
  return point;
}

function jsonSchemaToZod(jsonSchema: Record<string, unknown>): z.ZodTypeAny {
  const errors = [];

  try {
    return z.fromJSONSchema(jsonSchema);
  } catch (error) {
    errors.push(error);
  }

  try {
    return convertJsonSchemaToZod(jsonSchema);
  } catch (error) {
    errors.push(error);
  }

  throw new Error('Failed to convert JSON Schema to Zod schema.', { cause: errors });
}

function defaultNumber(schema: JSONSchema, fallback: number = 0): number {
  if (!schema || typeof schema === 'boolean') return fallback;

  const defaultValue = schema.default;
  if (typeof defaultValue === 'number') return defaultValue;

  if (!schema.examples || !Array.isArray(schema.examples)) return fallback;

  for (const example of schema.examples ?? []) {
    if (typeof example === 'number') return example;
  }

  return fallback;
}

function defaultString(schema: JSONSchema, fallback: string = ''): string {
  if (!schema || typeof schema === 'boolean') return fallback;

  const defaultValue = schema.default;
  if (typeof defaultValue === 'string') return defaultValue;

  if (!schema.examples || !Array.isArray(schema.examples)) return fallback;

  for (const example of schema.examples ?? []) {
    if (typeof example === 'string') return example;
  }

  return fallback;
}

function defaultCoordinate(schema: JSONSchema, fallback: [number, number] = [0, 0]): GeoJSONPoint {
  if (!schema || typeof schema === 'boolean') return geoJsonPointFeature(fallback);

  if (
    !schema.properties ||
    !(typeof schema.properties == 'object') ||
    !('value' in schema.properties)
  )
    return geoJsonPointFeature(fallback);

  const coordinateValue = schema.properties.value as JSONSchema;
  if (!coordinateValue || typeof coordinateValue === 'boolean')
    return geoJsonPointFeature(fallback);

  if (coordinateValue.default) {
    return coordinateValue.default as unknown as GeoJSONPoint;
  }

  if (!coordinateValue.examples || !Array.isArray(coordinateValue.examples))
    return geoJsonPointFeature(fallback);

  for (const example of coordinateValue.examples ?? []) {
    return example as GeoJSONPoint;
  }

  return geoJsonPointFeature(fallback);
}

function availableFieldsForRelativeJsonPointers(
  formModel: { inputs: Record<string, unknown> },
  inputs: { key: string; type: FieldType; metadata?: Metadata[] }[],
): Record<string, string[]> {
  const availableFields: Record<string, string[]> = {};

  for (const { key, type, metadata } of inputs) {
    if (!(type === FieldType.RelativeJsonPointer)) continue;

    availableFields[key] = [];
    const fields = availableFields[key];

    let href = metadata?.find((meta) => meta.role === 'json-pointer-base')?.href;
    if (!href) continue;

    if (href.startsWith('#')) href = href.substring(1); // remove leading hash

    let pointerBase: unknown;
    try {
      pointerBase = findByPointer(href, formModel).val;
    } catch (_error) {
      continue;
    }

    if (typeof pointerBase !== 'object' || pointerBase === null) continue;

    fields.push(...Object.keys(pointerBase));
  }

  return availableFields;
}
