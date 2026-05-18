import { Directive, output, signal } from '@angular/core';

@Directive({
  selector: '[appDnd]',
  standalone: true,
  host: {
    // eslint-disable-next-line @typescript-eslint/naming-convention
    '[class.fileover]': 'isOver()',
    // eslint-disable-next-line @typescript-eslint/naming-convention
    '(dragover)': 'onDragOver($event)',
    // eslint-disable-next-line @typescript-eslint/naming-convention
    '(dragleave)': 'onDragLeave()',
    // eslint-disable-next-line @typescript-eslint/naming-convention
    '(drop)': 'onDrop($event)',
  },
})
export class DndDirective {
  readonly isOver = signal(false);
  readonly fileDropped = output<FileList>();

  onDragOver(event: DragEvent): void {
    event.preventDefault();
    event.stopPropagation();
    this.isOver.set(true);
  }

  onDragLeave(): void {
    this.isOver.set(false);
  }

  onDrop(event: DragEvent): void {
    event.preventDefault();
    event.stopPropagation();
    this.isOver.set(false);

    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      this.fileDropped.emit(files);
    }
  }
}
