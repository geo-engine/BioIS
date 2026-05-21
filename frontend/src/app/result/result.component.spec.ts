import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ResultComponent, fixDataValue, columnTypeOfField } from './result.component';
import { RouterModule } from '@angular/router';
import { InlineOrRefData, Link as LinkValue, QualifiedInputValue } from '@geoengine/biois';

describe('ResultComponent', () => {
  let component: ResultComponent;
  let fixture: ComponentFixture<ResultComponent>;

  beforeEach(async () => {
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
});
