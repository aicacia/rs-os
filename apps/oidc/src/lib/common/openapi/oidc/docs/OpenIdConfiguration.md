
# OpenIdConfiguration


## Properties

Name | Type
------------ | -------------
`authorizationEndpoint` | string
`claimsSupported` | Array&lt;string&gt;
`codeChallengeMethodsSupported` | Array&lt;string&gt;
`deviceAuthorizationEndpoint` | string
`endSessionEndpoint` | string
`grantTypesSupported` | Array&lt;string&gt;
`idTokenSigningAlgValuesSupported` | Array&lt;string&gt;
`issuer` | string
`jwksUri` | string
`registrationEndpoint` | string
`responseModesSupported` | Array&lt;string&gt;
`responseTypesSupported` | Array&lt;string&gt;
`revocationEndpoint` | string
`scopesSupported` | Array&lt;string&gt;
`subjectTypesSupported` | Array&lt;string&gt;
`tokenEndpoint` | string
`tokenEndpointAuthMethodsSupported` | Array&lt;string&gt;
`userinfoEndpoint` | string

## Example

```typescript
import type { OpenIdConfiguration } from ''

// TODO: Update the object below with actual values
const example = {
  "authorizationEndpoint": null,
  "claimsSupported": null,
  "codeChallengeMethodsSupported": null,
  "deviceAuthorizationEndpoint": null,
  "endSessionEndpoint": null,
  "grantTypesSupported": null,
  "idTokenSigningAlgValuesSupported": null,
  "issuer": null,
  "jwksUri": null,
  "registrationEndpoint": null,
  "responseModesSupported": null,
  "responseTypesSupported": null,
  "revocationEndpoint": null,
  "scopesSupported": null,
  "subjectTypesSupported": null,
  "tokenEndpoint": null,
  "tokenEndpointAuthMethodsSupported": null,
  "userinfoEndpoint": null,
} satisfies OpenIdConfiguration

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as OpenIdConfiguration
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


