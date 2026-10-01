# .ContactApi

All URIs are relative to *http://localhost*

| Method                                           | HTTP request      | Description                                                     |
| ------------------------------------------------ | ----------------- | --------------------------------------------------------------- |
| [**submitContact**](ContactApi.md#submitContact) | **POST** /contact | Sends a request for pilot access to the &#x60;BioIS&#x60; team. |

# **submitContact**

> void submitContact(contactRequest)

### Example

```typescript
import { createConfiguration, ContactApi } from "";
import type { ContactApiSubmitContactRequest } from "";

const configuration = createConfiguration();
const apiInstance = new ContactApi(configuration);

const request: ContactApiSubmitContactRequest = {
  contactRequest: {
    name: "name_example",
    email: "email_example",
    organization: "organization_example",
    role: "largeCorporate",
    message: "message_example",
    privacyConsent: true,
    website: "website_example",
  },
};

const data = await apiInstance.submitContact(request);
console.log("API called successfully. Returned data:", data);
```

### Parameters

| Name               | Type               | Description | Notes |
| ------------------ | ------------------ | ----------- | ----- |
| **contactRequest** | **ContactRequest** |             |

### Return type

**void**

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

### HTTP response details

| Status code | Description                                | Response headers |
| ----------- | ------------------------------------------ | ---------------- |
| **204**     | The request was sent to the BioIS team.    | -                |
| **400**     | The request is invalid.                    | -                |
| **429**     | Too many requests. Please try again later. | -                |
| **500**     | A server error occurred.                   | -                |

[[Back to top]](#) [[Back to API list]](README.md#documentation-for-api-endpoints) [[Back to Model list]](README.md#documentation-for-models) [[Back to README]](README.md)
