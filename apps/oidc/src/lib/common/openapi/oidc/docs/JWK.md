
# JWK


## Properties

Name | Type
------------ | -------------
`alg` | string
`crv` | string
`e` | string
`keyOps` | Array&lt;string&gt;
`kid` | string
`kty` | string
`n` | string
`use` | string
`x` | string
`x5c` | string
`x5t` | string
`x5tS256` | string
`x5u` | string
`y` | string

## Example

```typescript
import type { JWK } from ''

// TODO: Update the object below with actual values
const example = {
  "alg": null,
  "crv": null,
  "e": null,
  "keyOps": null,
  "kid": null,
  "kty": null,
  "n": null,
  "use": null,
  "x": null,
  "x5c": null,
  "x5t": null,
  "x5tS256": null,
  "x5u": null,
  "y": null,
} satisfies JWK

console.log(example)

// Convert the instance to a JSON string
const exampleJSON: string = JSON.stringify(example)
console.log(exampleJSON)

// Parse the JSON string back to an object
const exampleParsed = JSON.parse(exampleJSON) as JWK
console.log(exampleParsed)
```

[[Back to top]](#) [[Back to API list]](../README.md#api-endpoints) [[Back to Model list]](../README.md#models) [[Back to README]](../README.md)


