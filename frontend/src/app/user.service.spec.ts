import { TestBed } from '@angular/core/testing';

import { UserService, __TEST__ } from './user.service';

describe('UserService', () => {
  let service: UserService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(UserService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});

describe('UserServiceHelpers', () => {
  it('should compare dates', () => {
    expect(
      __TEST__.sessionIsValid(
        {
          created: '',
          id: '',
          roles: [],
          user: {
            id: '',
          },
          validUntil: '2026-02-18T17:20:42.536Z',
        },
        Date.parse('2026-02-18T16:20:42.536Z'),
      ),
    ).toBeTruthy();
  });
});
