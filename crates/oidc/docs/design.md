# OIDC Crate Design

## Overview

The OIDC crate provides a standards-compliant, high-level implementation of OpenID Connect (OIDC) for Relying Parties (clients). It enables applications to authenticate end users via an OpenID Provider (OP) and to obtain and validate ID Tokens, Access Tokens, and user information in a secure, interoperable manner.

The crate focuses on developer-friendly building blocks for initiating authorization requests, performing token exchanges, validating tokens, retrieving user info, and managing session/logout semantics with minimal coupling to application frameworks. It is designed to be provider-agnostic and interoperable across compliant OPs.

## Goals

- Interoperable, standards-compliant OIDC Core functionality across providers
- Strong security guarantees: PKCE, nonce/state, token validation, key rotation
- Clear, composable flows for web, native, and device scenarios
- Robust discovery, caching, and resilience against provider/network issues
- Extensible configuration and hooks for advanced scenarios (ACR/AMR, prompts)
- Practical ergonomics while remaining implementation-agnostic and framework-neutral

## Non-Goals

- Serving as an Authorization Server/OP (this crate is for Relying Parties)
- Enforcing a specific HTTP client, storage, or runtime choice
- Comprehensive OAuth 2.0 resource server features (beyond what OIDC requires)

## Standards & Scope

The crate targets the following specifications:

- OpenID Connect Core 1.0
- OpenID Connect Discovery 1.0 (`/.well-known/openid-configuration`)
- JSON Web Token (JWT) and JSON Web Signature (JWS)
- JSON Web Key (JWK) Set for public key retrieval
- OAuth 2.0 (authorization code flow, refresh, device authorization)
- Optional: OpenID Connect Dynamic Client Registration 1.0 (configurable)
- Optional: RP-Initiated Logout, Front/Back-Channel Logout (where supported)

Where applicable, the crate follows current best practices, including the use of PKCE, refresh token rotation, and strict token validation.

## Supported Flows

- Authorization Code Flow (recommended for web backends)

  - PKCE (S256) required
  - `response_type=code`, `response_mode=query|form_post`
  - Optional: `prompt`, `max_age`, `ui_locales`, `acr_values`

- Device Authorization Flow (for devices/CLIs without browsers)

  - User code and verification URI handling
  - Polling with backoff; error handling per spec

- Hybrid/Implicit Flows
  - Not primary; support may be limited or disabled by default per modern guidance

## Architecture Overview

The crate exposes high-level operations for the typical OIDC journey while keeping the implementation modular:

1. Discovery

   - Fetch and cache the OP metadata from `/.well-known/openid-configuration`.
   - Validate and store endpoints and capabilities.

2. Client Configuration

   - Support static client registration (client ID/secret) and optional dynamic registration.
   - Configure redirect URIs, scopes, and default parameters.

3. Authorization Request Construction

   - Build authorization URLs with required parameters (`client_id`, `redirect_uri`, `scope`, `response_type`, `state`, `nonce`, PKCE values).
   - Provide helpers for multi-tenant/providers and advanced prompts/claims.

4. Token Exchange

   - Exchange authorization codes for tokens at the token endpoint.
   - Support refresh tokens, rotation, and revocation where applicable.

5. Token Validation

   - Validate ID Tokens: issuer, audience, azp, exp, nbf, iat, nonce, signature.
   - Validate Access Tokens where appropriate (signature or via UserInfo/Introspection if configured).
   - Manage JWK Set retrieval and key rotation with caching and backoff.

6. UserInfo Retrieval

   - Call the UserInfo endpoint using an Access Token.
   - Validate response consistency with ID Token claims per spec guidance.

7. Session & Logout
   - Provide RP-Initiated Logout URL construction when supported by OP.
   - Optionally integrate front/back-channel logout (feature-gated).

## Core Components

- Discovery Manager

  - Retrieves and caches OP metadata (endpoints, algorithms, capabilities).
  - Handles cache invalidation and refresh; resilient to network errors.

- Client Configuration

  - Holds client identifiers, secrets (if applicable), redirect URIs, and scopes.
  - Supports per-provider overrides; secure handling of secrets.
    - Integrates with a client lifecycle that requires explicit approval by authorized users before any authorization request is permitted.

- Authorization Builder

  - Constructs authorization requests with PKCE, state, nonce, and optional parameters.
  - Emits a serialized URL for redirecting the user to the OP.

- Token Processor

  - Exchanges codes/device codes for tokens; supports refresh.
  - Normalizes and validates token responses; maps errors to OIDC/OAuth codes.

- JWT & JWK Handling

  - Parses and verifies ID Tokens using OP’s JWK Set.
  - Implements key selection, algorithm validation, clock skew tolerance.

- UserInfo Client

  - Retrieves user claims securely; reconciles with ID Token if required.

- Logout Coordinator (optional)
  - Builds RP-Initiated Logout requests and coordinates session cleanup.

## Data & State Management

- Transient Authorization State

  - `state` and `nonce` values generated per request; stored to compare on return.
  - PKCE `code_verifier` securely stored during the flow.

- Provider Metadata Cache

  - Cached discovery document and JWK Set with TTL/backoff; persistent or in-memory.

- Token Storage (application-controlled)
  - Guidance for secure storage of Access/Refresh/ID Tokens, rotation, and revocation.
  - Hooks for applications to decide persistence model and lifecycle.

## Security Considerations

- PKCE (S256) required for authorization code flows.
- Nonce and State

  - Unique, high-entropy values per request; verified on callback.

- Token Validation

  - Signature verification using JWKs; allowed algorithms restricted.
  - Claim checks: `iss`, `aud`, `azp` (when applicable), `exp`, `nbf`, `iat`, `nonce`.
  - Optional validations: `c_hash`, `at_hash` when flows return fragments.

- Redirect URI Handling

  - Strict matching to registered URIs; prevent open redirects.

- Refresh Token Safety

  - Rotation support; detection of reuse; revocation guidance.

- Time & Clock Skew

  - Configurable skew tolerance for token validation.

- Key Rotation & Resilience

  - Periodic JWK refresh; fallback strategies when keys change.

- Privacy & Minimization
  - Request only necessary scopes; avoid over-collecting claims.

## Error Handling

- Normalize and surface OIDC/OAuth error codes/messages (e.g., `invalid_request`, `access_denied`, `invalid_grant`).
- Provide structured errors with context (endpoint, parameters, retryability).
- Backoff and retry strategies for discovery/JWK fetches; clear failure modes.

## Extensibility & Configuration

- Provider-Agnostic Configuration

  - Multiple providers; per-provider defaults for scopes, prompts, ACRs.

- Advanced Parameters

  - `prompt`, `login_hint`, `ui_locales`, `max_age`, `acr_values`, `claims`.

- Dynamic Client Registration (optional)

  - Feature-gated; supports registration where OP allows.

- Hooks & Policies
  - Pre/post-request hooks; claim transformation/validation policies.

## Client Lifecycle & Approval

- Clients must be created/registered and then explicitly approved by users with the appropriate permissions before any authorization flow is allowed to start.
- Authorization URL construction and token exchange entry points should fail fast for unapproved clients with clear, normalized errors (e.g., `unauthorized_client`).
- Admin/ops surfaces (outside this crate) handle the approval action; the crate consumes an approved-client flag from its configuration/store.
- Client updates (redirect URIs, scopes, secrets) should invalidate approval status until re-approved, ensuring only reviewed configurations can initiate login flows.

## Operational Concerns

- Observability

  - Structured logs for flow steps and validations; redaction of secrets.

- Caching

  - Configurable cache strategies for discovery/JWKs; TTL and refresh policies.

- Performance

  - Efficient JWT parsing/verification; minimal network calls; concurrency-safe caches.

- Internationalization
  - Support for `ui_locales` where OPs honor it; claim normalization guidance.

## Conformance & Testing

- Alignment with OpenID Foundation test suites (RP conformance profiles).
- Interoperability testing against popular OPs (e.g., Auth0, Okta, Keycloak, Azure AD, Google).
- Security audits: token validation correctness, replay protection, key rotation handling.
- Negative tests: malformed tokens, expired/nbf edge cases, algorithm mismatches.

## Documentation & Examples

- Clear flow guides: Web (Code + PKCE), Device.
- Provider setup notes: discovery URL, client registration, redirect URI requirements.
- Error reference: common OP responses and remediation.
- Security checklist: required and optional hardening.

## Open Questions / Future Work

- Logout support breadth (front/back-channel) and OP coverage.
- Federation, multi-tenant claim strategies, and advanced consent flows.
- FAPI (Financial-grade API) baseline considerations where relevant.

## Summary

This crate delivers a secure, interoperable OIDC client foundation: discovery, authorization, token exchange/validation, userinfo, and session/logout helpers. It emphasizes security best practices, provider-agnostic design, and extensibility, enabling applications to implement modern authentication reliably across OpenID Providers.
