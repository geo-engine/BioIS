import { ElementRef, Signal, signal, effect } from '@angular/core';

export interface ElementDimensions {
  width: number;
  height: number;
}

/**
 * Converts a DOM element query signal into a reactive size signal.
 */
export function fromResize(
  targetSignal: Signal<ElementRef<HTMLElement> | HTMLElement | undefined>,
): Signal<ElementDimensions> {
  const dimensions = signal<ElementDimensions>({ width: 0, height: 0 });

  effect((onCleanup) => {
    const target = targetSignal();
    if (!target) return;

    // Handle both template signal queries (ElementRef) and raw HTML elements
    const element = target instanceof ElementRef ? target.nativeElement : target;

    const observer = new ResizeObserver((entries) => {
      if (entries.length > 0) {
        const { width, height } = entries[0].contentRect;
        dimensions.set({ width, height });
      }
    });

    observer.observe(element);

    // Automatically clean up when the element changes or the component is destroyed
    onCleanup(() => {
      observer.disconnect();
    });
  });

  return dimensions.asReadonly();
}
