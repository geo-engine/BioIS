import { ApiException } from '@geoengine/biois';
import { BackendError } from './error';

describe('BackendError', () => {
  it('stores values passed to the constructor', () => {
    const error = new BackendError(
      'https://example.com/problem',
      400,
      'Bad Request',
      'Invalid input',
    );

    expect(error.type).toBe('https://example.com/problem');
    expect(error.status).toBe(400);
    expect(error.title).toBe('Bad Request');
    expect(error.detail).toBe('Invalid input');
  });

  it('returns the same instance when given a BackendError', () => {
    const input = new BackendError('AlreadyBackendError', 418, 'I am a teapot', 'Still a teapot');

    expect(BackendError.fromError(input)).toBe(input);
  });

  it('creates a BackendError from an ApiException body', () => {
    const apiException = new ApiException(
      400,
      'Bad Request',
      {
        type: 'https://example.com/problem',
        status: 400,
        title: 'Validation Failed',
        detail: 'Name must not be empty',
      },
      {},
    );

    const error = BackendError.fromError(apiException);

    expect(error.type).toBe('https://example.com/problem');
    expect(error.status).toBe(400);
    expect(error.title).toBe('Validation Failed');
    expect(error.detail).toBe('Name must not be empty');
  });

  it('creates a BackendError from a native Error', () => {
    const nativeError = new Error('Boom');
    nativeError.name = 'TypeError';
    nativeError.stack = 'stack trace';

    const error = BackendError.fromError(nativeError);

    expect(error.type).toBe('TypeError');
    expect(error.status).toBeUndefined();
    expect(error.title).toBe('Boom');
    expect(error.detail).toBe('stack trace');
  });

  it('creates a BackendError from a plain object', () => {
    const payload = { reason: 'some reason', nested: { value: 1 } };

    const error = BackendError.fromError(payload);

    expect(error.type).toBe('Error');
    expect(error.status).toBeUndefined();
    expect(error.title).toBeUndefined();
    expect(error.detail).toBe(JSON.stringify(payload));
  });

  it('creates a BackendError from a primitive value', () => {
    const error = BackendError.fromError(42);

    expect(error.type).toBe('Error');
    expect(error.status).toBeUndefined();
    expect(error.title).toBeUndefined();
    expect(error.detail).toBe('42');
  });
});
