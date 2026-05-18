import { processName } from './processes';

describe('processes', () => {
  it('converts process IDs to human-readable names', () => {
    expect(processName('ndvi')).toBe('Ndvi');
    expect(processName('habitatDistance')).toBe('Habitat Distance');
    expect(processName('biodiversity-sensitive-areas')).toBe('Biodiversity Sensitive Areas');
  });
});
