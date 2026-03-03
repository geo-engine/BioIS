import { ChangeDetectionStrategy, Component, computed, input } from '@angular/core';
import { MatGridListModule } from '@angular/material/grid-list';
import { MatMenuModule } from '@angular/material/menu';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinner } from '@angular/material/progress-spinner';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-number-indicator',
  template: `
    <mat-progress-spinner
      mode="determinate"
      [value]="scaledValue() * 100"
      [style]="colorVariable()"
    ></mat-progress-spinner>
    <p>{{ value() | number: '1.0-2' }}</p>
  `,
  styles: `
    :host {
      display: inline-block;
      position: relative;
    }

    p {
      position: absolute;
      top: calc(50% - 1.5rem); /* Adjust to vertically center the text */
      left: 50%;
      transform: translate(-50%, -50%);
      font-size: 1.5rem;
      font-weight: 500; /* Semi-bold */
    }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    CommonModule,
    MatButtonModule,
    MatCardModule,
    MatGridListModule,
    MatIconModule,
    MatMenuModule,
    MatProgressSpinner,
  ],
})
export class NumberIndicatorComponent {
  readonly value = input.required<number>();
  readonly min = input.required<number>();
  readonly max = input.required<number>();
  /**
   * An array of color breakpoints that define the color for specific value ranges.
   * Each breakpoint should specify a minimum and maximum value, as well as the corresponding color to be used for values that fall within that range.
   * Must be in ascending order based on the minimum value to ensure correct color mapping.
   */
  readonly colors = input.required<Array<ColorBreakpoint>>();
  readonly fallbackColor = input<string>('#808080'); // Fallback gray

  /** Scales the value between 0 and 100 */
  readonly scaledValue = computed(() => {
    const value = this.value();
    const min = this.min();
    const max = this.max();
    if (max === min) return 0; // Avoid division by zero, treat as zero
    const percentage = (value - min) / (max - min);
    return Math.max(0, Math.min(1, percentage)); // Clamp between 0 and 1
  });

  readonly colorVariable = computed(() => {
    const cssVariableName = `--mat-progress-spinner-active-indicator-color`;

    const value = this.value();
    const colors = this.colors();
    const fallbackColor = this.fallbackColor();

    return `${cssVariableName}: ${valueToColor(value, colors, fallbackColor)};`;
  });
}

/**
 * Maps a value to a color.
 *
 * @param value a numeric value to be mapped to a color
 * @param colors an array of color breakpoints that define the color for specific value ranges
 * @returns a string representing the color corresponding to the given value based on the provided color breakpoints.
 *          If the value does not fall within any of the specified ranges, a fallback color is returned.
 */
function valueToColor(
  value: number,
  colors: Array<ColorBreakpoint>,
  fallbackColor: string,
): string {
  for (const range of colors) {
    if (value >= range.min && value <= range.max) {
      return range.color;
    }
  }

  return fallbackColor;
}

export interface ColorBreakpoint {
  min: number;
  max: number;
  color: string;
}
