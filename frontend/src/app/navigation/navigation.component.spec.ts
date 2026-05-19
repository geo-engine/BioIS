import { ComponentFixture, TestBed } from '@angular/core/testing';
import { NavigationComponent } from './navigation.component';
import { RouterModule } from '@angular/router';
import { MatIconTestingModule } from '@angular/material/icon/testing';
import { ProcessesApi, ProcessList, ProcessSummary } from '@geoengine/biois';

describe('NavigationComponent', () => {
  let component: NavigationComponent;
  let fixture: ComponentFixture<NavigationComponent>;

  beforeEach(async () => {
    // mock ProcessesApi.processes early so resource loaders in the component don't perform real network fetches
    vi.spyOn(ProcessesApi.prototype, 'processes').mockResolvedValue(processes());

    await TestBed.configureTestingModule({
      imports: [MatIconTestingModule, NavigationComponent, RouterModule.forRoot([])],
    }).compileComponents();

    fixture = TestBed.createComponent(NavigationComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should compile', () => {
    expect(component).toBeTruthy();
  });
});

function processes(): ProcessList {
  const processList = new ProcessList();
  const ndviProcessSummary = new ProcessSummary();
  ndviProcessSummary.id = 'ndvi';
  ndviProcessSummary.version = '0.1.0';
  processList.processes = [ndviProcessSummary];
  return processList;
}
