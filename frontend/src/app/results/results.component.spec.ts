import { ComponentFixture, TestBed } from '@angular/core/testing';
import { of } from 'rxjs';
import { createConfiguration, ServerConfiguration, StatusCode, StatusInfo } from '@geoengine/biois';
import { vi } from 'vitest';
import { ResultsComponent } from './results.component';
import { JobsDataSource } from './jobs-datasource';
import { UserService } from '../user.service';
import { RouterModule } from '@angular/router';

describe('ResultsComponent', () => {
  let component: ResultsComponent;
  let fixture: ComponentFixture<ResultsComponent>;

  function setupWithRows(rows: StatusInfo[]): void {
    vi.spyOn(JobsDataSource.prototype, 'connect').mockReturnValue(of(rows));
    fixture = TestBed.createComponent(ResultsComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  }

  beforeEach(() => {
    vi.restoreAllMocks();

    TestBed.configureTestingModule({
      imports: [ResultsComponent, RouterModule.forRoot([])],
      providers: [
        {
          provide: UserService,
          useValue: {
            apiConfiguration: (): ReturnType<typeof createConfiguration> =>
              createConfiguration({
                baseServer: new ServerConfiguration('/api', {}),
              }),
          },
        },
      ],
    });
  });

  it('should compile', () => {
    setupWithRows([]);
    expect(component).toBeTruthy();
  });

  it('renders a row with job data', async () => {
    setupWithRows([
      {
        jobID: 'job-123',
        processID: 'biodiversity-sensitive-areas',
        status: StatusCode.Successful,
        progress: 100,
        message: 'Completed',
        created: new Date('2026-01-02T11:00:00Z'),
        updated: new Date('2026-01-02T11:05:00Z'),
        finished: new Date('2026-01-02T11:05:00Z'),
      },
    ]);

    await fixture.whenStable();
    fixture.detectChanges();

    const text = (fixture.nativeElement as HTMLElement).textContent ?? '';

    expect(text).toContain('biodiversity-sensitive-areas');
    expect(text).toContain('Completed');
    expect(text).toContain('Show Results');
    expect(text).toContain('check_circle');
  });

  it('renders the no-data row for an empty result set', async () => {
    setupWithRows([]);

    await fixture.whenStable();
    fixture.detectChanges();

    const noDataCell = (fixture.nativeElement as HTMLElement).querySelector('.no-data-row');

    expect(noDataCell?.textContent).toContain('No data to show');
  });
});
