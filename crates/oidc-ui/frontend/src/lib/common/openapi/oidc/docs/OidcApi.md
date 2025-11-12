# OidcApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**jwks**](OidcApi.md#jwks) | **GET** /oidc/api/.well-known/jwks.json |  |
| [**openidConfiguration**](OidcApi.md#openidconfiguration) | **GET** /oidc/api/.well-known/openid-configuration |  |
| [**token**](OidcApi.md#token) | **POST** /oidc/api/token |  |



## jwks

> JWKs jwks()



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { JwksRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new OidcApi();

  try {
    const data = await api.jwks();
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

[**JWKs**](JWKs.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | JSON Web Keys |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## openidConfiguration

> OpenIdConfiguration openidConfiguration()



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { OpenidConfigurationRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new OidcApi();

  try {
    const data = await api.openidConfiguration();
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

[**OpenIdConfiguration**](OpenIdConfiguration.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | OpenId Configuration |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## token

> Token token(clientId, scope, password, username, grantType, refreshToken, code)



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { TokenRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new OidcApi();

  const body = {
    // string (optional)
    clientId: clientId_example,
    // string (optional)
    scope: scope_example,
    // string (optional)
    password: password_example,
    // string (optional)
    username: username_example,
    // string (optional)
    grantType: grantType_example,
    // string (optional)
    refreshToken: refreshToken_example,
    // string (optional)
    code: code_example,
  } satisfies TokenRequest;

  try {
    const data = await api.token(body);
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
| **clientId** | `string` |  | [Optional] [Defaults to `undefined`] |
| **scope** | `string` |  | [Optional] [Defaults to `undefined`] |
| **password** | `string` |  | [Optional] [Defaults to `undefined`] |
| **username** | `string` |  | [Optional] [Defaults to `undefined`] |
| **grantType** | `authorization_code` |  | [Optional] [Defaults to `undefined`] [Enum: authorization_code] |
| **refreshToken** | `string` |  | [Optional] [Defaults to `undefined`] |
| **code** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

[**Token**](Token.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/x-www-form-urlencoded`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** | Token returned |  -  |
| **401** | Unauthorized Error |  -  |
| **403** | Forbiddon Error |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

