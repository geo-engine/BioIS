import { ComponentFixture, TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import { Process, ProcessesApi } from '@geoengine/biois';

import { CreateNewComponent } from './create-new.component';

describe('CreateNewComponent', () => {
  let component: CreateNewComponent;
  let fixture: ComponentFixture<CreateNewComponent>;

  beforeEach(async () => {
    // mock ProcessesApi.process early so resource loaders in the component don't perform real network fetches
    vi.spyOn(ProcessesApi.prototype, 'process').mockResolvedValue(ndviProcess());

    await TestBed.configureTestingModule({
      imports: [CreateNewComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(CreateNewComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});

function ndviProcess(): Process {
  const process = new Process();
  process.id = 'ndvi';
  process.inputs = {
    coordinate: {
      title: 'Coordinate',
      description: 'The coordinate to calculate the NDVI for',
      schema: null,
    },
    year: {
      title: 'Year',
      description: 'The year to calculate the NDVI for',
      schema: null,
    },
    month: {
      title: 'Month',
      description: 'The month to calculate the NDVI for',
      schema: null,
    },
  };
  process.outputs = {
    ndvi: {
      title: 'NDVI',
      description: 'The calculated NDVI value',
      schema: null,
    },
    kNdvi: {
      title: 'kNDVI',
      description: 'The calculated kNDVI value',
      schema: null,
    },
  };
  return process;
}
