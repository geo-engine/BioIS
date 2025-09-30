import createClient from "openapi-fetch";
import type { paths } from "./schema";

export type { paths } from "./schema";
export type { components } from "./schema";

export function createBioISClient(baseUrl: string = "http://localhost:3000") {
  return createClient<paths>({ baseUrl });
}

export default createBioISClient;
