# Client

## Properties

| Name                          | Type                |
| ----------------------------- | ------------------- |
| `accessTokenExpiresInSeconds` | number              |
| `active`                      | boolean             |
| `applicationType`             | string              |
| `audience`                    | Array&lt;string&gt; |
| `authMethod`                  | string              |
| `clientId`                    | string              |
| `clientSecret`                | string              |
| `clientUri`                   | string              |
| `createdAt`                   | Date                |
| `grantTypes`                  | Array&lt;string&gt; |
| `id`                          | number              |
| `idTokenExpiresInSeconds`     | number              |
| `logoUri`                     | string              |
| `name`                        | string              |
| `policyUri`                   | string              |
| `postLogoutRedirectUris`      | Array&lt;string&gt; |
| `redirectUris`                | Array&lt;string&gt; |
| `refreshExpiresInSeconds`     | number              |
| `responseTypes`               | Array&lt;string&gt; |
| `scopes`                      | Array&lt;string&gt; |
| `termsOfServiceUri`           | string              |
| `updatedAt`                   | Date                |

## Example

```typescript
import type { Client } from '';

// TODO: Update the object below with actual values
const example = {
	accessTokenExpiresInSeconds: null,
	active: null,
	applicationType: null,
	audience: null,
	authMethod: null,
	clientId: null,
	clientSecret: null,
	clientUri: null,
	createdAt: null,
	grantTypes: null,
	id: null,
	idTokenExpiresInSeconds: null,
	logoUri: null,
	name: null,
	policyUri: null,
	postLogoutRedirectUris: null,
	redirectUris: null,
	refreshExpiresInSeconds: null,
	responseTypes: null,
	scopes: null,
	termsOfServiceUri: null,
	updatedAt: null
} satisfies Client;

console.log(example);

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example);
console.log(exampleJSON);

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as Client;
console.log(exampleParsed);
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)
