
# Token


## Properties

Name | Type
------------ | -------------
`accessToken` | string
`expiresIn` | number
`idToken` | string
`issuedAt` | Date
`issuedTokenType` | string
`passwordResetRequired` | boolean
`refreshToken` | string
`refreshTokenExpiresIn` | number
`scope` | string
`tokenType` | string

## Example

```typescript
import type { Token } from ''

// TODO: Update the object below with actual values
const example = {
  "accessToken": null,
  "expiresIn": null,
  "idToken": null,
  "issuedAt": null,
  "issuedTokenType": null,
  "passwordResetRequired": null,
  "refreshToken": null,
  "refreshTokenExpiresIn": null,
  "scope": null,
  "tokenType": null,
} satisfies Token

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as Token
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


