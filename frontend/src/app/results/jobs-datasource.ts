import { DataSource } from '@angular/cdk/collections';
import { MatPaginator } from '@angular/material/paginator';
import { Observable, from } from 'rxjs';
import { Configuration, ProcessesApi, StatusInfo } from '@geoengine/biois';

/**
 * Data source for the Table view. This class should
 * encapsulate all logic for fetching and manipulating the displayed data
 * (including sorting, pagination, and filtering).
 */
export class JobsDataSource extends DataSource<StatusInfo> {
  // data: StatusInfo[] = [];
  paginator: MatPaginator | undefined;
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

    return from(
      this.queryDataPage(
        this.paginator.pageIndex * this.paginator.pageSize,
        this.paginator.pageSize,
      ),
    );

    // Combine everything that affects the rendered data into one update
    // stream for the data-table to consume.
    // return merge(observableOf(this.data), this.paginator.page, this.sort.sortChange).pipe(
    //   map((params) => {
    //     return this.getPagedData(this.getSortedData([...this.data]));
    //   }),
    // );
  }

  /**
   *  Called when the table is being destroyed. Use this function, to clean up
   * any open connections or free any held resources that were set up during connect.
   */
  disconnect(): void {
    // no-op
  }

  protected async queryDataPage(offset: number, limit: number): Promise<StatusInfo[]> {
    const jobs = (await this.api.jobs(/* TODO: pagination */)).jobs;

    if (this.paginator && limit <= jobs.length) {
      this.paginator.length += 1;
    }

    return jobs;
  }

  // /**
  //  * Paginate the data (client-side). If you're using server-side pagination,
  //  * this would be replaced by requesting the appropriate data from the server.
  //  */
  // private getPagedData(data: StatusInfo[]): StatusInfo[] {
  //   if (this.paginator) {
  //     const startIndex = this.paginator.pageIndex * this.paginator.pageSize;
  //     return data.splice(startIndex, this.paginator.pageSize);
  //   } else {
  //     return data;
  //   }
  // }

  //   /**
  //    * Sort the data (client-side). If you're using server-side sorting,
  //    * this would be replaced by requesting the appropriate data from the server.
  //    */
  //   private getSortedData(data: StatusInfo[]): StatusInfo[] {
  //     if (!this.sort?.active || this.sort.direction === '') {
  //       return data;
  //     }

  //     return data.sort((a, b) => {
  //       const isAsc = this.sort?.direction === 'asc';
  //       switch (this.sort?.active) {
  //         case 'name':
  //           return compare(a.name, b.name, isAsc);
  //         case 'id':
  //           return compare(+a.id, +b.id, isAsc);
  //         default:
  //           return 0;
  //       }
  //     });
  //   }
}

/** Simple sort comparator for example ID/Name columns (for client-side sorting). */
// function compare(a: string | number, b: string | number, isAsc: boolean): number {
//   return (a < b ? -1 : 1) * (isAsc ? 1 : -1);
// }
