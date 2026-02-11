import { ApplicationConfig, provideBrowserGlobalErrorListeners } from '@angular/core';
import { provideRouter } from '@angular/router';

import { routes } from './app.routes';

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
  ],
};
