import { GeoJSONFeature, GeoJsonInputMediaType } from '@geoengine/biois';

/**
 * A GeoJSON `FeatureCollection`.
 *
 * The API client only provides the specific collections of each process, so this is the common type.
 */
export interface GeoJsonFeatureCollection {
  type: 'FeatureCollection';
  features: GeoJSONFeature[];
  bbox?: number[];
}

/** A GeoJSON `FeatureCollection` process input. */
export interface FeatureCollectionGeoJsonInput {
  value: GeoJsonFeatureCollection;
  mediaType: GeoJsonInputMediaType;
}

/** Geometry types that users can draw on a map. Multi-geometries are displayed but not drawn. */
export type DrawableGeometryType = 'Point' | 'Polygon';

/** A feature property that users can edit, e.g., a name or a class label. */
export interface FeatureField {
  /** Name of the property in the feature's `properties` */
  key: string;
  title: string;
  description?: string;
  /** If set, the property is a class label with these options */
  enumValues?: string[];
}

export function emptyGeoJsonFeatureCollection(): GeoJsonFeatureCollection {
  return {
    type: 'FeatureCollection',
    features: [],
  };
}

export function emptyGeoJsonFeatureCollectionInput(): FeatureCollectionGeoJsonInput {
  return {
    value: emptyGeoJsonFeatureCollection(),
    mediaType: GeoJsonInputMediaType.ApplicationGeojson,
  };
}

/**
 * Marks features that users drew on the map (in contrast to uploaded ones).
 *
 * Being a symbol, it survives copies via spread (`{ ...feature }`),
 * but it is neither serialized (`JSON.stringify`) nor validated.
 */
export const DRAWN_FEATURE = Symbol('drawnFeature');

export function markAsDrawn(feature: GeoJSONFeature): GeoJSONFeature {
  return Object.assign(feature, { [DRAWN_FEATURE]: true });
}

export function isDrawnFeature(feature: GeoJSONFeature): boolean {
  return (feature as GeoJSONFeature & { [DRAWN_FEATURE]?: boolean })[DRAWN_FEATURE] === true;
}

/** Default properties for a new feature, i.e., an empty string or the first class label. */
export function defaultFeatureProperties(fields: FeatureField[]): Record<string, string> {
  return Object.fromEntries(fields.map(({ key, enumValues }) => [key, enumValues?.[0] ?? '']));
}

/**
 * Validates that all (selected) features have non-empty values for the given fields.
 *
 * @param filter - Only check features for which this returns `true`.
 * @returns an error message for the first invalid feature or `undefined` if all features are valid.
 */
export function missingFeatureProperties(
  collection: GeoJsonFeatureCollection,
  fields: FeatureField[],
  filter: (feature: GeoJSONFeature) => boolean = (): boolean => true,
): string | undefined {
  for (const [index, feature] of collection.features.entries()) {
    if (!filter(feature)) continue;
    const properties = (feature.properties ?? {}) as Record<string, unknown>;
    for (const { key, title } of fields) {
      const value = properties[key];
      if (value === undefined || value === null || value === '') {
        return `Feature ${index + 1} is missing a value for “${title}”.`;
      }
    }
  }
  return undefined;
}
