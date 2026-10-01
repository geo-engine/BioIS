export function mockResizeObserverClass(
  createdObservers: Array<{ callback: ResizeObserverCallback; disconnected: boolean }>,
): typeof ResizeObserver {
  return class MockResizeObserver {
    callback: ResizeObserverCallback;
    disconnected = false;

    constructor(cb: ResizeObserverCallback) {
      this.callback = cb;
      createdObservers.push(this);
    }

    observe(_el: Element): void {
      // jsdom does not implement ResizeObserver in the test environment, so we mock it to test our logic
    }

    unobserve(_el: Element): void {
      // not needed for the tests
    }

    disconnect(): void {
      this.disconnected = true;
    }
  };
}
