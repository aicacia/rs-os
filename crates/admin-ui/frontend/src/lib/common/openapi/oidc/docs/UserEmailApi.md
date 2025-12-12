# UserEmailApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**createUserEmailHandler**](UserEmailApi.md#createuseremailhandler) | **POST** /oidc/api/users/{user_id}/emails |  |
| [**deleteUserEmailHandler**](UserEmailApi.md#deleteuseremailhandler) | **DELETE** /oidc/api/users/{user_id}/emails/{email_id} |  |
| [**getUserEmail**](UserEmailApi.md#getuseremail) | **GET** /oidc/api/users/{user_id}/emails/{email_id} |  |
| [**listUserEmails**](UserEmailApi.md#listuseremails) | **GET** /oidc/api/users/{user_id}/emails |  |
| [**updateUserEmailHandler**](UserEmailApi.md#updateuseremailhandler) | **PATCH** /oidc/api/users/{user_id}/emails/{email_id} |  |
| [**verifyUserEmailHandler**](UserEmailApi.md#verifyuseremailhandler) | **POST** /oidc/api/users/{user_id}/emails/{email_id}/verify |  |



## createUserEmailHandler

> UserEmail createUserEmailHandler(userId, createUserEmailRequest)



### Example

```ts
import {
  Configuration,
  UserEmailApi,
} from '';
import type { CreateUserEmailHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserEmailApi(config);

  const body = {
    // string
    userId: userId_example,
    // CreateUserEmailRequest
    createUserEmailRequest: ...,
  } satisfies CreateUserEmailHandlerRequest;

  try {
    const data = await api.createUserEmailHandler(body);
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
| **createUserEmailRequest** | [CreateUserEmailRequest](CreateUserEmailRequest.md) |  | |

### Return type

[**UserEmail**](UserEmail.md)

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


## deleteUserEmailHandler

> deleteUserEmailHandler(userId, emailId)



### Example

```ts
import {
  Configuration,
  UserEmailApi,
} from '';
import type { DeleteUserEmailHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserEmailApi(config);

  const body = {
    // string
    userId: userId_example,
    // string
    emailId: emailId_example,
  } satisfies DeleteUserEmailHandlerRequest;

  try {
    const data = await api.deleteUserEmailHandler(body);
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
| **emailId** | `string` |  | [Defaults to `undefined`] |

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
| **204** | Email deleted successfully |  -  |
| **401** |  |  -  |
| **403** |  |  -  |
| **404** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## getUserEmail

> UserEmail getUserEmail(userId, emailId)



### Example

```ts
import {
  Configuration,
  UserEmailApi,
} from '';
import type { GetUserEmailRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserEmailApi(config);

  const body = {
    // string
    userId: userId_example,
    // string
    emailId: emailId_example,
  } satisfies GetUserEmailRequest;

  try {
    const data = await api.getUserEmail(body);
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
| **emailId** | `string` |  | [Defaults to `undefined`] |

### Return type

[**UserEmail**](UserEmail.md)

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


## listUserEmails

> Array&lt;UserEmail&gt; listUserEmails(userId)



### Example

```ts
import {
  Configuration,
  UserEmailApi,
} from '';
import type { ListUserEmailsRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserEmailApi(config);

  const body = {
    // string
    userId: userId_example,
  } satisfies ListUserEmailsRequest;

  try {
    const data = await api.listUserEmails(body);
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

[**Array&lt;UserEmail&gt;**](UserEmail.md)

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


## updateUserEmailHandler

> UserEmail updateUserEmailHandler(userId, emailId, updateUserEmailRequest)



### Example

```ts
import {
  Configuration,
  UserEmailApi,
} from '';
import type { UpdateUserEmailHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserEmailApi(config);

  const body = {
    // string
    userId: userId_example,
    // string
    emailId: emailId_example,
    // UpdateUserEmailRequest
    updateUserEmailRequest: ...,
  } satisfies UpdateUserEmailHandlerRequest;

  try {
    const data = await api.updateUserEmailHandler(body);
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
| **emailId** | `string` |  | [Defaults to `undefined`] |
| **updateUserEmailRequest** | [UpdateUserEmailRequest](UpdateUserEmailRequest.md) |  | |

### Return type

[**UserEmail**](UserEmail.md)

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


## verifyUserEmailHandler

> UserEmail verifyUserEmailHandler(userId, emailId)



### Example

```ts
import {
  Configuration,
  UserEmailApi,
} from '';
import type { VerifyUserEmailHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserEmailApi(config);

  const body = {
    // string
    userId: userId_example,
    // string
    emailId: emailId_example,
  } satisfies VerifyUserEmailHandlerRequest;

  try {
    const data = await api.verifyUserEmailHandler(body);
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
| **emailId** | `string` |  | [Defaults to `undefined`] |

### Return type

[**UserEmail**](UserEmail.md)

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

