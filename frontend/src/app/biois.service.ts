import { Injectable } from '@angular/core';
import { createBioISClient } from '@biois/bindings';
import type { components } from '@biois/bindings';

export type Indicator = components['schemas']['Indicator'];
export type IndicatorRequest = components['schemas']['IndicatorRequest'];
export type IndicatorResponse = components['schemas']['IndicatorResponse'];
export type HealthResponse = components['schemas']['HealthResponse'];

@Injectable({
  providedIn: 'root'
})
export class BioISService {
  private client = createBioISClient('http://localhost:3000');

  async getHealth(): Promise<HealthResponse> {
    const { data, error } = await this.client.GET('/health');
    if (error) throw error;
    return data!;
  }

  async getIndicators(): Promise<Indicator[]> {
    const { data, error } = await this.client.GET('/indicators');
    if (error) throw error;
    return data!;
  }

  async calculateIndicator(request: IndicatorRequest): Promise<IndicatorResponse> {
    const { data, error } = await this.client.POST('/indicators/calculate', {
      body: request
    });
    if (error) throw error;
    return data!;
  }
}
