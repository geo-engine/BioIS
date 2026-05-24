import { ComponentFixture, TestBed } from '@angular/core/testing';
import {
  ResultComponent,
  fixDataValue,
  columnTypeOfField,
  tableColumnInfoFromValue,
} from './result.component';
import { RouterModule } from '@angular/router';
import { InlineOrRefData, Link as LinkValue, QualifiedInputValue } from '@geoengine/biois';
import { vi } from 'vitest';

describe('ResultComponent', () => {
  let component: ResultComponent;
  let fixture: ComponentFixture<ResultComponent>;

  beforeEach(async () => {
    vi.restoreAllMocks();

    await TestBed.configureTestingModule({
      imports: [RouterModule.forRoot([])],
    }).compileComponents();
    fixture = TestBed.createComponent(ResultComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should compile', () => {
    expect(component).toBeTruthy();
  });

  it('converts plain link object to LinkValue', () => {
    const obj: InlineOrRefData = {
      href: 'https://example.com',
      rel: 'item',
      type: 'text',
    };
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const result = fixDataValue(obj);
    expect(result).toBeInstanceOf(LinkValue);
    const link = result as LinkValue;
    expect(link.href).toBe('https://example.com');
    expect(link.rel).toBe('item');
    expect(link.type).toBe('text');
  });

  it('converts qualified value object to QualifiedInputValue', () => {
    const obj: InlineOrRefData = { value: 42, mediaType: 'application/json', encoding: 'utf-8' };
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    const result = fixDataValue(obj);
    expect(result).toBeInstanceOf(QualifiedInputValue);
    const q = result as QualifiedInputValue;
    expect(q.value).toBe(42);
    expect(q.mediaType).toBe('application/json');
    expect(q.encoding).toBe('utf-8');
  });

  it('leaves primitives unchanged and is idempotent for instances', () => {
    expect(fixDataValue(123 as InlineOrRefData)).toBe(123);
    expect(fixDataValue('hello' as InlineOrRefData)).toBe('hello');

    const link = new LinkValue();
    link.href = 'a';
    link.rel = 'r';
    expect(fixDataValue(link)).toBe(link);

    const q = new QualifiedInputValue();
    q.value = 'v';
    q.mediaType = 'text/plain';
    expect(fixDataValue(q)).toBe(q);
  });

  it('maps field types to column types and detects URLs', () => {
    expect(columnTypeOfField(undefined, undefined)).toBe('string');
    expect(columnTypeOfField('number', 3)).toBe('number');
    expect(columnTypeOfField('integer', 5)).toBe('number');
    expect(columnTypeOfField('boolean', true)).toBe('boolean');
    expect(columnTypeOfField('string', 'hello')).toBe('string');
    expect(columnTypeOfField('string', 'http://example.com')).toBe('url');
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
      { name: 'Title', key: 'title', type: 'string' },
      { name: 'Reference', key: 'reference', type: 'url' },
      { name: 'Score', key: 'score', type: 'number' },
      { name: 'Active', key: 'active', type: 'boolean' },
      { name: 'tags', key: 'tags', type: 'list' },
    ]);
  });

  it('renders NDVI values with number indicator', async () => {
    const ndvi = new QualifiedInputValue();
    ndvi.mediaType = 'text/plain; spectral=ndvi';
    ndvi.value = 0.42;

    vi.spyOn(component.result, 'value').mockReturnValue({ ndvi } as Record<
      string,
      InlineOrRefData
    >);

    fixture.detectChanges();
    await fixture.whenStable();
    fixture.detectChanges();

    const root = fixture.nativeElement as HTMLElement;
    const indicator = root.querySelector('app-number-indicator');
    expect(indicator).not.toBeNull();
    expect(root.textContent).toContain('0.42');
  });

  it('renders JSON table values as a material table with typed columns', async () => {
    const table = new QualifiedInputValue();
    table.mediaType = 'application/vnd.dataresource+json';
    table.value = {
      data: [
        {
          title: 'Habitat A',
          reference: 'https://example.com/resource/42',
          score: 12.345,
          active: true,
        },
      ],
      schema: {
        fields: [
          { name: 'title', type: 'string', title: 'Title' },
          { name: 'reference', type: 'string', title: 'Reference' },
          { name: 'score', type: 'number', title: 'Score' },
          { name: 'active', type: 'boolean', title: 'Active' },
        ],
      },
    };

    vi.spyOn(component.result, 'value').mockReturnValue({ table } as Record<
      string,
      InlineOrRefData
    >);

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
    expect(root.textContent).toContain('True');

    const urlLink = root.querySelector('a[href="https://example.com/resource/42"]');
    expect(urlLink).not.toBeNull();
    expect(urlLink?.getAttribute('target')).toBe('_blank');
  });
});
