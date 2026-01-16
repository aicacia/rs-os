
# UserPhoneNumber


## Properties

Name | Type
------------ | -------------
`createdAt` | Date
`id` | string
`isPrimary` | boolean
`phoneNumber` | string
`updatedAt` | Date
`userId` | string
`verified` | boolean

## Example

```typescript
import type { UserPhoneNumber } from ''

// TODO: Update the object below with actual values
const example = {
  "createdAt": null,
  "id": null,
  "isPrimary": null,
  "phoneNumber": null,
  "updatedAt": null,
  "userId": null,
  "verified": null,
} satisfies UserPhoneNumber

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserPhoneNumber
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


