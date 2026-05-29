import { signal, ElementRef, runInInjectionContext, Injector } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { describe, it, expect, beforeEach } from 'vitest';
import { fromResize } from './resize-signal';

/**
 * jsdom does not implement ResizeObserver in the test environment, so we mock it to test our logic
 */
export function mockResizeObserverClass(
  createdObservers: Array<{ callback: ResizeObserverCallback; disconnected: boolean }>,
): typeof ResizeObserver {
  return class MockResizeObserver {
    // jsdom does not implement ResizeObserver in the test environment, so we mock it to test our logic
    callback: ResizeObserverCallback;
    disconnected = false;

    constructor(cb: ResizeObserverCallback) {
      this.callback = cb;
      createdObservers.push(this);
    }
    observe(_el: Element): void {
      // no-op
    }
    disconnect(): void {
      this.disconnected = true;
    }
  } as unknown as typeof ResizeObserver;
}

describe('fromResize', () => {
  let createdObservers: Array<{ callback: ResizeObserverCallback; disconnected: boolean }> = [];

  let injectorRef: Injector;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    injectorRef = TestBed.inject(Injector);
    createdObservers = [];

    globalThis.ResizeObserver = mockResizeObserverClass(createdObservers);
  });

  it('updates dimensions for ElementRef input', async () => {
    const element = document.createElement('div');
    const targetSignal = signal<ElementRef<HTMLElement> | HTMLElement | undefined>(
      new ElementRef(element),
    );
    const dimensions = runInInjectionContext(injectorRef, () => fromResize(targetSignal));

    // ensure the target signal is set and allow the effect to run and create the observer
    expect(targetSignal()).toBeDefined();
    await Promise.resolve();
    await new Promise((r) => setTimeout(r, 0));

    expect(dimensions()).toEqual({ width: 0, height: 0 });

    // ensure observer was created
    expect(createdObservers.length).toBe(1);
    const observer = createdObservers[0];
    expect(observer.callback).toBeTruthy();

    // simulate resize by calling the stored callback
    observer.callback(
      [
        { target: element, contentRect: { width: 100, height: 200 } },
      ] as unknown as ResizeObserverEntry[],
      observer as unknown as ResizeObserver,
    );

    TestBed.tick(); // trigger change detection to update the signal

    expect(dimensions().width).toBe(100);
    expect(dimensions().height).toBe(200);
  });

  it('updates dimensions for raw HTMLElement and disconnects on cleanup', async () => {
    const el = document.createElement('div');
    const targetSig = signal<ElementRef<HTMLElement> | HTMLElement | undefined>(el);
    const dims = runInInjectionContext(injectorRef, () => fromResize(targetSig));

    // ensure the target signal is set and allow the effect to run and create the observer
    expect(targetSig()).toBeDefined();
    await Promise.resolve();
    await new Promise((r) => setTimeout(r, 0));

    expect(dims()).toEqual({ width: 0, height: 0 });

    expect(createdObservers.length).toBe(1);
    const observer = createdObservers[0];
    expect(observer.callback).toBeTruthy();

    observer.callback(
      [{ target: el, contentRect: { width: 50, height: 60 } }] as unknown as ResizeObserverEntry[],
      observer as unknown as ResizeObserver,
    );

    TestBed.tick(); // trigger change detection to update the signal

    expect(dims()).toEqual({ width: 50, height: 60 });

    // trigger cleanup by removing the target and allow cleanup to run
    targetSig.set(undefined);

    TestBed.tick(); // trigger change detection to update the signal

    const lastObs = createdObservers[createdObservers.length - 1];
    expect(lastObs.disconnected).toBe(true);
  });
});
