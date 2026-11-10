// TODO: better import syntax?
import { BaseAPIRequestFactory } from "./baseapi";
import { Configuration } from "../configuration";
import {
  RequestContext,
  HttpMethod,
  ResponseContext,
  HttpInfo,
} from "../http/http";
import { ObjectSerializer } from "../models/ObjectSerializer";
import { ApiException } from "./exception";
import { isCodeInRange } from "../util";
import { SecurityAuthentication } from "../auth/auth";

/**
 * no description
 */
export class ProfilesApiRequestFactory extends BaseAPIRequestFactory {
  /**
   * The profile of the `schema.biois` extension: a Table Schema extension that declares per-column display metadata for `BioIS` result tables.
   */
  public async climateRiskTableSchemaProfile(
    _options?: Configuration,
  ): Promise<RequestContext> {
    let _config = _options || this.configuration;

    // Path Params
    const localVarPath = "/profiles/table-schema/climate-risk/1.0/schema.json";

    // Make Request Context
    const requestContext = _config.baseServer.makeRequestContext(
      localVarPath,
      HttpMethod.GET,
    );
    requestContext.setHeaderParam("Accept", "application/json, */*;q=0.8");

    const defaultAuth: SecurityAuthentication | undefined =
      _config?.authMethods?.default;
    if (defaultAuth?.applySecurityAuthentication) {
      await defaultAuth?.applySecurityAuthentication(requestContext);
    }

    return requestContext;
  }
}

export class ProfilesApiResponseProcessor {
  /**
   * Unwraps the actual response sent by the server from the response context and deserializes the response content
   * to the expected objects
   *
   * @params response Response returned by the server for a request to climateRiskTableSchemaProfile
   * @throws ApiException if the response code was not in [200, 299]
   */
  public async climateRiskTableSchemaProfileWithHttpInfo(
    response: ResponseContext,
  ): Promise<HttpInfo<string>> {
    const contentType = ObjectSerializer.normalizeMediaType(
      response.headers["content-type"],
    );
    if (isCodeInRange("200", response.httpStatusCode)) {
      const body: string = ObjectSerializer.deserialize(
        ObjectSerializer.parse(await response.body.text(), contentType),
        "string",
        "",
      ) as string;
      return new HttpInfo(
        response.httpStatusCode,
        response.headers,
        response.body,
        body,
      );
    }

    // Work around for missing responses in specification, e.g. for petstore.yaml
    if (response.httpStatusCode >= 200 && response.httpStatusCode <= 299) {
      const body: string = ObjectSerializer.deserialize(
        ObjectSerializer.parse(await response.body.text(), contentType),
        "string",
        "",
      ) as string;
      return new HttpInfo(
        response.httpStatusCode,
        response.headers,
        response.body,
        body,
      );
    }

    throw new ApiException<string | Blob | undefined>(
      response.httpStatusCode,
      "Unknown API Status Code!",
      await response.getBodyAsAny(),
      response.headers,
    );
  }
}
