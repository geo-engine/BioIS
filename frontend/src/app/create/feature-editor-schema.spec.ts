import { InputDescription as ApiInputDescription } from '@geoengine/biois';
import {
  allowedGeometryTypes,
  editableFeatureFields,
  featurePropertySchema,
  FieldType,
  jsonSchemaToZod,
  retrieveInputDescription,
} from './schema-info';

/** The `sites` input of the land-use process as described by the backend */
function sitesSchema(geometries: string[]): Record<string, unknown> {
  return {
    $defs: {
      GeoJsonInputMediaType: { enum: ['application/geo+json'], type: 'string' },
      LandUseSiteSpecification: {
        enum: ['site', 'natureOnSite', 'natureOffSite'],
        type: 'string',
      },
    },
    description: 'A `GeoJSON` `FeatureCollection` input',
    properties: {
      mediaType: { $ref: '#/$defs/GeoJsonInputMediaType' },
      value: {
        allOf: [
          { $ref: 'https://geojson.org/schema/FeatureCollection.json' },
          {
            properties: {
              features: {
                items: {
                  properties: {
                    geometry: {
                      anyOf: geometries.map((geometry) => ({
                        $ref: `https://geojson.org/schema/${geometry}.json`,
                      })),
                    },
                    properties: {
                      description: 'Expected properties of a site feature in the input `GeoJSON`.',
                      properties: {
                        name: { description: 'Name of the site', type: 'string' },
                        type: {
                          $ref: '#/$defs/LandUseSiteSpecification',
                          description: 'Land-use type of the site',
                        },
                      },
                      type: 'object',
                    },
                  },
                  type: 'object',
                },
                type: 'array',
              },
            },
            type: 'object',
          },
        ],
      },
    },
    required: ['value', 'mediaType'],
    title: 'FeatureCollectionGeoJsonInput',
    type: 'object',
  };
}

function pointerInput(title: string, defaultValue: string): ApiInputDescription {
  return {
    title,
    description: `Reference to the property for ${title}.`,
    metadata: [
      {
        title: 'GeoJSON Property Pointer',
        role: 'json-pointer-base',
        href: '#/inputs/sites/value/features/0/properties',
      },
    ],
    schema: {
      default: defaultValue,
      format: 'relative-json-pointer',
      minLength: 1,
      title: 'RelativeJsonPointer',
      type: 'string',
    },
  };
}

describe('allowedGeometryTypes', () => {
  it('returns the drawable geometry types of the schema', () => {
    expect(allowedGeometryTypes(sitesSchema(['Polygon', 'MultiPolygon']))).toEqual(['Polygon']);
    expect(allowedGeometryTypes(sitesSchema(['Point', 'MultiPoint']))).toEqual(['Point']);
    expect(
      allowedGeometryTypes(sitesSchema(['Point', 'MultiPoint', 'Polygon', 'MultiPolygon'])),
    ).toEqual(['Point', 'Polygon']);
  });

  it('ignores geometry types that cannot be drawn', () => {
    expect(allowedGeometryTypes(sitesSchema(['LineString', 'Polygon']))).toEqual(['Polygon']);
  });

  it('allows all geometry types for unrestricted schemas', () => {
    expect(allowedGeometryTypes({})).toEqual(['Point', 'Polygon']);
    expect(
      allowedGeometryTypes({
        properties: { value: { $ref: 'https://geojson.org/schema/FeatureCollection.json' } },
      }),
    ).toEqual(['Point', 'Polygon']);
  });
});

describe('featurePropertySchema', () => {
  it('returns the schema of a feature property', () => {
    expect(featurePropertySchema(sitesSchema(['Polygon']), 'name')).toEqual({
      description: 'Name of the site',
      type: 'string',
    });
  });

  it('resolves references', () => {
    expect(featurePropertySchema(sitesSchema(['Polygon']), 'type')).toMatchObject({
      enum: ['site', 'natureOnSite', 'natureOffSite'],
      type: 'string',
    });
  });

  it('returns undefined for unknown properties', () => {
    expect(featurePropertySchema(sitesSchema(['Polygon']), 'unknown')).toBeUndefined();
    expect(featurePropertySchema({}, 'name')).toBeUndefined();
  });
});

describe('editableFeatureFields', () => {
  const inputs = [
    retrieveInputDescription('sites', { title: 'Sites', schema: sitesSchema(['Polygon']) }),
    retrieveInputDescription('locationNameField', pointerInput('Location Name Field', 'name')),
    retrieveInputDescription('siteTypeField', pointerInput('Site Type Field', 'type')),
    retrieveInputDescription('year', { title: 'Year', schema: { type: 'integer' } }),
  ];

  it('detects the input types', () => {
    expect(inputs.map(({ type }) => type)).toEqual([
      FieldType.GeoJson,
      FieldType.RelativeJsonPointer,
      FieldType.RelativeJsonPointer,
      FieldType.Integer,
    ]);
  });

  it('derives the fields from the pointer inputs', () => {
    expect(
      editableFeatureFields(inputs, {
        locationNameField: 'name',
        siteTypeField: 'type',
        year: 2024,
      }),
    ).toEqual({
      sites: [
        {
          key: 'name',
          title: 'Name',
          description: 'Name of the site',
          enumValues: undefined,
        },
        {
          key: 'type',
          title: 'Type',
          description: 'Land-use type of the site',
          enumValues: ['site', 'natureOnSite', 'natureOffSite'],
        },
      ],
    });
  });

  it('follows the selected properties', () => {
    expect(
      editableFeatureFields(inputs, { locationNameField: 'label', siteTypeField: '' }),
    ).toEqual({
      sites: [
        {
          key: 'label',
          title: 'Label',
          description: 'Reference to the property for Location Name Field.',
          enumValues: undefined,
        },
      ],
    });
  });
});

describe('jsonSchemaToZod for GeoJSON inputs', () => {
  // the external GeoJSON schemas are not resolved, so all geometry types are indistinguishable
  it('accepts features for multiple geometry types', () => {
    const schema = jsonSchemaToZod(sitesSchema(['Point', 'MultiPoint', 'Polygon', 'MultiPolygon']));

    const result = schema.safeParse({
      mediaType: 'application/geo+json',
      value: {
        type: 'FeatureCollection',
        features: [
          {
            type: 'Feature',
            geometry: { type: 'Point', coordinates: [8.77, 50.81] },
            properties: { name: 'Site A', type: 'site' },
          },
        ],
      },
    });

    expect(result.success).toBe(true);
  });
});
