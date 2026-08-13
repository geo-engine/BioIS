import { ComponentFixture, TestBed } from '@angular/core/testing';
import { By } from '@angular/platform-browser';
import { vi } from 'vitest';
import { Process, ProcessesApi } from '@geoengine/biois';
import { CreateComponent, inputsForRequest } from './create.component';
import { inputBinding } from '@angular/core';
import { mockResizeObserverClass } from '../util/resize-signal.spec';
import { FieldType } from './schema-info';

describe('CreateComponent', () => {
  let component: CreateComponent;
  let fixture: ComponentFixture<CreateComponent>;

  beforeEach(async () => {
    globalThis.ResizeObserver = mockResizeObserverClass([]);

    vi.spyOn(ProcessesApi.prototype, 'process').mockResolvedValue(ndviProcess());

    await TestBed.configureTestingModule({
      imports: [CreateComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(CreateComponent, {
      bindings: [inputBinding('processId', () => 'ndvi')],
    });
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('computes the process name from fallback', () => {
    expect(component.processName()).toBe('Ndvi');
  });

  it('parses inputs into typed descriptors', () => {
    const inputs = component.inputs();
    expect(inputs.length).toBe(3);
    expect(inputs.find((i) => i.key === 'coordinate')?.type).toBe(FieldType.Coordinate);
    expect(inputs.find((i) => i.key === 'year')?.type).toBe(FieldType.Integer);
    expect(inputs.find((i) => i.key === 'month')?.type).toBe(FieldType.IntegerWithSmallRange);
  });

  it('parses outputs', () => {
    expect(component.outputs().length).toBe(2);
  });

  it('sets default form values from constructor effects', () => {
    const inputs = component.formModel().inputs;
    expect((inputs['coordinate'] as Record<string, unknown>)['value']).toBeDefined();
    expect(inputs['year']).toBe(2020);
  });

  it('enables outputs that are not disabled by default', () => {
    expect(component.formModel().outputs['ndvi']).toBe(true);
    expect(component.formModel().outputs['kNdvi']).toBeUndefined();
  });

  it('toggleOutput adds and removes outputs', () => {
    component.toggleOutput('ndvi', true);
    expect(component.formModel().outputs['ndvi']).toBe(true);
    component.toggleOutput('ndvi', false);
    expect(component.formModel().outputs['ndvi']).toBeUndefined();
  });

  it('toggleOutput preserves other outputs', () => {
    component.toggleOutput('ndvi', true);
    component.toggleOutput('kNdvi', true);
    component.toggleOutput('ndvi', false);
    expect(component.formModel().outputs['kNdvi']).toBe(true);
  });

  it('inputsForRequest omits undefined and null values', () => {
    expect(
      inputsForRequest({
        year: 2020,
        region: null,
        referenceYearBegin: undefined,
      }),
    ).toEqual({ year: 2020 });
  });

  it('renders title and fieldsets', () => {
    fixture.detectChanges();
    const titleEl = fixture.debugElement.query(By.css('app-page-title'));
    expect(titleEl).toBeTruthy();
    const fieldsets = fixture.debugElement.queryAll(By.css('fieldset'));
    expect(fieldsets.length).toBe(2);
  });

  it('renders output checkboxes', () => {
    fixture.detectChanges();
    const checkboxes = fixture.debugElement.queryAll(By.css('mat-checkbox'));
    expect(checkboxes.length).toBe(2);
  });

  it('submit button disabled when form invalid', () => {
    component.toggleOutput('ndvi', false);
    component.toggleOutput('kNdvi', false);
    fixture.detectChanges();
    const button = fixture.debugElement.query(By.css('button[type="submit"]'));
    expect((button.nativeElement as HTMLButtonElement).disabled).toBe(true);
  });

  it('renders all default outputs as selected', () => {
    fixture.detectChanges();
    expect(component.outputs().length).toBeGreaterThan(0);
    const outputKeys = Object.keys(component.formModel().outputs);
    expect(outputKeys.length).toBeGreaterThan(0);
  });

  it('renders input number fields for coordinate and integer types', () => {
    fixture.detectChanges();
    const numberInputs = fixture.debugElement.queryAll(By.css('input[type="number"]'));
    expect(numberInputs.length).toBe(3);
  });
});

describe('CreateComponent with diverse input types', () => {
  let component: CreateComponent;
  let fixture: ComponentFixture<CreateComponent>;

  beforeEach(async () => {
    globalThis.ResizeObserver = mockResizeObserverClass([]);

    vi.spyOn(ProcessesApi.prototype, 'process').mockResolvedValue(allTypesProcess());

    await TestBed.configureTestingModule({
      imports: [CreateComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(CreateComponent, {
      bindings: [inputBinding('processId', () => 'all-types')],
    });
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('parses boolean input', () => {
    expect(component.inputs().find((i) => i.key === 'boolInput')?.type).toBe(FieldType.Boolean);
    expect(component.formModel().inputs['boolInput']).toBe(false);
  });

  it('parses string array input', () => {
    expect(component.inputs().find((i) => i.key === 'stringArrayInput')?.type).toBe(
      FieldType.StringArray,
    );
    expect(component.formModel().inputs['stringArrayInput']).toEqual(['x', 'y', 'z']);
  });

  it('parses optional string array input as null', () => {
    const input = component.inputs().find((i) => i.key === 'optionalStringArrayInput');
    expect(input?.type).toBe(FieldType.StringArray);
    expect(input?.optional).toBe(true);
    expect(component.formModel().inputs['optionalStringArrayInput']).toBeNull();
  });

  it('parses GeoJson input', () => {
    expect(component.inputs().find((i) => i.key === 'geoJsonInput')?.type).toBe(FieldType.GeoJson);
    expect(component.formModel().inputs['geoJsonInput']).toBeInstanceOf(Error);
  });

  it('parses number input', () => {
    expect(component.inputs().find((i) => i.key === 'numberInput')?.type).toBe(FieldType.Number);
    expect(component.formModel().inputs['numberInput']).toBe(0);
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
      metadata: [{ title: '', role: 'default-disabled', href: '' }],
      schema: null,
    },
  };
  return process;
}

function allTypesProcess(): Process {
  const process = new Process();
  process.id = 'all-types';
  process.inputs = {
    boolInput: {
      title: 'Boolean Input',
      schema: { type: 'boolean' },
    },
    stringArrayInput: {
      title: 'String Array Input',
      schema: {
        type: 'array',
        items: { $ref: '#/$defs/MyEnum' },
        $defs: {
          MyEnum: { type: 'string', enum: ['x', 'y', 'z'] },
        },
      },
    },
    optionalStringArrayInput: {
      title: 'Optional String Array Input',
      schema: {
        anyOf: [
          {
            type: 'array',
            items: { $ref: '#/$defs/MyEnum' },
            $defs: {
              MyEnum: { type: 'string', enum: ['x', 'y', 'z'] },
            },
          },
          { type: 'null' },
        ],
      },
    },
    geoJsonInput: {
      title: 'GeoJSON Input',
      schema: {
        type: 'object',
        title: 'FeatureCollectionGeoJsonInput',
        properties: {},
      },
    },
    numberInput: {
      title: 'Number Input',
      schema: { type: 'number' },
    },
  };
  process.outputs = {
    output1: { title: 'Output 1', schema: null },
  };
  return process;
}
