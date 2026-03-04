import { bootstrapApplication } from '@angular/platform-browser';
import { appConfig } from './app/app.config';
import { App } from './app/app';

import '@fontsource/material-icons';
import '@fontsource/poppins';

bootstrapApplication(App, appConfig).catch((err) => console.error(err));
