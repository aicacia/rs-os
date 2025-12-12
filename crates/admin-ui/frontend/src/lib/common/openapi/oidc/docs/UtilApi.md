# UtilApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**health**](UtilApi.md#health) | **GET** /oidc/api/health |  |
| [**version**](UtilApi.md#version) | **GET** /oidc/api/version |  |



## health

> Health health()



### Example

```ts
import {
  Configuration,
  UtilApi,
} from '';
import type { HealthRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new UtilApi();

  try {
    const data = await api.health();
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

[**Health**](Health.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Health check response |  -  |
| **500** | Health check response |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## version

> Version version()



### Example

```ts
import {
  Configuration,
  UtilApi,
} from '';
import type { VersionRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new UtilApi();

  try {
    const data = await api.version();
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

[**Version**](Version.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Version response |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

