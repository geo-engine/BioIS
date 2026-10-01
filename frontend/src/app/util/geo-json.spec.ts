import { GeoJSONFeature } from '@geoengine/biois';
import {
  defaultFeatureProperties,
  GeoJsonFeatureCollection,
  isDrawnFeature,
  markAsDrawn,
  missingFeatureProperties,
} from './geo-json';

const fields = [
  { key: 'name', title: 'Name' },
  { key: 'type', title: 'Type', enumValues: ['office', 'mining'] },
];

function collection(
  ...properties: Array<Record<string, unknown> | null>
): GeoJsonFeatureCollection {
  return {
    type: 'FeatureCollection',
    features: properties.map(
      (props) =>
        ({
          type: 'Feature',
          geometry: { type: 'Point', coordinates: [0, 0] },
          properties: props,
        }) as GeoJSONFeature,
    ),
  };
}

describe('defaultFeatureProperties', () => {
  it('uses empty strings and the first class label', () => {
    expect(defaultFeatureProperties(fields)).toEqual({ name: '', type: 'office' });
    expect(defaultFeatureProperties([])).toEqual({});
  });
});

describe('missingFeatureProperties', () => {
  it('accepts features with all properties', () => {
    expect(
      missingFeatureProperties(collection({ name: 'A', type: 'office', other: 1 }), fields),
    ).toBeUndefined();
    expect(missingFeatureProperties(collection(null), [])).toBeUndefined();
  });

  it('reports the first feature with a missing property', () => {
    expect(
      missingFeatureProperties(
        collection({ name: 'A', type: 'office' }, { name: '', type: 'office' }),
        fields,
      ),
    ).toBe('Feature 2 is missing a value for “Name”.');
    expect(missingFeatureProperties(collection({ name: 'A' }), fields)).toBe(
      'Feature 1 is missing a value for “Type”.',
    );
    expect(missingFeatureProperties(collection(null), fields)).toBe(
      'Feature 1 is missing a value for “Name”.',
    );
  });
});

describe('drawn features', () => {
  it('keeps the mark on copies but does not serialize it', () => {
    const [feature] = collection({ name: 'A' }).features;
    expect(isDrawnFeature(feature)).toBe(false);

    markAsDrawn(feature);
    const copy = { ...feature, properties: { name: 'B' } };

    expect(isDrawnFeature(copy)).toBe(true);
    expect(JSON.parse(JSON.stringify(copy))).toEqual({
      type: 'Feature',
      geometry: { type: 'Point', coordinates: [0, 0] },
      properties: { name: 'B' },
    });
  });

  it('can restrict the check to drawn features', () => {
    const features = collection({ name: '' }, { name: '' });
    markAsDrawn(features.features[1]);

    expect(missingFeatureProperties(features, [fields[0]], isDrawnFeature)).toBe(
      'Feature 2 is missing a value for “Name”.',
    );
  });
});
