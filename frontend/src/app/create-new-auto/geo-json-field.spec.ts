import { ComponentFixture, TestBed } from '@angular/core/testing';
import { GeoJsonFormFieldComponent } from './geo-json-field';
import { GeoJsonInputMediaType } from '@geoengine/biois';
import { vi } from 'vitest';

describe('GeoJsonFormFieldComponent', () => {
  let component: GeoJsonFormFieldComponent;
  let fixture: ComponentFixture<GeoJsonFormFieldComponent>;

  beforeEach(async () => {
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
    const valueSetSpy = vi.spyOn(component.value, 'set');

    const file = new File(
      [
        JSON.stringify({
          type: 'FeatureCollection',
          features: [],
        }),
      ],
      'areas.geojson',
      { type: 'application/geo+json' },
    );

    await component.onFileSelected({
      target: {
        files: createFileList(file),
      },
    } as unknown as Event);

    expect(component.fileName()).toBe('areas.geojson');
    expect(valueSetSpy).toHaveBeenCalledWith({
      mediaType: GeoJsonInputMediaType.ApplicationGeojson,
      value: {
        type: 'FeatureCollection',
        features: [],
      },
    });
  });
});

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
