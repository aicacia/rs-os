# OidcApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**approveClientForUser**](OidcApi.md#approveclientforuser) | **POST** /oidc/api/approve-client |  |
| [**authorize**](OidcApi.md#authorize) | **GET** /oidc/api/authorize |  |
| [**authorizeClient**](OidcApi.md#authorizeclient) | **POST** /oidc/api/authorize-client |  |
| [**client**](OidcApi.md#client) | **GET** /oidc/api/client |  |
| [**deviceAuthorize**](OidcApi.md#deviceauthorize) | **POST** /oidc/api/device-authorize |  |
| [**endSession**](OidcApi.md#endsession) | **GET** /oidc/api/end-session |  |
| [**introspect**](OidcApi.md#introspect) | **POST** /oidc/api/introspect |  |
| [**isClientAllowedForUser**](OidcApi.md#isclientallowedforuser) | **GET** /oidc/api/client-allowed |  |
| [**jwks**](OidcApi.md#jwks) | **GET** /oidc/api/.well-known/jwks.json |  |
| [**openidConfiguration**](OidcApi.md#openidconfiguration) | **GET** /oidc/api/.well-known/openid-configuration |  |
| [**postAuthorize**](OidcApi.md#postauthorize) | **POST** /oidc/api/authorize |  |
| [**registerClient**](OidcApi.md#registerclient) | **POST** /oidc/api/register-client |  |
| [**revoke**](OidcApi.md#revoke) | **POST** /oidc/api/revoke |  |
| [**token**](OidcApi.md#token) | **POST** /oidc/api/token |  |
| [**userInfo**](OidcApi.md#userinfo) | **GET** /oidc/api/user-info |  |



## approveClientForUser

> approveClientForUser(clientId)



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { ApproveClientForUserRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OidcApi(config);

  const body = {
    // string
    clientId: clientId_example,
  } satisfies ApproveClientForUserRequest;

  try {
    const data = await api.approveClientForUser(body);
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


## authorize

> authorize(clientId, responseType, responseMode, scope, redirectUri, state, nonce, registration, codeChallenge, codeChallengeMethod)



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
    // string (optional)
    registration: registration_example,
    // string (optional)
    codeChallenge: codeChallenge_example,
    // string (optional)
    codeChallengeMethod: codeChallengeMethod_example,
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
| **registration** | `string` |  | [Optional] [Defaults to `undefined`] |
| **codeChallenge** | `string` |  | [Optional] [Defaults to `undefined`] |
| **codeChallengeMethod** | `string` |  | [Optional] [Defaults to `undefined`] |

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
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## authorizeClient

> ClientAuthorization authorizeClient(clientAuthorizeRequest)



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { AuthorizeClientRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OidcApi(config);

  const body = {
    // ClientAuthorizeRequest
    clientAuthorizeRequest: ...,
  } satisfies AuthorizeClientRequest;

  try {
    const data = await api.authorizeClient(body);
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
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## client

> Client client(clientId)



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { ClientRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OidcApi(config);

  const body = {
    // string
    clientId: clientId_example,
  } satisfies ClientRequest;

  try {
    const data = await api.client(body);
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
| **200** | Client fetched |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **404** | Not Found |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## deviceAuthorize

> deviceAuthorize()



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { DeviceAuthorizeRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new OidcApi();

  try {
    const data = await api.deviceAuthorize();
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

- **Content-Type**: `application/x-www-form-urlencoded`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Device authorization response |  -  |
| **400** | Invalid request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## endSession

> endSession(postLogoutRedirectUri, clientId, idTokenHint, state)



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
    // string (optional)
    state: state_example,
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
| **state** | `string` |  | [Optional] [Defaults to `undefined`] |

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
| **302** | Redirect to post_logout_redirect_uri |  -  |
| **400** | Bad Request Error |  -  |
| **401** | Unauthorized Error |  -  |
| **403** | Forbiddon Error |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## introspect

> BasicClaims introspect()



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { IntrospectRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OidcApi(config);

  try {
    const data = await api.introspect();
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

[**BasicClaims**](BasicClaims.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: `application/x-www-form-urlencoded`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Token introspection result |  -  |
| **400** | Invalid request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## isClientAllowedForUser

> ClientAllowed isClientAllowedForUser(clientId, scope)



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { IsClientAllowedForUserRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OidcApi(config);

  const body = {
    // string
    clientId: clientId_example,
    // string (optional)
    scope: scope_example,
  } satisfies IsClientAllowedForUserRequest;

  try {
    const data = await api.isClientAllowedForUser(body);
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
| **scope** | `string` |  | [Optional] [Defaults to `undefined`] |

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
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
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
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## revoke

> revoke(token, clientAssertion, clientAssertionType, clientId, clientSecret, tokenTypeHint)



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { RevokeRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const api = new OidcApi();

  const body = {
    // string
    token: token_example,
    // string (optional)
    clientAssertion: clientAssertion_example,
    // string (optional)
    clientAssertionType: clientAssertionType_example,
    // string (optional)
    clientId: clientId_example,
    // string (optional)
    clientSecret: clientSecret_example,
    // string (optional)
    tokenTypeHint: tokenTypeHint_example,
  } satisfies RevokeRequest;

  try {
    const data = await api.revoke(body);
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
| **token** | `string` |  | [Defaults to `undefined`] |
| **clientAssertion** | `string` |  | [Optional] [Defaults to `undefined`] |
| **clientAssertionType** | `string` |  | [Optional] [Defaults to `undefined`] |
| **clientId** | `string` |  | [Optional] [Defaults to `undefined`] |
| **clientSecret** | `string` |  | [Optional] [Defaults to `undefined`] |
| **tokenTypeHint** | `string` |  | [Optional] [Defaults to `undefined`] |

### Return type

`void` (Empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: `application/x-www-form-urlencoded`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **204** | Token revoked |  -  |
| **400** | Invalid request |  -  |
| **401** | Unauthorized |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## token

> Token token(clientAssertion, clientAssertionType, clientId, clientSecret, password, scope, username, grantType, refreshToken, code, codeVerifier)



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
    clientAssertion: clientAssertion_example,
    // string (optional)
    clientAssertionType: clientAssertionType_example,
    // string (optional)
    clientId: clientId_example,
    // string (optional)
    clientSecret: clientSecret_example,
    // string (optional)
    password: password_example,
    // string (optional)
    scope: scope_example,
    // string (optional)
    username: username_example,
    // string (optional)
    grantType: grantType_example,
    // string (optional)
    refreshToken: refreshToken_example,
    // string (optional)
    code: code_example,
    // string (optional)
    codeVerifier: codeVerifier_example,
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
| **clientAssertion** | `string` |  | [Optional] [Defaults to `undefined`] |
| **clientAssertionType** | `string` |  | [Optional] [Defaults to `undefined`] |
| **clientId** | `string` |  | [Optional] [Defaults to `undefined`] |
| **clientSecret** | `string` |  | [Optional] [Defaults to `undefined`] |
| **password** | `string` |  | [Optional] [Defaults to `undefined`] |
| **scope** | `string` |  | [Optional] [Defaults to `undefined`] |
| **username** | `string` |  | [Optional] [Defaults to `undefined`] |
| **grantType** | `authorization_code` |  | [Optional] [Defaults to `undefined`] [Enum: authorization_code] |
| **refreshToken** | `string` |  | [Optional] [Defaults to `undefined`] |
| **code** | `string` |  | [Optional] [Defaults to `undefined`] |
| **codeVerifier** | `string` |  | [Optional] [Defaults to `undefined`] |

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


## userInfo

> UserInfo userInfo()



### Example

```ts
import {
  Configuration,
  OidcApi,
} from '';
import type { UserInfoRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new OidcApi(config);

  try {
    const data = await api.userInfo();
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

[**UserInfo**](UserInfo.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **200** | Consented claims |  -  |
| **401** | Unauthorized |  -  |
| **403** | Forbidden |  -  |
| **500** | Application Error |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

