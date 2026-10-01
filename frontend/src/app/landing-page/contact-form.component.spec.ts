import { ComponentFixture, TestBed } from '@angular/core/testing';
import { MatIconTestingModule } from '@angular/material/icon/testing';
import { ApiException, ContactRole } from '@geoengine/biois';
import { Mock, vi } from 'vitest';
import { ContactFormComponent } from './contact-form.component';
import { ContactService } from './contact.service';

describe('ContactFormComponent', () => {
  let component: ContactFormComponent;
  let fixture: ComponentFixture<ContactFormComponent>;
  let submitMock: Mock<ContactService['submit']>;

  const validModel = {
    name: 'Max Muster',
    email: 'max@example.com',
    organization: 'ACME Corp',
    role: ContactRole.LargeCorporate,
    message: 'We would like to join the pilot.',
    privacyConsent: true,
    website: '',
  };

  beforeEach(async () => {
    submitMock = vi.fn<ContactService['submit']>().mockResolvedValue(undefined);

    await TestBed.configureTestingModule({
      imports: [ContactFormComponent, MatIconTestingModule],
      providers: [{ provide: ContactService, useValue: { submit: submitMock } }],
    }).compileComponents();

    fixture = TestBed.createComponent(ContactFormComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  function submitEvent(): Event {
    return new Event('submit', { cancelable: true });
  }

  function text(): string {
    return (fixture.nativeElement as HTMLElement).textContent ?? '';
  }

  it('renders the form fields', () => {
    const element = fixture.nativeElement as HTMLElement;
    expect(element.querySelectorAll('input[matInput]').length).toBe(3);
    expect(element.querySelector('textarea')).toBeTruthy();
    expect(element.querySelector('mat-select')).toBeTruthy();
    expect(element.querySelector('mat-checkbox')).toBeTruthy();
    expect(element.querySelector('.honeypot input')).toBeTruthy();
  });

  it('disables the submit button while the form is invalid', async () => {
    const button = (fixture.nativeElement as HTMLElement).querySelector<HTMLButtonElement>(
      'button[type="submit"]',
    );
    expect(button?.disabled).toBe(true);

    component.model.set({ ...validModel });
    await fixture.whenStable();
    expect(button?.disabled).toBe(false);

    component.model.set({ ...validModel, privacyConsent: false });
    await fixture.whenStable();
    expect(button?.disabled).toBe(true);
  });

  it('does not submit an invalid form', async () => {
    await component.onSubmit(submitEvent());
    await fixture.whenStable();

    expect(submitMock).not.toHaveBeenCalled();
    expect(component.state()).toBe('idle');
    expect(text()).toContain('Please enter your name.');
  });

  it('requires the privacy consent', async () => {
    component.model.set({ ...validModel, privacyConsent: false });

    await component.onSubmit(submitEvent());

    expect(submitMock).not.toHaveBeenCalled();
  });

  it('submits a valid form and shows a success message', async () => {
    component.model.set({ ...validModel });

    await component.onSubmit(submitEvent());
    await fixture.whenStable();

    expect(submitMock).toHaveBeenCalledExactlyOnceWith(validModel);
    expect(component.state()).toBe('success');
    expect(text()).toContain('Thank you for your interest!');
  });

  it('submits without a message', async () => {
    component.model.set({ ...validModel, message: '' });

    await component.onSubmit(submitEvent());

    expect(submitMock).toHaveBeenCalledExactlyOnceWith({ ...validModel, message: '' });
    expect(component.state()).toBe('success');
  });

  it('shows a rate limit message', async () => {
    submitMock.mockRejectedValue(new ApiException(429, 'Too many requests', undefined, {}));
    component.model.set({ ...validModel });

    await component.onSubmit(submitEvent());
    await fixture.whenStable();

    expect(component.state()).toBe('rate-limited');
    expect(text()).toContain('too many requests');
  });

  it('shows an error message if sending fails', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined);
    submitMock.mockRejectedValue(new ApiException(500, 'Server error', undefined, {}));
    component.model.set({ ...validModel });

    await component.onSubmit(submitEvent());
    await fixture.whenStable();

    expect(component.state()).toBe('error');
    expect(text()).toContain('could not be sent');
  });
});
