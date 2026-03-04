import {
  afterRenderEffect,
  ChangeDetectionStrategy,
  ChangeDetectorRef,
  Component,
  inject,
  TrackByFunction,
  viewChild,
} from '@angular/core';
import { MatTableModule, MatTable } from '@angular/material/table';
import { MatPaginatorModule, MatPaginator } from '@angular/material/paginator';
import { MatSortModule, MatSort } from '@angular/material/sort';
import { currentPageAsEvent, JobsDataSource as JobsDataSource } from './jobs-datasource';
import { UserService } from '../user.service';
import { StatusCode, StatusInfo } from '@geoengine/biois';
import { ScrollingModule } from '@angular/cdk/scrolling';
import { DatePipe } from '@angular/common';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatIconModule } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';
import { RouterLink } from '@angular/router';
import { MatAnchor, MatButtonModule } from '@angular/material/button';

@Component({
  selector: 'app-results',
  templateUrl: './results.component.html',
  styleUrl: './results.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    DatePipe,
    MatButtonModule,
    MatIconModule,
    MatPaginatorModule,
    MatProgressBarModule,
    MatSortModule,
    MatTableModule,
    MatTooltipModule,
    RouterLink,
    ScrollingModule,
    MatAnchor,
  ],
})
export class ResultsComponent {
  readonly userService = inject(UserService);
  readonly paginator = viewChild.required(MatPaginator);
  readonly sort = viewChild.required(MatSort);
  readonly table = viewChild.required(MatTable);
  readonly changeDetector = inject(ChangeDetectorRef);

  readonly StatusCode = StatusCode;

  readonly dataSource = new JobsDataSource(this.userService.apiConfiguration());

  /** Columns displayed in the table. Columns IDs can be added, removed, or reordered. */
  readonly displayedColumns = ['updated', 'jobID', 'processID', 'status', 'message'];

  readonly pageSize = 20; // TODO: get from server settings

  constructor() {
    afterRenderEffect(() => {
      const table = this.table();

      if (table.dataSource) return;

      this.dataSource.paginator = this.paginator();
      table.dataSource = this.dataSource;
    });
  }

  readonly trackByFn: TrackByFunction<StatusInfo> = (_index, item) => item.jobID;

  updatedTooltip(start: string | null, end: string | null): string {
    if (!start) {
      return '';
    }
    if (!end) {
      return `Started ${start}`;
    }
    return `Started ${start}; Finished: ${end}`;
  }

  refresh(): void {
    this.paginator().page.next(currentPageAsEvent(this.paginator()));
  }
}
