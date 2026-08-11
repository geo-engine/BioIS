import { YearAndMonth } from './credits.component';

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
