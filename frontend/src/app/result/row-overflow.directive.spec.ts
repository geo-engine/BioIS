import { ChangeDetectionStrategy, Component, signal, viewChild } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RowOverflowDirective } from './row-overflow.directive';

interface MockObserverInstance {
  callback: ResizeObserverCallback;
  disconnected: boolean;
  observedTargets: Element[];
}

function mockResizeObserverClass(instances: MockObserverInstance[]): typeof ResizeObserver {
  return class MockResizeObserver {
    callback: ResizeObserverCallback;
    disconnected = false;
    observedTargets: Element[] = [];

    constructor(callback: ResizeObserverCallback) {
      this.callback = callback;
      instances.push(this);
    }

    observe(target: Element): void {
      this.observedTargets.push(target);
    }

    disconnect(): void {
      this.disconnected = true;
    }
  } as unknown as typeof ResizeObserver;
}

function setElementHeights(target: HTMLElement, clientHeight: number, scrollHeight: number): void {
  Object.defineProperty(target, 'clientHeight', {
    configurable: true,
    get: () => clientHeight,
  });

  Object.defineProperty(target, 'scrollHeight', {
    configurable: true,
    get: () => scrollHeight,
  });
}

@Component({
  template: `
    <div appRowOverflow=".cell-content" [appRowOverflowExpanded]="expanded()">
      <div class="cell-content" id="target-a"></div>
      <div class="cell-content" id="target-b"></div>
    </div>
  `,
  imports: [RowOverflowDirective],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
class TestHostComponent {
  readonly expanded = signal(false);

  readonly directive = viewChild.required(RowOverflowDirective);
}

@Component({
  template: `<div appRowOverflow=".missing-target" [appRowOverflowExpanded]="false"></div>`,
  imports: [RowOverflowDirective],
})
class MissingTargetHostComponent {}

describe('RowOverflowDirective', () => {
  let createdObservers: MockObserverInstance[];

  beforeEach(() => {
    createdObservers = [];
    globalThis.ResizeObserver = mockResizeObserverClass(createdObservers);
  });

  async function createFixture(): Promise<ComponentFixture<TestHostComponent>> {
    await TestBed.configureTestingModule({
      imports: [TestHostComponent],
    }).compileComponents();

    const fixture = TestBed.createComponent(TestHostComponent);
    fixture.detectChanges();
    await fixture.whenStable();
    await new Promise((resolve) => setTimeout(resolve, 0));

    return fixture;
  }

  it('sets canExpand to true when at least one target overflows', async () => {
    const fixture = await createFixture();
    const hostElement = fixture.nativeElement as HTMLElement;

    const targetA = hostElement.querySelector('#target-a');
    const targetB = hostElement.querySelector('#target-b');
    expect(targetA).not.toBeNull();
    expect(targetB).not.toBeNull();

    setElementHeights(targetA as HTMLElement, 20, 20);
    setElementHeights(targetB as HTMLElement, 20, 40);

    const observer = createdObservers[0];
    observer.callback([], observer as unknown as ResizeObserver);

    expect(fixture.componentInstance.directive().canExpand()).toBe(true);
  });

  it('keeps canExpand false when no target overflows', async () => {
    const fixture = await createFixture();
    const hostElement = fixture.nativeElement as HTMLElement;

    const targetA = hostElement.querySelector('#target-a');
    const targetB = hostElement.querySelector('#target-b');
    expect(targetA).not.toBeNull();
    expect(targetB).not.toBeNull();

    setElementHeights(targetA as HTMLElement, 20, 20);
    setElementHeights(targetB as HTMLElement, 30, 30);

    const observer = createdObservers[0];
    observer.callback([], observer as unknown as ResizeObserver);

    expect(fixture.componentInstance.directive().canExpand()).toBe(false);
  });

  it('does not set canExpand when row is expanded', async () => {
    await TestBed.configureTestingModule({
      imports: [TestHostComponent],
    }).compileComponents();

    const fixture = TestBed.createComponent(TestHostComponent);
    fixture.componentInstance.expanded.set(true);
    fixture.detectChanges();
    await fixture.whenStable();
    await new Promise((resolve) => setTimeout(resolve, 0));

    const hostElement = fixture.nativeElement as HTMLElement;
    const targetA = hostElement.querySelector('#target-a');
    const targetB = hostElement.querySelector('#target-b');
    expect(targetA).not.toBeNull();
    expect(targetB).not.toBeNull();

    setElementHeights(targetA as HTMLElement, 20, 60);
    setElementHeights(targetB as HTMLElement, 20, 50);

    const observer = createdObservers[0];
    observer.callback([], observer as unknown as ResizeObserver);

    expect(fixture.componentInstance.directive().canExpand()).toBe(false);
  });

  it('does not create a ResizeObserver when no targets match', async () => {
    await TestBed.configureTestingModule({
      imports: [MissingTargetHostComponent],
    }).compileComponents();

    const fixture = TestBed.createComponent(MissingTargetHostComponent);
    fixture.detectChanges();
    await fixture.whenStable();
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(createdObservers).toHaveLength(0);
  });

  it('disconnects ResizeObserver on destroy', async () => {
    const fixture = await createFixture();

    expect(createdObservers).toHaveLength(1);
    const observer = createdObservers[0];
    expect(observer.disconnected).toBe(false);

    fixture.destroy();

    expect(observer.disconnected).toBe(true);
  });
});
