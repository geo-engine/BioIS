import { ComponentFixture, TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import { GeocodingResult, GeocodingService } from './geocoding.service';
import { LocationSearchComponent } from './location-search.component';

const result: GeocodingResult = {
  label: 'Philipps-Universität Marburg',
  detail: 'Biegenstraße 10, 35037 Marburg',
  coordinate: [8.77, 50.81],
};

describe('LocationSearchComponent', () => {
  let component: LocationSearchComponent;
  let fixture: ComponentFixture<LocationSearchComponent>;
  let search: ReturnType<typeof vi.fn>;

  beforeEach(async () => {
    search = vi.fn(() => Promise.resolve([result]));

    await TestBed.configureTestingModule({
      imports: [LocationSearchComponent],
      providers: [{ provide: GeocodingService, useValue: { search } }],
    }).compileComponents();

    fixture = TestBed.createComponent(LocationSearchComponent);
    component = fixture.componentInstance;
    fixture.componentRef.setInput('bias', [8.7, 50.8]);
    fixture.detectChanges();
  });

  it('does not search for short queries', async () => {
    component.query.set('Ma');
    fixture.detectChanges();
    await new Promise((resolve) => setTimeout(resolve, 400));

    expect(search).not.toHaveBeenCalled();
    expect(component.results.status()).toBe('idle');
  });

  it('searches after typing with a debounce', async () => {
    component.query.set('Mar');
    fixture.detectChanges();
    component.query.set('Marburg Uni');
    fixture.detectChanges();

    await vi.waitFor(() => expect(component.results.value()).toEqual([result]));

    // only the last query is sent
    expect(search).toHaveBeenCalledTimes(1);
    expect(search).toHaveBeenCalledWith('Marburg Uni', {
      bias: [8.7, 50.8],
      signal: expect.any(AbortSignal) as AbortSignal,
    });
  });

  it('emits the selected result without searching again', async () => {
    const selected = vi.fn();
    component.locationSelected.subscribe(selected);

    component.query.set('Marburg Uni');
    fixture.detectChanges();
    await vi.waitFor(() => expect(component.results.hasValue()).toBe(true));

    component.select(result);
    fixture.detectChanges();
    await new Promise((resolve) => setTimeout(resolve, 400));

    expect(selected).toHaveBeenCalledWith(result);
    expect(component.query()).toBe(result.label);
    expect(search).toHaveBeenCalledTimes(1);
  });

  it('does not submit surrounding forms on enter', () => {
    const input = (fixture.nativeElement as HTMLElement).querySelector('input')!;
    const event = new KeyboardEvent('keydown', { key: 'Enter', cancelable: true });
    input.dispatchEvent(event);

    expect(event.defaultPrevented).toBe(true);
  });
});
