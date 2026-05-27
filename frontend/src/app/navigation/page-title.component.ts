import { ChangeDetectionStrategy, Component, effect, inject, input } from '@angular/core';
import { TitleService } from './title.service';

@Component({
  selector: 'app-page-title',
  template: ``,
  styles: ``,
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PageTitleComponent {
  readonly title = input.required<string>();

  protected readonly titleService = inject(TitleService);

  constructor() {
    effect(() => {
      const title = this.title().trim();
      if (!title) return;
      this.titleService.title = title;
    });
  }
}
