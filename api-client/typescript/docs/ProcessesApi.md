# .ProcessesApi

All URIs are relative to *http://localhost*

Method | HTTP request | Description
------------- | ------------- | -------------
[**_delete**](ProcessesApi.md#_delete) | **DELETE** /jobs/{jobId} | Cancel a job execution, remove finished job
[**executeNdvi**](ProcessesApi.md#executeNdvi) | **POST** /processes/ndvi/execution | 
[**execution**](ProcessesApi.md#execution) | **POST** /processes/{processID}/execution | Execute a process
[**jobs**](ProcessesApi.md#jobs) | **GET** /jobs | Retrieve the list of jobs
[**process**](ProcessesApi.md#process) | **GET** /processes/{processID} | Retrieve a processes description
[**processes**](ProcessesApi.md#processes) | **GET** /processes | Retrieve the list of available processes
[**results**](ProcessesApi.md#results) | **GET** /jobs/{jobId}/results | Retrieve the result(s) of a job
[**status**](ProcessesApi.md#status) | **GET** /jobs/{jobId} | Retrieve the status of a job


# **_delete**
> StatusInfo _delete()

Cancel a job execution and remove it from the jobs list.  For more information, see [Section 13](https://docs.ogc.org/is/18-062/18-062.html#Dismiss).

### Example


```typescript
import { createConfiguration, ProcessesApi } from '';
import type { ProcessesApiDeleteRequest } from '';

const configuration = createConfiguration();
const apiInstance = new ProcessesApi(configuration);

const request: ProcessesApiDeleteRequest = {
  
  jobId: "jobId_example",
};

const data = await apiInstance._delete(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **jobId** | [**string**] |  | defaults to undefined


### Return type

**StatusInfo**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The status of a job |  -  |
**404** | The requested URI was not found. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **executeNdvi**
> NDVIProcessOutputs executeNdvi(nDVIProcessInputs)


### Example


```typescript
import { createConfiguration, ProcessesApi } from '';
import type { ProcessesApiExecuteNdviRequest } from '';

const configuration = createConfiguration();
const apiInstance = new ProcessesApi(configuration);

const request: ProcessesApiExecuteNdviRequest = {
  
  nDVIProcessInputs: {
    coordinate: {
      mediaType: "application/geo+json",
      value: {
        coordinates: [
          3.14,
        ],
        type: "Point",
      },
    },
    month: 0,
    year: 0,
  },
};

const data = await apiInstance.executeNdvi(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **nDVIProcessInputs** | **NDVIProcessInputs**|  |


### Return type

**NDVIProcessOutputs**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** |  |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **execution**
> { [key: string]: InlineOrRefData; } execution(execute)

Create a new job.  For more information, see [Section 7.11](https://docs.ogc.org/is/18-062/18-062.html#sc_create_job).

### Example


```typescript
import { createConfiguration, ProcessesApi } from '';
import type { ProcessesApiExecutionRequest } from '';

const configuration = createConfiguration();
const apiInstance = new ProcessesApi(configuration);

const request: ProcessesApiExecutionRequest = {
  
  processID: "processID_example",
  
  execute: {
    inputs: {
      "key": null,
    },
    outputs: {
      "key": {
        format: {
          encoding: "encoding_example",
          mediaType: "mediaType_example",
          schema: null,
        },
        transmissionMode: "value",
      },
    },
    response: "raw",
    subscriber: {
      failedUri: "failedUri_example",
      inProgressUri: "inProgressUri_example",
      successUri: "successUri_example",
    },
  },
};

const data = await apiInstance.execution(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **execute** | **Execute**|  |
 **processID** | [**string**] |  | defaults to undefined


### Return type

**{ [key: string]: InlineOrRefData; }**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: application/json
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Result of synchronous execution |  -  |
**404** | The requested URI was not found. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **jobs**
> JobList jobs()

For more information, see [Section 11](https://docs.ogc.org/is/18-062/18-062.html#sc_job_list).

### Example


```typescript
import { createConfiguration, ProcessesApi } from '';

const configuration = createConfiguration();
const apiInstance = new ProcessesApi(configuration);

const request = {};

const data = await apiInstance.jobs(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters
This endpoint does not need any parameter.


### Return type

**JobList**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | A list of jobs for this process. |  -  |
**404** | The requested URI was not found. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **process**
> Process process()

The process description contains information about inputs and outputs and a link to the execution-endpoint for the process. The Core does not mandate the use of a specific process description to specify the interface of a process. That said, the Core requirements class makes the following recommendation:  Implementations SHOULD consider supporting the OGC process description.  For more information, see Section 7.10.

### Example


```typescript
import { createConfiguration, ProcessesApi } from '';
import type { ProcessesApiProcessRequest } from '';

const configuration = createConfiguration();
const apiInstance = new ProcessesApi(configuration);

const request: ProcessesApiProcessRequest = {
  
  processID: "processID_example",
};

const data = await apiInstance.process(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **processID** | [**string**] |  | defaults to undefined


### Return type

**Process**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | A process description |  -  |
**404** | The requested URI was not found. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **processes**
> ProcessList processes()

The list of processes contains a summary of each process the OGC API - Processes offers, including the link to a more detailed description of the process.  For more information, see [Section 7.9](https://docs.ogc.org/is/18-062/18-062.html#sc_process_list).

### Example


```typescript
import { createConfiguration, ProcessesApi } from '';

const configuration = createConfiguration();
const apiInstance = new ProcessesApi(configuration);

const request = {};

const data = await apiInstance.processes(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters
This endpoint does not need any parameter.


### Return type

**ProcessList**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | Information about the available processe |  -  |
**500** | A server error occurred. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **results**
> { [key: string]: InlineOrRefData; } results()

Lists available results of a job. In case of a failure, lists exceptions instead.  For more information, see [Section 7.13](https://docs.ogc.org/is/18-062r2/18-062r2.html#sc_retrieve_job_results).  

### Example


```typescript
import { createConfiguration, ProcessesApi } from '';
import type { ProcessesApiResultsRequest } from '';

const configuration = createConfiguration();
const apiInstance = new ProcessesApi(configuration);

const request: ProcessesApiResultsRequest = {
  
  jobId: "jobId_example",
};

const data = await apiInstance.results(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **jobId** | [**string**] |  | defaults to undefined


### Return type

**{ [key: string]: InlineOrRefData; }**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The results of a job |  -  |
**404** | The requested URI was not found. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)

# **status**
> StatusInfo status()

Shows the status of a job.  For more information, see [Section 7.12](https://docs.ogc.org/is/18-062/18-062.html#sc_retrieve_status_info).

### Example


```typescript
import { createConfiguration, ProcessesApi } from '';
import type { ProcessesApiStatusRequest } from '';

const configuration = createConfiguration();
const apiInstance = new ProcessesApi(configuration);

const request: ProcessesApiStatusRequest = {
  
  jobId: "jobId_example",
};

const data = await apiInstance.status(request);
console.log('API called successfully. Returned data:', data);
```


### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **jobId** | [**string**] |  | defaults to undefined


### Return type

**StatusInfo**

### Authorization

No authorization required

### HTTP request headers

 - **Content-Type**: Not defined
 - **Accept**: application/json


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
**200** | The status of a job |  -  |
**404** | The requested URI was not found. |  -  |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)


