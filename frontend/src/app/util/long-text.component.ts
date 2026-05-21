import {
  ChangeDetectionStrategy,
  Component,
  computed,
  ElementRef,
  signal,
  viewChild,
} from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonModule } from '@angular/material/button';
import { fromResize } from './resize-signal';

@Component({
  selector: 'app-long-text',
  template: `
    <div #text class="text" [class.expanded]="isExpanded()">
      <ng-content />
    </div>
    @if (isButtonVisible()) {
      <button mat-button type="button" color="primary" (click)="toggleExpanded()">
        {{ isExpanded() ? 'Show Less' : 'Read More' }}
      </button>
    }
  `,
  styles: `
    .text {
      --line-height: 1.5em;
      line-height: var(--line-height);
      max-height: calc(5 * var(--line-height));

      overflow: hidden;
      position: relative;

      /* Smoothly animate the expansion */
      transition: max-height 0.4s ease-in-out;

      /* Adds a fading gradient over the last line when collapsed */
      &::after {
        content: '';
        position: absolute;
        bottom: 0;
        right: 0;
        width: 100%;
        height: var(--line-height);
        background: linear-gradient(to bottom, rgba(255 255 255 / 0), var(--mat-sys-surface));
        pointer-events: none;
        transition: opacity 0.3s ease;
      }

      /* When expanded, grow the height and hide the fade gradient */
      &.expanded {
        max-height: unset; /* Set to a value larger than your expected text block */
      }

      &.expanded::after {
        opacity: 0;
      }
    }
  `,
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [CommonModule, MatButtonModule],
})
export class LongTextComponent {
  readonly textContainer = viewChild.required<ElementRef<HTMLElement>>('text');
  readonly textContainerResize = fromResize(this.textContainer);

  readonly isExpanded = signal(false);
  readonly isTextOverflowing = computed(() => {
    const container = this.textContainer();
    this.textContainerResize();

    return container.nativeElement.scrollHeight > container.nativeElement.clientHeight;
  });
  readonly isButtonVisible = computed(() => this.isTextOverflowing() || this.isExpanded());

  toggleExpanded(): void {
    this.isExpanded.set(!this.isExpanded());
  }
}
