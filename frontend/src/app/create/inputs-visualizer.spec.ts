import { ComponentFixture, TestBed } from '@angular/core/testing';
import { inputBinding, signal } from '@angular/core';
import { defaultInputs, InputDescription, retrieveInputDescription } from './schema-info';
import { enumOptions, integerRangeList, InputsFormComponent } from './inputs-visualizer.component';
import { InputDescription as ApiInputDescription } from '@geoengine/biois';
import { form, MaybeFieldTree } from '@angular/forms/signals';

describe('enumOptions', () => {
  it('should handle invalid or missing schemas', () => {
    expect(enumOptions(undefined)).toEqual([]);
    expect(enumOptions(true)).toEqual([]);
    expect(enumOptions(false)).toEqual([]);
    expect(enumOptions({ type: 'string' })).toEqual([]);
    expect(enumOptions({ enum: [] })).toEqual([]);
  });

  it('should extract only string values from enum', () => {
    expect(enumOptions({ enum: ['option1', 'option2', 'option3'] })).toEqual([
      'option1',
      'option2',
      'option3',
    ]);
    expect(
      enumOptions({
        enum: ['string1', 42, 'string2', true, 'string3', null],
      }),
    ).toEqual(['string1', 'string2', 'string3']);
    expect(enumOptions({ enum: [1, 2, 3, true, false, null] })).toEqual([]);
  });

  it('should preserve string order and handle empty strings', () => {
    expect(enumOptions({ enum: ['z', 'a', 'm', 'b'] })).toEqual(['z', 'a', 'm', 'b']);
    expect(enumOptions({ enum: ['', 'non-empty', ''] })).toEqual(['', 'non-empty', '']);
  });
});

describe('integerRangeList', () => {
  it('should return empty array for invalid schemas', () => {
    expect(integerRangeList(undefined)).toEqual([]);
    expect(integerRangeList(true)).toEqual([]);
    expect(integerRangeList(false)).toEqual([]);
    expect(
      integerRangeList({
        type: 'string',
        minimum: 0,
        maximum: 5,
      }),
    ).toEqual([]);
    expect(
      integerRangeList({
        type: 'integer',
        maximum: 5,
      }),
    ).toEqual([]);
    expect(
      integerRangeList({
        type: 'integer',
        minimum: 0,
      }),
    ).toEqual([]);
    expect(
      integerRangeList({
        type: 'integer',
        minimum: 'zero',
        maximum: 5,
      }),
    ).toEqual([]);
    expect(
      integerRangeList({
        type: 'integer',
        minimum: 0,
        maximum: 'five',
      }),
    ).toEqual([]);
  });

  it('should generate correct integer ranges', () => {
    expect(
      integerRangeList({
        type: 'integer',
        minimum: 1,
        maximum: 5,
      }),
    ).toEqual([1, 2, 3, 4, 5]);
    expect(
      integerRangeList({
        type: 'integer',
        minimum: 3,
        maximum: 3,
      }),
    ).toEqual([3]);
    expect(
      integerRangeList({
        type: 'integer',
        minimum: -3,
        maximum: -1,
      }),
    ).toEqual([-3, -2, -1]);
    expect(
      integerRangeList({
        type: 'integer',
        minimum: -2,
        maximum: 2,
      }),
    ).toEqual([-2, -1, 0, 1, 2]);
    expect(
      integerRangeList({
        type: 'integer',
        minimum: 0,
        maximum: 3,
      }),
    ).toEqual([0, 1, 2, 3]);
  });

  it('should handle edge cases', () => {
    expect(
      integerRangeList({
        type: 'integer',
        minimum: 5,
        maximum: 1,
      }),
    ).toEqual([]);
    const result = integerRangeList({
      type: 'integer',
      minimum: 0,
      maximum: 100,
    });
    expect(result.length).toBe(101);
    expect(result[0]).toBe(0);
    expect(result[100]).toBe(100);
  });
});

describe('InputsFormComponent', () => {
  let component: InputsFormComponent;
  let fixture: ComponentFixture<InputsFormComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [InputsFormComponent],
    }).compileComponents();

    const inputs: InputDescription[] = [
      retrieveInputDescription('sites', testInputs.sites),
      retrieveInputDescription('locationNameField', testInputs.locationNameField),
      retrieveInputDescription('unitForArea', testInputs.unitForArea),
      retrieveInputDescription('previousYearData', testInputs.previousYearData),
      retrieveInputDescription('year', testInputs.year),
      retrieveInputDescription('siteTypeField', testInputs.siteTypeField),
    ];
    const formModel = signal<Record<string, unknown>>(defaultInputs(inputs));
    const inputForm: Record<
      string,
      MaybeFieldTree<unknown, string>
    > = TestBed.runInInjectionContext(() => form(formModel));
    const relativeJsonPointerAvailableFields = signal<Record<string, string[]>>({});

    fixture = TestBed.createComponent(InputsFormComponent, {
      bindings: [
        inputBinding('inputs', () => inputs),
        inputBinding('form', () => inputForm),
        inputBinding(
          'relativeJsonPointerAvailableFields',
          () => relativeJsonPointerAvailableFields,
        ),
      ],
    });
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });

  it('should have correct input bindings', () => {
    expect(component.inputs().length).toBe(6);
    expect(component.inputs()[0].key).toBe('sites');
    expect(component.inputs()[1].key).toBe('locationNameField');
    expect(component.inputs()[2].key).toBe('unitForArea');
    expect(component.inputs()[3].key).toBe('previousYearData');
    expect(component.inputs()[4].key).toBe('year');
    expect(component.inputs()[5].key).toBe('siteTypeField');
  });
});

const testInputs: {
  sites: ApiInputDescription;
  locationNameField: ApiInputDescription;
  unitForArea: ApiInputDescription;
  previousYearData: ApiInputDescription;
  year: ApiInputDescription;
  siteTypeField: ApiInputDescription;
} = {
  sites: {
    title: 'Sites',
    description: 'GeoJSON FeatureCollection of sites to analyze for land-use calculation.',
    schema: {
      $defs: {
        // eslint-disable-next-line @typescript-eslint/naming-convention
        'GeoJSON FeatureCollection': {
          $ref: 'https://geojson.org/schema/FeatureCollection.json',
        },
        GeoJsonInputMediaType: {
          enum: ['application/geo+json'],
          type: 'string',
        },
      },
      description: 'A `GeoJSON` `FeatureCollection` input',
      properties: {
        mediaType: {
          $ref: '#/$defs/GeoJsonInputMediaType',
        },
        value: {
          $ref: '#/$defs/GeoJSON%20FeatureCollection',
        },
      },
      required: ['value', 'mediaType'],
      title: 'FeatureCollectionGeoJsonInput',
      type: 'object',
    },
  },
  locationNameField: {
    title: 'Location Name Field',
    description:
      'Reference to the property in the input GeoJSON features that contains the location information.',
    metadata: [
      {
        title: 'GeoJSON Property Pointer',
        role: 'json-pointer-base',
        href: '#/inputs/sites/value/features/0/properties',
      },
    ],
    schema: {
      $defs: {
        // eslint-disable-next-line @typescript-eslint/naming-convention
        'GeoJSON FeatureCollection': {
          $ref: 'https://geojson.org/schema/FeatureCollection.json',
        },
        GeoJsonInputMediaType: {
          enum: ['application/geo+json'],
          type: 'string',
        },
      },
      description:
        'A property of the input data that is relevant for the process, e.g. a property field in a input `GeoJSON`.',
      format: 'relative-json-pointer',
      minLength: 1,
      title: 'RelativeJsonPointer',
      type: 'string',
    },
  },
  unitForArea: {
    title: 'Unit for Area',
    description: 'Unit for area measurement, with options for hectares (ha) or square meters (m²).',
    schema: {
      $defs: {
        // eslint-disable-next-line @typescript-eslint/naming-convention
        'GeoJSON FeatureCollection': {
          $ref: 'https://geojson.org/schema/FeatureCollection.json',
        },
        GeoJsonInputMediaType: {
          enum: ['application/geo+json'],
          type: 'string',
        },
      },
      enum: ['ha', 'm²'],
      examples: ['ha'],
      title: 'UnitForArea',
      type: 'string',
    },
  },
  previousYearData: {
    title: 'Previous Year Data (Optional)',
    description: 'GeoJSON FeatureCollection from previous reporting period for comparison.',
    schema: {
      $defs: {
        // eslint-disable-next-line @typescript-eslint/naming-convention
        'GeoJSON FeatureCollection': {
          $ref: 'https://geojson.org/schema/FeatureCollection.json',
        },
        GeoJsonInputMediaType: {
          enum: ['application/geo+json'],
          type: 'string',
        },
        JsonInput: {
          description: 'Helper struct to define complex input specifications for processes.',
          properties: {
            mediaType: {
              $ref: '#/$defs/JsonInputMediaType',
            },
            value: {
              $ref: '#/$defs/PreviousLandUseSummary',
            },
          },
          required: ['value', 'mediaType'],
          type: 'object',
        },
        JsonInputMediaType: {
          enum: ['application/json'],
          type: 'string',
        },
        PreviousLandUseSummary: {
          description:
            'If the previous year data is available, it will be used to calculate the percentage change for each land use category.',
          properties: {
            totalNatureOffSiteArea: {
              title: 'Total nature-oriented area off-site',
              description: 'Total nature-oriented area off-site',
              format: 'double',
              type: 'number',
            },
            totalNatureOnSiteArea: {
              title: 'Total nature-oriented area on-site',
              description: 'Total nature-oriented area on-site',
              format: 'double',
              type: 'number',
            },
            totalSealedArea: {
              title: 'Total sealed area',
              description: 'Total sealed area',
              format: 'double',
              type: 'number',
            },
            totalUseOfLand: {
              title: 'Total use of land',
              description: 'Total use of land',
              format: 'double',
              type: 'number',
            },
            unitForArea: {
              $ref: '#/$defs/UnitForArea',
              description: 'Unit for area values (e.g., "ha" for hectares, "m²" for square meters)',
            },
          },
          required: ['unitForArea'],
          type: 'object',
        },
        UnitForArea: {
          title: 'Unit for area values',
          enum: ['ha', 'm²'],
          examples: ['ha'],
          type: 'string',
        },
      },
      anyOf: [
        {
          $ref: '#/$defs/JsonInput',
        },
        {
          type: 'null',
        },
      ],
      title: 'Nullable_JsonInput',
    },
  },
  year: {
    title: 'Reporting Year',
    description: 'The reporting year for the land-use calculation.',
    schema: {
      $defs: {
        // eslint-disable-next-line @typescript-eslint/naming-convention
        'GeoJSON FeatureCollection': {
          $ref: 'https://geojson.org/schema/FeatureCollection.json',
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
  siteTypeField: {
    title: 'Site Type Field',
    description:
      "Reference to the property in the input GeoJSON features that indicates the site type (e.g., 'site', 'natureOnSite', 'natureOffSite').",
    metadata: [
      {
        title: 'GeoJSON Property Pointer',
        role: 'json-pointer-base',
        href: '#/inputs/sites/value/features/0/properties',
      },
    ],
    schema: {
      $defs: {
        // eslint-disable-next-line @typescript-eslint/naming-convention
        'GeoJSON FeatureCollection': {
          $ref: 'https://geojson.org/schema/FeatureCollection.json',
        },
        GeoJsonInputMediaType: {
          enum: ['application/geo+json'],
          type: 'string',
        },
      },
      description:
        'A property of the input data that is relevant for the process, e.g. a property field in a input `GeoJSON`.',
      format: 'relative-json-pointer',
      minLength: 1,
      title: 'RelativeJsonPointer',
      type: 'string',
    },
  },
} as const;
