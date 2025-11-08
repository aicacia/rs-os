# ClientApi

All URIs are relative to *http://0.0.0.0:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**clientByClientId**](ClientApi.md#clientbyclientid) | **GET** /oidc/api/clients/{client_id} |  |
| [**createClient**](ClientApi.md#createclient) | **POST** /oidc/api/clients |  |



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
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## createClient

> Client createClient()



### Example

```ts
import {
  Configuration,
  ClientApi,
} from '';
import type { CreateClientRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new ClientApi(config);

  try {
    const data = await api.createClient();
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

[**Client**](Client.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** |  |  -  |
| **400** |  |  -  |
| **401** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

