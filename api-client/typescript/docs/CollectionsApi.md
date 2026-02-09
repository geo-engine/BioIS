# .CollectionsApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create**](CollectionsApi.md#create) | **POST** /collections | Create new collection metadata
[**read**](CollectionsApi.md#read) | **GET** /collections/{collectionId} | Get collection metadata
[**remove**](CollectionsApi.md#remove) | **DELETE** /collections/{collectionId} | Delete collection metadata
[**update**](CollectionsApi.md#update) | **PUT** /collections/{collectionId} | Update collection metadata


# **create**
> void create(collection)


### Example


```typescript
import { createConfiguration, CollectionsApi } from '';
import type { CollectionsApiCreateRequest } from '';

const configuration = createConfiguration();
const apiInstance = new CollectionsApi(configuration);

const request: CollectionsApiCreateRequest = {
  
  collection: 
    key: null,
  ,
};

const data = await apiInstance.create(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **collection** | **Collection**|  |


### Return type

**void**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**201** | Created. |  * Location - URI of the newly added resource. <br>  |
**409** | Already exists. |  -  |
**500** | A server error occurred. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **read**
> Collection read()

Describe the feature collection with id `collectionId`

### Example


```typescript
import { createConfiguration, CollectionsApi } from '';
import type { CollectionsApiReadRequest } from '';

const configuration = createConfiguration();
const apiInstance = new CollectionsApi(configuration);

const request: CollectionsApiReadRequest = {
    // local identifier of a collection
  collectionId: "collectionId_example",
};

const data = await apiInstance.read(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **collectionId** | [**string**] | local identifier of a collection | defaults to undefined


### Return type

**Collection**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Information about the feature collection with id &#x60;collectionId&#x60;.   The response contains a link to the items in the collection (path &#x60;/collections/{collectionId}/items&#x60;, link relation &#x60;items&#x60;) as well as key information about the collection. This information includes:  * A local identifier for the collection that is unique for the dataset;  * A list of coordinate reference systems (CRS) in which geometries may be returned by the server. The first CRS is the default coordinate reference system (the default is always WGS 84 with axis order longitude/latitude);  * An optional title and description for the collection;  * An optional extent that can be used to provide an indication of the spatial and temporal extent of the collection - typically derived from the data;  * An optional indicator about the type of the items in the collection (the default value, if the indicator is not provided, is \&#39;feature\&#39;). |  -  |
**400** | General HTTP error response. |  -  |
**500** | A server error occurred. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **remove**
> void remove(collection)


### Example


```typescript
import { createConfiguration, CollectionsApi } from '';
import type { CollectionsApiRemoveRequest } from '';

const configuration = createConfiguration();
const apiInstance = new CollectionsApi(configuration);

const request: CollectionsApiRemoveRequest = {
    // local identifier of a collection
  collectionId: "collectionId_example",
  
  collection: 
    key: null,
  ,
};

const data = await apiInstance.remove(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **collection** | **Collection**|  |
 **collectionId** | [**string**] | local identifier of a collection | defaults to undefined


### Return type

**void**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Successfuly deleted, no content. |  -  |
**400** | General HTTP error response. |  -  |
**500** | A server error occurred. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **update**
> void update(collection)


### Example


```typescript
import { createConfiguration, CollectionsApi } from '';
import type { CollectionsApiUpdateRequest } from '';

const configuration = createConfiguration();
const apiInstance = new CollectionsApi(configuration);

const request: CollectionsApiUpdateRequest = {
    // local identifier of a collection
  collectionId: "collectionId_example",
  
  collection: 
    key: null,
  ,
};

const data = await apiInstance.update(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **collection** | **Collection**|  |
 **collectionId** | [**string**] | local identifier of a collection | defaults to undefined


### Return type

**void**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**204** | Successfuly updataed, no content. |  -  |
**400** | General HTTP error response. |  -  |
**500** | A server error occurred. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)


