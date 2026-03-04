# .UserApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**authHandler**](UserApi.md#authHandler) | **POST** /auth/accessTokenLogin | 
[**authRequestUrlHandler**](UserApi.md#authRequestUrlHandler) | **GET** /auth/authenticationRequestUrl | Generates a URL for initiating the OIDC code flow, which the frontend can use to redirect the user to the identity provider\&#39;s login page.


# **authHandler**
> UserSession authHandler(authCodeResponse)


### Example


```typescript
import { createConfiguration, UserApi } from '';
import type { UserApiAuthHandlerRequest } from '';

const configuration = createConfiguration();
const apiInstance = new UserApi(configuration);

const request: UserApiAuthHandlerRequest = {
    // The URI to which the identity provider should redirect after successful authentication.
  redirectUri: "redirectUri_example",
  
  authCodeResponse: {
    code: "code_example",
    sessionState: "sessionState_example",
    state: "state_example",
  },
};

const data = await apiInstance.authHandler(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **authCodeResponse** | **AuthCodeResponse**|  |
 **redirectUri** | [**string**] | The URI to which the identity provider should redirect after successful authentication. | defaults to undefined


### Return type

**UserSession**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The OIDC login flow was successful, and a user session has been created. |  -  |
**500** | A server error occurred. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **authRequestUrlHandler**
> string authRequestUrlHandler()


### Example


```typescript
import { createConfiguration, UserApi } from '';
import type { UserApiAuthRequestUrlHandlerRequest } from '';

const configuration = createConfiguration();
const apiInstance = new UserApi(configuration);

const request: UserApiAuthRequestUrlHandlerRequest = {
    // The URI to which the identity provider should redirect after successful authentication.
  redirectUri: "redirectUri_example",
};

const data = await apiInstance.authRequestUrlHandler(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **redirectUri** | [**string**] | The URI to which the identity provider should redirect after successful authentication. | defaults to undefined


### Return type

**string**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: text/plain, application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | A URL for initiating the OIDC code flow. |  -  |
**500** | A server error occurred. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)


