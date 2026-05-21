import { ComponentFixture, TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import { Process, ProcessesApi } from '@geoengine/biois';
import { CreateNewAutoComponent } from './create-new-auto.component';
import { inputBinding } from '@angular/core';
import { mockResizeObserverClass } from '../util/resize-signal.spec';

describe('CreateNewAutoComponent', () => {
  let component: CreateNewAutoComponent;
  let fixture: ComponentFixture<CreateNewAutoComponent>;

  beforeEach(async () => {
    globalThis.ResizeObserver = mockResizeObserverClass([]);

    // mock ProcessesApi.process early so resource loaders in the component don't perform real network fetches
    vi.spyOn(ProcessesApi.prototype, 'process').mockResolvedValue(ndviProcess());

    await TestBed.configureTestingModule({
      imports: [CreateNewAutoComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(CreateNewAutoComponent, {
      bindings: [inputBinding('processId', () => 'ndvi')],
    });
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    // fixture.componentRef.setInput('processId', 'ndvi');
    expect(component).toBeTruthy();
  });
});

function ndviProcess(): Process {
  const process = new Process();
  process.id = 'ndvi';
  process.inputs = {
    coordinate: {
      title: 'Coordinate',
      description: 'The coordinate to calculate the NDVI for',
      schema: {
        $defs: {
          // eslint-disable-next-line @typescript-eslint/naming-convention
          'GeoJSON Point': {
            $ref: 'https://geojson.org/schema/Point.json',
          },
          GeoJsonInputMediaType: {
            enum: ['application/geo+json'],
            type: 'string',
          },
        },
        properties: {
          mediaType: {
            $ref: '#/$defs/GeoJsonInputMediaType',
          },
          value: {
            $ref: '#/$defs/GeoJSON%20Point',
            examples: [
              {
                coordinates: [8.771796, 50.808453],
                type: 'Point',
              },
            ],
          },
        },
        required: ['value', 'mediaType'],
        title: 'PointGeoJsonInput',
        type: 'object',
      },
    },
    year: {
      title: 'Year',
      description: 'The year to calculate the NDVI for',
      schema: {
        $defs: {
          // eslint-disable-next-line @typescript-eslint/naming-convention
          'GeoJSON Point': {
            $ref: 'https://geojson.org/schema/Point.json',
          },
          GeoJsonInputMediaType: {
            enum: ['application/geo+json'],
            type: 'string',
          },
        },
        description: 'Year of reporting or change (e.g., 2023, 2024, etc.)',
        examples: [2020],
        format: 'uint16',
        maximum: 2100,
        minimum: 2000,
        title: 'Year',
        type: 'integer',
      },
    },
    month: {
      title: 'Month',
      description: 'The month to calculate the NDVI for',
      schema: {
        $defs: {
          // eslint-disable-next-line @typescript-eslint/naming-convention
          'GeoJSON Point': {
            $ref: 'https://geojson.org/schema/Point.json',
          },
          GeoJsonInputMediaType: {
            enum: ['application/geo+json'],
            type: 'string',
          },
        },
        examples: [1],
        format: 'uint8',
        maximum: 12,
        minimum: 1,
        title: 'Month',
        type: 'integer',
      },
    },
  };
  process.outputs = {
    ndvi: {
      title: 'NDVI',
      description: 'The calculated NDVI value',
      schema: null,
    },
    kNdvi: {
      title: 'kNDVI',
      description: 'The calculated kNDVI value',
      schema: null,
    },
  };
  return process;
}
