import {
  ChangeDetectionStrategy,
  Component,
  effect,
  input,
  model,
  viewChildren,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatFormFieldModule } from '@angular/material/form-field';
import { FormValueControl, ValidationError, WithOptionalFieldTree } from '@angular/forms/signals';
import { FieldType } from './create-new-auto.component';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatInput, MatInputModule } from '@angular/material/input';

@Component({
  selector: 'app-simple-form-field',
  template: `
    <mat-form-field>
      <mat-label>{{ title() }}</mat-label>

      @switch (type()) {
        @case (FieldType.String)
        @default {
          <input matInput type="text" [value]="value()" (input)="value.set($event.target.value)" />
        }
        @case (FieldType.Integer) {
          <input
            matInput
            type="number"
            step="1"
            [value]="value()"
            (input)="value.set($event.target.valueAsNumber)"
          />
        }
        @case (FieldType.Number) {
          <input
            matInput
            type="number"
            step="any"
            [value]="value()"
            (input)="value.set($event.target.valueAsNumber)"
          />
        }
        @case (FieldType.Boolean) {
          <mat-checkbox [checked]="value()" (change)="value.set($event.checked)"
            >True/False</mat-checkbox
          >
        }
      }

      @for (error of errors(); track error) {
        <mat-error>{{ error.message }}</mat-error>
      }
    </mat-form-field>
  `,
  styles: ``,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [CommonModule, MatFormFieldModule, MatInputModule, MatCheckboxModule],
})
export class SimpleFormFieldComponent implements FormValueControl<unknown> {
  readonly title = input.required<string>();
  readonly type = input.required<FieldType>();

  readonly value = model<unknown>();
  readonly errors = input.required<readonly WithOptionalFieldTree<ValidationError>[]>();

  readonly formInputFields = viewChildren(MatInput);

  readonly FieldType = FieldType; // Expose enum to template

  constructor() {
    effect(() => {
      const errors = this.errors();
      const formFields = this.formInputFields();

      const hasErrors = errors.length > 0;

      formFields.forEach((field) => (field.errorState = hasErrors));
    });
  }
}
