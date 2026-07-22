import {
  ChangeDetectionStrategy,
  Component,
  computed,
  forwardRef,
  input,
  output,
} from '@angular/core';
import { InputDescription, FieldType, defaultInput } from './schema-info';
import { CommonModule } from '@angular/common';
import { FormField, FieldTree, MaybeFieldTree } from '@angular/forms/signals';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { FeatureCollectionGeoJsonInput, PointGeoJsonInput } from '@geoengine/biois';
import { MatIcon } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';
import { processName } from '../util/processes';
import { type JSONSchema } from 'ya-json-schema-types';
import { SimpleFormFieldComponent } from './simple-form-field';
import { GeoJsonFormFieldComponent } from './geo-json-field.component';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { isNullOrUndefined } from '../util/assertions';

@Component({
  selector: 'app-inputs-form',
  template: `
    @for (input of inputs(); track input.key) {
      <p>
        @if (input.optional) {
          <mat-slide-toggle
            [checked]="isFieldSet()[input.key]"
            (change)="toggleOptionalField($event.checked, input)"
          ></mat-slide-toggle>
        }
        <span class="padding">{{ input.title }}</span>
        <mat-icon [matTooltip]="input.description">info</mat-icon>
      </p>
      @if (!input.optional || isFieldSet()[input.key]) {
        @switch (input.type) {
          @case (FieldType.Boolean)
          @case (FieldType.Integer)
          @case (FieldType.Number)
          @case (FieldType.String) {
            <app-simple-form-field
              [title]="fieldName(input.key)"
              [type]="input.type"
              [formField]="asPrimitiveInput(form()[input.key])"
            ></app-simple-form-field>
          }
          @case (FieldType.StringEnum) {
            <mat-form-field>
              <mat-label>{{ input.title }}</mat-label>
              <mat-select [formField]="asPrimitiveInput(form()[input.key])">
                @for (option of enumOptions(input.schema); track option) {
                  <mat-option [value]="option">{{ option }}</mat-option>
                }
              </mat-select>
            </mat-form-field>
          }
          @case (FieldType.IntegerWithSmallRange) {
            <mat-form-field>
              <mat-label>{{ input.title }}</mat-label>
              <mat-select [formField]="asPrimitiveInput(form()[input.key])">
                @for (option of integerRangeList(input.schema); track option) {
                  <mat-option [value]="option">{{ option }}</mat-option>
                }
              </mat-select>
            </mat-form-field>
          }
          @case (FieldType.RelativeJsonPointer) {
            <mat-form-field>
              <mat-label>{{ input.title }}</mat-label>
              <mat-select [formField]="asPrimitiveInput(form()[input.key])">
                @for (field of relativeJsonPointerAvailableFields()[input.key]; track field) {
                  <mat-option [value]="field">{{ field }}</mat-option>
                }
              </mat-select>
            </mat-form-field>
          }
          @case (FieldType.Coordinate) {
            @let coordinateInput = asGeoJsonPointFeature(form()[input.key]).value;
            <div>
              @for (
                coordinateValue of ['Longitude', 'Latitude'];
                track $index;
                let index = $index
              ) {
                <mat-form-field>
                  <mat-label>{{ coordinateValue }}</mat-label>
                  <input
                    matInput
                    type="number"
                    step="any"
                    [formField]="coordinateInput.coordinates[index]"
                  />
                  @for (error of coordinateInput.coordinates[index]().errors(); track error) {
                    <mat-error>{{ error.message }}</mat-error>
                  }
                </mat-form-field>
              }

              @for (error of coordinateInput.coordinates().errors(); track error) {
                <mat-error>{{ error.message }}</mat-error>
              }
            </div>
          }
          @case (FieldType.GeoJson) {
            <app-geo-json-field
              [title]="fieldName(input.key)"
              [geoJsonSchema]="input.schema"
              [formField]="asGeoJsonInput(form()[input.key])"
            ></app-geo-json-field>
          }
          @case (FieldType.NestedJson) {
            <fieldset>
              <app-inputs-form
                [inputs]="toInputs(input.children)"
                [form]="asNestedJsonInput(form()[input.key])"
                [relativeJsonPointerAvailableFields]="relativeJsonPointerAvailableFields()"
              ></app-inputs-form>
            </fieldset>
          }
          <!-- Prevent unhandled cases -->
          @default never;
        }
      }
    }
  `,
  styles: [
    `
      .padding {
        &:not(:first-child) {
          padding-left: 0.5rem;
        }
        padding-right: 0.5rem;
      }
      fieldset {
        border-color: color-mix(in srgb, var(--mat-sys-surface-container-lowest) 63%, transparent);
        border-radius: var(--mat-sys-corner-medium);
        padding: 1rem;

        legend {
          font: var(--mat-sys-body-small);
          color: var(--mat-sys-on-surface-variant);
        }
      }
    `,
  ],
  standalone: true,
  imports: [
    CommonModule,
    FormField,
    GeoJsonFormFieldComponent,
    MatFormFieldModule,
    MatIcon,
    MatInputModule,
    MatSelectModule,
    MatSlideToggleModule,
    MatTooltipModule,
    SimpleFormFieldComponent,
    forwardRef(() => InputsFormComponent),
  ],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class InputsFormComponent {
  readonly inputs = input.required<InputDescription[]>();
  readonly form = input.required<Record<string, MaybeFieldTree<unknown, string>>>();
  readonly relativeJsonPointerAvailableFields = input<Record<string, string[]>>({});
  readonly updateFormField = output<{ key: string; value: unknown }>();

  readonly fieldName = processName;
  readonly FieldType = FieldType;
  readonly enumOptions = enumOptions;
  readonly integerRangeList = integerRangeList;

  readonly isFieldSet = computed<Record<string, boolean>>(() => {
    const form = this.form();
    const inputs = this.inputs();

    const result: Record<string, boolean> = {};

    for (const { key } of inputs) {
      const formInput = form[key];
      result[key] = !!formInput && !isNullOrUndefined(formInput().value());
    }

    return result;
  });

  toggleOptionalField(checked: boolean, inputDescription: InputDescription): void {
    let value = undefined;

    if (checked) {
      // `Input` consists of `any` type
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      value = defaultInput(inputDescription, { ignoreOptional: true });
    }

    this.updateFormField.emit({ key: inputDescription.key, value });
  }

  asPrimitiveInput(
    formInput: MaybeFieldTree<unknown, string>,
  ): FieldTree<string | number | boolean, string> {
    return formInput as FieldTree<string | number | boolean, string>;
  }

  asGeoJsonInput(
    formInput: MaybeFieldTree<unknown, string>,
  ): FieldTree<FeatureCollectionGeoJsonInput, string> {
    return formInput as FieldTree<FeatureCollectionGeoJsonInput, string>;
  }

  asGeoJsonPointFeature(
    formInput: MaybeFieldTree<unknown, string>,
  ): FieldTree<PointGeoJsonInput, string> {
    return formInput as FieldTree<PointGeoJsonInput, string>;
  }

  asNestedJsonInput(
    formInput: MaybeFieldTree<unknown, string>,
  ): FieldTree<Record<string, unknown>, string> {
    const form = formInput as Record<string, MaybeFieldTree<unknown, string>>;

    return form['value'] as FieldTree<Record<string, unknown>, string>;
  }

  toInputs(value: Record<string, InputDescription> | undefined): Array<InputDescription> {
    if (value === undefined) throw new Error('Value is undefined');
    return Object.values(value);
  }
}

export function enumOptions(schema: JSONSchema | undefined): string[] {
  if (!schema || typeof schema === 'boolean' || !schema.enum || !Array.isArray(schema.enum))
    return [];

  const options = [];
  for (const value of schema.enum) {
    if (typeof value === 'string') options.push(value);
  }
  return options;
}

export function integerRangeList(schema: JSONSchema | undefined): number[] {
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
