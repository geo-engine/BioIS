import { TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import {
  GEOCODER_URL,
  GeocodingService,
  photonExtent,
  photonResultLabel,
} from './geocoding.service';

function photonResponse(): unknown {
  return {
    type: 'FeatureCollection',
    features: [
      {
        type: 'Feature',
        geometry: { type: 'Point', coordinates: [8.7722345, 50.8124763] },
        properties: {
          name: 'Philipps-Universität Marburg',
          street: 'Biegenstraße',
          housenumber: '10',
          postcode: '35037',
          city: 'Marburg',
          state: 'Hessen',
          country: 'Deutschland',
          extent: [8.7696339, 50.8153731, 8.7762961, 50.809536],
        },
      },
      // results without a point geometry are ignored
      { type: 'Feature', geometry: null, properties: { name: 'Nowhere' } },
    ],
  };
}

describe('GeocodingService', () => {
  let service: GeocodingService;
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    fetchMock = vi.fn(() => Promise.resolve(new Response(JSON.stringify(photonResponse()))));
    vi.stubGlobal('fetch', fetchMock);
    vi.spyOn(navigator, 'language', 'get').mockReturnValue('de-DE');

    TestBed.configureTestingModule({
      providers: [{ provide: GEOCODER_URL, useValue: 'https://geocoder.test/api/' }],
    });
    service = TestBed.inject(GeocodingService);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it('requests results near the bias', async () => {
    const controller = new AbortController();
    await service.search('Biegenstraße 10', { bias: [8.77, 50.8], signal: controller.signal });

    const [url, init] = fetchMock.mock.calls[0] as [URL, RequestInit];
    expect(url.origin + url.pathname).toBe('https://geocoder.test/api/');
    expect(Object.fromEntries(url.searchParams)).toEqual({
      q: 'Biegenstraße 10',
      limit: '5',
      lang: 'de',
      lon: '8.77',
      lat: '50.8',
    });
    expect(init.signal).toBe(controller.signal);
  });

  it('omits languages that Photon does not support', async () => {
    vi.spyOn(navigator, 'language', 'get').mockReturnValue('nl-NL');
    await service.search('Marburg');

    const [url] = fetchMock.mock.calls[0] as [URL];
    expect(url.searchParams.has('lang')).toBe(false);
    expect(url.searchParams.has('lon')).toBe(false);
  });

  it('parses the results', async () => {
    expect(await service.search('Biegenstraße 10')).toEqual([
      {
        label: 'Philipps-Universität Marburg',
        detail: 'Biegenstraße 10, 35037 Marburg, Hessen, Deutschland',
        coordinate: [8.7722345, 50.8124763],
        extent: [8.7696339, 50.809536, 8.7762961, 50.8153731],
      },
    ]);
  });

  it('fails for erroneous responses', async () => {
    fetchMock.mockResolvedValueOnce(new Response('{}', { status: 400 }));

    await expect(service.search('Marburg')).rejects.toThrow('Geocoding failed: 400');
  });
});

describe('photonResultLabel', () => {
  it('uses the street for addresses without a name', () => {
    expect(
      photonResultLabel({ street: 'Biegenstraße', housenumber: '10', city: 'Marburg' }),
    ).toEqual({ label: 'Biegenstraße 10', detail: 'Marburg' });
  });

  it('removes duplicates', () => {
    expect(photonResultLabel({ name: 'Marburg', city: 'Marburg', country: 'Deutschland' })).toEqual(
      { label: 'Marburg', detail: 'Deutschland' },
    );
  });

  it('has a fallback', () => {
    expect(photonResultLabel({})).toEqual({ label: 'Unknown location', detail: undefined });
  });
});

describe('photonExtent', () => {
  it('reorders to min/max coordinates', () => {
    expect(photonExtent([1, 4, 3, 2])).toEqual([1, 2, 3, 4]);
    expect(photonExtent(undefined)).toBeUndefined();
  });
});
