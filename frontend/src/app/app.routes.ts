import { Routes } from '@angular/router';
import { LogInGuard } from './log-in.guard';

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
