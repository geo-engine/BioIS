import { ComponentFixture, TestBed } from '@angular/core/testing';

import { NumberIndicatorComponent } from './number-indicator.component';
import { By } from '@angular/platform-browser';

describe('NumberIndicatorComponent', () => {
  let component: NumberIndicatorComponent;
  let fixture: ComponentFixture<NumberIndicatorComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [NumberIndicatorComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(NumberIndicatorComponent);
    component = fixture.componentInstance;
  });

  function setDefaultInputs(value = 50): void {
    fixture.componentRef.setInput('value', value);
    fixture.componentRef.setInput('min', 0);
    fixture.componentRef.setInput('max', 100);
    fixture.componentRef.setInput('colors', [
      { min: 0, max: 40, color: '#ef5350' },
      { min: 41, max: 80, color: '#ffca28' },
      { min: 81, max: 100, color: '#66bb6a' },
    ]);
  }

  it('creates the component', () => {
    setDefaultInputs();
    fixture.detectChanges();

    expect(component).toBeTruthy();
  });

  it('renders the formatted numeric value', () => {
    setDefaultInputs(12.3456);
    fixture.detectChanges();

    const valueElement = fixture.debugElement.query(By.css('p'))
      .nativeElement as HTMLParagraphElement;

    expect(valueElement.textContent?.trim()).toBe('12.35');
  });

  it('computes the progress percentage and applies matching range color', () => {
    setDefaultInputs(50);
    fixture.detectChanges();

    const spinner = fixture.debugElement.query(By.css('mat-progress-spinner'))
      .nativeElement as HTMLElement;

    expect(component.scaledValue()).toBe(0.5);
    expect(spinner.getAttribute('style')).toContain(
      '--mat-progress-spinner-active-indicator-color: #ffca28;',
    );
  });

  it('uses fallback color when no breakpoint matches', () => {
    setDefaultInputs(1000);
    fixture.componentRef.setInput('fallbackColor', '#808080');
    fixture.detectChanges();

    const spinner = fixture.debugElement.query(By.css('mat-progress-spinner'))
      .nativeElement as HTMLElement;

    expect(spinner.getAttribute('style')).toContain(
      '--mat-progress-spinner-active-indicator-color: #808080;',
    );
  });
});
