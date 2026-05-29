import { Component } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { By } from '@angular/platform-browser';
import { DndDirective } from './drag-and-drop.directive';

@Component({
  template: `<div appDnd></div>`,
  imports: [DndDirective],
})
class TestHostComponent {}

describe('DndDirective', () => {
  let fixture: ComponentFixture<TestHostComponent>;
  let directive: DndDirective;
  let hostElement: HTMLDivElement;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [TestHostComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(TestHostComponent);
    directive = fixture.debugElement.query(By.directive(DndDirective)).injector.get(DndDirective);
    hostElement = fixture.debugElement.query(By.css('div')).nativeElement as HTMLDivElement;
    fixture.detectChanges();
  });

  it('toggles the fileover class while dragging files over the drop zone', () => {
    expect(hostElement.classList.contains('fileover')).toBe(false);

    directive.onDragOver({
      preventDefault: () => undefined,
      stopPropagation: () => undefined,
    } as DragEvent);
    fixture.detectChanges();

    expect(hostElement.classList.contains('fileover')).toBe(true);

    directive.onDragLeave();
    fixture.detectChanges();

    expect(hostElement.classList.contains('fileover')).toBe(false);
  });

  it('emits dropped files and clears the hover state', () => {
    const file = new File(['content'], 'sample.geojson', { type: 'application/geo+json' });
    const files = [file] as unknown as FileList;

    let droppedFiles: FileList | undefined;
    directive.fileDropped.subscribe((value) => {
      droppedFiles = value;
    });

    directive.onDragOver({
      preventDefault: () => undefined,
      stopPropagation: () => undefined,
    } as DragEvent);
    fixture.detectChanges();

    directive.onDrop({
      preventDefault: () => undefined,
      stopPropagation: () => undefined,
      dataTransfer: { files },
    } as DragEvent);
    fixture.detectChanges();

    expect(directive.isOver()).toBe(false);
    expect(hostElement.classList.contains('fileover')).toBe(false);
    expect(droppedFiles).toBe(files);
  });
});
