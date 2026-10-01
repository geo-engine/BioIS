import { ComponentFixture, TestBed } from '@angular/core/testing';
import { GeoJSONFeature } from '@geoengine/biois';
import { geometryIcon, MapComponent, nameField } from './map.component';
import { mockResizeObserverClass } from '../util/mock-resize-observer';
import { GeoJsonFeatureCollection, isDrawnFeature } from '../util/geo-json';
import OlFeature from 'ol/Feature';
import View from 'ol/View';
import { Style } from 'ol/style';
import { vi } from 'vitest';
import { Point } from 'ol/geom';

describe('MapComponent', () => {
  let component: MapComponent;
  let fixture: ComponentFixture<MapComponent>;

  const collection: GeoJsonFeatureCollection = {
    type: 'FeatureCollection',
    features: [
      {
        type: 'Feature',
        geometry: { type: 'Point', coordinates: [8.77, 50.81] },
        properties: { name: 'A', type: 'office' },
      } as GeoJSONFeature,
      {
        type: 'Feature',
        geometry: {
          type: 'Polygon',
          coordinates: [
            [
              [8, 50],
              [9, 50],
              [9, 51],
              [8, 50],
            ],
          ],
        },
        properties: { name: 'B', type: 'mining' },
      } as GeoJSONFeature,
    ],
  };

  beforeEach(async () => {
    globalThis.ResizeObserver = mockResizeObserverClass([]);

    await TestBed.configureTestingModule({
      imports: [MapComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(MapComponent);
    component = fixture.componentInstance;

    fixture.componentRef.setInput('features', collection);
    fixture.componentRef.setInput('geometryTypes', ['Polygon']);
    fixture.componentRef.setInput('fields', [
      { key: 'name', title: 'Name' },
      { key: 'type', title: 'Type', enumValues: ['office', 'mining'] },
    ]);

    fixture.detectChanges();
  });

  it('creates', () => {
    expect(component).toBeTruthy();
  });

  it('lists the features with their fields', () => {
    const element = fixture.nativeElement as HTMLElement;
    const rows = element.querySelectorAll('.feature-list li');
    expect(rows.length).toBe(2);
    expect(rows[0].querySelector('input')?.value).toBe('A');
    expect(element.querySelectorAll('mat-select').length).toBe(2);
  });

  it('only offers the allowed geometry types for drawing', () => {
    const element = fixture.nativeElement as HTMLElement;
    const tools = element.querySelectorAll('mat-button-toggle');
    // select + polygon
    expect(tools.length).toBe(2);
  });

  it('sets a property of a feature', () => {
    component.setProperty(1, 'name', 'C');

    expect(component.features().features[1].properties).toEqual({ name: 'C', type: 'mining' });
    expect(component.features().features[0]).toBe(collection.features[0]);
  });

  it('labels features with their name', () => {
    const internals = component as unknown as {
      source: { getFeatures: () => OlFeature[] };
      featureStyle: (feature: OlFeature) => Style;
    };
    const labels = internals.source
      .getFeatures()
      .map((feature) => internals.featureStyle(feature).getText()?.getText());

    expect(labels.sort()).toEqual(['A', 'B']);
  });

  it('zooms to search results', () => {
    const view = (component as unknown as { map: { getView: () => View } }).map.getView();
    const fit = vi.spyOn(view, 'fit').mockImplementation(() => undefined);
    const animate = vi.spyOn(view, 'animate').mockImplementation(() => undefined);

    component.zoomToLocation({ label: 'Area', coordinate: [8.77, 50.81], extent: [8, 50, 9, 51] });
    expect(fit).toHaveBeenCalledOnce();

    component.zoomToLocation({ label: 'Address', coordinate: [8.77, 50.81] });
    expect(animate).toHaveBeenCalledWith(expect.objectContaining({ zoom: 17 }));

    // no features are added
    expect(component.features()).toBe(collection);
  });

  it('deletes a feature and updates the selection', () => {
    component.selectedIndex.set(1);
    component.deleteFeature(0);

    expect(component.features().features).toEqual([collection.features[1]]);
    expect(component.selectedIndex()).toBe(0);

    component.deleteFeature(0);
    expect(component.features().features).toEqual([]);
    expect(component.selectedIndex()).toBeUndefined();
  });
});

describe('MapComponent drawing', () => {
  it('marks drawn features', async () => {
    globalThis.ResizeObserver = mockResizeObserverClass([]);
    await TestBed.configureTestingModule({ imports: [MapComponent] }).compileComponents();
    const fixture = TestBed.createComponent(MapComponent);
    fixture.componentRef.setInput('fields', [{ key: 'name', title: 'Name' }]);
    fixture.detectChanges();
    const component = fixture.componentInstance;

    // `addFeature` is called on `drawend`
    (component as unknown as { addFeature: (feature: OlFeature) => void }).addFeature(
      new OlFeature(new Point([0, 0])),
    );

    const [feature] = component.features().features;
    expect(isDrawnFeature(feature)).toBe(true);
    expect(feature.properties).toEqual({ name: '' });
    expect(component.selectedIndex()).toBe(0);
  });
});

describe('MapComponent with a single feature', () => {
  it('replaces the point and hides the feature list', async () => {
    globalThis.ResizeObserver = mockResizeObserverClass([]);
    await TestBed.configureTestingModule({ imports: [MapComponent] }).compileComponents();
    const fixture = TestBed.createComponent(MapComponent);
    fixture.componentRef.setInput('maxFeatures', 1);
    fixture.componentRef.setInput('geometryTypes', ['Point']);
    fixture.componentRef.setInput('showFeatureList', false);
    fixture.detectChanges();
    const component = fixture.componentInstance;
    const addFeature = (component as unknown as { addFeature: (feature: OlFeature) => void })
      .addFeature;

    addFeature.call(component, new OlFeature(new Point([0, 0])));
    addFeature.call(component, new OlFeature(new Point([1_000_000, 0])));
    fixture.detectChanges();

    const { features } = component.features();
    expect(features.length).toBe(1);
    expect((features[0].geometry as { coordinates: number[] }).coordinates[0]).toBeCloseTo(
      8.983,
      3,
    );
    expect(component.selectedIndex()).toBe(0);

    const element = fixture.nativeElement as HTMLElement;
    expect(element.querySelector('.feature-list')).toBeNull();
    expect(element.querySelector('.hint')).toBeNull();

    // clicking places the point, so there are no other tools
    expect(component.activeTool()).toBe('Point');
    expect(element.querySelector('mat-button-toggle-group')).toBeNull();
  });
});

describe('nameField', () => {
  it('prefers free-text fields over class labels', () => {
    const name = { key: 'name', title: 'Name' };
    const type = { key: 'type', title: 'Type', enumValues: ['office'] };

    expect(nameField([type, name])).toBe(name);
    expect(nameField([name, type])).toBe(name);
    expect(nameField([type])).toBe(type);
    expect(nameField([])).toBeUndefined();
  });
});

describe('geometryIcon', () => {
  it('maps geometry types to icons', () => {
    expect(geometryIcon('Point')).toBe('place');
    expect(geometryIcon('MultiPolygon')).toBe('pentagon');
    expect(geometryIcon(undefined)).toBe('help_outline');
  });
});
