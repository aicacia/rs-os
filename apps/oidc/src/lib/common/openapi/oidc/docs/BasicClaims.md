# BasicClaims

## Properties

| Name     | Type   |
| -------- | ------ |
| `aud`    | string |
| `client` | string |
| `exp`    | number |
| `iat`    | number |
| `iss`    | string |
| `nbf`    | number |
| `scope`  | string |
| `sub`    | string |
| `type`   | string |
| `user`   | number |

## Example

```typescript
import type { BasicClaims } from '';

// TODO: Update the object below with actual values
const example = {
	aud: null,
	client: null,
	exp: null,
	iat: null,
	iss: null,
	nbf: null,
	scope: null,
	sub: null,
	type: null,
	user: null
} satisfies BasicClaims;

console.log(example);

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example);
console.log(exampleJSON);

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as BasicClaims;
console.log(exampleParsed);
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)
