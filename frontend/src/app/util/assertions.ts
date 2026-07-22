/**
 * A helper function that provides both compile-time type checking and runtime error throwing if invalid data bypasses TypeScript (e.g., untyped API responses).
 * @param x The value that should never occur
 */
export function assertNever(x: never): never {
  throw new Error(`Unexpected value: ${JSON.stringify(x)}`);
}

export function isNullOrUndefined<T>(value: T | null | undefined): boolean {
  return value === null || value === undefined;
}
