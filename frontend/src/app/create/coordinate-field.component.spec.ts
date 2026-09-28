import { signal, WritableSignal } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { form } from '@angular/forms/signals';
import { vi } from 'vitest';
import { GeoJSONPoint } from '@geoengine/biois';
import { mockResizeObserverClass } from '../util/mock-resize-observer';
import { GeoJsonFeatureCollection } from '../util/geo-json';
import { CoordinateFieldComponent, roundCoordinate } from './coordinate-field.component';

describe('CoordinateFieldComponent', () => {
  let component: CoordinateFieldComponent;
  let fixture: ComponentFixture<CoordinateFieldComponent>;
  let model: WritableSignal<GeoJSONPoint>;

  beforeEach(async () => {
    globalThis.ResizeObserver = mockResizeObserverClass([]);

    await TestBed.configureTestingModule({
      imports: [CoordinateFieldComponent],
    }).compileComponents();

    model = signal({ type: 'Point', coordinates: [8.77, 50.81] } as GeoJSONPoint);
    const pointForm = TestBed.runInInjectionContext(() => form(model));

    fixture = TestBed.createComponent(CoordinateFieldComponent);
    component = fixture.componentInstance;
    fixture.componentRef.setInput('field', pointForm);
    fixture.detectChanges();
  });

  it('shows the coordinate on the map and in the fields', () => {
    expect(component.collection().features).toEqual([
      {
        type: 'Feature',
        geometry: { type: 'Point', coordinates: [8.77, 50.81] },
        properties: {},
      },
    ]);

    const inputs = (fixture.nativeElement as HTMLElement).querySelectorAll<HTMLInputElement>(
      '.coordinates input',
    );
    expect([...inputs].map((input) => input.value)).toEqual(['8.77', '50.81']);
  });

  it('updates the map after typing', async () => {
    const initialCollection = component.collection();

    model.set({ type: 'Point', coordinates: [Number.NaN, 50.81] } as GeoJSONPoint);
    fixture.detectChanges();
    model.set({ type: 'Point', coordinates: [9, 50.81] } as GeoJSONPoint);
    fixture.detectChanges();

    // debounced
    expect(component.collection()).toBe(initialCollection);

    await vi.waitFor(
      () => expect(component.collection().features[0]?.geometry).toEqual(point([9, 50.81])),
      { timeout: 2000 },
    );
  });

  it('shows no point for incomplete coordinates', async () => {
    model.set({ type: 'Point', coordinates: [Number.NaN, 50.81] } as GeoJSONPoint);
    fixture.detectChanges();

    await vi.waitFor(() => expect(component.collection().features).toEqual([]), {
      timeout: 2000,
    });
  });

  it('sets the (rounded) coordinate from the map', () => {
    component.onFeaturesChange(pointCollection([7.123456789, 51.987654321]));

    expect(model()).toEqual({ type: 'Point', coordinates: [7.123457, 51.987654] });
    // without delay
    expect(component.collection().features[0].geometry).toEqual(point([7.123457, 51.987654]));
  });

  it('keeps the coordinate if the point was removed', () => {
    component.onFeaturesChange({ type: 'FeatureCollection', features: [] });

    expect(model().coordinates).toEqual([8.77, 50.81]);
  });
});

describe('roundCoordinate', () => {
  it('rounds to six decimals', () => {
    expect(roundCoordinate(8.7717964999)).toBe(8.771796);
    expect(roundCoordinate(-0.0000004)).toBe(-0);
  });
});

function point(coordinates: [number, number]): unknown {
  return { type: 'Point', coordinates };
}

function pointCollection(coordinates: [number, number]): GeoJsonFeatureCollection {
  return {
    type: 'FeatureCollection',
    features: [
      {
        type: 'Feature',
        geometry: { type: 'Point', coordinates },
        properties: {},
      },
    ],
  } as GeoJsonFeatureCollection;
}
