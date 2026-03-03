// TODO: better import syntax?
import {BaseAPIRequestFactory, RequiredError, COLLECTION_FORMATS} from './baseapi';
import {Configuration} from '../configuration';
import {RequestContext, HttpMethod, ResponseContext, HttpFile, HttpInfo} from '../http/http';
import {ObjectSerializer} from '../models/ObjectSerializer';
import {ApiException} from './exception';
import {canConsumeForm, isCodeInRange} from '../util';
import {SecurityAuthentication} from '../auth/auth';


import { AuthCodeResponse } from '../models/AuthCodeResponse';
import { Exception } from '../models/Exception';
import { UserSession } from '../models/UserSession';

/**
 * no description
 */
export class UserApiRequestFactory extends BaseAPIRequestFactory {

    /**
     * @param redirectUri The URI to which the identity provider should redirect after successful authentication.
     * @param authCodeResponse 
     */
    public async authHandler(redirectUri: string, authCodeResponse: AuthCodeResponse, _options?: Configuration): Promise<RequestContext> {
        let _config = _options || this.configuration;

        // verify required parameter 'redirectUri' is not null or undefined
        if (redirectUri === null || redirectUri === undefined) {
            throw new RequiredError("UserApi", "authHandler", "redirectUri");
        }


        // verify required parameter 'authCodeResponse' is not null or undefined
        if (authCodeResponse === null || authCodeResponse === undefined) {
            throw new RequiredError("UserApi", "authHandler", "authCodeResponse");
        }


        // Path Params
        const localVarPath = '/auth/accessTokenLogin';

        // Make Request Context
        const requestContext = _config.baseServer.makeRequestContext(localVarPath, HttpMethod.POST);
        requestContext.setHeaderParam("Accept", "application/json, */*;q=0.8")

        // Query Params
        if (redirectUri !== undefined) {
            requestContext.setQueryParam("redirectUri", ObjectSerializer.serialize(redirectUri, "string", "uri"));
        }


        // Body Params
        const contentType = ObjectSerializer.getPreferredMediaType([
            "application/json"
        ]);
        requestContext.setHeaderParam("Content-Type", contentType);
        const serializedBody = ObjectSerializer.stringify(
            ObjectSerializer.serialize(authCodeResponse, "AuthCodeResponse", ""),
            contentType
        );
        requestContext.setBody(serializedBody);

        
        const defaultAuth: SecurityAuthentication | undefined = _config?.authMethods?.default
        if (defaultAuth?.applySecurityAuthentication) {
            await defaultAuth?.applySecurityAuthentication(requestContext);
        }

        return requestContext;
    }

    /**
     * Generates a URL for initiating the OIDC code flow, which the frontend can use to redirect the user to the identity provider\'s login page.
     * @param redirectUri The URI to which the identity provider should redirect after successful authentication.
     */
    public async authRequestUrlHandler(redirectUri: string, _options?: Configuration): Promise<RequestContext> {
        let _config = _options || this.configuration;

        // verify required parameter 'redirectUri' is not null or undefined
        if (redirectUri === null || redirectUri === undefined) {
            throw new RequiredError("UserApi", "authRequestUrlHandler", "redirectUri");
        }


        // Path Params
        const localVarPath = '/auth/authenticationRequestUrl';

        // Make Request Context
        const requestContext = _config.baseServer.makeRequestContext(localVarPath, HttpMethod.GET);
        requestContext.setHeaderParam("Accept", "application/json, */*;q=0.8")

        // Query Params
        if (redirectUri !== undefined) {
            requestContext.setQueryParam("redirectUri", ObjectSerializer.serialize(redirectUri, "string", "uri"));
        }


        
        const defaultAuth: SecurityAuthentication | undefined = _config?.authMethods?.default
        if (defaultAuth?.applySecurityAuthentication) {
            await defaultAuth?.applySecurityAuthentication(requestContext);
        }

        return requestContext;
    }

}

export class UserApiResponseProcessor {

    /**
     * Unwraps the actual response sent by the server from the response context and deserializes the response content
     * to the expected objects
     *
     * @params response Response returned by the server for a request to authHandler
     * @throws ApiException if the response code was not in [200, 299]
     */
     public async authHandlerWithHttpInfo(response: ResponseContext): Promise<HttpInfo<UserSession >> {
        const contentType = ObjectSerializer.normalizeMediaType(response.headers["content-type"]);
        if (isCodeInRange("200", response.httpStatusCode)) {
            const body: UserSession = ObjectSerializer.deserialize(
                ObjectSerializer.parse(await response.body.text(), contentType),
                "UserSession", ""
            ) as UserSession;
            return new HttpInfo(response.httpStatusCode, response.headers, response.body, body);
        }
        if (isCodeInRange("500", response.httpStatusCode)) {
            const body: Exception = ObjectSerializer.deserialize(
                ObjectSerializer.parse(await response.body.text(), contentType),
                "Exception", ""
            ) as Exception;
            throw new ApiException<Exception>(response.httpStatusCode, "A server error occurred.", body, response.headers);
        }

        // Work around for missing responses in specification, e.g. for petstore.yaml
        if (response.httpStatusCode >= 200 && response.httpStatusCode <= 299) {
            const body: UserSession = ObjectSerializer.deserialize(
                ObjectSerializer.parse(await response.body.text(), contentType),
                "UserSession", ""
            ) as UserSession;
            return new HttpInfo(response.httpStatusCode, response.headers, response.body, body);
        }

        throw new ApiException<string | Blob | undefined>(response.httpStatusCode, "Unknown API Status Code!", await response.getBodyAsAny(), response.headers);
    }

    /**
     * Unwraps the actual response sent by the server from the response context and deserializes the response content
     * to the expected objects
     *
     * @params response Response returned by the server for a request to authRequestUrlHandler
     * @throws ApiException if the response code was not in [200, 299]
     */
     public async authRequestUrlHandlerWithHttpInfo(response: ResponseContext): Promise<HttpInfo<string >> {
        const contentType = ObjectSerializer.normalizeMediaType(response.headers["content-type"]);
        if (isCodeInRange("200", response.httpStatusCode)) {
            const body: string = ObjectSerializer.deserialize(
                ObjectSerializer.parse(await response.body.text(), contentType),
                "string", "uri"
            ) as string;
            return new HttpInfo(response.httpStatusCode, response.headers, response.body, body);
        }
        if (isCodeInRange("500", response.httpStatusCode)) {
            const body: Exception = ObjectSerializer.deserialize(
                ObjectSerializer.parse(await response.body.text(), contentType),
                "Exception", "uri"
            ) as Exception;
            throw new ApiException<Exception>(response.httpStatusCode, "A server error occurred.", body, response.headers);
        }

        // Work around for missing responses in specification, e.g. for petstore.yaml
        if (response.httpStatusCode >= 200 && response.httpStatusCode <= 299) {
            const body: string = ObjectSerializer.deserialize(
                ObjectSerializer.parse(await response.body.text(), contentType),
                "string", "uri"
            ) as string;
            return new HttpInfo(response.httpStatusCode, response.headers, response.body, body);
        }

        throw new ApiException<string | Blob | undefined>(response.httpStatusCode, "Unknown API Status Code!", await response.getBodyAsAny(), response.headers);
    }

}
