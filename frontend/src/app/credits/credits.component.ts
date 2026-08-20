import { ChangeDetectionStrategy, Component, inject, resource, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatDatepicker, MatDatepickerModule } from '@angular/material/datepicker';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { provideNativeDateAdapter } from '@angular/material/core';
import { PageTitleComponent } from '../navigation/page-title.component';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { UserService } from '../user.service';
import { MatTableModule } from '@angular/material/table';
import { RouterLink } from '@angular/router';

const YEAR_AND_MONTH_FORMAT = {
  parse: {
    dateInput: 'MM/yyyy',
  },
  display: {
    dateInput: 'MM/yyyy',
    monthYearLabel: 'MMM yyyy',
    dateA11yLabel: 'DD',
    monthYearA11yLabel: 'MMMM yyyy',
  },
};

@Component({
  selector: 'app-credits',
  template: `
    <app-page-title title="Credits" />
    <div class="month-picker">
      <button mat-icon-button (click)="previousMonth()">
        <mat-icon>chevron_left</mat-icon>
      </button>
      <span></span>
      <span>{{ yearAndMonth().date | date: 'MMM yyyy' }}</span>
      <input matInput [matDatepicker]="datepicker" hidden />
      <mat-datepicker-toggle matIconSuffix [for]="datepicker"></mat-datepicker-toggle>
      <mat-datepicker
        #datepicker
        startView="multi-year"
        (monthSelected)="setMonthAndYear($event, datepicker)"
        panelClass="example-month-picker"
      >
      </mat-datepicker>
      <button mat-icon-button (click)="nextMonth()">
        <mat-icon>chevron_right</mat-icon>
      </button>
    </div>
    <fieldset class="credits">
      @if (credits.isLoading()) {
        <mat-spinner></mat-spinner>
      } @else if (credits.error()) {
        <p class="error">Error loading credits: {{ credits.error() }}</p>
      } @else if (credits.value(); as value) {
        <legend>Credits for {{ yearAndMonth().date | date: 'MMM yyyy' }}</legend>
        <p class="credits-used">
          Credits used:
          {{ value.creditsUsed }}
        </p>
        @if (value.details.length) {
          <p>Details:</p>
          <mat-table [dataSource]="value.details" class="mat-elevation-z8">
            <ng-container matColumnDef="jobId">
              <mat-header-cell *matHeaderCellDef>Job Id</mat-header-cell>
              <mat-cell *matCellDef="let element">
                <a [routerLink]="['/app/results', element.jobId]">{{ element.jobId }}</a>
              </mat-cell>
            </ng-container>

            <ng-container matColumnDef="creditsUsed">
              <mat-header-cell *matHeaderCellDef>Credits used</mat-header-cell>
              <mat-cell *matCellDef="let element">{{ element.creditsUsed }}</mat-cell>
            </ng-container>

            <mat-header-row *matHeaderRowDef="['jobId', 'creditsUsed']"></mat-header-row>
            <mat-row *matRowDef="let row; columns: ['jobId', 'creditsUsed']"></mat-row>
          </mat-table>
        }
      }
    </fieldset>
  `,
  styles: `
    .month-picker {
      display: flex;
      flex-direction: row;
      justify-content: center;
      align-items: center;
      gap: 1rem;
    }

    mat-spinner {
      margin: 5rem auto;
    }

    .credits {
      margin: 1rem;

      border-color: var(--mat-sys-surface-container-lowest);
      border-radius: var(--mat-sys-corner-medium);
      padding: 1rem;

      legend {
        font: var(--mat-sys-body-small);
        color: var(--mat-sys-on-surface-variant);
      }

      .error {
        color: var(--mat-sys-error);
      }

      .credits-used {
        font: var(--mat-sys-title-large);
        color: var(--mat-sys-on-surface);
      }
    }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    CommonModule,
    MatButtonModule,
    MatDatepickerModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatTableModule,
    PageTitleComponent,
    RouterLink,
  ],
  providers: [provideNativeDateAdapter(YEAR_AND_MONTH_FORMAT)],
})
export class CreditsComponent {
  readonly userService = inject(UserService);

  readonly yearAndMonth = signal(YearAndMonth.fromCurrentDate());
  readonly credits = resource({
    params: () => ({
      year: this.yearAndMonth().year,
      month: this.yearAndMonth().month,
    }),
    loader: ({ params: { year, month } }) => this.userService.credits(year, month),
  });

  setMonthAndYear(date: Date, datepicker: MatDatepicker<Date>): void {
    this.yearAndMonth.set(YearAndMonth.fromDate(date));
    datepicker.close();
  }

  nextMonth(): void {
    this.yearAndMonth.set(this.yearAndMonth().nextMonth());
  }

  previousMonth(): void {
    this.yearAndMonth.set(this.yearAndMonth().previousMonth());
  }
}

/**
 * A simple class to represent a year and month combination.
 * It provides methods to create instances from a Date object, get the next and previous months,
 * and format the year and month as a string.
 */
export class YearAndMonth {
  year: number;
  month: number;

  constructor(year: number, month: number) {
    this.year = year;
    this.month = month;
  }

  static fromDate(date: Date): YearAndMonth {
    return new YearAndMonth(date.getFullYear(), date.getMonth() + 1);
  }

  static fromCurrentDate(): YearAndMonth {
    return YearAndMonth.fromDate(new Date());
  }

  nextMonth(): YearAndMonth {
    const year = this.year + Math.trunc(this.month / 12);
    const month = 1 + (this.month % 12);
    return new YearAndMonth(year, month);
  }

  previousMonth(): YearAndMonth {
    const year = this.year - Math.trunc((13 - this.month) / 12);
    const month = 1 + ((this.month + 10) % 12);
    return new YearAndMonth(year, month);
  }

  get date(): Date {
    return new Date(this.year, this.month - 1);
  }
}
