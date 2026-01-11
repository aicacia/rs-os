# UserPhoneNumberApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createUserPhoneNumberHandler**](UserPhoneNumberApi.md#createuserphonenumberhandler) | **POST** /oidc-admin/api/users/{user_id}/phone-numbers |  |
| [**deleteUserPhoneNumberHandler**](UserPhoneNumberApi.md#deleteuserphonenumberhandler) | **DELETE** /oidc-admin/api/users/{user_id}/phone-numbers/{phone_id} |  |
| [**getUserPhoneNumber**](UserPhoneNumberApi.md#getuserphonenumber) | **GET** /oidc-admin/api/users/{user_id}/phone-numbers/{phone_id} |  |
| [**listUserPhoneNumbers**](UserPhoneNumberApi.md#listuserphonenumbers) | **GET** /oidc-admin/api/users/{user_id}/phone-numbers |  |
| [**updateUserPhoneNumberHandler**](UserPhoneNumberApi.md#updateuserphonenumberhandler) | **PATCH** /oidc-admin/api/users/{user_id}/phone-numbers/{phone_id} |  |
| [**verifyUserPhoneNumberHandler**](UserPhoneNumberApi.md#verifyuserphonenumberhandler) | **POST** /oidc-admin/api/users/{user_id}/phone-numbers/{phone_id}/verify |  |



## createUserPhoneNumberHandler

> UserPhoneNumber createUserPhoneNumberHandler(userId, createUserPhoneNumberRequest)



### Example

```ts
import {
  Configuration,
  UserPhoneNumberApi,
} from '';
import type { CreateUserPhoneNumberHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserPhoneNumberApi(config);

  const body = {
    // string
    userId: userId_example,
    // CreateUserPhoneNumberRequest
    createUserPhoneNumberRequest: ...,
  } satisfies CreateUserPhoneNumberHandlerRequest;

  try {
    const data = await api.createUserPhoneNumberHandler(body);
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
| **userId** | `string` |  | [Defaults to `undefined`] |
| **createUserPhoneNumberRequest** | [CreateUserPhoneNumberRequest](CreateUserPhoneNumberRequest.md) |  | |

### Return type

[**UserPhoneNumber**](UserPhoneNumber.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **201** |  |  -  |
| **400** |  |  -  |
| **401** |  |  -  |
| **403** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## deleteUserPhoneNumberHandler

> deleteUserPhoneNumberHandler(userId, phoneId)



### Example

```ts
import {
  Configuration,
  UserPhoneNumberApi,
} from '';
import type { DeleteUserPhoneNumberHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserPhoneNumberApi(config);

  const body = {
    // string
    userId: userId_example,
    // string
    phoneId: phoneId_example,
  } satisfies DeleteUserPhoneNumberHandlerRequest;

  try {
    const data = await api.deleteUserPhoneNumberHandler(body);
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
| **userId** | `string` |  | [Defaults to `undefined`] |
| **phoneId** | `string` |  | [Defaults to `undefined`] |

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
| **204** | Phone number deleted successfully |  -  |
| **401** |  |  -  |
| **403** |  |  -  |
| **404** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getUserPhoneNumber

> UserPhoneNumber getUserPhoneNumber(userId, phoneId)



### Example

```ts
import {
  Configuration,
  UserPhoneNumberApi,
} from '';
import type { GetUserPhoneNumberRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserPhoneNumberApi(config);

  const body = {
    // string
    userId: userId_example,
    // string
    phoneId: phoneId_example,
  } satisfies GetUserPhoneNumberRequest;

  try {
    const data = await api.getUserPhoneNumber(body);
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
| **userId** | `string` |  | [Defaults to `undefined`] |
| **phoneId** | `string` |  | [Defaults to `undefined`] |

### Return type

[**UserPhoneNumber**](UserPhoneNumber.md)

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
| **404** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## listUserPhoneNumbers

> Array&lt;UserPhoneNumber&gt; listUserPhoneNumbers(userId)



### Example

```ts
import {
  Configuration,
  UserPhoneNumberApi,
} from '';
import type { ListUserPhoneNumbersRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserPhoneNumberApi(config);

  const body = {
    // string
    userId: userId_example,
  } satisfies ListUserPhoneNumbersRequest;

  try {
    const data = await api.listUserPhoneNumbers(body);
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
| **userId** | `string` |  | [Defaults to `undefined`] |

### Return type

[**Array&lt;UserPhoneNumber&gt;**](UserPhoneNumber.md)

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


## updateUserPhoneNumberHandler

> UserPhoneNumber updateUserPhoneNumberHandler(userId, phoneId, updateUserPhoneNumberRequest)



### Example

```ts
import {
  Configuration,
  UserPhoneNumberApi,
} from '';
import type { UpdateUserPhoneNumberHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserPhoneNumberApi(config);

  const body = {
    // string
    userId: userId_example,
    // string
    phoneId: phoneId_example,
    // UpdateUserPhoneNumberRequest
    updateUserPhoneNumberRequest: ...,
  } satisfies UpdateUserPhoneNumberHandlerRequest;

  try {
    const data = await api.updateUserPhoneNumberHandler(body);
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
| **userId** | `string` |  | [Defaults to `undefined`] |
| **phoneId** | `string` |  | [Defaults to `undefined`] |
| **updateUserPhoneNumberRequest** | [UpdateUserPhoneNumberRequest](UpdateUserPhoneNumberRequest.md) |  | |

### Return type

[**UserPhoneNumber**](UserPhoneNumber.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: `application/json`
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


## verifyUserPhoneNumberHandler

> UserPhoneNumber verifyUserPhoneNumberHandler(userId, phoneId)



### Example

```ts
import {
  Configuration,
  UserPhoneNumberApi,
} from '';
import type { VerifyUserPhoneNumberHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserPhoneNumberApi(config);

  const body = {
    // string
    userId: userId_example,
    // string
    phoneId: phoneId_example,
  } satisfies VerifyUserPhoneNumberHandlerRequest;

  try {
    const data = await api.verifyUserPhoneNumberHandler(body);
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
| **userId** | `string` |  | [Defaults to `undefined`] |
| **phoneId** | `string` |  | [Defaults to `undefined`] |

### Return type

[**UserPhoneNumber**](UserPhoneNumber.md)

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
| **404** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

