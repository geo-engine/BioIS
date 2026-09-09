import { ChangeDetectionStrategy, Component, input, model } from '@angular/core';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { FormValueControl } from '@angular/forms/signals';

@Component({
  selector: 'app-boolean-field',
  template: `
    <mat-checkbox [checked]="value()" (change)="value.set($event.checked)">{{
      title()
    }}</mat-checkbox>
  `,
  imports: [MatCheckboxModule],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class BooleanFieldComponent implements FormValueControl<unknown> {
  readonly title = input.required<string>();
  readonly value = model<unknown>();
}
