# CurrentUserApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**currentUser**](CurrentUserApi.md#currentuser) | **GET** /oidc/api/current-user |  |
| [**updatePassword**](CurrentUserApi.md#updatepassword) | **PATCH** /oidc/api/current-user/password |  |
| [**updateUserInfo**](CurrentUserApi.md#updateuserinfooperation) | **PATCH** /oidc/api/current-user/info |  |
| [**updateUsername**](CurrentUserApi.md#updateusernameoperation) | **PATCH** /oidc/api/current-user |  |



## currentUser

> CurrentUser currentUser()



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

[**CurrentUser**](CurrentUser.md)

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


## updatePassword

> updatePassword(updateUserPassword)



### Example

```ts
import {
  Configuration,
  CurrentUserApi,
} from '';
import type { UpdatePasswordRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CurrentUserApi(config);

  const body = {
    // UpdateUserPassword
    updateUserPassword: ...,
  } satisfies UpdatePasswordRequest;

  try {
    const data = await api.updatePassword(body);
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
| **updateUserPassword** | [UpdateUserPassword](UpdateUserPassword.md) |  | |

### Return type

`void` (Empty response body)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **204** |  |  -  |
| **400** |  |  -  |
| **401** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateUserInfo

> UserInfo updateUserInfo(updateUserInfoRequest)



### Example

```ts
import {
  Configuration,
  CurrentUserApi,
} from '';
import type { UpdateUserInfoOperationRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CurrentUserApi(config);

  const body = {
    // UpdateUserInfoRequest
    updateUserInfoRequest: ...,
  } satisfies UpdateUserInfoOperationRequest;

  try {
    const data = await api.updateUserInfo(body);
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
| **updateUserInfoRequest** | [UpdateUserInfoRequest](UpdateUserInfoRequest.md) |  | |

### Return type

[**UserInfo**](UserInfo.md)

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
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


## updateUsername

> updateUsername(updateUsernameRequest)



### Example

```ts
import {
  Configuration,
  CurrentUserApi,
} from '';
import type { UpdateUsernameOperationRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new CurrentUserApi(config);

  const body = {
    // UpdateUsernameRequest
    updateUsernameRequest: ...,
  } satisfies UpdateUsernameOperationRequest;

  try {
    const data = await api.updateUsername(body);
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
| **updateUsernameRequest** | [UpdateUsernameRequest](UpdateUsernameRequest.md) |  | |

### Return type

`void` (Empty response body)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`


### HTTP response details
| Status code | Description | Response headers |
|-------------|-------------|------------------|
| **204** |  |  -  |
| **400** |  |  -  |
| **401** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

