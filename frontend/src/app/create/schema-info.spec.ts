import { InputDescription as ApiInputDescription } from '@geoengine/biois';
import {
  retrieveInputDescription,
  FieldType,
  jsonSchemaToZod,
  defaultInput,
  defaultInputs,
} from './schema-info';
import { enumOptions } from './inputs-visualizer.component';

const testInputs: {
  sites: ApiInputDescription;
  locationNameField: ApiInputDescription;
  unitForArea: ApiInputDescription;
  previousYearData: ApiInputDescription;
  year: ApiInputDescription;
  yearRange: ApiInputDescription;
  referenceYearBegin: ApiInputDescription;
  siteTypeField: ApiInputDescription;
  region: ApiInputDescription;
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
  yearRange: {
    title: 'Range (years)',
    description: 'Length of the climate-risk aggregation window in years (5-30).',
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
      default: 20,
      description: 'Length of the climate-risk aggregation window in years (5-30).',
      examples: [20],
      maximum: 30,
      minimum: 5,
      title: 'YearRange',
      type: 'integer',
    },
  },
  referenceYearBegin: {
    title: 'Reference period start',
    description:
      'First year of the reference period used to compute anomalies. Uses the same range as the analysis window. Set to null to disable anomaly computation.',
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
        Year: {
          description: 'Year of reporting or change (e.g., 2023, 2024, etc.)',
          examples: [2020],
          format: 'uint16',
          maximum: 2100,
          minimum: 2000,
          title: 'Year',
          type: 'integer',
        },
      },
      anyOf: [
        {
          $ref: '#/$defs/Year',
        },
        {
          type: 'null',
        },
      ],
      default: 2006,
      title: 'Nullable_Year',
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
  region: {
    title: 'CORDEX/CMIP5 region',
    description: 'The CORDEX/CMIP5 region to use for the climate-risk aggregation.',
    schema: {
      $defs: {
        CordexRegion: {
          title: 'CordexRegion',
          type: 'string',
          enum: ['Eur'],
        },
      },
      anyOf: [
        {
          $ref: '#/$defs/CordexRegion',
        },
        {
          type: 'null',
        },
      ],
      title: 'Nullable_CordexRegion',
    },
  },
} as const;

describe('retrieveInputDescription', () => {
  it('should process GeoJSON input (sites)', () => {
    const result = retrieveInputDescription('sites', testInputs.sites);

    expect(result).toMatchObject({
      key: 'sites',
      title: 'Sites',
      type: FieldType.GeoJson,
      optional: false,
      description: 'GeoJSON FeatureCollection of sites to analyze for land-use calculation.',
    });
  });

  it('should process RelativeJsonPointer input (locationNameField)', () => {
    const result = retrieveInputDescription('locationNameField', testInputs.locationNameField);

    expect(result).toMatchObject({
      key: 'locationNameField',
      title: 'Location Name Field',
      type: FieldType.RelativeJsonPointer,
      optional: false,
    });
    expect(result.metadata).toHaveLength(1);
    expect(result.metadata?.[0].role).toBe('json-pointer-base');
  });

  it('should process StringEnum input (unitForArea)', () => {
    const result = retrieveInputDescription('unitForArea', testInputs.unitForArea);

    expect(result).toMatchObject({
      key: 'unitForArea',
      title: 'Unit for Area',
      type: FieldType.StringEnum,
      optional: false,
    });
  });

  it('should process Integer input (year)', () => {
    const result = retrieveInputDescription('year', testInputs.year);

    expect(result).toMatchObject({
      key: 'year',
      title: 'Reporting Year',
      type: FieldType.Integer,
      optional: false,
    });
  });

  it('should process IntegerWithSmallRange input (yearRange)', () => {
    const result = retrieveInputDescription('yearRange', testInputs.yearRange);

    expect(result).toMatchObject({
      key: 'yearRange',
      title: 'Range (years)',
      type: FieldType.IntegerWithSmallRange,
      optional: false,
    });
  });

  it('should process nullable Integer input with default (referenceYearBegin)', () => {
    const result = retrieveInputDescription('referenceYearBegin', testInputs.referenceYearBegin);

    expect(result).toMatchObject({
      key: 'referenceYearBegin',
      title: 'Reference period start',
      type: FieldType.Integer,
      optional: true,
    });

    expect(defaultInput(result)).toBeNull();
    expect(defaultInput(result, { ignoreOptional: true })).toBe(2006);
  });

  it('should process nullable StringEnum input (region) with a usable default', () => {
    const result = retrieveInputDescription('region', testInputs.region);

    expect(result).toMatchObject({
      key: 'region',
      title: 'CORDEX/CMIP5 region',
      type: FieldType.StringEnum,
      optional: true,
    });

    expect(defaultInput(result)).toBeNull();
    expect(defaultInput(result, { ignoreOptional: true })).toBe('Eur');
    expect(enumOptions(result.schema)).toEqual(['Eur']);
  });

  it('should process nullable input (previousYearData)', () => {
    const result = retrieveInputDescription('previousYearData', testInputs.previousYearData);

    expect(result).toMatchObject({
      key: 'previousYearData',
      title: 'Previous Year Data (Optional)',
      optional: true,
      type: FieldType.NestedJson,
      children: {
        totalNatureOffSiteArea: {
          type: FieldType.Number,
          optional: false,
        },
        totalNatureOnSiteArea: {
          type: FieldType.Number,
          optional: false,
        },
        totalSealedArea: {
          type: FieldType.Number,
          optional: false,
        },
        totalUseOfLand: {
          type: FieldType.Number,
          optional: false,
        },
        unitForArea: {
          type: FieldType.StringEnum,
          optional: false,
          title: 'Unit for area values',
          schema: { enum: ['ha', 'm²'] },
        },
      },
    });
  });
});

describe('defaultInputs', () => {
  it('enables an optional input with a default only when it opts in via metadata', () => {
    const input = retrieveInputDescription('referenceYearBegin', {
      ...testInputs.referenceYearBegin,
      metadata: [{ title: '', role: 'enabled-by-default', href: '' }],
    });
    const result = defaultInputs([input]);
    expect(result['referenceYearBegin']).toBe(2006);
  });

  it('keeps an optional input with a schema default disabled without the metadata role', () => {
    const input = retrieveInputDescription('referenceYearBegin', testInputs.referenceYearBegin);
    const result = defaultInputs([input]);
    expect(result['referenceYearBegin']).toBeNull();
  });
});

describe('jsonSchemaToZod', () => {
  it('should convert GeoJSON input schema (sites) to Zod schema', () => {
    const zodSchema = jsonSchemaToZod(retrieveInputDescription('sites', testInputs.sites).schema);

    expect(zodSchema).toBeDefined();
    expect(zodSchema).not.toBeNull();
  });

  it('should convert nested JSON input schema (previousYearData) to Zod schema', () => {
    const zodSchema = jsonSchemaToZod(
      retrieveInputDescription('previousYearData', testInputs.previousYearData).schema,
    );

    expect(zodSchema).toBeDefined();
    expect(zodSchema).not.toBeNull();
  });
});
