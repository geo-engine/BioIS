# .CapabilitiesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**api**](CapabilitiesApi.md#api) | **GET** /api | API definition
[**conformance**](CapabilitiesApi.md#conformance) | **GET** /conformance | API conformance definition
[**root**](CapabilitiesApi.md#root) | **GET** / | Landing page


# **api**
> { [key: string]: any; } api()


### Example


```typescript
import { createConfiguration, CapabilitiesApi } from '';

const configuration = createConfiguration();
const apiInstance = new CapabilitiesApi(configuration);

const request = {};

const data = await apiInstance.api(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters
This endpoint does not need any parameter.


### Return type

**{ [key: string]: any; }**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The Open API definition. |  -  |
**500** | A server error occurred. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **conformance**
> Conformance conformance()

A list of all conformance classes specified in a standard that the server conforms to.

### Example


```typescript
import { createConfiguration, CapabilitiesApi } from '';

const configuration = createConfiguration();
const apiInstance = new CapabilitiesApi(configuration);

const request = {};

const data = await apiInstance.conformance(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters
This endpoint does not need any parameter.


### Return type

**Conformance**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The URIs of all conformance classes supported by the server.   To support \&quot;generic\&quot; clients that want to access multiple OGC API Features implementations - and not \&quot;just\&quot; a specific API / server, the server declares the conformance classes it implements and conforms to |  -  |
**400** | General HTTP error response. |  -  |
**500** | A server error occurred. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **root**
> LandingPage root()

The landing page provides links to the API definition and the conformance statements for this API.

### Example


```typescript
import { createConfiguration, CapabilitiesApi } from '';

const configuration = createConfiguration();
const apiInstance = new CapabilitiesApi(configuration);

const request = {};

const data = await apiInstance.root(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters
This endpoint does not need any parameter.


### Return type

**LandingPage**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The landing page provides links to the API definition (link relations &#x60;service-desc&#x60; and &#x60;service-doc&#x60;), and the Conformance declaration (path &#x60;/conformance&#x60;, link relation &#x60;conformance&#x60;). |  -  |
**400** | General HTTP error response. |  -  |
**500** | A server error occurred. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)


