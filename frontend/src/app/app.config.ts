import { ApplicationConfig, provideBrowserGlobalErrorListeners } from '@angular/core';
import { provideRouter } from '@angular/router';
import { routes } from './app.routes';
import { MAT_CARD_CONFIG } from '@angular/material/card';
import { MAT_FORM_FIELD_DEFAULT_OPTIONS } from '@angular/material/form-field';

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideRouter(routes),
    // provideAppInitializer(() => {
    //   const iconRegistry = inject(MatIconRegistry);
    //   console.log(
    //     'Setting default font set class for Material Icons',
    //     iconRegistry.getDefaultFontSetClass(),
    //   );
    // iconRegistry.setDefaultFontSetClass('Material-Icons');
    // }),
    // { provide: MAT_ICON_DEFAULT_OPTIONS, useValue: { fontSet: 'Material Icons' } },
    { provide: MAT_FORM_FIELD_DEFAULT_OPTIONS, useValue: { appearance: 'fill' } },
    { provide: MAT_CARD_CONFIG, useValue: { appearance: 'outlined' } },
  ],
};
