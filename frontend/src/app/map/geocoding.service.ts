import { inject, Injectable, InjectionToken } from '@angular/core';

/** URL of a Photon geocoder (https://github.com/komoot/photon), e.g., a self-hosted instance. */
export const GEOCODER_URL = new InjectionToken<string>('GEOCODER_URL', {
  providedIn: 'root',
  factory: (): string => 'https://photon.komoot.io/api/',
});

/** Languages that Photon supports for results (others are rejected). */
const PHOTON_LANGUAGES = ['de', 'en'];

const RESULT_LIMIT = 5;

export interface GeocodingResult {
  /** e.g., the name of a place or the street */
  label: string;
  /** e.g., the address and country */
  detail?: string;
  /** [lon, lat] */
  coordinate: [number, number];
  /** [minLon, minLat, maxLon, maxLat] */
  extent?: [number, number, number, number];
}

export interface GeocodingOptions {
  /** [lon, lat] to prefer nearby results */
  bias?: [number, number];
  signal?: AbortSignal;
}

/** Searches for addresses and places. */
@Injectable({ providedIn: 'root' })
export class GeocodingService {
  private readonly url = inject(GEOCODER_URL);

  async search(query: string, { bias, signal }: GeocodingOptions = {}): Promise<GeocodingResult[]> {
    const url = new URL(this.url);
    url.searchParams.set('q', query);
    url.searchParams.set('limit', String(RESULT_LIMIT));

    const language = photonLanguage(navigator.language);
    if (language) url.searchParams.set('lang', language);

    if (bias) {
      url.searchParams.set('lon', String(bias[0]));
      url.searchParams.set('lat', String(bias[1]));
    }

    const response = await fetch(url, { signal });
    if (!response.ok) throw new Error(`Geocoding failed: ${response.status}`);

    const collection = (await response.json()) as PhotonResponse;
    return collection.features.flatMap(photonResult);
  }
}

interface PhotonResponse {
  features: Array<{
    geometry?: { type: string; coordinates: [number, number] } | null;
    properties?: PhotonProperties | null;
  }>;
}

export interface PhotonProperties {
  name?: string;
  street?: string;
  housenumber?: string;
  postcode?: string;
  city?: string;
  state?: string;
  country?: string;
  /** [west, north, east, south] */
  extent?: [number, number, number, number];
}

function photonLanguage(browserLanguage: string | undefined): string | undefined {
  const language = browserLanguage?.split('-')[0].toLowerCase();
  return language && PHOTON_LANGUAGES.includes(language) ? language : undefined;
}

function photonResult(feature: PhotonResponse['features'][number]): GeocodingResult[] {
  if (feature.geometry?.type !== 'Point') return [];

  const properties = feature.properties ?? {};
  const { label, detail } = photonResultLabel(properties);

  return [
    {
      label,
      detail,
      coordinate: feature.geometry.coordinates,
      extent: photonExtent(properties.extent),
    },
  ];
}

/**
 * Creates a label from a Photon result, e.g.,
 * `Philipps-Universität Marburg` with detail `Biegenstraße 10, 35037 Marburg, Deutschland`.
 */
export function photonResultLabel(properties: PhotonProperties): {
  label: string;
  detail?: string;
} {
  const street = join(' ', properties.street, properties.housenumber);
  const place = join(' ', properties.postcode, properties.city);

  const parts = unique([properties.name, street, place, properties.state, properties.country]);
  const [label = 'Unknown location', ...detail] = parts;

  return { label, detail: detail.length > 0 ? detail.join(', ') : undefined };
}

/** Reorders Photon's `[west, north, east, south]` to `[minLon, minLat, maxLon, maxLat]`. */
export function photonExtent(
  extent: PhotonProperties['extent'],
): GeocodingResult['extent'] | undefined {
  if (extent?.length !== 4) return undefined;

  const [west, north, east, south] = extent;
  return [west, south, east, north];
}

function join(separator: string, ...parts: Array<string | undefined>): string | undefined {
  const filtered = parts.filter((part): part is string => !!part);
  return filtered.length > 0 ? filtered.join(separator) : undefined;
}

function unique(parts: Array<string | undefined>): string[] {
  return [...new Set(parts.filter((part): part is string => !!part))];
}
