import { DataSource } from '@angular/cdk/collections';
import { MatPaginator, PageEvent } from '@angular/material/paginator';
import { defer, Observable, repeat, startWith, switchMap, tap, timer } from 'rxjs';
import { Configuration, ProcessesApi, StatusInfo } from '@geoengine/biois';
import { ChangeDetectorRef } from '@angular/core';

/**
 * Data source for the Table view. This class should
 * encapsulate all logic for fetching and manipulating the displayed data
 * (including sorting, pagination, and filtering).
 */
export class JobsDataSource extends DataSource<StatusInfo> {
  paginator?: MatPaginator;
  // sort: MatSort | undefined;

  protected readonly api: ProcessesApi;

  constructor(
    config: Configuration,
    protected readonly changeDetectorRef: ChangeDetectorRef,
  ) {
    super();
    this.api = new ProcessesApi(config);
  }

  /**
   * Connect this data source to the table. The table will only update when
   * the returned stream emits new items.
   * @returns A stream of the items to be rendered.
   */
  connect(): Observable<StatusInfo[]> {
    if (!this.paginator) {
      throw Error('Please set the paginator on the data source before connecting.');
    }

    return this.paginator.page.pipe(
      startWith(currentPageAsEvent(this.paginator)),
      switchMap((pageEvent) =>
        defer(() =>
          this.queryDataPage(pageEvent.pageIndex * pageEvent.pageSize, pageEvent.pageSize),
        ).pipe(
          repeat({
            delay: (count) => {
              const baseDelay = 5_000; // Start with a 5-second delay
              const maxDelay = 60_000; // Cap the delay at 60 seconds

              // Calculate exponential delay: 5s, 10s, 20s, 40s, 60s…
              const nextDelay = Math.min(baseDelay * Math.pow(2, count - 1), maxDelay);

              return timer(nextDelay);
            },
          }),
        ),
      ),
      tap(
        () => this.changeDetectorRef.markForCheck(), // Ensure the table updates when new data arrives
      ),
    );
  }

  /**
   *  Called when the table is being destroyed. Use this function, to clean up
   * any open connections or free any held resources that were set up during connect.
   */
  disconnect(): void {
    // no-op
  }

  protected async queryDataPage(offset: number, limit: number): Promise<StatusInfo[]> {
    const jobs = (await this.api.jobs(limit, offset)).jobs;

    if (this.paginator && limit <= jobs.length) {
      this.paginator.length = Math.max(
        this.paginator.length,
        offset + jobs.length + /* no page is shown if we won't expect another item */ 1,
      );
    }

    return jobs;
  }
}

/**
 * Converts the current page of the paginator into a PageEvent, which can be emitted to trigger a reload of the current page.
 * @param paginator
 * @returns
 */
export function currentPageAsEvent(paginator: MatPaginator): PageEvent {
  const pageEvent = new PageEvent();
  pageEvent.pageIndex = paginator.pageIndex;
  pageEvent.pageSize = paginator.pageSize;
  pageEvent.length = paginator.length;
  return pageEvent;
}
