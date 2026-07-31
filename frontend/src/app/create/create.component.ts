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
import { form, validateStandardSchema } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { Input, Metadata, Output, ProcessesApi, Response } from '@geoengine/biois';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { UserService } from '../user.service';
import { Router } from '@angular/router';
import { processName } from '../util/processes';
import * as z from 'zod';
import { findByPointer } from '@jsonjoy.com/json-pointer';
import { marked } from 'marked';
import { LongTextComponent } from '../util/long-text.component';
import { PageTitleComponent } from '../navigation/page-title.component';
import { InputsFormComponent } from './inputs-visualizer.component';
import { MatError } from '@angular/material/select';
import { MatIcon } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';
import {
  FieldType,
  InputDescription,
  retrieveInputDescription,
  jsonSchemaToZod,
  defaultInputs,
} from './schema-info';
import { assertNever, isNullOrUndefined } from '../util/assertions';

@Component({
  selector: 'app-create',
  imports: [
    CommonModule,
    InputsFormComponent,
    LongTextComponent,
    MatButtonModule,
    MatCheckboxModule,
    MatError,
    MatIcon,
    MatTooltipModule,
    PageTitleComponent,
  ],
  templateUrl: './create.component.html',
  styleUrls: ['./create.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class CreateComponent {
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
    for (const { key, schema } of inputs) {
      // console.log(`Converting JSON Schema for input "${key}" to Zod schema...`, schema);
      zodInputs[key] = jsonSchemaToZod(schema);
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

  readonly inputs = computed((): InputDescription[] => {
    const descriptionInputs = this.description.value()?.inputs;
    if (!descriptionInputs) return [];

    return Object.entries(descriptionInputs)
      .map(([key, processInput]) => retrieveInputDescription(key, processInput))
      .sort(compareInputDescriptionsForSorting);
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

  constructor() {
    // initially, set all inputs
    effect(() => {
      const inputDescriptions = this.inputs();
      const inputs: Record<string, Input> = defaultInputs(inputDescriptions);

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
      inputs: inputsForRequest(this.formModel().inputs),
      outputs: outputsForRequest(this.formModel().outputs),
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

  updateFormInputField(key: string, value: unknown): void {
    this.formModel.update((current) => ({
      ...current,
      inputs: { ...current.inputs, [key]: value },
    }));
  }
}

/**
 * Map the outputs from the form model to the format expected by the API request.
 * Only include outputs that are set to true.
 *
 * @param outputs - The outputs from the form model.
 * @returns A new object containing only the outputs that are set to true.
 */
function outputsForRequest(outputs: Record<string, boolean>): Record<string, Output> {
  return Object.fromEntries(
    Object.entries(outputs)
      .filter(([_, value]) => value)
      .map(([key, _]) => [key, {}]),
  );
}

/**
 * Filter out undefined values from the inputs and return a new object with only defined values.
 * This is necessary because the API expects only defined inputs to be sent.
 *
 * @param inputs - The input object to filter.
 * @returns A new object containing only the defined inputs.
 */
function inputsForRequest(inputs: Record<string, unknown>): Record<string, Input> {
  return Object.fromEntries(
    Object.entries(inputs).filter(([_, value]) => !isNullOrUndefined(value)),
  );
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

/**
 * An ordering function for InputDescription objects that sorts them based on their FieldType and key.
 *
 * It is important to have a strict and stable ordering.
 *
 * TODO: get order from process description when available
 */
function compareInputDescriptionsForSorting(
  leftInfo: InputDescription,
  rightInfo: InputDescription,
): number {
  const operatorPrecedence = (fieldType: FieldType): number => {
    switch (fieldType) {
      case FieldType.Number:
      case FieldType.Integer:
      case FieldType.IntegerWithSmallRange:
      case FieldType.Boolean:
      case FieldType.String:
      case FieldType.RelativeJsonPointer:
      case FieldType.StringEnum:
        return 0;
      case FieldType.Coordinate:
      case FieldType.GeoJson:
        return 1;
      case FieldType.NestedJson:
        return -1;
      default:
        assertNever(fieldType);
    }
  };

  const leftPrecedence = operatorPrecedence(leftInfo.type);
  const rightPrecedence = operatorPrecedence(rightInfo.type);

  if (leftPrecedence !== rightPrecedence) {
    return rightPrecedence - leftPrecedence; // higher precedence first
  }

  return leftInfo.key.localeCompare(rightInfo.key);
}
