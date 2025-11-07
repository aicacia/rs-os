
# UserInfo


## Properties

Name | Type
------------ | -------------
`address` | string
`birthdate` | number
`createdAt` | Date
`familyName` | string
`gender` | string
`givenName` | string
`locale` | string
`middleName` | string
`name` | string
`nickname` | string
`profilePicture` | string
`updatedAt` | Date
`website` | string
`zoneInfo` | string

## Example

```typescript
import type { UserInfo } from ''

// TODO: Update the object below with actual values
const example = {
  "address": null,
  "birthdate": null,
  "createdAt": null,
  "familyName": null,
  "gender": null,
  "givenName": null,
  "locale": null,
  "middleName": null,
  "name": null,
  "nickname": null,
  "profilePicture": null,
  "updatedAt": null,
  "website": null,
  "zoneInfo": null,
} satisfies UserInfo

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as UserInfo
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


