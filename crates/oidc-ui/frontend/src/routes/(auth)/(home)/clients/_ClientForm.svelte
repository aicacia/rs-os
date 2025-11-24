<script lang="ts" context="module">
	import * as v from 'valibot';

	export const ClientFormSchema = () =>
		v.object({
			clientId: v.pipe(v.string(), v.minLength(1, 'Client ID is required')),
			name: v.pipe(v.string(), v.minLength(1, 'Name is required')),
			clientSecret: v.optional(v.nullable(v.string())),
			applicationType: v.pipe(v.string(), v.minLength(1, 'Application type is required')),
			authMethod: v.pipe(v.string(), v.minLength(1, 'Auth method is required')),
			grantTypes: v.pipe(
				v.array(v.string()),
				v.minLength(1, 'At least one grant type is required')
			),
			responseTypes: v.pipe(
				v.array(v.string()),
				v.minLength(1, 'At least one response type is required')
			),
			scopes: v.pipe(v.array(v.string()), v.minLength(1, 'At least one scope is required')),
			redirectUris: v.optional(v.nullable(v.array(v.string()))),
			postLogoutRedirectUris: v.optional(v.nullable(v.array(v.string()))),
			audience: v.optional(v.nullable(v.array(v.string()))),
			logoUri: v.optional(v.nullable(v.string())),
			clientUri: v.optional(v.nullable(v.string())),
			policyUri: v.optional(v.nullable(v.string())),
			termsOfServiceUri: v.optional(v.nullable(v.string())),
			accessTokenExpiresInSeconds: v.pipe(v.number(), v.minValue(1, 'Must be at least 1 second')),
			idTokenExpiresInSeconds: v.pipe(v.number(), v.minValue(1, 'Must be at least 1 second')),
			refreshExpiresInSeconds: v.pipe(v.number(), v.minValue(1, 'Must be at least 1 second'))
		});

	export type ClientFormData = v.InferOutput<ReturnType<typeof ClientFormSchema>>;
</script>

<script lang="ts">
	import type { ClientRegisterRequest, Client } from '$lib/common/openapi/oidc/models/index';
	import { createForm } from '$lib/common/util/form.svelte';
	import Issues from '$lib/common/components/Issues.svelte';
	import { X, Plus } from '@lucide/svelte';
	import type { Snippet } from 'svelte';

	let {
		initialValues,
		onsubmit,
		submitLabel = 'Save Client',
		readonly = false,
		actions
	}: {
		initialValues: Partial<ClientRegisterRequest> | Partial<Client>;
		onsubmit: (data: ClientFormData) => void | Promise<void>;
		submitLabel?: string;
		readonly?: boolean;
		actions?: Snippet;
	} = $props();

	const form = createForm(ClientFormSchema(), {
		clientId: initialValues.clientId ?? '',
		name: initialValues.name ?? '',
		clientSecret: ('clientSecret' in initialValues ? initialValues.clientSecret : null) ?? null,
		applicationType: initialValues.applicationType ?? 'web',
		authMethod: initialValues.authMethod ?? 'client_secret_basic',
		grantTypes: initialValues.grantTypes ?? ['authorization_code'],
		responseTypes: initialValues.responseTypes ?? ['code'],
		scopes: initialValues.scopes ?? ['openid'],
		redirectUris: initialValues.redirectUris ?? null,
		postLogoutRedirectUris: initialValues.postLogoutRedirectUris ?? null,
		audience: initialValues.audience ?? null,
		logoUri: initialValues.logoUri ?? null,
		clientUri: initialValues.clientUri ?? null,
		policyUri: initialValues.policyUri ?? null,
		termsOfServiceUri: initialValues.termsOfServiceUri ?? null,
		accessTokenExpiresInSeconds: initialValues.accessTokenExpiresInSeconds ?? 3600,
		idTokenExpiresInSeconds: initialValues.idTokenExpiresInSeconds ?? 3600,
		refreshExpiresInSeconds: initialValues.refreshExpiresInSeconds ?? 86400
	});

	// State for managing arrays - we'll use simple state tracking instead of the complex form array fields
	let grantTypes = $state<string[]>(initialValues.grantTypes ?? ['authorization_code']);
	let responseTypes = $state<string[]>(initialValues.responseTypes ?? ['code']);
	let scopesState = $state<string[]>(initialValues.scopes ?? ['openid']);
	let redirectUris = $state<string[]>(
		(initialValues.redirectUris as string[] | null | undefined) ?? []
	);
	let postLogoutRedirectUris = $state<string[]>(
		(initialValues.postLogoutRedirectUris as string[] | null | undefined) ?? []
	);
	let audienceState = $state<string[]>(
		(initialValues.audience as string[] | null | undefined) ?? []
	);

	// String array management
	let redirectUriInput = $state('');
	let postLogoutRedirectUriInput = $state('');
	let audienceInput = $state('');

	function addRedirectUri() {
		if (redirectUriInput.trim()) {
			redirectUris = [...redirectUris, redirectUriInput.trim()];
			redirectUriInput = '';
		}
	}

	function removeRedirectUri(index: number) {
		redirectUris = redirectUris.filter((_, i) => i !== index);
	}

	function addPostLogoutRedirectUri() {
		if (postLogoutRedirectUriInput.trim()) {
			postLogoutRedirectUris = [...postLogoutRedirectUris, postLogoutRedirectUriInput.trim()];
			postLogoutRedirectUriInput = '';
		}
	}

	function removePostLogoutRedirectUri(index: number) {
		postLogoutRedirectUris = postLogoutRedirectUris.filter((_, i) => i !== index);
	}

	function addAudience() {
		if (audienceInput.trim()) {
			audienceState = [...audienceState, audienceInput.trim()];
			audienceInput = '';
		}
	}

	function removeAudience(index: number) {
		audienceState = audienceState.filter((_, i) => i !== index);
	}

	// Multi-select management for grant types, response types, and scopes
	const availableGrantTypes = [
		'authorization_code',
		'implicit',
		'password',
		'client_credentials',
		'refresh_token',
		'urn:ietf:params:oauth:grant-type:device_code'
	];

	const availableResponseTypes = ['code', 'token', 'id_token'];

	const availableScopes = ['openid', 'profile', 'email', 'address', 'phone', 'offline_access'];

	function toggleGrantType(grantType: string) {
		if (grantTypes.includes(grantType)) {
			grantTypes = grantTypes.filter((gt) => gt !== grantType);
		} else {
			grantTypes = [...grantTypes, grantType];
		}
	}

	function toggleResponseType(responseType: string) {
		if (responseTypes.includes(responseType)) {
			responseTypes = responseTypes.filter((rt) => rt !== responseType);
		} else {
			responseTypes = [...responseTypes, responseType];
		}
	}

	function toggleScope(scope: string) {
		if (scopesState.includes(scope)) {
			scopesState = scopesState.filter((s) => s !== scope);
		} else {
			scopesState = [...scopesState, scope];
		}
	}

	async function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

		// Manually construct the form data with array values
		const formData: ClientFormData = {
			clientId: form.fields.clientId.value ?? '',
			name: form.fields.name.value ?? '',
			clientSecret: form.fields.clientSecret.value,
			applicationType: form.fields.applicationType.value ?? '',
			authMethod: form.fields.authMethod.value ?? '',
			grantTypes,
			responseTypes,
			scopes: scopesState,
			redirectUris: redirectUris.length > 0 ? redirectUris : null,
			postLogoutRedirectUris: postLogoutRedirectUris.length > 0 ? postLogoutRedirectUris : null,
			audience: audienceState.length > 0 ? audienceState : null,
			logoUri: form.fields.logoUri.value,
			clientUri: form.fields.clientUri.value,
			policyUri: form.fields.policyUri.value,
			termsOfServiceUri: form.fields.termsOfServiceUri.value,
			accessTokenExpiresInSeconds: form.fields.accessTokenExpiresInSeconds.value ?? 3600,
			idTokenExpiresInSeconds: form.fields.idTokenExpiresInSeconds.value ?? 3600,
			refreshExpiresInSeconds: form.fields.refreshExpiresInSeconds.value ?? 86400
		};

		// Validate basic fields
		const [, basicErr] = await form.validate();
		if (basicErr) {
			return;
		}

		await onsubmit(formData);
	}
</script>

<form onsubmit={handleSubmit} class="space-y-6">
	<!-- Basic Information -->
	<section class="card">
		<h3 class="mb-4 text-lg font-semibold">Basic Information</h3>
		<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
			<label class="block">
				<span class="text-sm font-medium">Client ID *</span>
				<input
					type="text"
					class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
					bind:value={form.fields.clientId.value}
					placeholder="my-client-id"
					{readonly}
					aria-label="Client ID"
				/>
				<Issues issues={form.fields.clientId.issues} />
			</label>

			<label class="block">
				<span class="text-sm font-medium">Name *</span>
				<input
					type="text"
					class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
					bind:value={form.fields.name.value}
					placeholder="My Application"
					{readonly}
					aria-label="Client Name"
				/>
				<Issues issues={form.fields.name.issues} />
			</label>

			<label class="block">
				<span class="text-sm font-medium">Client Secret</span>
				<input
					type="text"
					class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
					bind:value={form.fields.clientSecret.value}
					placeholder="Leave empty for public clients"
					{readonly}
					aria-label="Client Secret"
				/>
				<Issues issues={form.fields.clientSecret.issues} />
			</label>
		</div>
	</section>

	<!-- Application Configuration -->
	<section class="card">
		<h3 class="mb-4 text-lg font-semibold">Application Configuration</h3>
		<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
			<label class="block">
				<span class="text-sm font-medium">Application Type *</span>
				<select
					class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
					bind:value={form.fields.applicationType.value}
					disabled={readonly}
				>
					<option value="web">Web</option>
					<option value="native">Native</option>
					<option value="service">Service</option>
				</select>
				<Issues issues={form.fields.applicationType.issues} />
			</label>

			<label class="block">
				<span class="text-sm font-medium">Authentication Method *</span>
				<select
					class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
					bind:value={form.fields.authMethod.value}
					disabled={readonly}
				>
					<option value="client_secret_basic">Client Secret Basic</option>
					<option value="client_secret_post">Client Secret Post</option>
					<option value="none">None (Public)</option>
				</select>
				<Issues issues={form.fields.authMethod.issues} />
			</label>
		</div>
	</section>

	<!-- OAuth2 Configuration -->
	<section class="card">
		<h3 class="mb-4 text-lg font-semibold">OAuth2 Configuration</h3>

		<div class="space-y-4">
			<div>
				<span class="text-sm font-medium">Grant Types *</span>
				<div class="mt-2 flex flex-wrap gap-2">
					{#each availableGrantTypes as grantType}
						<label class="flex items-center gap-2">
							<input
								type="checkbox"
								checked={grantTypes.includes(grantType)}
								onchange={() => toggleGrantType(grantType)}
								disabled={readonly}
							/>
							<span class="text-sm">{grantType}</span>
						</label>
					{/each}
				</div>
			</div>

			<div>
				<span class="text-sm font-medium">Response Types *</span>
				<div class="mt-2 flex flex-wrap gap-2">
					{#each availableResponseTypes as responseType}
						<label class="flex items-center gap-2">
							<input
								type="checkbox"
								checked={responseTypes.includes(responseType)}
								onchange={() => toggleResponseType(responseType)}
								disabled={readonly}
							/>
							<span class="text-sm">{responseType}</span>
						</label>
					{/each}
				</div>
			</div>

			<div>
				<span class="text-sm font-medium">Scopes *</span>
				<div class="mt-2 flex flex-wrap gap-2">
					{#each availableScopes as scope}
						<label class="flex items-center gap-2">
							<input
								type="checkbox"
								checked={scopesState.includes(scope)}
								onchange={() => toggleScope(scope)}
								disabled={readonly}
							/>
							<span class="text-sm">{scope}</span>
						</label>
					{/each}
				</div>
			</div>
		</div>
	</section>

	<!-- URIs -->
	<section class="card">
		<h3 class="mb-4 text-lg font-semibold">URIs</h3>
		<div class="space-y-4">
			<!-- Redirect URIs -->
			<div>
				<span class="text-sm font-medium">Redirect URIs</span>
				{#if !readonly}
					<div class="mt-2 flex gap-2">
						<input
							type="url"
							class="block flex-1 rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
							bind:value={redirectUriInput}
							placeholder="https://example.com/callback"
							onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addRedirectUri())}
						/>
						<button type="button" class="btn primary" onclick={addRedirectUri}>
							<Plus class="h-4 w-4" />
						</button>
					</div>
				{/if}
				<div class="mt-2 flex flex-wrap gap-2">
					{#each redirectUris as uri, index}
						<span
							class="inline-flex items-center gap-1 rounded-full bg-gray-100 px-3 py-1 text-sm dark:bg-gray-700"
						>
							{uri}
							{#if !readonly}
								<button
									type="button"
									onclick={() => removeRedirectUri(index)}
									class="text-red-500 hover:text-red-700"
								>
									<X class="h-3 w-3" />
								</button>
							{/if}
						</span>
					{/each}
				</div>
			</div>

			<!-- Post Logout Redirect URIs -->
			<div>
				<span class="text-sm font-medium">Post Logout Redirect URIs</span>
				{#if !readonly}
					<div class="mt-2 flex gap-2">
						<input
							type="url"
							class="block flex-1 rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
							bind:value={postLogoutRedirectUriInput}
							placeholder="https://example.com/logout"
							onkeydown={(e) =>
								e.key === 'Enter' && (e.preventDefault(), addPostLogoutRedirectUri())}
						/>
						<button type="button" class="btn primary" onclick={addPostLogoutRedirectUri}>
							<Plus class="h-4 w-4" />
						</button>
					</div>
				{/if}
				<div class="mt-2 flex flex-wrap gap-2">
					{#each postLogoutRedirectUris as uri, index}
						<span
							class="inline-flex items-center gap-1 rounded-full bg-gray-100 px-3 py-1 text-sm dark:bg-gray-700"
						>
							{uri}
							{#if !readonly}
								<button
									type="button"
									onclick={() => removePostLogoutRedirectUri(index)}
									class="text-red-500 hover:text-red-700"
								>
									<X class="h-3 w-3" />
								</button>
							{/if}
						</span>
					{/each}
				</div>
			</div>

			<!-- Additional URIs -->
			<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
				<label class="block">
					<span class="text-sm font-medium">Logo URI</span>
					<input
						type="url"
						class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
						bind:value={form.fields.logoUri.value}
						placeholder="https://example.com/logo.png"
						{readonly}
					/>
				</label>

				<label class="block">
					<span class="text-sm font-medium">Client URI</span>
					<input
						type="url"
						class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
						bind:value={form.fields.clientUri.value}
						placeholder="https://example.com"
						{readonly}
					/>
				</label>

				<label class="block">
					<span class="text-sm font-medium">Policy URI</span>
					<input
						type="url"
						class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
						bind:value={form.fields.policyUri.value}
						placeholder="https://example.com/policy"
						{readonly}
					/>
				</label>

				<label class="block">
					<span class="text-sm font-medium">Terms of Service URI</span>
					<input
						type="url"
						class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
						bind:value={form.fields.termsOfServiceUri.value}
						placeholder="https://example.com/terms"
						{readonly}
					/>
				</label>
			</div>
		</div>
	</section>

	<!-- Audience -->
	<section class="card">
		<h3 class="mb-4 text-lg font-semibold">Audience</h3>
		{#if !readonly}
			<div class="flex gap-2">
				<input
					type="text"
					class="block flex-1 rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
					bind:value={audienceInput}
					placeholder="https://api.example.com"
					onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addAudience())}
				/>
				<button type="button" class="btn primary" onclick={addAudience}>
					<Plus class="h-4 w-4" />
				</button>
			</div>
		{/if}
		<div class="mt-2 flex flex-wrap gap-2">
			{#each audienceState as aud, index}
				<span
					class="inline-flex items-center gap-1 rounded-full bg-gray-100 px-3 py-1 text-sm dark:bg-gray-700"
				>
					{aud}
					{#if !readonly}
						<button
							type="button"
							onclick={() => removeAudience(index)}
							class="text-red-500 hover:text-red-700"
						>
							<X class="h-3 w-3" />
						</button>
					{/if}
				</span>
			{/each}
		</div>
	</section>

	<!-- Token Expiration -->
	<section class="card">
		<h3 class="mb-4 text-lg font-semibold">Token Expiration (seconds)</h3>
		<div class="grid grid-cols-1 gap-4 md:grid-cols-3">
			<label class="block">
				<span class="text-sm font-medium">Access Token *</span>
				<input
					type="number"
					class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
					bind:value={form.fields.accessTokenExpiresInSeconds.value}
					min="1"
					{readonly}
				/>
				<Issues issues={form.fields.accessTokenExpiresInSeconds.issues} />
			</label>

			<label class="block">
				<span class="text-sm font-medium">ID Token *</span>
				<input
					type="number"
					class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
					bind:value={form.fields.idTokenExpiresInSeconds.value}
					min="1"
					{readonly}
				/>
				<Issues issues={form.fields.idTokenExpiresInSeconds.issues} />
			</label>

			<label class="block">
				<span class="text-sm font-medium">Refresh Token *</span>
				<input
					type="number"
					class="mt-1 block w-full rounded-lg border border-gray-300 px-3 py-2 dark:border-gray-600 dark:bg-gray-800"
					bind:value={form.fields.refreshExpiresInSeconds.value}
					min="1"
					{readonly}
				/>
				<Issues issues={form.fields.refreshExpiresInSeconds.issues} />
			</label>
		</div>
	</section>

	{#if !readonly}
		<div class="flex justify-end gap-2">
			{#if actions}
				{@render actions()}
			{/if}
			<button type="submit" class="btn primary">{submitLabel}</button>
		</div>
	{/if}
</form>
