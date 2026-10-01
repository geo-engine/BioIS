import { Injectable } from '@angular/core';
import { ContactApi, ContactRequest } from '@geoengine/biois';
import { publicApiConfiguration } from '../user.service';

@Injectable({
  providedIn: 'root',
})
export class ContactService {
  private readonly contactApi = new ContactApi(publicApiConfiguration());

  /**
   * Sends a request for pilot access to the BioIS team.
   * @throws `ApiException` if the request failed, e.g., with code 429 if the rate limit was exceeded.
   */
  async submit(request: ContactRequest): Promise<void> {
    await this.contactApi.submitContact(request);
  }
}
