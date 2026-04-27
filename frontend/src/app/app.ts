import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { MatIconRegistry } from '@angular/material/icon';
import { DomSanitizer } from '@angular/platform-browser';
import { RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet],
  templateUrl: './app.html',
  styleUrl: './app.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class App {
  private readonly matIconRegistry = inject(MatIconRegistry);
  private readonly sanitizer = inject(DomSanitizer);

  protected readonly title = signal('BioIS');

  constructor() {
    this.matIconRegistry.addSvgIconInNamespace(
      'geoengine',
      'logo',
      this.sanitizer.bypassSecurityTrustResourceUrl('assets/geoengine-black.svg'),
    );
    this.matIconRegistry.addSvgIconInNamespace(
      'geoengine',
      'logo-white',
      this.sanitizer.bypassSecurityTrustResourceUrl('assets/geoengine-white.svg'),
    );
    this.matIconRegistry.addSvgIconInNamespace(
      'biois',
      'logo',
      this.sanitizer.bypassSecurityTrustResourceUrl('assets/BioIS_Black.svg'),
    );
    this.matIconRegistry.addSvgIconInNamespace(
      'biois',
      'logo-white',
      this.sanitizer.bypassSecurityTrustResourceUrl('assets/BioIS_White.svg'),
    );
  }
}
