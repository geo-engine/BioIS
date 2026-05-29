import { ComponentFixture, TestBed } from '@angular/core/testing';
import {
  Column,
  ColumnType,
  columnTypeOfField,
  DataResourceTableComponent,
  tableColumnInfoFromValue,
} from './data-resource-table.component';
import { mockResizeObserverClass } from '../util/resize-signal.spec';

describe('DataResourceTableComponent', () => {
  let component: DataResourceTableComponent;
  let fixture: ComponentFixture<DataResourceTableComponent>;

  beforeEach(async () => {
    globalThis.ResizeObserver = mockResizeObserverClass([]);

    await TestBed.configureTestingModule({
      imports: [DataResourceTableComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(DataResourceTableComponent);
    component = fixture.componentInstance;
  });

  it('should compile', () => {
    fixture.componentRef.setInput('columns', []);
    fixture.componentRef.setInput('rows', []);
    fixture.detectChanges();

    expect(component).toBeTruthy();
  });

  it('maps field types to column types and detects URLs', () => {
    expect(columnTypeOfField(undefined, undefined)).toBe(ColumnType.String);
    expect(columnTypeOfField('number', 3)).toBe(ColumnType.Number);
    expect(columnTypeOfField('integer', 5)).toBe(ColumnType.Number);
    expect(columnTypeOfField('boolean', true)).toBe(ColumnType.Boolean);
    expect(columnTypeOfField('list', ['a'])).toBe(ColumnType.List);
    expect(columnTypeOfField('string', 'hello')).toBe(ColumnType.String);
    expect(columnTypeOfField('string', 'http://example.com')).toBe(ColumnType.Url);
  });

  it('builds table columns from schema fields', () => {
    const columns = tableColumnInfoFromValue(
      {
        fields: [
          { name: 'title', type: 'string', title: 'Title' },
          { name: 'reference', type: 'string', title: 'Reference' },
          { name: 'score', type: 'number', title: 'Score' },
          { name: 'active', type: 'boolean', title: 'Active' },
          { name: 'tags', type: 'list' },
        ],
        primaryKey: ['title'],
      },
      [
        {
          title: 'Habitat A',
          reference: 'https://example.com/resource/42',
          score: 12.345,
          active: true,
          tags: ['forest', 'protected'],
        },
      ],
    );

    expect(columns).toEqual([
      { name: 'Title', key: 'title', type: ColumnType.String, isPrimaryKey: true },
      { name: 'Reference', key: 'reference', type: ColumnType.Url, isPrimaryKey: false },
      { name: 'Score', key: 'score', type: ColumnType.Number, isPrimaryKey: false },
      { name: 'Active', key: 'active', type: ColumnType.Boolean, isPrimaryKey: false },
      { name: 'tags', key: 'tags', type: ColumnType.List, isPrimaryKey: false },
    ]);
  });

  it('renders typed columns for row values', async () => {
    const columns: Column[] = [
      { name: 'Title', key: 'title', type: ColumnType.String, isPrimaryKey: true },
      { name: 'Reference', key: 'reference', type: ColumnType.Url, isPrimaryKey: false },
      { name: 'Score', key: 'score', type: ColumnType.Number, isPrimaryKey: false },
      { name: 'Active', key: 'active', type: ColumnType.Boolean, isPrimaryKey: false },
      { name: 'tags', key: 'tags', type: ColumnType.List, isPrimaryKey: false },
    ];

    fixture.componentRef.setInput('columns', columns);
    fixture.componentRef.setInput('rows', [
      {
        title: 'Habitat A\nline 2',
        reference: 'https://example.com/resource/42',
        score: 12.345,
        active: true,
        tags: ['forest', 'protected'],
      },
    ]);

    fixture.detectChanges();
    await fixture.whenStable();
    fixture.detectChanges();

    const root = fixture.nativeElement as HTMLElement;

    const matTable = root.querySelector('table[mat-table], table');
    expect(matTable).not.toBeNull();
    expect(root.textContent).toContain('Title');
    expect(root.textContent).toContain('Reference');
    expect(root.textContent).toContain('Score');
    expect(root.textContent).toContain('Active');
    expect(root.textContent).toContain('Habitat A');
    expect(root.textContent).toContain('12.35');
    expect(root.textContent).toContain('TRUE');

    const urlLink = root.querySelector('a[href="https://example.com/resource/42"]');
    expect(urlLink).not.toBeNull();
    expect(urlLink?.getAttribute('target')).toBe('_blank');

    const newlineRendered = root.querySelector('tbody td br');
    expect(newlineRendered).not.toBeNull();

    const listItems = root.querySelectorAll('tbody td div li');
    expect(listItems.length).toBe(2);
    expect(root.textContent).toContain('forest');
    expect(root.textContent).toContain('protected');
  });
});
