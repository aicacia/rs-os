# JwkApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**jwkById**](JwkApi.md#jwkbyid) | **GET** /oidc/api/jwks/{kid} |  |



## jwkById

> JWK jwkById(kid)



### Example

```ts
import {
  Configuration,
  JwkApi,
} from '';
import type { JwkByIdRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new JwkApi();

  const body = {
    // number
    kid: 789,
  } satisfies JwkByIdRequest;

  try {
    const data = await api.jwkById(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters


| Name | Type | Description  | Notes |
|------------- | ------------- | ------------- | -------------|
| **kid** | `number` |  | [Defaults to `undefined`] |

### Return type

[**JWK**](JWK.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** |  |  -  |
| **400** |  |  -  |
| **401** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

