import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { email, form, FormField, maxLength, required, submit } from '@angular/forms/signals';
import { MatButtonModule } from '@angular/material/button';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSelectModule } from '@angular/material/select';
import { ApiException, ContactRole } from '@geoengine/biois';
import { ContactService } from './contact.service';

// cf. backend `contact::handler`
const MAX_SHORT_FIELD_LENGTH = 200;
const MAX_MESSAGE_LENGTH = 5000;

interface ContactFormModel {
  name: string;
  email: string;
  organization: string;
  role: ContactRole | null;
  message: string;
  privacyConsent: boolean;
  /** Honeypot field, must stay empty */
  website: string;
}

type SubmitState = 'idle' | 'sending' | 'success' | 'error' | 'rate-limited';

const EMPTY_MODEL: ContactFormModel = {
  name: '',
  email: '',
  organization: '',
  role: null,
  message: '',
  privacyConsent: false,
  website: '',
};

@Component({
  selector: 'app-contact-form',
  changeDetection: ChangeDetectionStrategy.OnPush,
  imports: [
    FormField,
    MatButtonModule,
    MatCheckboxModule,
    MatFormFieldModule,
    MatIconModule,
    MatInputModule,
    MatProgressSpinnerModule,
    MatSelectModule,
  ],
  template: `
    @if (state() === 'success') {
      <div class="success" role="status">
        <mat-icon aria-hidden="true">mark_email_read</mat-icon>
        <p>
          <strong>Thank you for your interest!</strong><br />
          We have received your request and will get back to you shortly.
        </p>
      </div>
    } @else {
      <form (submit)="onSubmit($event)" novalidate aria-label="Request pilot access">
        <mat-form-field>
          <mat-label>Name</mat-label>
          <input matInput autocomplete="name" [formField]="form.name" />
          @for (error of form.name().errors(); track error) {
            <mat-error>{{ error.message }}</mat-error>
          }
        </mat-form-field>

        <mat-form-field>
          <mat-label>Business email</mat-label>
          <input matInput type="email" autocomplete="email" [formField]="form.email" />
          @for (error of form.email().errors(); track error) {
            <mat-error>{{ error.message }}</mat-error>
          }
        </mat-form-field>

        <mat-form-field>
          <mat-label>Organization</mat-label>
          <input matInput autocomplete="organization" [formField]="form.organization" />
          @for (error of form.organization().errors(); track error) {
            <mat-error>{{ error.message }}</mat-error>
          }
        </mat-form-field>

        <mat-form-field>
          <mat-label>I am a…</mat-label>
          <mat-select [formField]="form.role">
            @for (role of roles; track role.value) {
              <mat-option [value]="role.value">{{ role.label }}</mat-option>
            }
          </mat-select>
          @for (error of form.role().errors(); track error) {
            <mat-error>{{ error.message }}</mat-error>
          }
        </mat-form-field>

        <mat-form-field class="full-width">
          <mat-label>Anything else we should know? (optional)</mat-label>
          <textarea
            matInput
            rows="5"
            placeholder="e.g., number of sites, reporting deadline, frameworks you report on"
            [formField]="form.message"
          ></textarea>
          <mat-hint align="end"
            >{{ form.message().value().length }} / {{ maxMessageLength }}</mat-hint
          >
          @for (error of form.message().errors(); track error) {
            <mat-error>{{ error.message }}</mat-error>
          }
        </mat-form-field>

        <!-- Honeypot for spam bots: hidden from users and assistive technology -->
        <div class="honeypot" aria-hidden="true">
          <label>
            Website
            <input type="text" tabindex="-1" autocomplete="off" [formField]="form.website" />
          </label>
        </div>

        <div class="full-width consent">
          <mat-checkbox [formField]="form.privacyConsent">
            I agree that my data will be processed to answer my request, as described in the
            <a [href]="privacyPolicyUrl" target="_blank" rel="noopener">privacy policy</a>.
          </mat-checkbox>
          @if (form.privacyConsent().touched()) {
            @for (error of form.privacyConsent().errors(); track error) {
              <mat-error>{{ error.message }}</mat-error>
            }
          }
        </div>

        @switch (state()) {
          @case ('rate-limited') {
            <p class="full-width submit-error" role="alert">
              You have sent too many requests. Please try again later or email us at
              <a href="mailto:info@geoengine.de">info&#64;geoengine.de</a>.
            </p>
          }
          @case ('error') {
            <p class="full-width submit-error" role="alert">
              Your request could not be sent. Please try again later or email us at
              <a href="mailto:info@geoengine.de">info&#64;geoengine.de</a>.
            </p>
          }
        }

        <div class="full-width actions">
          <button
            matButton="filled"
            type="submit"
            [disabled]="form().invalid() || state() === 'sending'"
          >
            @if (state() === 'sending') {
              <mat-spinner diameter="20" aria-label="Sending"></mat-spinner>
            } @else {
              <mat-icon aria-hidden="true">send</mat-icon>
            }
            Request Pilot Access
          </button>
        </div>
      </form>
    }
  `,
  styles: `
    $small-breakpoint: 600px;

    :host {
      display: block;
      margin: 1.5rem 0;
    }

    form {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      column-gap: 1rem;
      row-gap: 0.5rem;

      @media (max-width: $small-breakpoint) {
        grid-template-columns: minmax(0, 1fr);
      }
    }

    .full-width {
      grid-column: 1 / -1;
    }

    .consent mat-error {
      display: block;
      padding-left: 2.5rem;
      font-size: smaller;
    }

    .honeypot {
      position: absolute;
      left: -10000px;
      width: 1px;
      height: 1px;
      overflow: hidden;
    }

    .submit-error {
      color: var(--mat-sys-error);
    }

    .actions button mat-spinner {
      display: inline-block;
      margin-right: 0.5rem;
    }

    .success {
      display: flex;
      align-items: center;
      gap: 1rem;
      padding: 1rem 1.5rem;
      border-radius: 1rem;
      background: var(--mat-sys-primary-container);
      color: var(--mat-sys-on-primary-container);

      mat-icon {
        flex-shrink: 0;
      }

      p {
        margin: 0;
      }
    }
  `,
})
export class ContactFormComponent {
  private readonly contactService = inject(ContactService);

  readonly privacyPolicyUrl = 'https://www.geoengine.de/en/privacy-policy/';
  readonly maxMessageLength = MAX_MESSAGE_LENGTH;

  readonly roles: ReadonlyArray<{ value: ContactRole; label: string }> = [
    { value: ContactRole.LargeCorporate, label: 'Large corporate (CSRD, Supply Chain Risks)' },
    { value: ContactRole.Sme, label: 'SME (VSME, Supply Chain Risks)' },
    { value: ContactRole.Consultancy, label: 'ESG consultancy' },
    { value: ContactRole.Other, label: 'Other' },
  ];

  readonly model = signal<ContactFormModel>({ ...EMPTY_MODEL });

  readonly form = form(this.model, (path) => {
    required(path.name, { message: 'Please enter your name.' });
    maxLength(path.name, MAX_SHORT_FIELD_LENGTH, { message: 'The name is too long.' });

    required(path.email, { message: 'Please enter your email address.' });
    email(path.email, { message: 'Please enter a valid email address.' });

    required(path.organization, { message: 'Please enter your organization.' });
    maxLength(path.organization, MAX_SHORT_FIELD_LENGTH, {
      message: 'The organization is too long.',
    });

    required(path.role, { message: 'Please select what describes you best.' });

    maxLength(path.message, MAX_MESSAGE_LENGTH, { message: 'The message is too long.' });

    required(path.privacyConsent, { message: 'Please accept the privacy policy.' });
  });

  readonly state = signal<SubmitState>('idle');

  async onSubmit(event: Event): Promise<void> {
    event.preventDefault();

    await submit(this.form, async () => {
      const { role, ...model } = this.model();
      if (!role) return; // guarded by validation

      this.state.set('sending');
      try {
        await this.contactService.submit({ ...model, role });
        this.state.set('success');
        this.model.set({ ...EMPTY_MODEL });
      } catch (error) {
        console.error('Failed to send contact request:', error);
        this.state.set(
          error instanceof ApiException && error.code === 429 ? 'rate-limited' : 'error',
        );
      }
      return undefined;
    });
  }
}
