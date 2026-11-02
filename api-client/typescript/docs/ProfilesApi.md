# .ProfilesApi

All URIs are relative to *http://localhost*

| Method                                                                            | HTTP request                                                | Description                                                                                                                                                    |
| --------------------------------------------------------------------------------- | ----------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [**climateRiskTableSchemaProfile**](ProfilesApi.md#climateRiskTableSchemaProfile) | **GET** /profiles/table-schema/climate-risk/1.0/schema.json | The profile of the &#x60;schema.biois&#x60; extension: a Table Schema extension that declares per-column display metadata for &#x60;BioIS&#x60; result tables. |

# **climateRiskTableSchemaProfile**

> string climateRiskTableSchemaProfile()

### Example

```typescript
import { createConfiguration, ProfilesApi } from "";

const configuration = createConfiguration();
const apiInstance = new ProfilesApi(configuration);

const request = {};

const data = await apiInstance.climateRiskTableSchemaProfile(request);
console.log("API called successfully. Returned data:", data);
```

### Parameters

This endpoint does not need any parameter.

### Return type

**string**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/schema+json

### HTTP response details

| Status code | Description                     | Response headers |
| ----------- | ------------------------------- | ---------------- |
| **200**     | The BioIS table schema profile. | -                |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)
