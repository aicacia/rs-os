# UserRoleApi

All URIs are relative to *http://localhost:3000*

| Method | HTTP request | Description |
|------------- | ------------- | -------------|
| [**assignUserRoleHandler**](UserRoleApi.md#assignuserrolehandler) | **POST** /oidc/api/users/{user_id}/roles |  |
| [**listUserPermissions**](UserRoleApi.md#listuserpermissions) | **GET** /oidc/api/users/{user_id}/permissions |  |
| [**listUserRoles**](UserRoleApi.md#listuserroles) | **GET** /oidc/api/users/{user_id}/roles |  |
| [**removeUserRoleHandler**](UserRoleApi.md#removeuserrolehandler) | **DELETE** /oidc/api/users/{user_id}/roles/{role_id} |  |



## assignUserRoleHandler

> UserRole assignUserRoleHandler(userId, assignUserRoleRequest)



### Example

```ts
import {
  Configuration,
  UserRoleApi,
} from '';
import type { AssignUserRoleHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserRoleApi(config);

  const body = {
    // string
    userId: userId_example,
    // AssignUserRoleRequest
    assignUserRoleRequest: ...,
  } satisfies AssignUserRoleHandlerRequest;

  try {
    const data = await api.assignUserRoleHandler(body);
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
| **assignUserRoleRequest** | [AssignUserRoleRequest](AssignUserRoleRequest.md) |  | |

### Return type

[**UserRole**](UserRole.md)

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


## listUserPermissions

> UserPermissions listUserPermissions(userId)



### Example

```ts
import {
  Configuration,
  UserRoleApi,
} from '';
import type { ListUserPermissionsRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserRoleApi(config);

  const body = {
    // string
    userId: userId_example,
  } satisfies ListUserPermissionsRequest;

  try {
    const data = await api.listUserPermissions(body);
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

[**UserPermissions**](UserPermissions.md)

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


## listUserRoles

> Array&lt;UserRole&gt; listUserRoles(userId)



### Example

```ts
import {
  Configuration,
  UserRoleApi,
} from '';
import type { ListUserRolesRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserRoleApi(config);

  const body = {
    // string
    userId: userId_example,
  } satisfies ListUserRolesRequest;

  try {
    const data = await api.listUserRoles(body);
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

[**Array&lt;UserRole&gt;**](UserRole.md)

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


## removeUserRoleHandler

> removeUserRoleHandler(userId, roleId)



### Example

```ts
import {
  Configuration,
  UserRoleApi,
} from '';
import type { RemoveUserRoleHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({ 
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserRoleApi(config);

  const body = {
    // string
    userId: userId_example,
    // string
    roleId: roleId_example,
  } satisfies RemoveUserRoleHandlerRequest;

  try {
    const data = await api.removeUserRoleHandler(body);
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
| **roleId** | `string` |  | [Defaults to `undefined`] |

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
| **204** | Role removed successfully |  -  |
| **401** |  |  -  |
| **403** |  |  -  |
| **404** |  |  -  |
| **500** |  |  -  |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

