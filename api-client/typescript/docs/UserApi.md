# .UserApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**authHandler**](UserApi.md#authHandler) | **POST** /auth | 


# **authHandler**
> UserSession authHandler(authCodeResponse)


### Example


```typescript
import { createConfiguration, UserApi } from '';
import type { UserApiAuthHandlerRequest } from '';

const configuration = createConfiguration();
const apiInstance = new UserApi(configuration);

const request: UserApiAuthHandlerRequest = {
  
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


