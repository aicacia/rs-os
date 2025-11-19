# ClientApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**clientAuthorize**](ClientApi.md#clientauthorizeoperation) | **POST** /oidc/api/client/{client_id}/authorize |  |
| [**clientByClientId**](ClientApi.md#clientbyclientid) | **GET** /oidc/api/clients/{client_id} |  |
| [**clientUserAllowed**](ClientApi.md#clientuserallowed) | **GET** /oidc/api/clients/{client_id}/allowed |  |
| [**clientUserApprove**](ClientApi.md#clientuserapprove) | **POST** /oidc/api/clients/{client_id}/approve |  |



## clientAuthorize

> ClientAuthorization clientAuthorize(clientId, clientAuthorizeRequest)



### Example

```ts
import {
  Configuration,
  ClientApi,
} from '';
import type { ClientAuthorizeOperationRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ClientApi(config);

  const body = {
    // string
    clientId: clientId_example,
    // ClientAuthorizeRequest
    clientAuthorizeRequest: ...,
  } satisfies ClientAuthorizeOperationRequest;

  try {
    const data = await api.clientAuthorize(body);
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
| **clientId** | `string` |  | [Defaults to `undefined`] |
| **clientAuthorizeRequest** | [ClientAuthorizeRequest](ClientAuthorizeRequest.md) |  | |

### Return type

[**ClientAuthorization**](ClientAuthorization.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Authorized |  -  |
| **400** | Application Error |  -  |
| **401** | Application Error |  -  |
| **403** | Application Error |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## clientByClientId

> Client clientByClientId(clientId)



### Example

```ts
import {
  Configuration,
  ClientApi,
} from '';
import type { ClientByClientIdRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ClientApi(config);

  const body = {
    // string
    clientId: clientId_example,
  } satisfies ClientByClientIdRequest;

  try {
    const data = await api.clientByClientId(body);
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
| **clientId** | `string` |  | [Defaults to `undefined`] |

### Return type

[**Client**](Client.md)

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
| **403** |  |  -  |
| **404** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## clientUserAllowed

> ClientAllowed clientUserAllowed(clientId)



### Example

```ts
import {
  Configuration,
  ClientApi,
} from '';
import type { ClientUserAllowedRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ClientApi(config);

  const body = {
    // string
    clientId: clientId_example,
  } satisfies ClientUserAllowedRequest;

  try {
    const data = await api.clientUserAllowed(body);
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
| **clientId** | `string` |  | [Defaults to `undefined`] |

### Return type

[**ClientAllowed**](ClientAllowed.md)

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
| **403** |  |  -  |
| **404** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## clientUserApprove

> clientUserApprove(clientId)



### Example

```ts
import {
  Configuration,
  ClientApi,
} from '';
import type { ClientUserApproveRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ClientApi(config);

  const body = {
    // string
    clientId: clientId_example,
  } satisfies ClientUserApproveRequest;

  try {
    const data = await api.clientUserApprove(body);
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
| **clientId** | `string` |  | [Defaults to `undefined`] |

### Return type

`void` (Empty response body)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **204** |  |  -  |
| **400** |  |  -  |
| **401** |  |  -  |
| **403** |  |  -  |
| **404** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

