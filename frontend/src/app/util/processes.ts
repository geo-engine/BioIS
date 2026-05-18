/**
 * Generates a human-readable name for a process based on its ID.
 *
 * TODO: Use title from process metadata once available instead of relying on hardcoded mappings.
 *
 * @param processId
 * @returns The human-readable name of the process.
 */
export function processName(id: string): string {
  return id
    .replace(/([a-z])([A-Z])/g, '$1 $2') // camelCase
    .replace(/[-_]/g, ' ') // snake_case and kebab-case
    .split(' ')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1).toLowerCase())
    .join(' ');
}
