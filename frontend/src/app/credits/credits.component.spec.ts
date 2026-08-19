import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RouterModule } from '@angular/router';
import { vi } from 'vitest';
import { UserService } from '../user.service';
import { CreditsComponent, YearAndMonth } from './credits.component';

describe('YearAndMonth', () => {
  it('should create an instance from a Date object', () => {
    const date = new Date(2023, 4, 15); // May 15, 2023
    const yearAndMonth = YearAndMonth.fromDate(date);
    expect(yearAndMonth.year).toBe(2023);
    expect(yearAndMonth.month).toBe(5);
  });

  it('should return the next month correctly', () => {
    const yearAndMonth = new YearAndMonth(2023, 12); // December 2023
    const next = yearAndMonth.nextMonth();
    expect(next.year).toBe(2024);
    expect(next.month).toBe(1);
  });

  it('should return the previous month correctly', () => {
    const yearAndMonth = new YearAndMonth(2023, 1); // January 2023
    const previous = yearAndMonth.previousMonth();
    expect(previous.year).toBe(2022);
    expect(previous.month).toBe(12);
  });

  it('should convert to a Date object correctly', () => {
    const yearAndMonth = new YearAndMonth(2023, 5); // May 2023
    const date = yearAndMonth.date;
    expect(date.getFullYear()).toBe(2023);
    expect(date.getMonth()).toBe(4); // Months are zero-based in JavaScript Date
  });
});

describe('CreditsComponent', () => {
  let component: CreditsComponent;
  let fixture: ComponentFixture<CreditsComponent>;
  const creditsMock = vi.fn();

  beforeEach(async () => {
    creditsMock.mockReset();
    creditsMock.mockResolvedValue({
      creditsUsed: 42,
      details: [
        { jobId: 'job-123', creditsUsed: 17 },
        { jobId: 'job-456', creditsUsed: 25 },
      ],
    });

    await TestBed.configureTestingModule({
      imports: [CreditsComponent, RouterModule.forRoot([])],
      providers: [
        {
          provide: UserService,
          useValue: {
            credits: creditsMock,
          },
        },
      ],
    }).compileComponents();

    fixture = TestBed.createComponent(CreditsComponent);
    component = fixture.componentInstance;
  });

  it('loads and renders the credits for a month', async () => {
    fixture.detectChanges();
    await fixture.whenStable();
    fixture.detectChanges();

    const text = (fixture.nativeElement as HTMLElement).textContent ?? '';

    expect(text).toContain('Credits used:');
    expect(text).toContain('42');
    expect(text).toContain('job-123');
    expect(text).toContain('17');
    expect(creditsMock).toHaveBeenCalledWith(expect.any(Number), expect.any(Number));
  });

  it('renders the error state when the credits request fails', async () => {
    creditsMock.mockRejectedValueOnce(new Error('network failure'));

    fixture.detectChanges();
    await fixture.whenStable();
    fixture.detectChanges();

    const text = (fixture.nativeElement as HTMLElement).textContent ?? '';

    expect(text).toContain('Error loading credits:');
    expect(text).toContain('network failure');
  });

  it('updates the selected month and closes the datepicker on month selection', () => {
    const close = vi.fn();

    component.setMonthAndYear(new Date(2024, 6, 15), { close } as never);

    expect(component.yearAndMonth().year).toBe(2024);
    expect(component.yearAndMonth().month).toBe(7);
    expect(close).toHaveBeenCalledTimes(1);
  });

  it('moves to the next and previous month correctly', () => {
    component.yearAndMonth.set(new YearAndMonth(2023, 12));
    component.nextMonth();
    expect(component.yearAndMonth()).toEqual(new YearAndMonth(2024, 1));

    component.previousMonth();
    expect(component.yearAndMonth()).toEqual(new YearAndMonth(2023, 12));

    component.yearAndMonth.set(new YearAndMonth(2023, 1));
    component.previousMonth();
    expect(component.yearAndMonth()).toEqual(new YearAndMonth(2022, 12));
  });
});
