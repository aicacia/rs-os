
# UserEmail


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`email` | string
`id` | string
`isPrimary` | boolean
`updatedAt` | Date
`userId` | string
`verified` | boolean

## Example

```typescript
import type { UserEmail } from ''

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "email": null,
  "id": null,
  "isPrimary": null,
  "updatedAt": null,
  "userId": null,
  "verified": null,
} satisfies UserEmail

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserEmail
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


