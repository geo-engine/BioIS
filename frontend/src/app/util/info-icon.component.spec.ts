import { ComponentFixture, TestBed } from '@angular/core/testing';
import { InfoIconComponent } from './info-icon.component';
import { inputBinding } from '@angular/core';

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
  });

  it('should create and render button', () => {
    fixture.detectChanges();

    expect(component).toBeTruthy();
    const icon = (fixture.nativeElement as HTMLElement).querySelector('mat-icon');
    expect(icon).toBeTruthy();
  });

  it('should extract and display description from text content', () => {
    (fixture.nativeElement as HTMLElement).textContent = 'The info';
    fixture.detectChanges();

    expect(component.description()).toBe('The info');
  });
});
