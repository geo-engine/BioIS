import {
  AfterViewInit,
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
import { JobsDataSource as JobsDataSource } from './jobs-datasource';
import { UserService } from '../user.service';
import { StatusCode, StatusInfo } from '@geoengine/biois';
import { ScrollingModule } from '@angular/cdk/scrolling';
import { DatePipe } from '@angular/common';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatIconModule } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-results',
  templateUrl: './results.component.html',
  styleUrl: './results.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    DatePipe,
    MatIconModule,
    MatPaginatorModule,
    MatProgressBarModule,
    MatSortModule,
    MatTableModule,
    MatTooltipModule,
    RouterLink,
    ScrollingModule,
  ],
})
export class ResultsComponent implements AfterViewInit {
  readonly userService = inject(UserService);
  readonly paginator = viewChild.required(MatPaginator);
  readonly sort = viewChild.required(MatSort);
  readonly table = viewChild.required(MatTable);
  readonly changeDetector = inject(ChangeDetectorRef);

  readonly StatusCode = StatusCode;

  readonly dataSource = new JobsDataSource(this.userService.apiConfiguration());

  /** Columns displayed in the table. Columns IDs can be added, removed, or reordered. */
  readonly displayedColumns = ['jobID', 'processID', 'status', 'message', 'updated'];

  readonly pageSize = 20; // TODO: get from server settings

  readonly trackByFn: TrackByFunction<StatusInfo> = (_index, item) => item.jobID;

  ngAfterViewInit(): void {
    this.dataSource.paginator = this.paginator();
    this.table().dataSource = this.dataSource;

    setTimeout(() => {
      // triggers change detection to update the table with the new data source
      this.changeDetector.markForCheck();
    });
  }

  updatedTooltip(start: string | null, end: string | null): string {
    if (!start) {
      return '';
    }
    if (!end) {
      return `Started ${start}`;
    }
    return `Started ${start}; Finished: ${end}`;
  }
}
