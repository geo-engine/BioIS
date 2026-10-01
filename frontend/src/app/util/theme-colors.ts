/** An RGB color, e.g., for canvas-based libraries like OpenLayers */
export type RgbColor = [red: number, green: number, blue: number];

/**
 * Resolves a CSS color (e.g., `var(--mat-sys-primary)`) to RGB in the context of an element.
 *
 * Material's system variables may contain values like `light-dark(…)` that canvas styles cannot parse,
 * so we let the browser compute the actual color of a probe element.
 *
 * @returns the color or `fallback` if it cannot be resolved, e.g., outside of a browser.
 */
export function resolveCssColor(
  context: HTMLElement,
  cssColor: string,
  fallback: RgbColor,
): RgbColor {
  const probe = context.ownerDocument.createElement('span');
  probe.style.display = 'none';
  probe.style.color = cssColor;
  context.appendChild(probe);

  const computedColor = getComputedStyle(probe).color;
  probe.remove();

  return parseRgbColor(computedColor) ?? fallback;
}

/** Parses computed colors like `rgb(88, 99, 49)` or `rgba(88, 99, 49, 0.5)` (alpha is ignored). */
export function parseRgbColor(color: string): RgbColor | undefined {
  const match = /^rgba?\(\s*([\d.]+)[\s,]+([\d.]+)[\s,]+([\d.]+)/.exec(color.trim());
  if (!match) return undefined;

  return [Number(match[1]), Number(match[2]), Number(match[3])];
}
