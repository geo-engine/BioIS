import { Routes } from '@angular/router';
import { LogInGuard } from './log-in.guard';
import { inject } from '@angular/core';
import { UserService } from './user.service';

const appRoutes: Routes = [
  {
    path: 'results',
    title: 'Results',
    loadComponent: () => import('./results/results.component').then((m) => m.ResultsComponent),
  },
  {
    path: 'results/:resultId',
    title: 'Result Details',
    loadComponent: () => import('./result/result.component').then((m) => m.DashboardComponent),
  },
  {
    path: 'create',
    title: 'Create new',
    loadComponent: () =>
      import('./create-new/create-new.component').then((m) => m.CreateNewComponent),
  },
  {
    path: 'signout',
    title: 'BioIS – Sign Out',
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
    title: 'BioIS – Biodiversity Indicator Service',
    loadComponent: () =>
      import('./landing-page/landing-page.component').then((m) => m.LandingPageComponent),
  },
  {
    path: 'app/signin',
    title: 'BioIS – Sign In',
    loadComponent: () =>
      import('./signin.component/signin.component').then((m) => m.SigninComponent),
  },
  {
    path: 'app',
    title: 'BioIS – App',
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
