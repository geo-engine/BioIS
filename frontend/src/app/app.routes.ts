import { Routes } from '@angular/router';

const appRoutes: Routes = [
  {
    path: 'dashboard',
    title: 'Dashboard',
    loadComponent: () =>
      import('./dashboard/dashboard.component').then((m) => m.DashboardComponent),
  },
  {
    path: 'table',
    title: 'Table',
    loadComponent: () => import('./table/table.component').then((m) => m.TableComponent),
  },
];

export const routes: Routes = [
  {
    path: 'app',
    title: 'App',
    loadComponent: () =>
      import('./navigation/navigation.component').then((m) => m.NavigationComponent),
    children: appRoutes,
  },
  {
    path: '',
    title: 'BioIS',
    loadComponent: () =>
      import('./landing-page/landing-page.component').then((m) => m.LandingPageComponent),
  },
];
