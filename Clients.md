# Prompt: Generate Clients CRUD Pages and Components

## Context

You are building the frontend UI for managing OAuth2/OIDC clients in a SvelteKit application. The backend API already exists with full CRUD operations for clients.

## Backend API Endpoints

The following API endpoints are already implemented and available via the `ClientApi` class:

### List Clients

- **Endpoint**: `GET /oidc/api/clients`
- **Method**: `clientList()`
- **Returns**: `Promise<Array<Client>>`
- **Permission**: `client:read`

### Get Client by ID

- **Endpoint**: `GET /oidc/api/clients/{client_id}`
- **Method**: `clientByClientId({ clientId: string })`
- **Returns**: `Promise<Client>`
- **Permission**: `client:read`

### Create Client

- **Endpoint**: `POST /oidc/api/clients`
- **Method**: `clientCreate({ clientRegisterRequest: ClientRegisterRequest })`
- **Returns**: `Promise<Client>`
- **Permission**: `client:write`

### Update Client

- **Endpoint**: `PUT /oidc/api/clients/{client_id}`
- **Method**: `clientUpdate({ clientId: string, clientRegisterRequest: ClientRegisterRequest })`
- **Returns**: `Promise<Client>`
- **Permission**: `client:write`

### Delete Client

- **Endpoint**: `DELETE /oidc/api/clients/{client_id}`
- **Method**: `clientDelete({ clientId: string })`
- **Returns**: `Promise<void>`
- **Permission**: `client:delete`

## Client Model Structure

The `Client` TypeScript interface includes the following key fields:

```typescript
interface Client {
  id: number;
  clientId: string;
  clientSecret?: string | null;
  name: string;
  active: boolean;
  applicationType: string;
  authMethod: string;
  grantTypes: Array<string>;
  responseTypes: Array<string>;
  scopes: Array<string>;
  redirectUris?: Array<string> | null;
  postLogoutRedirectUris?: Array<string> | null;
  audience?: Array<string> | null;
  logoUri?: string | null;
  clientUri?: string | null;
  policyUri?: string | null;
  termsOfServiceUri?: string | null;
  accessTokenExpiresInSeconds: number;
  idTokenExpiresInSeconds: number;
  refreshExpiresInSeconds: number;
  createdAt: Date;
  updatedAt: Date;
}
```

## Required Pages and Components

### 1. Clients List Page

**Location**: `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/+page.svelte`

**Requirements**:

- Display a table/list of all clients
- Show key information: name, clientId, active status, applicationType
- Include "Create New Client" button at the top
- Each row should have action buttons: View, Edit, Delete
- Implement search/filter functionality
- Show loading state while fetching
- Handle empty state when no clients exist
- Use responsive design (table on desktop, cards on mobile)

### 2. Client Detail/View Page

**Location**: `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/[clientId]/+page.svelte`

**Requirements**:

- Display all client details in a readable format
- Show all fields from the Client model
- Display arrays (scopes, grantTypes, etc.) as tags/chips
- Include "Edit" and "Delete" buttons
- Include "Back to Clients" navigation
- Handle loading and error states

### 3. Create Client Page

**Location**: `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/create/+page.svelte`

**Requirements**:

- Form with all required fields from ClientRegisterRequest
- Use Valibot for form validation (following existing patterns)
- Text inputs for: clientId, name, clientSecret
- Multi-select or tag inputs for: scopes, grantTypes, responseTypes, redirectUris, etc.
- Dropdowns for: applicationType, authMethod
- Number inputs for: accessTokenExpiresInSeconds, idTokenExpiresInSeconds, refreshExpiresInSeconds
- Checkbox for: active
- Submit button that calls `clientCreate()`
- Cancel button to go back
- Display validation errors using the `Issues` component
- Show success notification and redirect on successful creation

### 4. Edit Client Page

**Location**: `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/[clientId]/edit/+page.svelte`

**Requirements**:

- Pre-populate form with existing client data
- Same form structure as create page
- Submit button that calls `clientUpdate()`
- Show success notification and redirect on successful update
- Handle loading state while fetching initial data

### 5. Reusable Components

#### ClientForm Component

**Location**: `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/_ClientForm.svelte`

**Requirements**:

- Shared form component used by both create and edit pages
- Accept initial values as props
- Emit submit event with form data
- Use `createForm()` utility from `$lib/common/util/form.svelte`
- Implement Valibot schema for validation
- Include all Client fields with appropriate input types
- Use `Issues` component for displaying validation errors

#### ClientCard Component

**Location**: `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/_ClientCard.svelte`

**Requirements**:

- Display client summary in card format
- Show: name, clientId, active status, applicationType
- Include action buttons (View, Edit, Delete)
- Used in mobile view of list page

## Code Patterns to Follow

### 1. API Usage

Import and use the existing `clientApi` instance:

```typescript
import { clientApi } from "$lib/common/openapi";

// In component
const clients = await clientApi.clientList();
```

### 2. Form Validation with Valibot

```typescript
import * as v from "valibot";
import { createForm } from "$lib/common/util/form.svelte";

const ClientFormSchema = () =>
  v.object({
    clientId: v.pipe(v.string(), v.minLength(1)),
    name: v.pipe(v.string(), v.minLength(1)),
    // ... other fields
  });

const form = createForm(ClientFormSchema(), initialValues);
```

### 3. Error Handling

```typescript
import { handleError } from "$lib/common/errors";

try {
  await clientApi.clientCreate({ clientRegisterRequest: value });
} catch (e) {
  handleError(e);
}
```

### 4. Navigation

```typescript
import { goto } from "$app/navigation";
import { resolve } from "$app/paths";

await goto(resolve("/clients"));
```

### 5. Styling

- Use existing Tailwind CSS classes
- Follow the card-based layout: `<div class="card">...</div>`
- Use buttons: `<button class="btn primary">...</button>`
- Responsive grid: `<div class="grid grid-cols-1 gap-4 md:grid-cols-2">...</div>`

### 6. Page Loading

```typescript
// +page.ts
import type { PageLoad } from "./$types";
import { clientApi } from "$lib/common/openapi";

export const load: PageLoad = async () => {
  const clients = await clientApi.clientList();
  return { clients };
};
```

## Deliverables

Generate the following files with complete, production-ready code:

1. `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/+page.svelte` - List page
2. `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/+page.ts` - List page loader
3. `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/create/+page.svelte` - Create page
4. `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/[clientId]/+page.svelte` - Detail page
5. `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/[clientId]/+page.ts` - Detail page loader
6. `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/[clientId]/edit/+page.svelte` - Edit page
7. `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/[clientId]/edit/+page.ts` - Edit page loader
8. `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/_ClientForm.svelte` - Shared form component
9. `/crates/oidc-ui/frontend/src/routes/(auth)/(home)/clients/_ClientCard.svelte` - Card component

## Additional Notes

- Include proper TypeScript types for all variables and functions
- Add appropriate `<svelte:head>` tags with page titles
- Include proper ARIA labels for accessibility
- Handle all edge cases (loading, errors, empty states)
- Follow existing code patterns from the profile pages
- Use Lucide Svelte icons where appropriate (Trash, Edit, Plus, ArrowLeft, etc.)
- Implement confirmation dialogs for delete operations
- Add success notifications after create/update/delete operations
