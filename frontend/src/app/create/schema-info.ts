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
  if (!schema) return true;

  if (typeof schema === 'boolean') return false; // boolean schemas don't have a concept of optionality

  // Check for nullable types
  if (Array.isArray(schema.type) && schema.type.includes('null')) return true;

  // Check for anyOf with null type
  if (schema.anyOf && Array.isArray(schema.anyOf)) {
    for (const subSchema of schema.anyOf as JSONSchema[]) {
      if (typeof subSchema === 'object' && subSchema.type === 'null') {
        return true;
      }
    }
  }

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
  if (!schema || typeof schema !== 'object') return schema;

  // Handle anyOf - find the non-null type
  if (schema.anyOf && Array.isArray(schema.anyOf)) {
    for (const item of schema.anyOf as JSONSchema[]) {
      if (typeof item === 'object' && item.type !== 'null') {
        const resolved = resolveSchemaRef(rootSchema, item);
        return getActualObjectSchema(resolved, rootSchema);
      }
    }
  }

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
