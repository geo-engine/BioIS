import { ApiException } from '@geoengine/biois';

/**
 * Error based on [RFC 9457](https://www.rfc-editor.org/rfc/rfc9457.html)
 */
export class BackendError {
  readonly type: string;
  readonly status?: number;
  readonly title?: string;
  readonly detail?: string;

  constructor(type: string, status?: number, title?: string, detail?: string) {
    this.type = type;
    this.status = status;
    this.title = title;
    this.detail = detail;
  }

  static fromError(error: unknown): BackendError {
    if (error instanceof BackendError) {
      return error;
    }

    if (error instanceof ApiException) {
      const body = error.body as {
        type: string;
        status?: number;
        title?: string;
        detail?: string;
      }; // TODO: Use proper type checking here
      return new BackendError(body.type, body.status, body.title, body.detail);
    }

    if (error instanceof Error) {
      return new BackendError(error.name, undefined, error.message, error.stack);
    }

    if (typeof error === 'object' && error !== null) {
      return new BackendError('Error', undefined, undefined, JSON.stringify(error));
    }

    return new BackendError('Error', undefined, undefined, String(error));
  }
}
