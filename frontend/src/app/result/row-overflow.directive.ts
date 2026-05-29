import {
  Directive,
  ElementRef,
  inject,
  input,
  signal,
  afterNextRender,
  DestroyRef,
} from '@angular/core';

@Directive({
  selector: '[appRowOverflow]',
  standalone: true,
  exportAs: 'rowOverflow',
})
export class RowOverflowDirective {
  readonly targetSelector = input.required<string>({ alias: 'appRowOverflow' });
  // eslint-disable-next-line @angular-eslint/no-input-rename
  readonly isExpanded = input<boolean>(false, { alias: 'appRowOverflowExpanded' });
  readonly canExpand = signal(false);

  protected readonly element = inject<ElementRef<HTMLElement>>(ElementRef);
  protected readonly destroyRef = inject(DestroyRef);

  constructor() {
    afterNextRender(() => {
      const targets = this.element.nativeElement.querySelectorAll(this.targetSelector());
      if (!targets?.length) return;

      const resizeObserver = new ResizeObserver(() => {
        if (this.isExpanded()) return;

        for (const target of targets) {
          if (target.scrollHeight <= target.clientHeight) continue;
          this.canExpand.set(true);
          return;
        }
      });

      targets.forEach((target) => resizeObserver.observe(target));

      this.destroyRef.onDestroy(() => resizeObserver.disconnect());
    });
  }
}
