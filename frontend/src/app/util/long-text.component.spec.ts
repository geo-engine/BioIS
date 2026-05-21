import { Component } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { LongTextComponent } from './long-text.component';
import { mockResizeObserverClass } from './resize-signal.spec';

@Component({
  template: `<app-long-text>Projected content for test</app-long-text>`,
  imports: [LongTextComponent],
})
class TestHostComponent {}

describe('LongTextComponent', () => {
  let fixture: ComponentFixture<TestHostComponent>;

  beforeEach(async () => {
    globalThis.ResizeObserver = mockResizeObserverClass([]);

    await TestBed.configureTestingModule({
      imports: [TestHostComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(TestHostComponent);
    fixture.detectChanges();
  });

  it('renders projected content', () => {
    const hostElement: HTMLElement = fixture.nativeElement as HTMLElement;
    const textElement = hostElement.querySelector('app-long-text .text');
    expect(textElement).not.toBeNull();
    if (textElement) {
      expect(textElement.textContent).toContain('Projected content for test');
    }
  });
});
