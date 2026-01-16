
# OpenIdProfile


## Properties

Name | Type
------------ | -------------
`address` | string
`birthdate` | Date
`email` | string
`emailVerified` | boolean
`familyName` | string
`gender` | string
`givenName` | string
`locale` | string
`middleName` | string
`name` | string
`nickname` | string
`phone` | string
`phoneVerified` | boolean
`preferredUsername` | string
`profilePicture` | string
`website` | string
`zoneInfo` | string

## Example

```typescript
import type { OpenIdProfile } from ''

// TODO: Update the object below with actual values
const example = {
  "address": null,
  "birthdate": null,
  "email": null,
  "emailVerified": null,
  "familyName": null,
  "gender": null,
  "givenName": null,
  "locale": null,
  "middleName": null,
  "name": null,
  "nickname": null,
  "phone": null,
  "phoneVerified": null,
  "preferredUsername": null,
  "profilePicture": null,
  "website": null,
  "zoneInfo": null,
} satisfies OpenIdProfile

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as OpenIdProfile
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


