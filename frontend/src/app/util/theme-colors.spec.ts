import { parseRgbColor, resolveCssColor } from './theme-colors';

describe('parseRgbColor', () => {
  it('parses computed colors', () => {
    expect(parseRgbColor('rgb(88, 99, 49)')).toEqual([88, 99, 49]);
    expect(parseRgbColor('rgba(88, 99, 49, 0.5)')).toEqual([88, 99, 49]);
    expect(parseRgbColor('rgb(88 99 49)')).toEqual([88, 99, 49]);
  });

  it('rejects other formats', () => {
    expect(parseRgbColor('')).toBeUndefined();
    expect(parseRgbColor('#586331')).toBeUndefined();
    expect(parseRgbColor('var(--mat-sys-primary)')).toBeUndefined();
  });
});

describe('resolveCssColor', () => {
  it('resolves colors in the context of an element', () => {
    expect(resolveCssColor(document.body, 'rgb(1, 2, 3)', [0, 0, 0])).toEqual([1, 2, 3]);
    // the probe is removed again
    expect(document.body.querySelector('span')).toBeNull();
  });

  it('falls back for unresolvable colors', () => {
    expect(resolveCssColor(document.body, 'var(--undefined-color)', [4, 5, 6])).toEqual([4, 5, 6]);
  });
});
