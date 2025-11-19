
# AuthorizeRequest


## Properties

Name | Type
------------ | -------------
`clientId` | string
`nonce` | string
`redirectUri` | string
`responseMode` | [ResponseMode](ResponseMode.md)
`responseType` | [ResponseType](ResponseType.md)
`scope` | string
`state` | string

## Example

```typescript
import type { AuthorizeRequest } from ''

// TODO: Update the object below with actual values
const example = {
  "clientId": null,
  "nonce": null,
  "redirectUri": null,
  "responseMode": null,
  "responseType": null,
  "scope": null,
  "state": null,
} satisfies AuthorizeRequest

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as AuthorizeRequest
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


