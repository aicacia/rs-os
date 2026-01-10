# UserOauth2ProviderApi

All URIs are relative to _http://localhost:3000_

| Method                                                                                          | HTTP request                                                              | Description |
| ----------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- | ----------- |
| [**getUserOauth2Provider**](UserOauth2ProviderApi.md#getuseroauth2provider)                     | **GET** /oidc-admin/api/users/{user_id}/oauth2-providers/{provider_id}    |             |
| [**linkUserOauth2ProviderHandler**](UserOauth2ProviderApi.md#linkuseroauth2providerhandler)     | **POST** /oidc-admin/api/users/{user_id}/oauth2-providers                 |             |
| [**listUserOauth2Providers**](UserOauth2ProviderApi.md#listuseroauth2providers)                 | **GET** /oidc-admin/api/users/{user_id}/oauth2-providers                  |             |
| [**unlinkUserOauth2ProviderHandler**](UserOauth2ProviderApi.md#unlinkuseroauth2providerhandler) | **DELETE** /oidc-admin/api/users/{user_id}/oauth2-providers/{provider_id} |             |

## getUserOauth2Provider

> UserOAuth2Provider getUserOauth2Provider(userId, providerId)

### Example

```ts
import { Configuration, UserOauth2ProviderApi } from '';
import type { GetUserOauth2ProviderRequest } from '';

async function example() {
	console.log('🚀 Testing  SDK...');
	const config = new Configuration({
		// Configure HTTP bearer authorization: Authorization
		accessToken: 'YOUR BEARER TOKEN'
	});
	const api = new UserOauth2ProviderApi(config);

	const body = {
		// string
		userId: userId_example,
		// string
		providerId: providerId_example
	} satisfies GetUserOauth2ProviderRequest;

	try {
		const data = await api.getUserOauth2Provider(body);
		console.log(data);
	} catch (error) {
		console.error(error);
	}
}

// Run the test
example().catch(console.error);
```

### Parameters

| Name           | Type     | Description | Notes                     |
| -------------- | -------- | ----------- | ------------------------- |
| **userId**     | `string` |             | [Defaults to `undefined`] |
| **providerId** | `string` |             | [Defaults to `undefined`] |

### Return type

[**UserOAuth2Provider**](UserOAuth2Provider.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

### HTTP response details

| Status code | Description | Response headers |
| ----------- | ----------- | ---------------- |
| **200**     |             | -                |
| **401**     |             | -                |
| **403**     |             | -                |
| **404**     |             | -                |
| **500**     |             | -                |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

## linkUserOauth2ProviderHandler

> UserOAuth2Provider linkUserOauth2ProviderHandler(userId, linkUserOAuth2ProviderRequest)

### Example

```ts
import {
  Configuration,
  UserOauth2ProviderApi,
} from '';
import type { LinkUserOauth2ProviderHandlerRequest } from '';

async function example() {
  console.log("🚀 Testing  SDK...");
  const config = new Configuration({
    // Configure HTTP bearer authorization: Authorization
    accessToken: "YOUR BEARER TOKEN",
  });
  const api = new UserOauth2ProviderApi(config);

  const body = {
    // string
    userId: userId_example,
    // LinkUserOAuth2ProviderRequest
    linkUserOAuth2ProviderRequest: ...,
  } satisfies LinkUserOauth2ProviderHandlerRequest;

  try {
    const data = await api.linkUserOauth2ProviderHandler(body);
    console.log(data);
  } catch (error) {
    console.error(error);
  }
}

// Run the test
example().catch(console.error);
```

### Parameters

| Name                              | Type                                                              | Description | Notes                     |
| --------------------------------- | ----------------------------------------------------------------- | ----------- | ------------------------- |
| **userId**                        | `string`                                                          |             | [Defaults to `undefined`] |
| **linkUserOAuth2ProviderRequest** | [LinkUserOAuth2ProviderRequest](LinkUserOAuth2ProviderRequest.md) |             |                           |

### Return type

[**UserOAuth2Provider**](UserOAuth2Provider.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: `application/json`
- **Accept**: `application/json`

### HTTP response details

| Status code | Description | Response headers |
| ----------- | ----------- | ---------------- |
| **201**     |             | -                |
| **400**     |             | -                |
| **401**     |             | -                |
| **403**     |             | -                |
| **500**     |             | -                |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

## listUserOauth2Providers

> Array&lt;UserOAuth2Provider&gt; listUserOauth2Providers(userId)

### Example

```ts
import { Configuration, UserOauth2ProviderApi } from '';
import type { ListUserOauth2ProvidersRequest } from '';

async function example() {
	console.log('🚀 Testing  SDK...');
	const config = new Configuration({
		// Configure HTTP bearer authorization: Authorization
		accessToken: 'YOUR BEARER TOKEN'
	});
	const api = new UserOauth2ProviderApi(config);

	const body = {
		// string
		userId: userId_example
	} satisfies ListUserOauth2ProvidersRequest;

	try {
		const data = await api.listUserOauth2Providers(body);
		console.log(data);
	} catch (error) {
		console.error(error);
	}
}

// Run the test
example().catch(console.error);
```

### Parameters

| Name       | Type     | Description | Notes                     |
| ---------- | -------- | ----------- | ------------------------- |
| **userId** | `string` |             | [Defaults to `undefined`] |

### Return type

[**Array&lt;UserOAuth2Provider&gt;**](UserOAuth2Provider.md)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

### HTTP response details

| Status code | Description | Response headers |
| ----------- | ----------- | ---------------- |
| **200**     |             | -                |
| **401**     |             | -                |
| **403**     |             | -                |
| **500**     |             | -                |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)

## unlinkUserOauth2ProviderHandler

> unlinkUserOauth2ProviderHandler(userId, providerId)

### Example

```ts
import { Configuration, UserOauth2ProviderApi } from '';
import type { UnlinkUserOauth2ProviderHandlerRequest } from '';

async function example() {
	console.log('🚀 Testing  SDK...');
	const config = new Configuration({
		// Configure HTTP bearer authorization: Authorization
		accessToken: 'YOUR BEARER TOKEN'
	});
	const api = new UserOauth2ProviderApi(config);

	const body = {
		// string
		userId: userId_example,
		// string
		providerId: providerId_example
	} satisfies UnlinkUserOauth2ProviderHandlerRequest;

	try {
		const data = await api.unlinkUserOauth2ProviderHandler(body);
		console.log(data);
	} catch (error) {
		console.error(error);
	}
}

// Run the test
example().catch(console.error);
```

### Parameters

| Name           | Type     | Description | Notes                     |
| -------------- | -------- | ----------- | ------------------------- |
| **userId**     | `string` |             | [Defaults to `undefined`] |
| **providerId** | `string` |             | [Defaults to `undefined`] |

### Return type

`void` (Empty response body)

### Authorization

[Authorization](../README.md#Authorization)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: `application/json`

### HTTP response details

| Status code | Description                           | Response headers |
| ----------- | ------------------------------------- | ---------------- |
| **204**     | OAuth2 provider unlinked successfully | -                |
| **401**     |                                       | -                |
| **403**     |                                       | -                |
| **404**     |                                       | -                |
| **500**     |                                       | -                |

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)
