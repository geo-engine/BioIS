import { ApplicationConfig, provideBrowserGlobalErrorListeners } from '@angular/core';
import { provideRouter } from '@angular/router';
import { routes } from './app.routes';
import { MAT_CARD_CONFIG } from '@angular/material/card';
import { MAT_FORM_FIELD_DEFAULT_OPTIONS } from '@angular/material/form-field';
import { PreventDefaultOnSubmitEventPlugin } from './util/prevent-default';
import { EVENT_MANAGER_PLUGINS } from '@angular/platform-browser';
import { DATE_PIPE_DEFAULT_OPTIONS } from '@angular/common';

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideRouter(routes),
    { provide: MAT_FORM_FIELD_DEFAULT_OPTIONS, useValue: { appearance: 'outline' } },
    { provide: MAT_CARD_CONFIG, useValue: { appearance: 'outlined' } },
    {
      provide: EVENT_MANAGER_PLUGINS,
      useClass: PreventDefaultOnSubmitEventPlugin,
      multi: true,
    },
    {
      provide: DATE_PIPE_DEFAULT_OPTIONS,
      useValue: {
        /* timezone: 'CET' */
        dateFormat: "dd.MM.yyyy 'at' H:mm",
      },
    },
  ],
};
