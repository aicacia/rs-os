
# User


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`id` | string
`updatedAt` | Date
`username` | string

## Example

```typescript
import type { User } from ''

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "id": null,
  "updatedAt": null,
  "username": null,
} satisfies User

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as User
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


