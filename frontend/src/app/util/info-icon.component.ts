import { Component, input } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';

/**
 * A button that displays an info icon and shows a tooltip with the provided description when hovered over.
 *
 * @example
 * <app-info-icon [description]="'This is an info icon'"></app-info-icon>
 */
@Component({
  selector: 'app-info-icon',
  template: `
    <mat-icon [matTooltip]="description()">info</mat-icon
    ><span class="cdk-visually-hidden">{{ description() }}</span>
  `,
  styles: [
    `
      @use '@angular/material' as mat;

      @include mat.icon-overrides(
        (
          color: var(--mat-sys-primary),
        )
      );

      mat-icon {
        cursor: help;
        vertical-align: text-bottom;
      }
    `,
  ],
  imports: [MatButtonModule, MatIconModule, MatTooltipModule],
})
export class InfoIconComponent {
  readonly description = input.required<string>();
}
