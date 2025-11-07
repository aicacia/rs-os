# CurrentUserApi

All URIs are relative to *http://0.0.0.0:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**currentUser**](CurrentUserApi.md#currentuser) | **GET** /oidc/api/current-user |  |



## currentUser

> User currentUser()



### Example

```ts
import {
  Configuration,
  CurrentUserApi,
} from '';
import type { CurrentUserRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CurrentUserApi(config);

  try {
    const data = await api.currentUser();
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

[**User**](User.md)

### Authorization

[Authorization](../README.md#Authorization)

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

