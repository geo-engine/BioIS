import { DataSource } from '@angular/cdk/collections';
import { MatPaginator, PageEvent } from '@angular/material/paginator';
import { Observable, startWith, switchMap } from 'rxjs';
import { Configuration, ProcessesApi, StatusInfo } from '@geoengine/biois';

/**
 * Data source for the Table view. This class should
 * encapsulate all logic for fetching and manipulating the displayed data
 * (including sorting, pagination, and filtering).
 */
export class JobsDataSource extends DataSource<StatusInfo> {
  paginator?: MatPaginator;
  // sort: MatSort | undefined;

  protected readonly api: ProcessesApi;

  constructor(config: Configuration) {
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

    const initialPageEvent = new PageEvent();
    initialPageEvent.length = this.paginator.length;
    initialPageEvent.pageIndex = this.paginator.pageIndex;
    initialPageEvent.pageSize = this.paginator.pageSize;

    return this.paginator.page.pipe(
      startWith(initialPageEvent),
      switchMap((pageEvent) =>
        this.queryDataPage(pageEvent.pageIndex * pageEvent.pageSize, pageEvent.pageSize),
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
