# ClientRegisterRequest

## Properties

| Name                          | Type                |
| ----------------------------- | ------------------- |
| `accessTokenExpiresInSeconds` | number              |
| `applicationType`             | string              |
| `audience`                    | Array&lt;string&gt; |
| `authMethod`                  | string              |
| `clientId`                    | string              |
| `clientUri`                   | string              |
| `grantTypes`                  | Array&lt;string&gt; |
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

## Example

```typescript
import type { ClientRegisterRequest } from '';

// TODO: Update the object below with actual values
const example = {
	accessTokenExpiresInSeconds: null,
	applicationType: null,
	audience: null,
	authMethod: null,
	clientId: null,
	clientUri: null,
	grantTypes: null,
	idTokenExpiresInSeconds: null,
	logoUri: null,
	name: null,
	policyUri: null,
	postLogoutRedirectUris: null,
	redirectUris: null,
	refreshExpiresInSeconds: null,
	responseTypes: null,
	scopes: null,
	termsOfServiceUri: null
} satisfies ClientRegisterRequest;

console.log(example);

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example);
console.log(exampleJSON);

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ClientRegisterRequest;
console.log(exampleParsed);
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)
