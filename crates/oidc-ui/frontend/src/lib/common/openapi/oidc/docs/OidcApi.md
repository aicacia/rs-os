# OidcApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**authorize**](OidcApi.md#authorize) | **GET** /oidc/api/authorize |  |
| [**endSession**](OidcApi.md#endsession) | **GET** /oidc/api/end-session |  |
| [**jwks**](OidcApi.md#jwks) | **GET** /oidc/api/.well-known/jwks.json |  |
| [**openidConfiguration**](OidcApi.md#openidconfiguration) | **GET** /oidc/api/.well-known/openid-configuration |  |
| [**postAuthorize**](OidcApi.md#postauthorize) | **POST** /oidc/api/authorize |  |
| [**registerClient**](OidcApi.md#registerclient) | **POST** /oidc/api/register-client |  |
| [**token**](OidcApi.md#token) | **POST** /oidc/api/token |  |



## authorize

> authorize(clientId, responseType, responseMode, scope, redirectUri, state, nonce)



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { AuthorizeRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new OidcApi();

  const body = {
    // string
    clientId: clientId_example,
    // ResponseType
    responseType: ...,
    // ResponseMode
    responseMode: ...,
    // string
    scope: scope_example,
    // string
    redirectUri: redirectUri_example,
    // string (optional)
    state: state_example,
    // string (optional)
    nonce: nonce_example,
  } satisfies AuthorizeRequest;

  try {
    const data = await api.authorize(body);
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
| **responseType** | `ResponseType` |  | [Defaults to `undefined`] [Enum: none, code, token, id_token, code token, code id_token, id_token token, code id_token token] |
| **responseMode** | `ResponseMode` |  | [Defaults to `undefined`] [Enum: query, fragment, form_post, web_message] |
| **scope** | `string` |  | [Defaults to `undefined`] |
| **redirectUri** | `string` |  | [Defaults to `undefined`] |
| **state** | `string` |  | [Optional] [Defaults to `undefined`] |
| **nonce** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **302** | Redirect |  -  |
| **400** | Application Error |  -  |
| **401** | Application Error |  -  |
| **403** | Application Error |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## endSession

> endSession(postLogoutRedirectUri, clientId, idTokenHint)



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { EndSessionRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new OidcApi();

  const body = {
    // string
    postLogoutRedirectUri: postLogoutRedirectUri_example,
    // string (optional)
    clientId: clientId_example,
    // string (optional)
    idTokenHint: idTokenHint_example,
  } satisfies EndSessionRequest;

  try {
    const data = await api.endSession(body);
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
| **postLogoutRedirectUri** | `string` |  | [Defaults to `undefined`] |
| **clientId** | `string` |  | [Optional] [Defaults to `undefined`] |
| **idTokenHint** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **204** | Session ended |  -  |
| **401** | Unauthorized Error |  -  |
| **403** | Forbiddon Error |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


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


## postAuthorize

> postAuthorize(authorizeRequest)



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { PostAuthorizeRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new OidcApi();

  const body = {
    // AuthorizeRequest
    authorizeRequest: ...,
  } satisfies PostAuthorizeRequest;

  try {
    const data = await api.postAuthorize(body);
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
| **authorizeRequest** | [AuthorizeRequest](AuthorizeRequest.md) |  | |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **302** | Redirect |  -  |
| **400** | Application Error |  -  |
| **401** | Application Error |  -  |
| **403** | Application Error |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## registerClient

> Client registerClient(clientRegisterRequest)



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { RegisterClientRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OidcApi(config);

  const body = {
    // ClientRegisterRequest
    clientRegisterRequest: ...,
  } satisfies RegisterClientRequest;

  try {
    const data = await api.registerClient(body);
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
| **clientRegisterRequest** | [ClientRegisterRequest](ClientRegisterRequest.md) |  | |

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
| **200** | Client registation updated |  -  |
| **201** | Client registered |  -  |
| **401** | Application Error |  -  |
| **403** | Application Error |  -  |
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

