# ClientApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**clientByClientId**](ClientApi.md#clientbyclientid) | **GET** /admin/api/clients/{client_id} |  |
| [**clientCreate**](ClientApi.md#clientcreate) | **POST** /admin/api/clients |  |
| [**clientDelete**](ClientApi.md#clientdelete) | **DELETE** /admin/api/clients/{client_id} |  |
| [**clientList**](ClientApi.md#clientlist) | **GET** /admin/api/clients |  |
| [**clientUpdate**](ClientApi.md#clientupdate) | **PUT** /admin/api/clients/{client_id} |  |



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
    // number
    clientId: 789,
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
| **clientId** | `number` |  | [Defaults to `undefined`] |

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


## clientCreate

> Client clientCreate(clientUpsertRequest)



### Example

```ts
import {
  Configuration,
  ClientApi,
} from '';
import type { ClientCreateRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ClientApi(config);

  const body = {
    // ClientUpsertRequest
    clientUpsertRequest: ...,
  } satisfies ClientCreateRequest;

  try {
    const data = await api.clientCreate(body);
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
| **clientUpsertRequest** | [ClientUpsertRequest](ClientUpsertRequest.md) |  | |

### Return type

[**Client**](Client.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Client updated |  -  |
| **201** | Client created |  -  |
| **400** | Application Error |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## clientDelete

> clientDelete(clientId)



### Example

```ts
import {
  Configuration,
  ClientApi,
} from '';
import type { ClientDeleteRequest } from '';

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
  } satisfies ClientDeleteRequest;

  try {
    const data = await api.clientDelete(body);
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
| **204** | Client deleted |  -  |
| **400** | Application Error |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## clientList

> Array&lt;Client&gt; clientList()



### Example

```ts
import {
  Configuration,
  ClientApi,
} from '';
import type { ClientListRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ClientApi(config);

  try {
    const data = await api.clientList();
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

[**Array&lt;Client&gt;**](Client.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** |  |  -  |
| **401** |  |  -  |
| **403** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## clientUpdate

> Client clientUpdate(clientId, clientUpsertRequest)



### Example

```ts
import {
  Configuration,
  ClientApi,
} from '';
import type { ClientUpdateRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ClientApi(config);

  const body = {
    // number
    clientId: 789,
    // ClientUpsertRequest
    clientUpsertRequest: ...,
  } satisfies ClientUpdateRequest;

  try {
    const data = await api.clientUpdate(body);
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
| **clientId** | `number` |  | [Defaults to `undefined`] |
| **clientUpsertRequest** | [ClientUpsertRequest](ClientUpsertRequest.md) |  | |

### Return type

[**Client**](Client.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Client updated |  -  |
| **400** | Application Error |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

