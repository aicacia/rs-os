# UserPermissions

## Properties

| Name          | Type                                     |
| ------------- | ---------------------------------------- |
| `permissions` | [Array&lt;Permission&gt;](Permission.md) |

## Example

```typescript
import type { UserPermissions } from '';

// TODO: Update the object below with actual values
const example = {
	permissions: null
} satisfies UserPermissions;

console.log(example);

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example);
console.log(exampleJSON);

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserPermissions;
console.log(exampleParsed);
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)
