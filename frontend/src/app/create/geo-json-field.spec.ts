import { ComponentFixture, TestBed } from '@angular/core/testing';
import { GeoJsonFormFieldComponent } from './geo-json-field.component';
import { GeoJsonInputMediaType } from '@geoengine/biois';
import { mockResizeObserverClass } from '../util/mock-resize-observer';
import { GeoJsonFeatureCollection, isDrawnFeature, markAsDrawn } from '../util/geo-json';
import { vi } from 'vitest';

describe('GeoJsonFormFieldComponent', () => {
  let component: GeoJsonFormFieldComponent;
  let fixture: ComponentFixture<GeoJsonFormFieldComponent>;

  beforeEach(async () => {
    globalThis.ResizeObserver = mockResizeObserverClass([]);

    await TestBed.configureTestingModule({
      imports: [GeoJsonFormFieldComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(GeoJsonFormFieldComponent);
    component = fixture.componentInstance;

    fixture.componentRef.setInput('title', 'Upload area');
    fixture.componentRef.setInput('geoJsonSchema', featureCollectionInputSchema());
    fixture.componentRef.setInput('errors', []);
    fixture.componentRef.setInput('value', new Error('No file selected'));

    fixture.detectChanges();
  });

  it('creates', () => {
    expect(component).toBeTruthy();
  });

  it('sets a valid GeoJSON input when a valid file is selected', async () => {
    const collection = featureCollection({ name: 'Site A' });

    await component.onFileSelected(fileSelectedEvent(collection));

    expect(component.fileName()).toBe('areas.geojson');
    expect(component.collection()).toEqual(collection);
    expect(component.value()).toEqual({
      mediaType: GeoJsonInputMediaType.ApplicationGeojson,
      value: collection,
    });
  });

  it('sets an error for files that are not a FeatureCollection', async () => {
    await component.onFileSelected(fileSelectedEvent({ type: 'Point', coordinates: [0, 0] }));

    expect(component.errorValue()?.message).toContain('not a GeoJSON FeatureCollection');
  });

  it('sets an error if there are no features', async () => {
    await component.onCollectionChange({ type: 'FeatureCollection', features: [] });

    expect(component.errorValue()?.message).toContain('at least one feature');
  });

  it('warns about missing feature properties without invalidating the input', async () => {
    // e.g., an uploaded file with other property names than the defaults
    fixture.componentRef.setInput('fields', [{ key: 'name', title: 'Name' }]);

    const collection = featureCollection({ location: 'Site A' });
    await component.onCollectionChange(collection);

    expect(component.value()).toEqual({
      mediaType: GeoJsonInputMediaType.ApplicationGeojson,
      value: collection,
    });
    expect(component.missingPropertiesWarning()).toBe('Feature 1 is missing a value for “Name”.');

    // e.g., the user selected the `location` property
    fixture.componentRef.setInput('fields', [{ key: 'location', title: 'Location' }]);

    expect(component.missingPropertiesWarning()).toBeUndefined();
  });

  it('requires the feature properties of drawn features', async () => {
    fixture.componentRef.setInput('fields', [{ key: 'name', title: 'Name' }]);

    const collection = featureCollection({ name: '' });
    markAsDrawn(collection.features[0]);
    await component.onCollectionChange(collection);

    expect(component.errorValue()?.message).toBe('Feature 1 is missing a value for “Name”.');
    expect(component.missingPropertiesWarning()).toBeUndefined();
    // the features are kept to be edited
    expect(component.collection()).toBe(collection);

    // edits keep the mark
    const edited: GeoJsonFeatureCollection = {
      ...collection,
      features: [{ ...collection.features[0], properties: { name: 'Site A' } }],
    };
    expect(isDrawnFeature(edited.features[0])).toBe(true);
    await component.onCollectionChange(edited);

    expect(component.value()).toEqual({
      mediaType: GeoJsonInputMediaType.ApplicationGeojson,
      value: edited,
    });
  });

  it('does not revalidate for equal fields', async () => {
    fixture.componentRef.setInput('fields', [{ key: 'name', title: 'Name' }]);
    fixture.detectChanges();
    await fixture.whenStable();

    const valueSetSpy = vi.spyOn(component.value, 'set');
    fixture.componentRef.setInput('fields', [{ key: 'name', title: 'Name' }]);
    fixture.detectChanges();
    await fixture.whenStable();

    expect(valueSetSpy).not.toHaveBeenCalled();
  });

  it('removes the file and its features', async () => {
    await component.onFileSelected(fileSelectedEvent(featureCollection({ name: 'Site A' })));
    await component.removeFile();

    expect(component.fileName()).toBeUndefined();
    expect(component.collection().features).toEqual([]);
    expect(component.errorValue()).toBeDefined();
  });
});

function featureCollection(properties: Record<string, unknown>): GeoJsonFeatureCollection {
  return {
    type: 'FeatureCollection',
    features: [
      {
        type: 'Feature',
        geometry: { type: 'Point', coordinates: [8.77, 50.81] },
        properties,
      },
    ],
  } as GeoJsonFeatureCollection;
}

function fileSelectedEvent(content: unknown): Event {
  const file = new File([JSON.stringify(content)], 'areas.geojson', {
    type: 'application/geo+json',
  });
  return {
    target: {
      files: createFileList(file),
    },
  } as unknown as Event;
}

function featureCollectionInputSchema(): Record<string, unknown> {
  return {
    type: 'object',
    additionalProperties: false,
    properties: {
      value: {
        type: 'object',
        additionalProperties: true,
        properties: {
          type: {
            const: 'FeatureCollection',
          },
          features: {
            type: 'array',
          },
        },
        required: ['type', 'features'],
      },
      mediaType: {
        const: GeoJsonInputMediaType.ApplicationGeojson,
      },
    },
    required: ['value', 'mediaType'],
  };
}

function createFileList(file: File): FileList {
  return [file] as unknown as FileList;
}
