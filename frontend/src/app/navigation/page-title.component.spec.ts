import { ComponentFixture, TestBed } from '@angular/core/testing';
import { PageTitleComponent } from './page-title.component';
import { TitleService } from './title.service';

describe('PageTitleComponent', () => {
  let component: PageTitleComponent;
  let fixture: ComponentFixture<PageTitleComponent>;
  let titleService: TitleService;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PageTitleComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(PageTitleComponent);
    component = fixture.componentInstance;
    titleService = TestBed.inject(TitleService);
  });

  it('should compile', () => {
    fixture.componentRef.setInput('title', 'Overview');
    fixture.detectChanges();

    expect(component).toBeTruthy();
  });

  it('sets the title service to a trimmed value', () => {
    fixture.componentRef.setInput('title', '  Results  ');
    fixture.detectChanges();

    expect(titleService.title()).toBe('Results');
  });

  it('ignores blank titles', () => {
    titleService.title = 'Existing';

    fixture.componentRef.setInput('title', '   ');
    fixture.detectChanges();

    expect(titleService.title()).toBe('Existing');
  });
});
