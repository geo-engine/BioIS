import { AfterViewInit, ChangeDetectionStrategy, Component, viewChild } from '@angular/core';
import { MatTableModule, MatTable } from '@angular/material/table';
import { MatPaginatorModule, MatPaginator } from '@angular/material/paginator';
import { MatSortModule, MatSort } from '@angular/material/sort';
import { TableDataSource } from './table-datasource';

@Component({
  selector: 'app-table',
  templateUrl: './table.component.html',
  styleUrl: './table.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [MatTableModule, MatPaginatorModule, MatSortModule],
})
export class TableComponent implements AfterViewInit {
  readonly paginator = viewChild.required(MatPaginator);
  readonly sort = viewChild.required(MatSort);
  readonly table = viewChild.required(MatTable);
  dataSource = new TableDataSource();

  /** Columns displayed in the table. Columns IDs can be added, removed, or reordered. */
  displayedColumns = ['id', 'name'];

  ngAfterViewInit(): void {
    this.dataSource.sort = this.sort();
    this.dataSource.paginator = this.paginator();
    this.table().dataSource = this.dataSource;
  }
}
