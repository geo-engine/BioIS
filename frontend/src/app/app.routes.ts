import { Routes } from '@angular/router';
import { LogInGuard } from './log-in.guard';
import { inject } from '@angular/core';
import { UserService } from './user.service';

export const TITLE = 'BioIS';
export const LONG_TITLE = 'Biodiversity Indicator Service';

const appRoutes: Routes = [
  {
    path: 'results',
    title: 'Results',
    loadComponent: () => import('./results/results.component').then((m) => m.ResultsComponent),
  },
  {
    path: 'results/:resultId',
    title: 'Result Details',
    loadComponent: () => import('./result/result.component').then((m) => m.ResultComponent),
  },
  {
    path: 'create',
    title: 'Create new',
    loadComponent: () =>
      import('./create-new/create-new.component').then((m) => m.CreateNewComponent),
  },
  {
    path: 'create/:processId',
    title: 'Create new',
    loadComponent: () =>
      import('./create-new-auto/create-new-auto.component').then((m) => m.CreateNewAutoComponent),
  },
  {
    path: 'signout',
    title: `${TITLE} – Sign Out`,
    redirectTo: (): string => {
      const userService = inject(UserService);
      userService.logout();
      return '/';
    },
  },
  {
    path: '**',
    redirectTo: 'results',
  },
];

export const routes: Routes = [
  {
    path: '',
    title: `${TITLE} – ${LONG_TITLE}`,
    loadComponent: () =>
      import('./landing-page/landing-page.component').then((m) => m.LandingPageComponent),
  },
  {
    path: 'app/signin',
    title: `${TITLE} – Sign In`,
    loadComponent: () => import('./signin/signin.component').then((m) => m.SigninComponent),
  },
  {
    path: 'app',
    title: `${TITLE} – App`,
    children: appRoutes,
    loadComponent: () =>
      import('./navigation/navigation.component').then((m) => m.NavigationComponent),
    canActivate: [LogInGuard],
  },
  {
    path: '**',
    redirectTo: '/',
  },
];
