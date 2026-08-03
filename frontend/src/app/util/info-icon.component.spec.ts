import { ComponentFixture, TestBed } from '@angular/core/testing';
import { InfoIconComponent } from './info-icon.component';
import { inputBinding } from '@angular/core';
import { MatTooltip } from '@angular/material/tooltip';

describe('InfoIconComponent', () => {
  let component: InfoIconComponent;
  let fixture: ComponentFixture<InfoIconComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [InfoIconComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(InfoIconComponent, {
      bindings: [inputBinding('description', () => 'The info')],
    });
    component = fixture.componentInstance;

    await fixture.whenStable();
  });

  it('should create and render button', () => {
    expect(component).toBeTruthy();
    const icon = (fixture.nativeElement as HTMLElement).querySelector('mat-icon');
    expect(icon).toBeTruthy();
  });

  it('should bind description to matTooltip and aria-label', () => {
    const iconDebugElement = fixture.debugElement.query((el) => el.name === 'mat-icon');
    expect(iconDebugElement).toBeTruthy();

    const icon = iconDebugElement?.nativeElement as HTMLElement;
    expect(icon.getAttribute('aria-label')).toBe('The info');
    expect(component.description()).toBe('The info');

    const tooltipDirective = iconDebugElement?.injector.get(MatTooltip);
    expect(tooltipDirective).toBeTruthy();
    expect(tooltipDirective.message).toBe('The info');
  });
});
