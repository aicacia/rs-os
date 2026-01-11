
# CurrentUser


## Properties

Name | Type
------------ | -------------
`active` | boolean
`createdAt` | Date
`email` | [UserEmail](UserEmail.md)
`emails` | [Array&lt;UserEmail&gt;](UserEmail.md)
`id` | number
`info` | [UserInfo](UserInfo.md)
`oauth2Providers` | [Array&lt;UserOAuth2Provider&gt;](UserOAuth2Provider.md)
`phoneNumber` | [UserPhoneNumber](UserPhoneNumber.md)
`phoneNumbers` | [Array&lt;UserPhoneNumber&gt;](UserPhoneNumber.md)
`roles` | [Array&lt;UserRole&gt;](UserRole.md)
`updatedAt` | Date
`username` | string

## Example

```typescript
import type { CurrentUser } from ''

// TODO: Update the object below with actual values
const example = {
  "active": null,
  "createdAt": null,
  "email": null,
  "emails": null,
  "id": null,
  "info": null,
  "oauth2Providers": null,
  "phoneNumber": null,
  "phoneNumbers": null,
  "roles": null,
  "updatedAt": null,
  "username": null,
} satisfies CurrentUser

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as CurrentUser
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


