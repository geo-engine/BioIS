// TODO: better import syntax?
import { BaseAPIRequestFactory, RequiredError } from "./baseapi";
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

import { ContactRequest } from "../models/ContactRequest";
import { Exception } from "../models/Exception";

/**
 * no description
 */
export class ContactApiRequestFactory extends BaseAPIRequestFactory {
  /**
   * Sends a request for pilot access to the `BioIS` team.
   * @param contactRequest
   */
  public async submitContact(
    contactRequest: ContactRequest,
    _options?: Configuration,
  ): Promise<RequestContext> {
    let _config = _options || this.configuration;

    // verify required parameter 'contactRequest' is not null or undefined
    if (contactRequest === null || contactRequest === undefined) {
      throw new RequiredError("ContactApi", "submitContact", "contactRequest");
    }

    // Path Params
    const localVarPath = "/contact";

    // Make Request Context
    const requestContext = _config.baseServer.makeRequestContext(
      localVarPath,
      HttpMethod.POST,
    );
    requestContext.setHeaderParam("Accept", "application/json, */*;q=0.8");

    // Body Params
    const contentType = ObjectSerializer.getPreferredMediaType([
      "application/json",
    ]);
    requestContext.setHeaderParam("Content-Type", contentType);
    const serializedBody = ObjectSerializer.stringify(
      ObjectSerializer.serialize(contactRequest, "ContactRequest", ""),
      contentType,
    );
    requestContext.setBody(serializedBody);

    const defaultAuth: SecurityAuthentication | undefined =
      _config?.authMethods?.default;
    if (defaultAuth?.applySecurityAuthentication) {
      await defaultAuth?.applySecurityAuthentication(requestContext);
    }

    return requestContext;
  }
}

export class ContactApiResponseProcessor {
  /**
   * Unwraps the actual response sent by the server from the response context and deserializes the response content
   * to the expected objects
   *
   * @params response Response returned by the server for a request to submitContact
   * @throws ApiException if the response code was not in [200, 299]
   */
  public async submitContactWithHttpInfo(
    response: ResponseContext,
  ): Promise<HttpInfo<void>> {
    const contentType = ObjectSerializer.normalizeMediaType(
      response.headers["content-type"],
    );
    if (isCodeInRange("204", response.httpStatusCode)) {
      return new HttpInfo(
        response.httpStatusCode,
        response.headers,
        response.body,
        undefined,
      );
    }
    if (isCodeInRange("400", response.httpStatusCode)) {
      const body: Exception = ObjectSerializer.deserialize(
        ObjectSerializer.parse(await response.body.text(), contentType),
        "Exception",
        "",
      ) as Exception;
      throw new ApiException<Exception>(
        response.httpStatusCode,
        "The request is invalid.",
        body,
        response.headers,
      );
    }
    if (isCodeInRange("429", response.httpStatusCode)) {
      const body: Exception = ObjectSerializer.deserialize(
        ObjectSerializer.parse(await response.body.text(), contentType),
        "Exception",
        "",
      ) as Exception;
      throw new ApiException<Exception>(
        response.httpStatusCode,
        "Too many requests. Please try again later.",
        body,
        response.headers,
      );
    }
    if (isCodeInRange("500", response.httpStatusCode)) {
      const body: Exception = ObjectSerializer.deserialize(
        ObjectSerializer.parse(await response.body.text(), contentType),
        "Exception",
        "",
      ) as Exception;
      throw new ApiException<Exception>(
        response.httpStatusCode,
        "A server error occurred.",
        body,
        response.headers,
      );
    }

    // Work around for missing responses in specification, e.g. for petstore.yaml
    if (response.httpStatusCode >= 200 && response.httpStatusCode <= 299) {
      const body: void = ObjectSerializer.deserialize(
        ObjectSerializer.parse(await response.body.text(), contentType),
        "void",
        "",
      ) as void;
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
