# OpenapiApi

All URIs are relative to _http://localhost:3000_

| Method                                     | HTTP request                         | Description |
| ------------------------------------------ | ------------------------------------ | ----------- |
| [**getOpenapi**](OpenapiApi.md#getopenapi) | **GET** /oidc-admin/api/openapi.json |             |

## getOpenapi

> getOpenapi()

### Example

```ts
import { Configuration, OpenapiApi } from '';
import type { GetOpenapiRequest } from '';

async function example() {
	console.log('🚀 Testing  SDK...');
	const api = new OpenapiApi();

	try {
		const data = await api.getOpenapi();
		console.log(data);
	} catch (error) {
		console.error(error);
	}
}

// Run the test
example().catch(console.error);
```

### Parameters

This endpoint does not need any parameter.

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

### HTTP response details

| Status code | Description           | Response headers |
| ----------- | --------------------- | ---------------- |
| **200**     | OpenApi documenation  | -                |
| **500**     | Internal server error | -                |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)
