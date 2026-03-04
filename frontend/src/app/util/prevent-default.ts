import { EventManagerPlugin } from '@angular/platform-browser';

/**
 * An `EventManagerPlugin` that prevents the default action of `submit` events.
 * This is necessary to prevent the page from reloading when a form is submitted.
 */
export class PreventDefaultOnSubmitEventPlugin extends EventManagerPlugin {
  supports(eventName: string): boolean {
    return eventName == 'submit';
  }

  // eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
  addEventListener(element: HTMLElement, eventName: string, handler: Function) {
    const [actualEvent] = eventName.split('.');

    const callback = (event: Event): void => {
      event.preventDefault();
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call
      handler(event);
    };

    element.addEventListener(actualEvent, callback);

    return (): void => {
      element.removeEventListener(actualEvent, callback);
    };
  }
}
