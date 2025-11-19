
# ClientAuthorizationOneOf


## Properties

Name | Type
------------ | -------------
`accessToken` | string
`expiresIn` | number
`idToken` | string
`issuedAt` | Date
`issuedTokenType` | string
`refreshToken` | string
`refreshTokenExpiresIn` | number
`scope` | string
`tokenType` | string
`type` | string

## Example

```typescript
import type { ClientAuthorizationOneOf } from ''

// TODO: Update the object below with actual values
const example = {
  "accessToken": null,
  "expiresIn": null,
  "idToken": null,
  "issuedAt": null,
  "issuedTokenType": null,
  "refreshToken": null,
  "refreshTokenExpiresIn": null,
  "scope": null,
  "tokenType": null,
  "type": null,
} satisfies ClientAuthorizationOneOf

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as ClientAuthorizationOneOf
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


