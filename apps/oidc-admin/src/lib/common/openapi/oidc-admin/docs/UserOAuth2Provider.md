# UserOAuth2Provider

## Properties

| Name        | Type   |
| ----------- | ------ |
| `createdAt` | Date   |
| `email`     | string |
| `id`        | number |
| `name`      | string |
| `updatedAt` | Date   |
| `uri`       | string |

## Example

```typescript
import type { UserOAuth2Provider } from '';

// TODO: Update the object below with actual values
const example = {
	createdAt: null,
	email: null,
	id: null,
	name: null,
	updatedAt: null,
	uri: null
} satisfies UserOAuth2Provider;

console.log(example);

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example);
console.log(exampleJSON);

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserOAuth2Provider;
console.log(exampleParsed);
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)
