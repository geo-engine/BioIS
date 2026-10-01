import {
  GeoJsonInputMediaType,
  GeoJSONPoint,
  Input,
  InputDescription as ApiInputDescription,
  JsonInputMediaType,
  Metadata,
  PointGeoJsonInput,
  QualifiedInputValue,
  GeoJSONPointTypeEnum,
} from '@geoengine/biois';
import { processName as fieldName } from '../util/processes';
import { BaseJSONSchema, JSONSchema } from 'ya-json-schema-types';
import * as z from 'zod';
import { convertJsonSchemaToZod } from 'zod-from-json-schema';
import { assertNever } from '../util/assertions';
import { DrawableGeometryType, FeatureField } from '../util/geo-json';

export interface InputDescription {
  key: string;
  title: string;
  description?: string;
  type: FieldType;
  optional: boolean;
  metadata?: Metadata[];
  schema: JSONSchema;
  children?: Record<string, InputDescription>;
}

export enum FieldType {
  Boolean = 'boolean',
  Coordinate = 'coordinate',
  GeoJson = 'geoJson',
  Integer = 'integer',
  IntegerWithSmallRange = 'integerWithSmallRange',
  Number = 'number',
  RelativeJsonPointer = 'relativeJsonPointer',
  String = 'string',
  StringEnum = 'stringEnum',
  NestedJson = 'nestedJson',
}

export function retrieveInputDescription(
  key: string,
  processInput: ApiInputDescription,
): InputDescription {
  const inputDescription: InputDescription = {
    key,
    title: processInput.title ?? fieldName(key),
    description: processInput.description,
    type: typeFromSchema(processInput.schema as JSONSchema),
    optional: isOptional(processInput.schema as JSONSchema),
    metadata: processInput.metadata,
    schema: processInput.schema as Record<string, unknown>,
  };

  if (inputDescription.type === FieldType.NestedJson) {
    const actualObjectSchema = getActualObjectSchema(
      inputDescription.schema,
      inputDescription.schema,
    );
    const children: Record<string, InputDescription> = {};
    for (const childKey of retrieveSubSchemaKeys(actualObjectSchema)) {
      const childSchema = retrieveSubSchema(
        actualObjectSchema,
        childKey,
        inputDescription.schema,
      ) as BaseJSONSchema;
      children[childKey] = {
        key: childKey,
        title: childSchema.title ?? fieldName(childKey),
        description: childSchema.description,
        type: typeFromSchema(childSchema),
        optional: isOptional(childSchema),
        metadata: [],
        schema: childSchema,
      };
    }
    inputDescription.children = children;
  }

  return inputDescription;
}

/**
 * Determine the field type from the JSON schema.
 * This is a simplified version and may need to be expanded to handle more complex schemas (e.g., arrays, nested objects, etc.).
 */
function typeFromSchema(schema: JSONSchema | undefined): FieldType {
  if (!schema) return FieldType.String;

  // JSON Schema may be a boolean (true/false) or an object. If it's a boolean,
  // it doesn't have a `type` property, so handle that case first.
  if (typeof schema === 'boolean') return FieldType.String;

  // Handle array types like ["number", "null"] - extract the non-null type
  let type = schema.type;
  if (Array.isArray(type)) {
    type = type.find((t) => t !== 'null');
  }

  if (type === 'string') {
    if (schema.format === 'relative-json-pointer') return FieldType.RelativeJsonPointer;
    if (schema.enum) return FieldType.StringEnum;

    return FieldType.String;
  }
  if (type === 'number') return FieldType.Number;
  if (type === 'integer') {
    if (
      typeof schema.maximum === 'number' &&
      typeof schema.minimum === 'number' &&
      schema.maximum - schema.minimum <= 12
    ) {
      return FieldType.IntegerWithSmallRange;
    }
    return FieldType.Integer;
  }
  if (type === 'boolean') return FieldType.Boolean;

  if (type === 'object') {
    if (schema.title === 'PointGeoJsonInput') return FieldType.Coordinate;
    if (schema.title === 'FeatureCollectionGeoJsonInput') return FieldType.GeoJson;
  }

  // nested types (for now)
  if (!type) {
    return FieldType.NestedJson;
  }

  return FieldType.String; // fallback to string if type cannot be determined
}

function isOptional(schema: JSONSchema | undefined): boolean {
  function anySubSchemaIsNull(subSchemas: JSONSchema[] | undefined): boolean {
    if (!subSchemas) return false;
    for (const subSchema of subSchemas) {
      if (typeof subSchema === 'object' && subSchema.type === 'null') return true;
    }
    return false;
  }

  if (!schema) return true;

  if (typeof schema === 'boolean') return false; // boolean schemas don't have a concept of optionality

  // Check for nullable types
  if (Array.isArray(schema.type) && schema.type.includes('null')) return true;

  // Check for sub-schemas with null type
  if (anySubSchemaIsNull(schema.anyOf as JSONSchema[] | undefined)) return true;
  if (anySubSchemaIsNull(schema.oneOf as JSONSchema[] | undefined)) return true;

  return false;
}

function retrieveSubSchemaKeys(schema: JSONSchema | undefined): string[] {
  if (!schema || typeof schema !== 'object') return [];

  const properties = schema['properties'];
  if (!properties || typeof properties !== 'object') return [];

  return Object.keys(properties);
}

function retrieveSubSchema(
  schema: JSONSchema | undefined,
  key: string,
  rootSchema?: JSONSchema,
): JSONSchema {
  if (!schema || typeof schema !== 'object') return {};

  const properties = schema['properties'];
  if (!properties || typeof properties !== 'object') return {};

  const propSchema = (properties as Record<string, JSONSchema>)[key];

  // Resolve $ref in property
  if (
    propSchema &&
    typeof propSchema === 'object' &&
    (propSchema as Record<string, unknown>)['$ref'] &&
    typeof (propSchema as Record<string, unknown>)['$ref'] === 'string' &&
    rootSchema
  ) {
    return resolveSchemaRef(rootSchema, propSchema);
  }

  return propSchema;
}

function getActualObjectSchema(schema: JSONSchema, rootSchema: JSONSchema): JSONSchema {
  function subSchemaType(subSchemas: JSONSchema[] | undefined): JSONSchema | undefined {
    if (!subSchemas) return undefined;
    for (const subSchema of subSchemas) {
      if (typeof subSchema !== 'object' || subSchema.type === 'null') continue;
      const resolved = resolveSchemaRef(rootSchema, subSchema);
      return getActualObjectSchema(resolved, rootSchema);
    }
    return undefined;
  }

  if (!schema || typeof schema !== 'object') return schema;

  // Handle sub schemas (anyOf, oneOf) - find the non-null type
  let subSchema = subSchemaType(schema.anyOf as JSONSchema[] | undefined);
  if (subSchema) return subSchema;
  subSchema = subSchemaType(schema.oneOf as JSONSchema[] | undefined);
  if (subSchema) return subSchema;

  // Handle $ref at top level
  if (schema.$ref && typeof schema.$ref === 'string') {
    const resolved = resolveSchemaRef(rootSchema, schema);
    return getActualObjectSchema(resolved, rootSchema);
  }

  // For wrapped objects that have a 'value' property pointing to the actual data, follow that
  const properties = schema.properties;
  if (properties && typeof properties === 'object') {
    const valueSchema = (properties as Record<string, unknown>)['value'];
    if (
      valueSchema &&
      typeof valueSchema === 'object' &&
      (((valueSchema as Record<string, unknown>)['$ref'] &&
        typeof (valueSchema as Record<string, unknown>)['$ref'] === 'string') ||
        (valueSchema as Record<string, unknown>)['type'] === 'object')
    ) {
      const resolved = resolveSchemaRef(rootSchema, valueSchema as Record<string, unknown>);
      return getActualObjectSchema(resolved, rootSchema);
    }
  }

  return schema;
}

function resolveSchemaRef(rootSchema: JSONSchema, schema?: JSONSchema): JSONSchema {
  const workSchema = schema ?? rootSchema;

  if (!workSchema || typeof workSchema !== 'object') return workSchema;
  if (!rootSchema || typeof rootSchema !== 'object') return workSchema;

  // Handle $ref
  if (workSchema.$ref && typeof workSchema.$ref === 'string') {
    const ref = workSchema.$ref;
    if (ref.startsWith('#/')) {
      const parts = ref.substring(2).split('/');
      let current: BaseJSONSchema = rootSchema;

      for (const part of parts) {
        if (current && typeof current === 'object') {
          current = current[part] as BaseJSONSchema;
        } else {
          return workSchema;
        }
      }

      if (current && typeof current === 'object') {
        // Only add $defs from root for further resolution, don't merge other properties
        const resolved = current;
        const $defs = rootSchema.$defs;
        if ($defs && typeof $defs === 'object') {
          const resolvedCopy = Object.assign({}, resolved);
          resolvedCopy.$defs = $defs;
          return resolvedCopy;
        }
        return resolved;
      }
    }
  }

  return workSchema;
}

export function jsonSchemaToZod(jsonSchema: JSONSchema): z.ZodTypeAny {
  const errors = [];

  try {
    return z.fromJSONSchema(jsonSchema as Record<string, unknown>);
  } catch (error) {
    errors.push(error);
  }

  try {
    return convertJsonSchemaToZod(jsonSchema as Record<string, unknown>);
  } catch (error) {
    errors.push(error);
  }

  throw new Error('Failed to convert JSON Schema to Zod schema.', { cause: errors });
}

export function defaultInputs(inputDescriptions: Array<InputDescription>): Record<string, Input> {
  const inputs: Record<string, Input> = {};
  for (const input of inputDescriptions) {
    // `Input` consists of `any` type
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    inputs[input.key] = defaultInput(input);
  }
  return inputs;
}

export function defaultInput(
  { type, schema, children, optional }: InputDescription,
  { ignoreOptional }: { ignoreOptional?: boolean } = { ignoreOptional: false },
): Input {
  if (optional && !ignoreOptional) return null; // validator does not accept `undefined`

  switch (type) {
    case FieldType.Number:
    case FieldType.Integer:
    case FieldType.IntegerWithSmallRange:
      return defaultNumber(schema, 0);
    case FieldType.Boolean:
      return false;
    case FieldType.Coordinate:
      return {
        value: defaultCoordinate(schema),
        mediaType: GeoJsonInputMediaType.ApplicationGeojson,
      } as PointGeoJsonInput;
    case FieldType.GeoJson:
      return new Error('Missing GeoJSON input.'); // Placeholder value to indicate that the user needs to upload a file
    case FieldType.String:
    case FieldType.RelativeJsonPointer:
    case FieldType.StringEnum:
      return defaultString(schema, '');
    case FieldType.NestedJson:
      return {
        value: defaultInputs(Object.values(children ?? {})),
        mediaType: JsonInputMediaType.ApplicationJson,
      } as QualifiedInputValue;
    default:
      assertNever(type);
  }
}

function defaultNumber(schema: JSONSchema, fallback: number = 0): number {
  if (!schema || typeof schema === 'boolean') return fallback;

  const defaultValue = schema.default;
  if (typeof defaultValue === 'number') return defaultValue;

  if (!schema.examples || !Array.isArray(schema.examples)) return fallback;

  for (const example of schema.examples ?? []) {
    if (typeof example === 'number') return example;
  }

  return fallback;
}

function defaultString(schema: JSONSchema, fallback: string = ''): string {
  if (!schema || typeof schema === 'boolean') return fallback;

  const defaultValue = schema.default;
  if (typeof defaultValue === 'string') return defaultValue;

  if (!schema.examples || !Array.isArray(schema.examples)) return fallback;

  for (const example of schema.examples ?? []) {
    if (typeof example === 'string') return example;
  }

  return fallback;
}

function defaultCoordinate(schema: JSONSchema, fallback: [number, number] = [0, 0]): GeoJSONPoint {
  if (!schema || typeof schema === 'boolean') return geoJsonPointFeature(fallback);

  if (
    !schema.properties ||
    !(typeof schema.properties == 'object') ||
    !('value' in schema.properties)
  )
    return geoJsonPointFeature(fallback);

  const coordinateValue = schema.properties.value as JSONSchema;
  if (!coordinateValue || typeof coordinateValue === 'boolean')
    return geoJsonPointFeature(fallback);

  if (coordinateValue.default) {
    return coordinateValue.default as unknown as GeoJSONPoint;
  }

  if (!coordinateValue.examples || !Array.isArray(coordinateValue.examples))
    return geoJsonPointFeature(fallback);

  for (const example of coordinateValue.examples ?? []) {
    return example as GeoJSONPoint;
  }

  return geoJsonPointFeature(fallback);
}

function geoJsonPointFeature(coordinates: [number, number]): GeoJSONPoint {
  const point = new GeoJSONPoint();
  point.type = GeoJSONPointTypeEnum.Point;
  point.coordinates = coordinates;
  return point;
}

export function enumOptions(schema: JSONSchema | undefined): string[] {
  if (!schema || typeof schema === 'boolean' || !schema.enum || !Array.isArray(schema.enum))
    return [];

  const options = [];
  for (const value of schema.enum) {
    if (typeof value === 'string') options.push(value);
  }
  return options;
}

const DRAWABLE_GEOMETRY_TYPES: readonly DrawableGeometryType[] = ['Point', 'Polygon'];

/** Maps the GeoJSON schemas (https://geojson.org/schema/…) to the geometry type that can be drawn. */
function drawableGeometryType(schemaUrl: string): DrawableGeometryType | undefined {
  switch (schemaUrl.split('/').pop()) {
    case 'Point.json':
    case 'MultiPoint.json':
      return 'Point';
    case 'Polygon.json':
    case 'MultiPolygon.json':
      return 'Polygon';
    default:
      return undefined;
  }
}

/**
 * Retrieves the schemas of the feature members (`geometry`, `properties`, …)
 * of a GeoJSON FeatureCollection input, i.e., `value.features.items.properties`.
 */
function featureMemberSchemas(inputSchema: JSONSchema): Record<string, JSONSchema> | undefined {
  if (!inputSchema || typeof inputSchema !== 'object') return undefined;

  const valueSchema = retrieveSubSchema(inputSchema, 'value', inputSchema);
  if (!valueSchema || typeof valueSchema !== 'object') return undefined;

  const candidates = [valueSchema, ...((valueSchema.allOf as JSONSchema[] | undefined) ?? [])];
  for (const candidate of candidates) {
    const featuresSchema = retrieveSubSchema(candidate, 'features', inputSchema);
    if (!featuresSchema || typeof featuresSchema !== 'object') continue;

    const items = featuresSchema.items;
    if (!items || typeof items !== 'object' || Array.isArray(items)) continue;

    const members = (items as BaseJSONSchema)['properties'];
    if (members && typeof members === 'object') return members as Record<string, JSONSchema>;
  }

  return undefined;
}

/**
 * Determines which geometry types can be drawn for a GeoJSON FeatureCollection input.
 * If the schema does not restrict the geometry, all drawable geometry types are allowed.
 */
export function allowedGeometryTypes(inputSchema: JSONSchema): DrawableGeometryType[] {
  const geometrySchema = featureMemberSchemas(inputSchema)?.['geometry'];
  if (!geometrySchema || typeof geometrySchema !== 'object') return [...DRAWABLE_GEOMETRY_TYPES];

  const subSchemas = geometrySchema.oneOf ?? geometrySchema.anyOf;
  if (!Array.isArray(subSchemas)) return [...DRAWABLE_GEOMETRY_TYPES];

  const types = new Set<DrawableGeometryType>();
  for (const subSchema of subSchemas as JSONSchema[]) {
    if (!subSchema || typeof subSchema !== 'object' || typeof subSchema.$ref !== 'string') continue;

    const type = drawableGeometryType(subSchema.$ref);
    if (type) types.add(type);
  }

  return DRAWABLE_GEOMETRY_TYPES.filter((type) => types.has(type));
}

/**
 * Retrieves the schema of a feature property of a GeoJSON FeatureCollection input,
 * e.g., to get the class labels of a `type` property.
 */
export function featurePropertySchema(
  inputSchema: JSONSchema,
  propertyKey: string,
): JSONSchema | undefined {
  const propertiesSchema = featureMemberSchemas(inputSchema)?.['properties'];
  if (!propertiesSchema || typeof propertiesSchema !== 'object') return undefined;

  const resolvedPropertiesSchema = resolveSchemaRef(inputSchema, propertiesSchema);
  if (!resolvedPropertiesSchema || typeof resolvedPropertiesSchema !== 'object') return undefined;

  const propertySchema = (
    resolvedPropertiesSchema.properties as Record<string, JSONSchema> | undefined
  )?.[propertyKey];
  if (!propertySchema || typeof propertySchema !== 'object') return undefined;

  // keep keywords next to a `$ref`, e.g., the `description`
  const { $ref: _, ...siblings } = propertySchema;
  const resolved = resolveSchemaRef(inputSchema, propertySchema);
  return typeof resolved === 'object' ? { ...resolved, ...siblings } : resolved;
}

/**
 * Determines the editable feature properties for each GeoJSON FeatureCollection input.
 *
 * The properties are given by the `RelativeJsonPointer` inputs that point into the features of a GeoJSON input
 * (cf. `json-pointer-base` metadata), with their current value as the property name.
 *
 * @param inputs - The input descriptions of the process.
 * @param formInputs - The current input values of the form.
 * @returns The editable fields per GeoJSON input key.
 */
export function editableFeatureFields(
  inputs: InputDescription[],
  formInputs: Record<string, unknown>,
): Record<string, FeatureField[]> {
  const result: Record<string, FeatureField[]> = {};

  for (const geoJsonInput of inputs) {
    if (geoJsonInput.type !== FieldType.GeoJson) continue;

    const pointerBasePrefix = `#/inputs/${geoJsonInput.key}/`;
    const fields: FeatureField[] = [];

    for (const pointerInput of inputs) {
      if (pointerInput.type !== FieldType.RelativeJsonPointer) continue;

      const pointerBase = pointerInput.metadata?.find(
        (meta) => meta.role === 'json-pointer-base',
      )?.href;
      if (!pointerBase?.startsWith(pointerBasePrefix)) continue;

      const key = formInputs[pointerInput.key];
      if (typeof key !== 'string' || key === '') continue;

      const propertySchema = featurePropertySchema(geoJsonInput.schema, key);
      const baseSchema = typeof propertySchema === 'object' ? propertySchema : undefined;
      const enumValues = enumOptions(propertySchema);

      fields.push({
        key,
        title: baseSchema?.title ?? fieldName(key),
        description: baseSchema?.description ?? pointerInput.description,
        enumValues: enumValues.length > 0 ? enumValues : undefined,
      });
    }

    result[geoJsonInput.key] = fields;
  }

  return result;
}
